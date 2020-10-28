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
use fltk::button::RadioRoundButton;
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
use widgets::{make_window, make_table};

mod database;
use database::{populate_table, Database};

mod error;
use error::Error;

mod connector;
use connector::SocketClient;

#[derive(Debug, Copy, Clone)]
pub enum Message {
    Redraw,
}

lazy_static! {
    static ref CHANNEL: (Sender<Message>, Receiver<Message>) = channel::<Message>();
}

fn get_alpha_upper_char(char_index: i32) -> char {
    (char_index + 65) as u8 as char
}



fn main() -> Result<(), Error> {
    let mut bind_port = 10001;
    
    let dest_socket = "127.0.0.1:10000";

    // Try to bind free port
    let mut raw_connector = Err(Error::new("Not initialized"));
    while bind_port < 10010 {
        raw_connector = SocketClient::new(format!("127.0.0.1:{}", bind_port).as_str(), dest_socket);
        if raw_connector.is_err() {
            bind_port += 1;
        } else {
            break;
        }
    }
    let connector = Rc::from(RefCell::from(raw_connector.unwrap()));


    let db = Database::new("mysql://zotho:zotho@localhost:3306/rust".to_owned()).unwrap();

    // If we need populate table
    if let Some(populate_flag) = std::env::args().nth(1) {
        if populate_flag == "--populate" {
            populate_table(&db);
        }
    }

    let raw_data: Vec<Vec<String>> = db.get_rows().unwrap().iter()
        .map(|row| row.into()).collect();
    let n_rows = raw_data.len();
    let n_cols = raw_data.first().unwrap_or(&Vec::new()).len();

    let data: Rc<RefCell<Vec<Vec<String>>>> = Rc::from(RefCell::from(raw_data));
    let cell = Rc::from(RefCell::from(CellData::default()));

    let fltk_app = App::default().with_scheme(Scheme::Gtk);

    let mut window = make_window();

    let mut bind_socket_input = Input::new(120, 5, 200, 30, "Bind socket:");
    bind_socket_input.set_value(connector.borrow().bind_addr());
    let connector_clone = connector.clone();
    let bind_socket_input_clone = bind_socket_input.clone();
    bind_socket_input.handle(Box::new(move |event| match event {
        Event::Unfocus => {
            let value = bind_socket_input_clone.value();
            let mut connector = connector_clone.borrow_mut();
            if connector.bind_addr() != value.as_str() {
                if let Err(error) = connector.set_bind_addr(value.as_str()) {
                    bind_socket_input_clone.set_value(connector.bind_addr());
                    drop(connector);
                    alert(0, 0, error.details.as_str());
                } else {
                    println!("Rebind address: {}", value.as_str());
                }
                return true;
            }
            false
        }
        _ => false
    }));

    let mut connection_socket_input = Input::new(120, 45, 200, 30, "Connect socket:");
    connection_socket_input.set_value(connector.borrow().connect_addr());
    let connector_clone = connector.clone();
    let connection_socket_input_clone = connection_socket_input.clone();
    connection_socket_input.handle(Box::new(move |event| match event {
        Event::Unfocus => {
            let value = connection_socket_input_clone.value();
            let mut connector = connector_clone.borrow_mut();
            if connector.connect_addr() != value.as_str() {
                if let Err(error) = connector.set_connect_addr(value.as_str()) {
                    connection_socket_input_clone.set_value(connector.connect_addr());
                    drop(connector);
                    alert(0, 0, error.details.as_str());
                } else {
                    println!("Reconnect address: {}", value.as_str());
                }
                return true;
            }
            false
        }
        _ => false
    }));

    let mut table = make_table(n_rows, n_cols);

    let mut rb_send = RadioRoundButton::new(5, 75, 100, 40, "Send");
    rb_send.toggle(true);

    let mut table_clone = table.clone();
    let rb_send_clone = rb_send.clone();
    rb_send.handle(Box::new(move |event| match event {
        Event::Push | Event::Shortcut | Event::Focus => {
            if rb_send_clone.is_toggled() {
                table_clone.activate();
            } else {
                table_clone.deactivate();
            }
            false
        }
        _ => false
    }));
    let mut rb_recieve = RadioRoundButton::new(5, 115, 100, 40, "Recieve");
    let mut table_clone = table.clone();
    let rb_recieve_clone = rb_recieve.clone();
    rb_recieve.handle(Box::new(move |event| match event {
        Event::Push | Event::Shortcut | Event::Focus => {
            if !rb_recieve_clone.is_toggled() {
                table_clone.activate();
            } else {
                table_clone.deactivate();
            }
            false
        }
        _ => false
    }));

    window.add(&rb_send);
    window.add(&rb_recieve);

    // We need an input widget for table
    let mut input = Input::new(0, 0, 0, 0, "");
    input.hide();
    table.add(&input);

    window.show();

    let table_clone = table.clone();
    let cell_clone = cell.clone();
    let data_clone = data.clone();
    let mut input_clone = input.clone();

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
                if input_clone.visible() {
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
    let cell_clone = cell.clone();
    let data_clone = data.clone();
    let mut table_clone = table.clone();
    let mut window_clone = window.clone();
    let mut input_clone = input.clone();

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
                    return true;
                }
                false
            }
            Event::KeyDown if app::event_key() == Key::Enter => {
                let cell = cell.borrow();
                let (row, col) = (cell.row as usize, cell.col as usize);
                if input.visible() {
                    let value = input.value();
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
                            Ok(n_bytes) => println!("send_data {} bytes", n_bytes),
                            Err(error) => println!("{}", error.details),
                        };
                    }

                    input.resize(cell.x, cell.y, cell.w, cell.h);
                    input.set_value("");
                    input.hide();

                    window_clone.set_cursor(Cursor::Default); // If we don't do this, cursor can disappear!

                    table_clone.redraw();
                } else {
                    input.resize(cell.x, cell.y, cell.w, cell.h);
                    input.set_value(&data_clone.borrow()[row][col]);
                    input.show();
                }
                return true;
            }
            _ => false,
        }
    }));

    app::add_timeout(1.0, callback);

    let receiver = CHANNEL.1;

    while fltk_app.wait() {
        if let Some(Message::Redraw) = receiver.recv() {
            let connector = connector.borrow_mut();
            if rb_send.is_toggled() {
                match connector.send_data(&data.borrow()) {
                    Ok(n_bytes) => println!("send_data {} bytes", n_bytes),
                    Err(error) => println!("{}", error.details),
                };
            } else if rb_recieve.is_toggled() {
                for _ in 0..2 {
                    match connector.receive_data() {
                        Ok(incoming_data) => {
                            let mut data = data.borrow_mut();
                            data.clear();
                            data.extend(incoming_data);
                            println!("receive_data {:?}", data);
                        },
                        Err(error) => {
                            // end_of_recieve = true;
                            println!("{}", error.details);
                        },
                    }
                }
            } else {
                unreachable!();
            };
            
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