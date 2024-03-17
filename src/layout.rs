use std::cmp;

use crate::vec2::Vec2;

pub enum LayoutKind {
    Vert,
    Horz,
}

pub struct Layout {
    pub kind: LayoutKind,
    pub pos: Vec2,
    pub size: Vec2,
}

impl Layout {
    pub fn available_pos(&self) -> Vec2 {
        use LayoutKind::*;
        match self.kind {
            Horz => self.pos + self.size * Vec2::new(1, 0),
            Vert => self.pos + self.size * Vec2::new(0, 1),
        }
    }

    pub fn add_widget(&mut self, size: Vec2) {
        use LayoutKind::*;
        match self.kind {
            Horz => {
                self.size.x += size.x;
                self.size.y = cmp::max(self.size.y, size.y);
            }
            Vert => {
                self.size.x = cmp::max(self.size.x, size.x);
                self.size.y += size.y;
            }
        }
    }
}
