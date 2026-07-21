//! Shared 2D math: vectors and facing.

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn add(self, o: Self) -> Self {
        Self {
            x: self.x + o.x,
            y: self.y + o.y,
        }
    }

    pub fn sub(self, o: Self) -> Self {
        Self {
            x: self.x - o.x,
            y: self.y - o.y,
        }
    }

    pub fn scale(self, s: f32) -> Self {
        Self {
            x: self.x * s,
            y: self.y * s,
        }
    }

    pub fn len(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn len_sq(self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    pub fn normalize_or_zero(self) -> Self {
        let l = self.len();
        if l < 1e-6 {
            Self::ZERO
        } else {
            self.scale(1.0 / l)
        }
    }

    pub fn dot(self, o: Self) -> f32 {
        self.x * o.x + self.y * o.y
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Dir4 {
    Up,
    #[default]
    Down,
    Left,
    Right,
}

impl Dir4 {
    pub fn from_vec(v: Vec2, prev: Self) -> Self {
        if v.x.abs() < 1e-6 && v.y.abs() < 1e-6 {
            return prev;
        }
        if v.x.abs() >= v.y.abs() {
            if v.x >= 0.0 {
                Self::Right
            } else {
                Self::Left
            }
        } else if v.y >= 0.0 {
            Self::Down
        } else {
            Self::Up
        }
    }

    pub fn unit(self) -> Vec2 {
        match self {
            Self::Up => Vec2::new(0.0, -1.0),
            Self::Down => Vec2::new(0.0, 1.0),
            Self::Left => Vec2::new(-1.0, 0.0),
            Self::Right => Vec2::new(1.0, 0.0),
        }
    }
}
