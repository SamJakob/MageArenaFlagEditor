use crate::error::Error;
use crate::error::Error::{AccessFailure, External, UnexpectedValue};
use bitmap_rs::{Bitmap, Pixel24Bit};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::PathBuf;
use windows_registry::{Key, Value, CURRENT_USER};

/// The key for Mage Arena's registry data in the [Hive::CurrentUser] registry.
pub const MAGE_ARENA_KEY: &str = r"Software\jrsjams\MageArena";

/// The key for the flag, relative to [MAGE_ARENA_KEY] in the [Hive::CurrentUser] registry.
pub const MAGE_ARENA_FLAG_KEY_PREFIX: &str = r"flagGrid_";

/// The width of the flag in pixels.
pub const MAGE_ARENA_FLAG_WIDTH: i32 = 100;

/// The height of the flag in pixels.
pub const MAGE_ARENA_FLAG_HEIGHT: i32 = 66;

/// The number of bytes used to represent a pixel.
const MAGE_ARENA_FLAG_PIXEL_SIZE: usize = 10;

/// Locate the user-specific flag grid key under the Mage Arena settings key.
///
/// This function indexes [`COMPUTER\HKEY_CURRENT_USER\{MAGE_ARENA_KEY}`](MAGE_ARENA_KEY) for keys
/// that start with [MAGE_ARENA_FLAG_KEY_PREFIX], returning the full name of the flag key if it is
/// found, or an error if it is not.
fn locate_flag_grid_key(mage_arena_key: &Key) -> Result<String, Error> {
    mage_arena_key.values()
        .map_err(|err| AccessFailure(format!(r"failed to index the subkeys of COMPUTER\HKEY_CURRENT_USER\{MAGE_ARENA_KEY} in the registry: {err}")))?
        .find_map(|(key, _)| if key.starts_with(MAGE_ARENA_FLAG_KEY_PREFIX) { Some(key) } else { None })
        .ok_or_else(|| AccessFailure(format!("failed to find flag grid key (expected registry key with prefix {MAGE_ARENA_FLAG_KEY_PREFIX})")))
}

/// Read the flag from the registry.
fn read_raw_flag_data() -> Result<Vec<u8>, Error> {
    let mage_arena_key = CURRENT_USER.open(MAGE_ARENA_KEY)
        .map_err(|_| AccessFailure(format!(r"could not access the COMPUTER\HKEY_CURRENT_USER\{MAGE_ARENA_KEY} registry key")))?;

    let flag_key = mage_arena_key.get_value(locate_flag_grid_key(&mage_arena_key)?)
        .map_err(|_| AccessFailure("could not access MageArena flag registry key".to_string()))?;

    Ok(flag_key.to_vec())
}

/// Write the flag to the registry.
fn write_raw_flag_data(data: &[u8]) -> Result<(), Error> {
    let mage_arena_key = CURRENT_USER.create(MAGE_ARENA_KEY)
        .map_err(|_| AccessFailure(format!(r"could not access the COMPUTER\HKEY_CURRENT_USER\{MAGE_ARENA_KEY} registry key")))?;

    mage_arena_key.set_value(locate_flag_grid_key(&mage_arena_key)?, &Value::from(data))
        .map_err(|_| AccessFailure("could not access MageArena flag registry key".to_string()))
}

fn read_bitmap_file(bitmap_file: &PathBuf) -> Result<Bitmap<Pixel24Bit>, Error> {
    let mut reader = BufReader::new(File::open(bitmap_file)
        .map_err(|err| AccessFailure(format!("failed to access bitmap file: {err}")))?);

    let mut file_data: Vec<u8> = vec![];
    reader.read_to_end(&mut file_data)
        .map_err(|err| AccessFailure(format!("failed to read bitmap file: {err}")))?;

    Bitmap::new_from_bytes(file_data)
        .map_err(|err| External(format!("failed to parse bitmap data in palette file: {err}")))
}

