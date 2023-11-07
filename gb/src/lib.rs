pub enum Button {
    A,
    B,
    Left,
    Right,
    Up,
    Down,
    Start,
    Select,
}

pub struct GameBoy {
    width: u8,
    height: u8,
}

impl GameBoy {
    pub fn new(_data: Vec<u8>) -> GameBoy {
        GameBoy {
            width: 160,
            height: 144,
        }
    }

    #[inline]
    pub fn width(&self) -> u8 {
        self.width
    }

    #[inline]
    pub fn height(&self) -> u8 {
        self.height
    }

    #[inline]
    pub fn data(&self) -> &[u8] {
        &[]
    }

    pub fn frame(&self) {}

    pub fn keydown(&self, button: Button) {}

    pub fn keyup(&self, button: Button) {}
}
