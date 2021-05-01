use graphics::types::Rectangle;

/// Center = 0, 0
/// Left = -240, right = +240
/// Top = +180, bottom = -180
#[derive(Copy, Clone, Debug, Default)]
pub struct SpriteCoordinate {
    pub x: f64,
    pub y: f64,
}

impl SpriteCoordinate {
    pub fn add(&self, other: &Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }

    pub fn apply_vector(&self, direction: f64, magnitude: f64) -> Self {
        Self {
            x: self.x + direction.to_radians().sin() * magnitude,
            y: self.y + direction.to_radians().cos() * magnitude,
        }
    }
}

impl PartialEq for SpriteCoordinate {
    fn eq(&self, other: &Self) -> bool {
        (self.x - other.x).abs() < f64::EPSILON && (self.y - other.y).abs() < f64::EPSILON
    }
}

impl From<CanvasCoordinate> for SpriteCoordinate {
    fn from(c: CanvasCoordinate) -> Self {
        Self {
            x: c.x - canvas_const::X_MAX / 2.0,
            y: -c.y + canvas_const::Y_MAX / 2.0,
        }
    }
}

/// Left = 0, right = +480
/// Top = 0, bottom +360
#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub struct CanvasCoordinate {
    pub x: f64,
    pub y: f64,
}

impl From<SpriteCoordinate> for CanvasCoordinate {
    fn from(sprite_coordinate: SpriteCoordinate) -> Self {
        Self {
            x: canvas_const::X_MAX / 2.0 + sprite_coordinate.x,
            y: canvas_const::Y_MAX / 2.0 - sprite_coordinate.y,
        }
    }
}

pub mod canvas_const {
    /// Right edge
    pub const X_MAX: f64 = 480.0;
    /// Bottom edge
    pub const Y_MAX: f64 = 360.0;
}

#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}

