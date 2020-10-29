use std::cell::RefCell;
use std::rc::Rc;

use lazy_static::lazy_static;

use fltk::{
    Event,
    Font,
    Key,
    GroupExt,
    InputExt,
    TableExt,
    WidgetExt,
    WindowExt,
};
use fltk::app;
use fltk::app::{App, Scheme, channel, Sender, Receiver};
use fltk::button::{Button, RadioRoundButton};
use fltk::Cursor;
use fltk::draw;
use fltk::dialog::alert;
use fltk::input::Input;
use fltk::table::TableContext::{
    StartPage,
    ColHeader,
    RowHeader,
    Cell,
};

mod cell_data;
use cell_data::CellData;

mod draw_table;
use draw_table::{draw_header, draw_data};

mod widgets;
use widgets::{make_window, make_table, make_input, InputType, VisibleFlag};

mod database;
use database::{populate_table, Database, Row};

mod error;
use error::Error;

mod connector;
use connector::make_connector;

#[derive(Debug, Copy, Clone)]
pub enum Message {
    Redraw,
    SetSend,
    SetRecieve,
    AddRow,
    UpdateTable,
}

lazy_static! {
    static ref CHANNEL: (Sender<Message>, Receiver<Message>) = channel::<Message>();
}

fn get_alpha_upper_char(char_index: i32) -> char {
    (char_index + 65) as u8 as char
}

