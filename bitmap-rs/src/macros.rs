use crate::{Error, Pixel24Bit};
use crate::Error::IllegalParameter;

#[macro_export]
macro_rules! rgb {
    ($r:expr, $g:expr, $b:expr) => { bitmap_rs::Pixel24Bit { red: $r, green: $g, blue: $b } };
}

const fn hex_digit_to_u8(digit: u8) -> u8 {
    match digit {
        0x30 => 0,
        0x31 => 1,
        0x32 => 2,
        0x33 => 3,
        0x34 => 4,
        0x35 => 5,
        0x36 => 6,
        0x37 => 7,
        0x38 => 8,
        0x39 => 9,
        0x41 | 0x61 => 10,
        0x42 | 0x62 => 11,
        0x43 | 0x63 => 12,
        0x44 | 0x64 => 13,
        0x45 | 0x65 => 14,
        0x46 | 0x66 => 15,
        _ => panic!("invalid hex digit")
    }
}

const fn hex_digits_to_u8(msn: u8, lsn: u8) -> u8 {
    hex_digit_to_u8(msn) << 4 | hex_digit_to_u8(lsn)
}

pub const fn hex_to_rgb(hex: &str) -> Result<Pixel24Bit, Error> {
    let as_bytes = hex.as_bytes();

    if as_bytes[0] != 0x23 || hex.len() != 7 {
        return Err(IllegalParameter("expected '#AAAAAA' where A is a hexadecimal digit."));
    }

    Ok(Pixel24Bit {
        red: hex_digits_to_u8(as_bytes[1], as_bytes[2]),
        green: hex_digits_to_u8(as_bytes[3], as_bytes[4]),
        blue: hex_digits_to_u8(as_bytes[5], as_bytes[6]),
    })
}

#[macro_export]
macro_rules! hex {
    ($hex:expr) => { const { bitmap_rs::hex_to_rgb($hex) } };
}

/// Convert hue, saturation and value to red, green and blue.
///
/// This function will return an error if the hue, saturation or value are outside of the permitted
/// domain.
///
/// - `hue`: 0.0 <= hue < 1.0
/// - `saturation`: 0.0 <= saturation <= 1.0
/// - `value`: 0.0 <= value <= 1.0
///
/// The formula used is from rapidtables.com:
/// https://www.rapidtables.com/convert/color/hsv-to-rgb.html
pub const fn hsv_to_rgb(hue: f64, saturation: f64, value: f64) -> Result<Pixel24Bit, Error> {
    if hue < 0.0 || hue >= 1.0 { return Err(IllegalParameter("hue must be in the range of [0.0, 1.0)")) }
    if saturation < 0.0 || saturation > 1.0 { return Err(IllegalParameter("saturation must be in range of [0.0, 1.0]")) }
    if value < 0.0 || value > 1.0 { return Err(IllegalParameter("value must be in range of [0.0, 1.0]")) }

    // Normalize the hue to 360 degrees.
    let hue = hue * 360f64;

    let c = value * saturation;
    let x = c * (1f64 - ((((hue / 60f64) as i8) % 2) - 1).abs() as f64);
    let m = value - c;

    let (r, g, b) = match hue {
        hue if hue >= 0.0 && hue < 60.0 => (c, x, 0.0),
        hue if hue >= 60.0 && hue < 120.0 => (x, c, 0.0),
        hue if hue >= 120.0 && hue < 180.0 => (0.0, c, x),
        hue if hue >= 180.0 && hue < 240.0 => (0.0, x, c),
        hue if hue >= 240.0 && hue < 300.0 => (x, 0.0, c),
        hue if hue >= 300.0 && hue < 360.0 => (c, 0.0, x),
        _ => { return Err(IllegalParameter("hue exceeded range [0, 360)")); }
    };

    Ok(Pixel24Bit {
        red: ((r + m) * 255.0) as u8,
        green: ((g + m) * 255.0) as u8,
        blue: ((b + m) * 255.0) as u8,
    })
}

#[macro_export]
macro_rules! hsv {
    ($hue:expr, $saturation:expr, $value:expr) => { const { bitmap_rs::hsv_to_rgb($hue, $saturation, $value) } }
}