impl Size {
    pub fn multiply(&self, scale: &Scale) -> Self {
        Self {
            width: self.width * scale.x,
            height: self.height * scale.y,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Scale {
    pub x: f64,
    pub y: f64,
}

impl Default for Scale {
    fn default() -> Self {
        Self { x: 1.0, y: 1.0 }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub struct SpriteRectangle {
    pub center: SpriteCoordinate,
    pub size: Size,
}

impl SpriteRectangle {
    pub fn contains(&self, coordinate: &SpriteCoordinate) -> bool {
        let top_left = self.top_left();
        coordinate.x >= top_left.x
            && coordinate.y <= top_left.y
            && coordinate.x <= top_left.x + self.size.width
            && coordinate.y >= top_left.y - self.size.height
    }

    fn top_left(&self) -> SpriteCoordinate {
        self.center.add(&SpriteCoordinate {
            x: self.size.width / -2.0,
            y: self.size.height / 2.0,
        })
    }

    fn bottom_right(&self) -> SpriteCoordinate {
        self.center.add(&SpriteCoordinate {
            x: self.size.width / 2.0,
            y: self.size.height / -2.0,
        })
    }

    pub fn intersects(&self, other: &SpriteRectangle) -> bool {
        let self_top_left = self.top_left();
        let self_bottom_right = self.bottom_right();
        let other_top_left = other.top_left();
        let other_bottom_right = other.bottom_right();
        !(self_top_left.x > other_bottom_right.x
            || self_bottom_right.x < other_top_left.x
            || self_top_left.y < other_bottom_right.y
            || self_bottom_right.y > other_top_left.y)
    }
}

#[allow(clippy::from_over_into)]
impl Into<Rectangle> for SpriteRectangle {
    fn into(self) -> Rectangle {
        let top_left: CanvasCoordinate = self.top_left().into();
        [top_left.x, top_left.y, self.size.width, self.size.height]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod sprite_coordinate {
        use super::*;
        

        #[test]
        #[allow(clippy::eq_op)]
        fn partial_eq() {
            assert_eq!(SpriteCoordinate::default(), SpriteCoordinate::default());
            assert_ne!(
                SpriteCoordinate {
                    x: f64::NAN,
                    y: f64::NAN
                },
                SpriteCoordinate {
                    x: f64::NAN,
                    y: f64::NAN
                }
            );
        }

        #[test]
        fn apply_vector() {
            assert_eq!(
                SpriteCoordinate::default().apply_vector(0.0, 0.0),
                SpriteCoordinate { x: 0.0, y: 0.0 }
            );
            assert_eq!(
                SpriteCoordinate::default().apply_vector(0.0, 1.0),
                SpriteCoordinate { x: 0.0, y: 1.0 }
            );
            assert_eq!(
                SpriteCoordinate::default().apply_vector(45.0, 1.0),
                SpriteCoordinate {
                    x: f64::sqrt(2.0) / 2.0,
                    y: f64::sqrt(2.0) / 2.0
                }
            );
            assert_eq!(
                SpriteCoordinate::default().apply_vector(90.0, 1.0),
                SpriteCoordinate { x: 1.0, y: 0.0 }
            );
            assert_eq!(
                SpriteCoordinate::default().apply_vector(180.0, 1.0),
                SpriteCoordinate { x: 0.0, y: -1.0 }
            );
            assert_eq!(
                SpriteCoordinate::default().apply_vector(270.0, 1.0),
                SpriteCoordinate { x: -1.0, y: 0.0 }
            );
        }
    }

    mod canvas_coordinate {
        use super::*;

        #[test]
        fn conversion() {
            assert_eq!(
                CanvasCoordinate::from(SpriteCoordinate { x: 0.0, y: 0.0 }),
                CanvasCoordinate { x: 240.0, y: 180.0 }
            );
            assert_eq!(
                SpriteCoordinate::from(CanvasCoordinate { x: 240.0, y: 180.0 }),
                SpriteCoordinate { x: 0.0, y: 0.0 }
            );
            assert_eq!(
                CanvasCoordinate::from(SpriteCoordinate {
                    x: -240.0,
                    y: 180.0
                }),
                CanvasCoordinate { x: 0.0, y: 0.0 }
            );
            assert_eq!(
                SpriteCoordinate::from(CanvasCoordinate { x: 0.0, y: 0.0 }),
                SpriteCoordinate {
                    x: -240.0,
                    y: 180.0,
                }
            );
        }
    }

    mod sprite_rectangle {
        use super::*;

        #[test]
        fn test_contains() {
            struct Test {
                rect: SpriteRectangle,
                coordinate: SpriteCoordinate,
                expected: bool,
            }

            let tests = vec![
                Test {
                    rect: SpriteRectangle {
                        center: SpriteCoordinate { x: 0.0, y: 0.0 },
                        size: Size {
                            width: 0.0,
                            height: 0.0,
                        },
                    },
                    coordinate: SpriteCoordinate { x: 0.0, y: 0.0 },
                    expected: true,
                },
                Test {
                    rect: SpriteRectangle {
                        center: SpriteCoordinate { x: 0.0, y: 0.0 },
                        size: Size {
                            width: 1.0,
                            height: 1.0,
                        },
                    },
                    coordinate: SpriteCoordinate { x: 0.0, y: 0.0 },
                    expected: true,
                },
                Test {
                    rect: SpriteRectangle {
                        center: SpriteCoordinate { x: 0.0, y: 0.0 },
                        size: Size {
                            width: 2.0,
                            height: 2.0,
                        },
                    },
                    coordinate: SpriteCoordinate { x: 1.0, y: 1.0 },
                    expected: true,
                },
                Test {
                    rect: SpriteRectangle {
                        center: SpriteCoordinate { x: 0.0, y: 0.0 },
                        size: Size {
                            width: 1.0,
                            height: 1.0,
                        },
                    },
                    coordinate: SpriteCoordinate { x: 1.0, y: 1.0 },
                    expected: false,
                },
                Test {
                    rect: SpriteRectangle {
                        center: SpriteCoordinate { x: 0.0, y: 0.0 },
                        size: Size {
                            width: 1.0,
                            height: 1.0,
                        },
                    },
                    coordinate: SpriteCoordinate { x: -1.0, y: -1.0 },
                    expected: false,
                },
                Test {
                    rect: SpriteRectangle {
                        center: SpriteCoordinate { x: 0.0, y: 0.0 },
                        size: Size {
                            width: 1.0,
                            height: 1.0,
                        },
                    },
                    coordinate: SpriteCoordinate { x: -2.0, y: 0.0 },
                    expected: false,
                },
                Test {
                    rect: SpriteRectangle {
                        center: SpriteCoordinate { x: 1.0, y: 1.0 },
                        size: Size {
                            width: 1.0,
                            height: 1.0,
                        },
                    },
                    coordinate: SpriteCoordinate { x: 1.0, y: 0.0 },
                    expected: false,
                },
                Test {
                    rect: SpriteRectangle {
                        center: SpriteCoordinate { x: 0.0, y: 0.0 },
                        size: Size {
                            width: 1.0,
                            height: 1.0,
                        },
                    },
                    coordinate: SpriteCoordinate { x: 1.0, y: 2.0 },
                    expected: false,
                },
                Test {
                    rect: SpriteRectangle {
                        center: SpriteCoordinate { x: -79.0, y: -41.0 },
                        size: Size {
                            width: 127.0,
                            height: 108.0,
                        },
                    },
                    coordinate: SpriteCoordinate {
                        x: -123.0,
                        y: -72.0,
                    },
                    expected: true,
                },
            ];

            for (i, test) in tests.iter().enumerate() {
                assert_eq!(test.rect.contains(&test.coordinate), test.expected, "{}", i);
            }
        }

        #[test]
        fn intersects() {
            struct Test {
                a: SpriteRectangle,
                b: SpriteRectangle,
                expected: bool,
            }

            let tests = vec![
                Test {
                    a: SpriteRectangle {
                        center: SpriteCoordinate { x: 0.0, y: 0.0 },
                        size: Size {
                            width: 0.0,
                            height: 0.0,
                        },
                    },
                    b: SpriteRectangle {
                        center: SpriteCoordinate { x: 0.0, y: 0.0 },
                        size: Size {
                            width: 0.0,
                            height: 0.0,
                        },
                    },
                    expected: true,
                },
                Test {
                    a: SpriteRectangle {
                        center: SpriteCoordinate { x: 1.0, y: 1.0 },
                        size: Size {
                            width: 2.0,
                            height: 2.0,
                        },
                    },
                    b: SpriteRectangle {
                        center: SpriteCoordinate { x: 1.0, y: 1.0 },
                        size: Size {
                            width: 2.0,
                            height: 2.0,
                        },
                    },
                    expected: true,
                },
                Test {
                    a: SpriteRectangle {
                        center: SpriteCoordinate { x: 1.0, y: 1.0 },
                        size: Size {
                            width: 2.0,
                            height: 2.0,
                        },
                    },
                    b: SpriteRectangle {
                        center: SpriteCoordinate { x: 2.0, y: 2.0 },
                        size: Size {
                            width: 2.0,
                            height: 2.0,
                        },
                    },
                    expected: true,
                },
                Test {
                    a: SpriteRectangle {
                        center: SpriteCoordinate { x: 1.0, y: 1.0 },
                        size: Size {
                            width: 4.0,
                            height: 4.0,
                        },
                    },
                    b: SpriteRectangle {
                        center: SpriteCoordinate { x: 2.0, y: 2.0 },
                        size: Size {
                            width: 4.0,
                            height: 4.0,
                        },
                    },
                    expected: true,
                },
            ];

            for (i, test) in tests.iter().enumerate() {
                assert_eq!(test.a.intersects(&test.b), test.expected, "{}", i);
            }
        }
    }
}
