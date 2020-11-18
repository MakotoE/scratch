#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Coordinate {
    pub x: f64,
    pub y: f64,
}

impl Coordinate {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn add(&self, other: &Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }

    pub fn x(&self) -> f64 {
        self.x
    }

    pub fn y(&self) -> f64 {
        self.y
    }
}

/// Center = 0, 0
/// Left = -x, right = +x
/// Top = -y, bottom = +y
pub type SpriteCoordinate = Coordinate;

/// Left = 0, right = +x
/// Top = 0, bottom + y
pub type CanvasCoordinate = Coordinate;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Size {
    pub width: f64,
    pub length: f64,
}

impl Size {
    pub fn new(width: f64, length: f64) -> Self {
        Self { width, length }
    }

    pub fn width(&self) -> f64 {
        self.width
    }

    pub fn length(&self) -> f64 {
        self.length
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Rectangle {
    center: SpriteCoordinate,
    size: Size,
}

impl Rectangle {
    pub fn new(center: SpriteCoordinate, size: Size) -> Self {
        Self { center, size }
    }

    pub fn center(&self) -> SpriteCoordinate {
        self.center
    }

    pub fn size(&self) -> Size {
        self.size
    }

    pub fn contains(&self, coordinate: &SpriteCoordinate) -> bool {
        let top_left = self.center.add(&SpriteCoordinate::new(
            self.size.width() / -2.0,
            self.size.length() / -2.0,
        ));
        coordinate.x >= top_left.x
            && coordinate.y >= top_left.y
            && coordinate.x <= top_left.x + self.size.width()
            && coordinate.y <= top_left.y + self.size.length()
    }

    pub fn translate(&self, coordinate: &SpriteCoordinate) -> Rectangle {
        Rectangle {
            center: self.center.add(coordinate),
            size: self.size,
        }
    }
}
