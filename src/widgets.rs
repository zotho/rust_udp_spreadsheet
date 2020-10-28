use fltk::{
    Event,
    Key,
    TableExt,
    WidgetExt,
    WindowExt,
};
use fltk::app;

use fltk::window::DoubleWindow;
use fltk::table::Table;

pub fn make_window() -> DoubleWindow {
    let mut window = DoubleWindow::new(100, 100, 410, 600, "Spreadsheet")
        .center_screen();
    window.make_resizable(true);
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

pub fn make_table(n_rows: usize, n_cols: usize) -> Table {
    let mut table = Table::new(5, 175, 400, 400, "Data");

    table.set_rows(n_rows as u32);
    table.set_row_header(true);
    table.set_row_height_all(27);
    table.set_row_resize(true);

    table.set_cols(n_cols as u32);
    table.set_col_header(true);
    table.set_col_width_all(175);
    table.set_col_resize(true);

    table.set_selection(0, 0, 0, 0);
    table
}