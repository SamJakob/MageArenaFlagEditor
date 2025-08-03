use crate::error::Error;
use crate::error::Error::IllegalParameter;
use crate::helpers::array_from_slice;
use crate::Error::Unsupported;
use std::iter::repeat_n;

/// The set of supported bitmap type identifiers.
#[derive(Debug)]
pub enum BitmapIdentifier {
    /// Windows 3.x, 95, NT, etc.,
    BM
}

impl BitmapIdentifier {
    pub fn from_bytes(bytes: [u8; 2]) -> Result<Self, Error> {
        match bytes {
            [0x42, 0x4D] => Ok(BitmapIdentifier::BM),
            _ => Err(IllegalParameter("unsupported bitmap identifier"))
        }
    }

    pub fn to_bytes(&self) -> [u8; 2] {
        match self {
            BitmapIdentifier::BM => [0x42, 0x4D]
        }
    }
}

/// The bitmap file header.
#[derive(Debug)]
pub struct Header {
    /// The identifier that indicates the type of BMP file.
    pub identifier: BitmapIdentifier,

    /// The size of the entire bitmap file, in bytes.
    pub size: u32,

    /// Reserved for application - set to 0.
    pub reserved_1: u16,

    /// Reserved for application - set to 0.
    pub reserved_2: u16,

    /// Starting address for the bitmap image data.
    pub offset: u32,
}

impl Header {
    /// The size of the header in bytes.
    pub const SIZE: usize = 14;

    pub fn new(size: u32, offset: u32) -> Self {
        Self {
            identifier: BitmapIdentifier::BM,
            size,
            reserved_1: 0,
            reserved_2: 0,
            offset,
        }
    }

    pub fn new_from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let identifier = BitmapIdentifier::from_bytes(*array_from_slice(&bytes[..2])?)?;
        let size = u32::from_le_bytes(*array_from_slice(&bytes[2..6])?);
        let reserved_1 = u16::from_le_bytes(*array_from_slice(&bytes[6..8])?);
        let reserved_2 = u16::from_le_bytes(*array_from_slice(&bytes[8..10])?);
        let offset = u32::from_le_bytes(*array_from_slice(&bytes[10..14])?);

        Ok(Self {
            identifier,
            size,
            reserved_1,
            reserved_2,
            offset,
        })
    }

    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut bytes = [0u8; Self::SIZE];
        bytes[..2].copy_from_slice(&self.identifier.to_bytes());
        bytes[2..6].copy_from_slice(&self.size.to_le_bytes());
        bytes[6..8].copy_from_slice(&self.reserved_1.to_le_bytes());
        bytes[8..10].copy_from_slice(&self.reserved_2.to_le_bytes());
        bytes[10..14].copy_from_slice(&self.offset.to_le_bytes());
        bytes
    }
}

/// The set of supported bitmap compression methods.
#[derive(Debug)]
pub enum CompressionMethod {
    /// No compression.
    BiRgb,
}

impl CompressionMethod {
    /// Get the 32-bit identifier for the compression method.
    ///
    /// This can be used directly as the [InformationHeader::compression_method].
    pub fn get_identifier(&self) -> u32 {
        match self {
            CompressionMethod::BiRgb => 0
        }
    }

    pub fn from_identifier(identifier: u32) -> Result<Self, Error> {
        match identifier {
            0 => Ok(CompressionMethod::BiRgb),
            _ => Err(IllegalParameter("unknown identifier"))
        }
    }
}

/// The DIB header (bitmap information header).
///
/// Also known as the Windows `BITMAPINFOHEADER`; this is the conventionally supported bitmap
/// DIB header.
///
/// See: https://en.wikipedia.org/wiki/BMP_file_format#DIB_header_(bitmap_information_header)
#[derive(Debug)]
pub struct InformationHeader {
    /// The size of this header in bytes (40 bytes).
    pub size: u32,

    /// The width of the image in bytes.
    pub width: i32,

    /// The height of the image in bytes.
    ///
    /// Negative for top-to-bottom pixel order, positive for bottom-to-top pixel order.
    pub height: i32,

    /// The number of color planes (must be 1).
    pub color_plane_count: u16,

    /// The number of bits per pixel (typically 1, 4, 8, 16, 32).
    pub bits_per_pixel: u16,

    /// The compression method in use.
    pub compression_method: CompressionMethod,

