use std::cell::RefCell;
use std::rc::Rc;

use fltk::{
    Event,
    Key,
    GroupExt,
    InputExt,
    TableExt,
    WidgetExt,
    WindowExt,
};
use fltk::app;
use fltk::dialog::alert;
use fltk::input::Input;
use fltk::table::Table;
use fltk::window::DoubleWindow;

use crate::connector::{Connector, MyConnectorResult};

pub fn make_window(
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    title: &str,
) -> DoubleWindow {
    let mut window = DoubleWindow::new(x, y, w, h, title)
        .center_screen();
    window.set_callback(Box::new(|| {
        let event = app::event();
        let close = event == Event::Close;
        let keyboard_close = event == Event::Shortcut && app::event_key() == Key::Escape;
        if close || keyboard_close {
            // let x = fltk::dialog::choice(0, 0, "Would you like to save your work?", "No", "Cancel", "Yes");
            app::quit();
        }
    }));
    window
}

pub struct VisibleFlag {
    pub visible: bool
}

pub fn make_table(
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    title: &str,
    n_rows: usize,
    n_cols: usize,
) -> (Table, Input) {
    let mut table = Table::new(x, y, w, h, title);

    table.set_rows(n_rows as u32);
    table.set_row_header(true);
    table.set_row_height_all(27);
    table.set_row_resize(true);

    table.set_cols(n_cols as u32);
    table.set_col_header(true);
    table.set_col_width_all(170);
    table.set_col_resize(true);

    table.set_selection(0, 0, 0, 0);

    // We need an input widget for table
    let mut input = Input::new(0, 0, 0, 0, "");
    input.hide();
    table.add(&input);

    (table, input)
}

#[derive(Copy, Clone)]
pub enum InputType {
    BindAddress,
    ConnectAddress,
}

pub fn make_input(
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    title: &str,
    connector: Rc<RefCell<Connector>>,
    input_type: InputType,
    log_on_change: &'static str,
) -> Input {
    fn get_addr(connector: &Rc<RefCell<Connector>>, input_type: InputType) -> String {
        let connector = connector.borrow();
        match input_type {
            InputType::BindAddress => connector.bind_addr(),
            InputType::ConnectAddress => connector.connect_addr()
        }.to_owned()
    }
    
    fn set_addr(connector: &Rc<RefCell<Connector>>, input_type: InputType, value: &str) -> MyConnectorResult<()> {
        let mut connector = connector.borrow_mut();
        match input_type {
            InputType::BindAddress => connector.set_bind_addr(value),
            InputType::ConnectAddress => connector.set_connect_addr(value)
        }
    }

    let mut input = Input::new(x, y, w, h, title);
    let connector_clone = connector.clone();
    input.set_value(get_addr(&connector_clone, input_type).as_str());

    let input_clone = input.clone();
    input.handle(Box::new(move |event| match event {
        Event::Unfocus => {
            let value = input_clone.value();
            let value = value.as_str();
            if get_addr(&connector_clone, input_type) != value {
                if let Err(error) = set_addr(&connector_clone, input_type, value) {
                    input_clone.set_value(get_addr(&connector_clone, input_type).as_str());
                    alert(0, 0, error.details.as_str());
                } else {
                    println!("{}{}", log_on_change, value);
                }
                return true;
            }
            false
        }
        _ => false
    }));
    input
}