fn main() -> Result<(), Error> {
    let connector = Rc::from(RefCell::from(make_connector()?));

    let db = Rc::from(RefCell::from(Database::new("mysql://zotho:zotho@localhost:3306/rust".to_owned())?));

    // If we need populate table
    if let Some(populate_flag) = std::env::args().nth(1) {
        if populate_flag == "--populate" {
            populate_table(&db.borrow());
        }
    }

    let raw_data: Vec<Vec<String>> = db.borrow().get_rows().unwrap().iter()
        .map(|row| row.into()).collect();
    let n_rows = raw_data.len();
    let n_cols = raw_data.first().unwrap_or(&Vec::new()).len();

    let data: Rc<RefCell<Vec<Vec<String>>>> = Rc::from(RefCell::from(raw_data));
    let cell = Rc::from(RefCell::from(CellData::default()));

    let sender = CHANNEL.0;
    let receiver = CHANNEL.1;
    
    let fltk_app = App::default().with_scheme(Scheme::Gtk);

    let mut window = make_window(100, 100, 410, 640, "Spreadsheet");

    let mut db_input = Input::new(35, 5, 370, 30, "DB:");
    db_input.set_value(&db.borrow().url().clone());

    let db_clone = db.clone();
    let data_clone = data.clone();
    let db_input_clone = db_input.clone();

    db_input.handle(Box::new(move |event| match event {
        Event::Unfocus => {
            let value = db_input_clone.value();
            let mut db = db_clone.borrow_mut();
            let initial_url = db.url();
            if initial_url != value {
                if let Err(error) = db.set_url(value) {
                    db_input_clone.set_value(&initial_url);
                    drop(db);
                    alert(0, 0, &error.to_string())
                } else {
                    match db.get_rows() {
                        Ok(rows) => {
                            println!("ROWS {:?}", rows);
                            let mut data = data_clone.borrow_mut();
                            data.clear();
                            let raw_data: Vec<Vec<String>> = rows.iter().map(|row| row.into()).collect();
                            data.extend(raw_data);
                            sender.send(Message::UpdateTable);
                        }
                        Err(error) => {
                            db_input_clone.set_value(&initial_url);
                            drop(db);
                            alert(0, 0, &error.to_string())
                        }
                    }
                    
                }
            }
            false
        }
        _ => false
    }));

    make_input(
        125,
        45,
        200,
        30,
        "Bind socket:",
        connector.clone(),
        InputType::BindAddress,
        "Rebind address: ",
    );

    make_input(
        125,
        85,
        200,
        30,
        "Connect socket:",
        connector.clone(),
        InputType::ConnectAddress,
        "Reconnect address: ",
    );

    let mut rb_send = RadioRoundButton::new(5, 115, 100, 30, "Send");
    rb_send.toggle(true);
    rb_send.set_callback(Box::new(move || sender.send(Message::SetSend)));

    let mut rb_recieve = RadioRoundButton::new(5, 145, 100, 30, "Recieve");
    rb_recieve.set_callback(Box::new(move || sender.send(Message::SetRecieve)));

    let (mut table, input) = make_table(5, 205, 400, 400, "Data", n_rows, n_cols);
    let input_visible = Rc::from(RefCell::from(VisibleFlag {visible: false}));

    let mut button = Button::new(5, 610, 400, 25, "Add row");
    button.set_callback(Box::new(move || sender.send(Message::AddRow)));

    window.add(&button);

    window.show();

    let table_clone = table.clone();
    let cell_clone = cell.clone();
    let data_clone = data.clone();
    let input_visible_clone = input_visible.clone();

    // Called when the table is drawn then when it's redrawn due to events
    table.draw_cell(Box::new(move |ctx, row, col, x, y, w, h| match ctx {
        StartPage => draw::set_font(Font::Helvetica, 14),
        ColHeader => draw_header(&format!("{}", get_alpha_upper_char(col)), x, y, w, h),
        RowHeader => draw_header(&format!("{}", row + 1), x, y, w, h),
        Cell => {
            let selected = table_clone.is_selected(row, col);
            if selected {
                cell_clone
                    .borrow_mut()
                    .select(row, col, x, y, w, h); // Captures the cell information
                if input_visible_clone.borrow().visible {
                    return; // Don't redraw cell if input is visible
                }
            }
            let data = data_clone.borrow();
            let (row, col) = (row as usize, col as usize);
            draw_data(&data[row][col], x, y, w, h, selected);
        }
        _ => (),
    }));


    let connector_clone = connector.clone();
    let db_clone = db.clone();
    let cell_clone = cell.clone();
    let data_clone = data.clone();
    let mut table_clone = table.clone();
    let mut window_clone = window.clone();
    let mut input_clone = input.clone();
    let input_visible_clone = input_visible.clone();

    // Handle double clicks
    // Handle Enter: store the data into the cell or start writing
    table.handle(Box::new(move |event| {
        match event {
            Event::Push => {
                table_clone.take_focus().unwrap_or(());
                if app::event_clicks() {
                    let data = data_clone.borrow();
                    let cell = cell_clone.borrow_mut();
                    input_clone.resize(cell.x, cell.y, cell.w, cell.h);
                    let (row, col) = (cell.row as usize, cell.col as usize);
                    input_clone.set_value(&data[row][col]);
                    input_clone.show();
                    input_visible_clone.borrow_mut().visible = true;
                    return true;
                }
                false
            }
            Event::KeyDown if app::event_key() == Key::Enter => {
                let db = db_clone.borrow();
                let cell = cell.borrow();
                let (row, col) = (cell.row as usize, cell.col as usize);
                if input_visible_clone.borrow().visible {
                    let value = input_clone.value();
                    let value_clone = value.clone();

                    let db_row = row + 1;
                    let result: Result<(), Error> = match col {
                        0 => {
                            match value.parse() {
                                Ok(parsed_value) => {
                                    db.update_number(db_row, parsed_value)
                                        .map_err(|err| err.into())
                                }
                                Err(error) => {
                                    alert(0, 0, format!("Can't parse \"{}\" as int", value).as_str());
                                    Err(error.into())
                                }
                            }
                        }
                        1 => db.update_text(db_row, if value.len() > 0 {Some(value)} else {None})
                                .map_err(|err| err.into()),
                        _ => unreachable!()
                    };

                    if result.is_ok() {
                        let mut data = data_clone.borrow_mut();
                        data[row][col] = value_clone;
                        match connector_clone.borrow().send_data(data.as_ref()) {
                            Ok(n_bytes) => println!("Send {} bytes", n_bytes),
                            Err(error) => println!("Send error: {}", error.details),
                        };
                    }

                    input_clone.resize(cell.x, cell.y, cell.w, cell.h);
                    input_clone.set_value("");
                    input_clone.hide();
                    input_visible_clone.borrow_mut().visible = false;

                    window_clone.set_cursor(Cursor::Default); // If we don't do this, cursor can disappear!

                    table_clone.redraw();
                } else {
                    input_clone.resize(cell.x, cell.y, cell.w, cell.h);
                    input_clone.set_value(&data_clone.borrow()[row][col]);
                    input_clone.show();
                    input_visible_clone.borrow_mut().visible = true;
                }
                return true;
            }
            _ => false,
        }
    }));

    app::add_timeout(1.0, Box::new(callback));

    while fltk_app.wait().unwrap() {
        match receiver.recv() {
            Some(Message::Redraw) => {
                let connector = connector.borrow_mut();
                // println!("{} {}", connector.bind_addr(), connector.connect_addr());
                if rb_send.is_toggled() {
                    match connector.send_data(&data.borrow()) {
                        Ok(n_bytes) => println!("Send {} bytes", n_bytes),
                        Err(error) => println!("Send error: {}", error.details),
                    };
                } else if rb_recieve.is_toggled() {
                    for _ in 0..2 {
                        match connector.receive_data() {
                            Ok(incoming_data) => {
                                let mut data = data.borrow_mut();
                                data.clear();
                                data.extend(incoming_data);
                                table.set_rows(data.len() as u32);
                                println!("Receive {:?}", data);
                            },
                            Err(error) => {
                                println!("Receive error: {}", error.details);
                            },
                        }
                    }
                } else {
                    unreachable!();
                };
            }
            Some(Message::SetSend) => {
                table.activate();
                button.activate();
                db_input.activate();
            }
            Some(Message::SetRecieve) => {
                table.deactivate();
                button.deactivate();
                db_input.deactivate();
            }
            Some(Message::AddRow) => {
                let mut data = data.borrow_mut();
                data.push(vec!["0".to_owned(), "".to_owned()]);
                table.set_rows(data.len() as u32);
                db.borrow().insert_row(Row::default()).unwrap();
            }
            Some(Message::UpdateTable) => {
                table.set_rows(data.borrow().len() as u32);
            }
            None => ()
        }
    }

    Ok(())
}

fn callback() {
    let sender = CHANNEL.0;
    sender.send(Message::Redraw);
    app::redraw();
    app::repeat_timeout(1.0, Box::new(callback));
}