    /// This can be set to 0 for [CompressionMethod::BiRgb].
    pub raw_image_size: u32,

    /// Horizontal resolution of the image in pixels per meter.
    pub horizontal_resolution: i32,

    /// Vertical resolution of the image in pixels per meter.
    pub vertical_resolution: i32,

    /// The number of colors in the color palette, or 0 to default to 2^n.
    pub color_palette_count: u32,

    /// The number of important colors in the color palette, or 0 if all are important.
    ///
    /// Generally ignored.
    pub important_color_count: u32,
}

impl InformationHeader {
    pub const SIZE: usize = 40;

    pub fn new<P: Pixel>(width: i32, height: i32) -> Self {
        Self {
            size: Self::SIZE as u32,
            width,
            height,
            color_plane_count: 1,
            bits_per_pixel: P::bits_per_pixel(),
            compression_method: CompressionMethod::BiRgb,
            raw_image_size: 0,
            horizontal_resolution: P::pixels_per_meter(),
            vertical_resolution: P::pixels_per_meter(),
            color_palette_count: 0,
            important_color_count: 0,
        }
    }

    pub fn new_from_bytes(bytes: &[u8]) -> Result<InformationHeader, Error> {
        let size = u32::from_le_bytes(*array_from_slice(&bytes[0..4])?);
        let width = i32::from_le_bytes(*array_from_slice(&bytes[4..8])?);
        let height = i32::from_le_bytes(*array_from_slice(&bytes[8..12])?);
        let color_plane_count = u16::from_le_bytes(*array_from_slice(&bytes[12..14])?);
        let bits_per_pixel = u16::from_le_bytes(*array_from_slice(&bytes[14..16])?);
        let compression_method = CompressionMethod::from_identifier(u32::from_le_bytes(*array_from_slice(&bytes[16..20])?))?;
        let raw_image_size = u32::from_le_bytes(*array_from_slice(&bytes[20..24])?);
        let horizontal_resolution = i32::from_le_bytes(*array_from_slice(&bytes[24..28])?);
        let vertical_resolution = i32::from_le_bytes(*array_from_slice(&bytes[28..32])?);
        let color_palette_count = u32::from_le_bytes(*array_from_slice(&bytes[32..36])?);
        let important_color_count = u32::from_le_bytes(*array_from_slice(&bytes[36..40])?);

        if size != 40 {
            return Err(IllegalParameter("unexpected bitmap information header size"));
        }

        if bits_per_pixel != 24 {
            return Err(Unsupported("only 24bpp bitmaps are supported"));
        }

        if color_plane_count != 1 {
            return Err(IllegalParameter("color plane count must be 1"));
        }

        Ok(Self {
            size,
            width,
            height,
            color_plane_count,
            bits_per_pixel,
            compression_method,
            raw_image_size,
            horizontal_resolution,
            vertical_resolution,
            color_palette_count,
            important_color_count,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = [0; 40];
        bytes[0..4].copy_from_slice(&self.size.to_le_bytes());
        bytes[4..8].copy_from_slice(&self.width.to_le_bytes());
        bytes[8..12].copy_from_slice(&self.height.to_le_bytes());
        bytes[12..14].copy_from_slice(&self.color_plane_count.to_le_bytes());
        bytes[14..16].copy_from_slice(&self.bits_per_pixel.to_le_bytes());
        bytes[16..20].copy_from_slice(&self.compression_method.get_identifier().to_le_bytes());
        bytes[20..24].copy_from_slice(&self.raw_image_size.to_le_bytes());
        bytes[24..28].copy_from_slice(&self.horizontal_resolution.to_le_bytes());
        bytes[28..32].copy_from_slice(&self.vertical_resolution.to_le_bytes());
        bytes[32..36].copy_from_slice(&self.color_palette_count.to_le_bytes());
        bytes[36..40].copy_from_slice(&self.important_color_count.to_le_bytes());
        bytes.to_vec()
    }
}

pub trait Pixel {
    /// The number of bits used to represent each pixel.
    fn bits_per_pixel() -> u16;

    /// The number of pixels that should be printed per meter when the pixels are physically
    /// printed.
    fn pixels_per_meter() -> i32;

    /// Create a new pixel from the given bytes.
    fn new_from_bytes(bytes: &[u8]) -> Result<Self, Error> where Self: Sized;

    /// Returns true if the pixel represents pure black.
    fn is_black(&self) -> bool;

    /// Returns true if the pixel represents pure white.
    fn is_white(&self) -> bool;

    /// Serialize the pixel to bytes.
    fn to_bytes(&self) -> Vec<u8>;

    /// Get the normalized difference between this value and the other value.
    fn difference(&self, other: &Self) -> f64;
}

#[derive(Copy, Clone, Debug)]
pub struct Pixel24Bit {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Pixel for Pixel24Bit {
    fn bits_per_pixel() -> u16 {
        24
    }

    fn pixels_per_meter() -> i32 {
        2835
    }

    fn new_from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() != 3 {
            return Err(IllegalParameter("expected exactly 3 bytes for a pixel"));
        }

        Ok(Self {
            red: bytes[0],
            green: bytes[1],
            blue: bytes[2],
        })
    }

    fn is_black(&self) -> bool {
        self.red == 0 && self.green == 0 && self.blue == 0
    }

    fn is_white(&self) -> bool {
        self.red == 255 && self.green == 255 && self.blue == 255
    }

    fn to_bytes(&self) -> Vec<u8> {
        [self.red, self.green, self.blue].to_vec()
    }

    fn difference(&self, other: &Self) -> f64 {
        (
            (f64::from(other.red) - f64::from(self.red)).powi(2) +
                (f64::from(other.green) - f64::from(self.green)).powi(2) +
                (f64::from(other.blue) - f64::from(self.blue)).powi(2)
        ).sqrt()
    }
}

/// Represents a bitmap image.
///
/// ## Example
///
/// ```rust
/// use tempfile::NamedTempFile;
/// use std::io::{BufWriter, Write};
/// use std::iter::repeat_n;
/// use bitmap_rs::{hex, Bitmap, Pixel24Bit};
///
/// // Create an image of 100px x 66px where each pixel is the color green.
/// let width = 100;
/// let height = 66;
/// let pixels: Vec<Pixel24Bit> = repeat_n(
///     hex!("#4CAF50").unwrap(),
///     (width * height) as usize
/// ).collect();
///
/// let bitmap = Bitmap::new_from_pixels(width, height, pixels).unwrap();
///
/// // Write the bitmap image data into a temporary file.
/// let mut output_file = NamedTempFile::new().unwrap();
/// let mut output_file_writer = BufWriter::new(output_file);
/// output_file_writer.write_all(&bitmap.to_bytes()).unwrap();
/// output_file_writer.flush().unwrap();
/// ```
#[derive(Debug)]
pub struct Bitmap<P: Pixel> {
    pub header: Header,
    pub information_header: InformationHeader,
    pub pixels: Vec<P>,
}

impl<P: Pixel + std::fmt::Debug> Bitmap<P> {
    /// Construct a new [Bitmap] from the given dimensions and pixel array.
    ///
    /// The height is automatically negated (this means the default for a positive height is that
    /// the pixels are interpreted top-to-bottom).
    pub fn new_from_pixels(width: i32, height: i32, pixels: Vec<P>) -> Result<Self, Error> {
        let unsigned_abs_height = height.unsigned_abs();
        if pixels.len() != (width.unsigned_abs() * unsigned_abs_height) as usize {
            return Err(IllegalParameter("pixel length is not equal to width * height"));
        }

        let information_header = InformationHeader::new::<P>(width, height);
        let headers_size = (Header::SIZE + information_header.size as usize) as u32;

        let (_, padded_bytes_per_image) = Self::compute_padding(pixels.len() as u32, unsigned_abs_height);

        Ok(Self {
            header: Header::new(
                headers_size + padded_bytes_per_image,
                headers_size
            ),
            information_header,
            pixels
        })
    }

    /// Construct a new [Bitmap] from the given bitmap file bytes.
    pub fn new_from_bytes(bytes: Vec<u8>) -> Result<Bitmap<P>, Error> {
        let header = Header::new_from_bytes(&bytes[0..Header::SIZE])?;
        let information_header = InformationHeader::new_from_bytes(&bytes[Header::SIZE..(Header::SIZE + InformationHeader::SIZE)])?;

        let bytes_per_pixel = information_header.bits_per_pixel.div_ceil(8) as usize;
        let pixel_count = information_header.height.unsigned_abs() * information_header.width.unsigned_abs();

        let (padding_bytes_per_row, _) = Self::compute_padding(pixel_count, information_header.height.unsigned_abs());
        let bytes_per_row = information_header.width.unsigned_abs() as usize * bytes_per_pixel;
        let bytes_per_padded_row = bytes_per_row + padding_bytes_per_row as usize;

        let mut pixels = Vec::with_capacity(pixel_count as usize);
        let mut has_bad_pixel = false;

        bytes[(header.offset as usize)..].chunks_exact(bytes_per_padded_row).for_each(|row| {
            row[0..bytes_per_row].chunks_exact(bytes_per_pixel).for_each(|pixel| {
                if let Ok(pixel) = P::new_from_bytes(pixel).map_err(|_| has_bad_pixel = true) {
                    pixels.push(pixel);
                }
            });
        });

        if has_bad_pixel {
            return Err(IllegalParameter("bad pixel data"));
        }

        Ok(Bitmap {
            header,
            information_header,
            pixels
        })
    }

    /// Get the width of the image, in pixels.
    pub fn get_width(&self) -> u32 {
        self.get_raw_width().unsigned_abs()
    }

    /// Get the height of the image, in pixels.
    pub fn get_height(&self) -> u32 {
        self.get_raw_height().unsigned_abs()
    }

    /// Get the raw width of the image.
    pub fn get_raw_width(&self) -> i32 {
        self.information_header.width
    }

    /// Get the raw height of the image.
    ///
    /// Negative means the image is drawn bottom-to-top, positive means the image is drawn
    /// top-to-bottom.
    pub fn get_raw_height(&self) -> i32 {
        self.information_header.height
    }

    /// Get the pixel at the given coordinates.
    pub fn get_pixel_at(&self, x: u32, y: u32) -> Option<&P> {
        let width = self.get_width();
        let height = self.get_height();

        // Return none if the given coordinate is out-of-bounds.
        if x >= width || y >= height {
            return None;
        }

        Some(&self.pixels[((y * width) + x) as usize])
    }

    /// Find the location of the pixel in this bitmap with the closest match to the specified other
    /// pixel.
    pub fn find_pixel_by_closest_match(&self, other: &P) -> Option<(u32, u32)> {
        let width = self.get_width();

        let mut best_match_difference: f64 = f64::INFINITY;
        let mut best_match_location: Option<(u32, u32)> = None;

        for (y, row) in self.pixels.chunks_exact(width as usize).enumerate() {
            for (x, current_pixel) in row.iter().enumerate() {
                let new_difference = current_pixel.difference(other);
                if new_difference < best_match_difference {
                    best_match_difference = new_difference;
                    best_match_location = Some((x as u32, y as u32));
                }
            }
        }

        best_match_location
    }

    fn compute_padding(pixel_count: u32, unsigned_abs_height: u32) -> (u32, u32) {
        // Each row must begin at a memory address that is a multiple of four.
        let bytes_per_image = pixel_count * (P::bits_per_pixel() as u32).div_ceil(8);
        let bytes_per_row = bytes_per_image / unsigned_abs_height;

        // The padding is the amount needed to ensure the number of bytes per row is divisible by 4.
        let row_remainder = bytes_per_row % 4;
        let padding_bytes_per_row = if row_remainder != 0 {
            4 - row_remainder
        } else {
            0
        };

        // Re-compute the row and image with the padding applied.
        let bytes_per_padded_row = bytes_per_row + padding_bytes_per_row;
        let bytes_per_padded_image = bytes_per_padded_row * unsigned_abs_height;
        (padding_bytes_per_row, bytes_per_padded_image)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![0; Header::SIZE];

        // Apply the headers.
        bytes.copy_from_slice(&self.header.to_bytes());
        bytes.append(&mut self.information_header.to_bytes());

        // Compute the padded pixel bytes.
        let (padding_per_row, _) = Self::compute_padding(self.pixels.len() as u32, self.information_header.height.unsigned_abs());

        bytes.append(&mut self.pixels
            .chunks_exact(self.information_header.height.unsigned_abs() as usize)
            .flat_map(|row| {
                let row_bytes: Vec<u8> = row.iter()
                    .flat_map(Pixel::to_bytes)
                    .chain(repeat_n(0u8, padding_per_row as usize))
                    .collect();

                row_bytes
            })
            .collect());

        bytes
    }
}
