use fltk::draw::{
    draw_box, draw_rect, draw_rectf, draw_text2, pop_clip, push_clip, set_draw_color,
};
use fltk::{Align, Color, FrameType};

pub fn draw_header(txt: &str, x: i32, y: i32, w: i32, h: i32) {
    push_clip(x, y, w, h);
    draw_box(FrameType::ThinUpBox, x, y, w, h, Color::FrameDefault);
    set_draw_color(Color::Black);
    draw_text2(txt, x, y, w, h, Align::Center);
    pop_clip();
}

// The selected flag sets the color of the cell to a grayish color, otherwise white
pub fn draw_data(txt: &str, x: i32, y: i32, w: i32, h: i32, selected: bool) {
    push_clip(x, y, w, h);

    set_draw_color(if selected {
        Color::from_u32(0xD3D3D3)
    } else {
        Color::White
    });
    draw_rectf(x, y, w, h);
    set_draw_color(Color::Gray0);
    draw_text2(txt, x, y, w, h, Align::Center);
    draw_rect(x, y, w, h);
    pop_clip();
}