pub fn read_flag(palette_file: PathBuf, output_file: PathBuf) -> Result<(), Error> {
    let palette = read_bitmap_file(&palette_file)?;

    let raw_data = read_raw_flag_data()?;
    if raw_data.is_empty() {
        return Err(UnexpectedValue("flag data is missing".to_string()));
    }

    // Split the raw data into chunks.
    let (raw_pixels, []) = raw_data.as_chunks::<MAGE_ARENA_FLAG_PIXEL_SIZE>() else {
        return Err(UnexpectedValue(format!("raw flag data length is not divisible by the pixel size ({MAGE_ARENA_FLAG_PIXEL_SIZE})")));
    };

    // Perform a matrix transposition on the pixels - as the registry values are column-ordered
    // while bitmap images are row-ordered.
    let pixels: Vec<[u8; 10]> = (0..MAGE_ARENA_FLAG_HEIGHT as usize)
        .flat_map(|i| {
            (0..MAGE_ARENA_FLAG_WIDTH as usize).map(move |j| {
                raw_pixels[j * MAGE_ARENA_FLAG_HEIGHT as usize + i]
            })
        }).collect();

    // Ensure that all chunks have a comma as the last byte (except the last chunk, which must have
    // null).
    let mut bad_pixels: Vec<Error> = vec![];
    let pixels: Vec<Pixel24Bit> = pixels.iter()
        .enumerate()
        .map(|(i, pixel)| {
            // Ensure pixel data ends with an ASCII comma (or a null if it's the last pixel).
            let is_last_pixel = i == pixels.len() - 1;

            let expected_last_char = if is_last_pixel { 0 } else { 0x2C };
            let actual_last_char = pixel[9];

            if actual_last_char != expected_last_char {
                return Err(UnexpectedValue(format!("pixel {i} contains an invalid last character (expected: {expected_last_char}, got: {actual_last_char})")))
            }

            let Some(divider) = pixel.iter().position(|&b| b == 0x3A) else {
                return Err(UnexpectedValue(format!("pixel {i} is missing the expected divider character (:)")))
            };

            let x_str = String::from_utf8(pixel[0..divider].to_vec())
                .map_err(|err| UnexpectedValue(format!("pixel {i}'s x-coordinate was not valid UTF-8: {err}")))?;
            let x = x_str.parse::<f64>()
                .map_err(|err| UnexpectedValue(format!("pixel {i}'s x-coordinate ({x_str}) was not a valid float: {err}")))?;
            let x = if x > 1.0 { x / 100.0 } else { x };
            let x_coordinate = (x * f64::from(palette.get_width())) as u32;

            let y_str = String::from_utf8(pixel[divider+1..9].to_vec())
                .map_err(|err| UnexpectedValue(format!("pixel {i}'s y-coordinate was not valid UTF-8: {err}")))?;
            let y = y_str.parse::<f64>()
                .map_err(|err| UnexpectedValue(format!("pixel {i}'s y-coordinate ({y_str}) was not a valid float: {err}")))?;
            let y = if y > 1.0 { y / 100.0 } else { y };
            let y_coordinate = (y * f64::from(palette.get_height())) as u32;

            let Some(palette_pixel) = palette.get_pixel_at(x_coordinate, y_coordinate) else {
                return Err(UnexpectedValue(format!("failed to resolve palette pixel ({x_coordinate}, {y_coordinate}) for pixel {i}")));
            };

            Ok(*palette_pixel)
        })
        .filter_map(|pixel| pixel.map_err(|e| bad_pixels.push(e)).ok())
        .collect();

    if !bad_pixels.is_empty() {
        return Err(UnexpectedValue(format!(
            "bad pixels\n\n{}",
            bad_pixels.iter().map(|err| err.to_string()).collect::<Vec<String>>().join("\n")
        )));
    }

    let width = MAGE_ARENA_FLAG_WIDTH;
    let height = MAGE_ARENA_FLAG_HEIGHT;
    let bitmap = Bitmap::new_from_pixels(width, height, pixels)
        .map_err(|err| External(format!("failed to create bitmap image: {err}")))?;

    let mut output_file_writer = BufWriter::new(File::create(&output_file)
        .map_err(|err| AccessFailure(format!("could not create or access the requested output file: {err}")))?);

    output_file_writer.write_all(&bitmap.to_bytes())
        .map_err(|err| AccessFailure(format!("failed to write bytes to file: {err}")))?;

    output_file_writer.flush()
        .map_err(|err| AccessFailure(format!("failed to flush output file: {err}")))?;

    Ok(())
}

pub fn write_flag(palette_file: PathBuf, input_file: PathBuf) -> Result<(), Error> {
    let palette = read_bitmap_file(&palette_file)?;
    let flag = read_bitmap_file(&input_file)?;

    let palette_width = f64::from(palette.get_width());
    let palette_height = f64::from(palette.get_height());
    let pixel_count = flag.pixels.len();

    // Perform a matrix transposition on the pixels - as the registry values are column-ordered
    // while bitmap images are row-ordered.
    let pixels: Vec<Pixel24Bit> = (0..MAGE_ARENA_FLAG_WIDTH as usize)
        .flat_map(|i| {
            (0..MAGE_ARENA_FLAG_HEIGHT as usize).map(move |j| {
                j * MAGE_ARENA_FLAG_WIDTH as usize + i
            })
        }).map(|index| flag.pixels[index]).collect();

    let mut bad_pixels: Vec<Error> = vec![];
    let pixels: Vec<String> = pixels.iter()
        .map(|pixel| {
            let Some(closest_pixel) = palette.find_pixel_by_closest_match(pixel) else {
                return Err(UnexpectedValue("failed to find match for pixel".to_string()));
            };

            Ok(closest_pixel)
        })
        .filter_map(|pixel| pixel.map_err(|err| bad_pixels.push(err)).ok())
        .enumerate()
        .map(|(i, (x, y))| {
            let trailing_character = if i == pixel_count - 1 {
                '\0'
            } else {
                ','
            };

            format!("{:.2}:{:.2}{}", f64::from(x) / palette_width, f64::from(y) / palette_height, trailing_character)
        })
        .collect();

    if !bad_pixels.is_empty() {
        return Err(UnexpectedValue(format!(
            "error mapping pixels\n\n{}",
            bad_pixels.iter().map(|err| err.to_string()).collect::<Vec<String>>().join("\n")
        )));
    }

    write_raw_flag_data(pixels.join("").as_bytes())
}
