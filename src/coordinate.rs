#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Coordinate {
    pub x: f64,
    pub y: f64,
}

impl Coordinate {
    pub fn add(&self, other: &Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
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

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Rectangle {
    top_left: SpriteCoordinate,
    size: Size,
}

impl Rectangle {
    pub fn new(top_left: SpriteCoordinate, size: Size) -> Self {
        Self { top_left, size }
    }

    pub fn with_center(center: SpriteCoordinate, size: Size) -> Self {
        Self {
            top_left: center.add(&SpriteCoordinate {
                x: size.width / -2.0,
                y: size.length / -2.0,
            }),
            size,
        }
    }

    pub fn top_left(&self) -> SpriteCoordinate {
        self.top_left
    }

    pub fn center(&self) -> SpriteCoordinate {
        self.top_left.add(&SpriteCoordinate {
            x: self.size.width / 2.0,
            y: self.size.length / 2.0,
        })
    }

    pub fn size(&self) -> Size {
        self.size
    }

    pub fn contains(&self, coordinate: &SpriteCoordinate) -> bool {
        coordinate.x >= self.top_left.x
            && coordinate.y >= self.top_left.y
            && coordinate.x <= self.top_left.x + self.size.width
            && coordinate.y <= self.top_left.y + self.size.length
    }

    pub fn translate(&self, coordinate: &SpriteCoordinate) -> Rectangle {
        Rectangle {
            top_left: self.top_left.add(coordinate),
            size: self.size,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_with_center() {
        assert_eq!(
            Rectangle::with_center(
                SpriteCoordinate { x: 0.0, y: 0.0 },
                Size {
                    width: 0.0,
                    length: 0.0,
                }
            ),
            Rectangle::new(
                SpriteCoordinate { x: 0.0, y: 0.0 },
                Size {
                    width: 0.0,
                    length: 0.0,
                }
            )
        );
        assert_eq!(
            Rectangle::with_center(
                SpriteCoordinate { x: 0.0, y: 0.0 },
                Size {
                    width: 1.0,
                    length: 1.0,
                }
            ),
            Rectangle::new(
                SpriteCoordinate { x: -0.5, y: -0.5 },
                Size {
                    width: 1.0,
                    length: 1.0,
                }
            )
        );
        assert_eq!(
            Rectangle::new(
                SpriteCoordinate { x: -0.5, y: -0.5 },
                Size {
                    width: 1.0,
                    length: 1.0,
                }
            )
            .center(),
            SpriteCoordinate { x: 0.0, y: 0.0 },
        )
    }

    #[test]
    fn test_contains() {
        struct Test {
            rect: Rectangle,
            coordinate: SpriteCoordinate,
            expected: bool,
        }

        let tests = vec![
            // 0
            Test {
                rect: Rectangle::new(
                    SpriteCoordinate { x: 0.0, y: 0.0 },
                    Size {
                        width: 0.0,
                        length: 0.0,
                    },
                ),
                coordinate: SpriteCoordinate { x: 0.0, y: 0.0 },
                expected: true,
            },
            // 1
            Test {
                rect: Rectangle::new(
                    SpriteCoordinate { x: 0.0, y: 0.0 },
                    Size {
                        width: 1.0,
                        length: 1.0,
                    },
                ),
                coordinate: SpriteCoordinate { x: 0.0, y: 0.0 },
                expected: true,
            },
            // 2
            Test {
                rect: Rectangle::new(
                    SpriteCoordinate { x: 0.0, y: 0.0 },
                    Size {
                        width: 1.0,
                        length: 1.0,
                    },
                ),
                coordinate: SpriteCoordinate { x: 1.0, y: 1.0 },
                expected: true,
            },
            // 3
            Test {
                rect: Rectangle::new(
                    SpriteCoordinate { x: 0.0, y: 0.0 },
                    Size {
                        width: 1.0,
                        length: 1.0,
                    },
                ),
                coordinate: SpriteCoordinate { x: 2.0, y: 2.0 },
                expected: false,
            },
            // 4
            Test {
                rect: Rectangle::new(
                    SpriteCoordinate { x: 0.0, y: 0.0 },
                    Size {
                        width: 1.0,
                        length: 1.0,
                    },
                ),
                coordinate: SpriteCoordinate { x: 1.0, y: 0.0 },
                expected: true,
            },
            // 5
            Test {
                rect: Rectangle::new(
                    SpriteCoordinate { x: 1.0, y: 1.0 },
                    Size {
                        width: 1.0,
                        length: 1.0,
                    },
                ),
                coordinate: SpriteCoordinate { x: 1.0, y: 0.0 },
                expected: false,
            },
        ];

        for (i, test) in tests.iter().enumerate() {
            assert_eq!(test.rect.contains(&test.coordinate), test.expected, "{}", i);
        }
    }
}
