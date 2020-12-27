use super::*;
use crate::error;
use palette::{Hsv, IntoColor};

pub fn hex_to_color(s: &str) -> error::Result<Hsv> {
    if s.len() != 7 || s.bytes().next() != Some(b'#') {
        return Err(wrap_err!(format!("s is invalid: {}", s)));
    }

    let rgb = palette::Srgb::new(
        u8::from_str_radix(&s[1..3], 16)? as f32 / 255.0,
        u8::from_str_radix(&s[3..5], 16)? as f32 / 255.0,
        u8::from_str_radix(&s[5..7], 16)? as f32 / 255.0,
    );
    Ok(Hsv::from(rgb))
}

pub fn color_to_hex(color: &Hsv) -> String {
    let rgb = palette::Srgb::from_linear(color.into_rgb());
    format!(
        "#{:02x}{:02x}{:02x}",
        (rgb.red * 255.0).round() as u8,
        (rgb.green * 255.0).round() as u8,
        (rgb.blue * 255.0).round() as u8
    )
}

#[cfg(test)]
mod tests {
    use crate::color::{color_to_hex, hex_to_color};
    use crate::sprite_runtime::*;

    #[test]
    fn test_hex_to_color() {
        struct Test {
            s: &'static str,
            expected: Hsv,
            expect_err: bool,
        }

        let tests = vec![
            Test {
                s: "",
                expected: Hsv::new(0.0, 0.0, 0.0),
                expect_err: true,
            },
            Test {
                s: "#",
                expected: Hsv::new(0.0, 0.0, 0.0),
                expect_err: true,
            },
            Test {
                s: "#000000",
                expected: Hsv::new(0.0, 0.0, 0.0),
                expect_err: false,
            },
            Test {
                s: "#ffffff",
                expected: Hsv::new(0.0, 0.0, 1.0),
                expect_err: false,
            },
            Test {
                s: "#ffffffa",
                expected: Hsv::new(0.0, 0.0, 0.0),
                expect_err: true,
            },
        ];

        for (i, test) in tests.iter().enumerate() {
            let result = hex_to_color(test.s);
            assert_eq!(result.is_err(), test.expect_err, "{}", i);
            if !test.expect_err {
                assert_eq!(result.unwrap(), test.expected, "{}", i);
            }
        }
    }

    #[test]
    fn test_color_to_hex() {
        assert_eq!(color_to_hex(&Hsv::new(0.0, 1.0, 1.0)), "#ff0000");
    }
}
