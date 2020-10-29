// Needed to store cell information during the draw_cell call
#[derive(Default)]
pub struct CellData {
    pub row: i32,
    pub col: i32,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl CellData {
    pub fn select(&mut self, row: i32, col: i32, x: i32, y: i32, w: i32, h: i32) {
        self.row = row;
        self.col = col;
        self.x = x;
        self.y = y;
        self.w = w;
        self.h = h;
    }
}
