use wasm_bindgen::prelude::*;
use base64::{encode, decode};
use image::{DynamicImage, RgbImage};
use std::io::Cursor;
use std::str;

#[wasm_bindgen]
pub fn get_image_width(base64_image: &str) -> u32 {
    let image_bytes = decode(base64_image).unwrap();
    let img = image::load_from_memory(&image_bytes).unwrap();
    img.width()
}

#[wasm_bindgen]
pub fn get_image_height(base64_image: &str) -> u32 {
    let image_bytes = decode(base64_image).unwrap();
    let img = image::load_from_memory(&image_bytes).unwrap();
    img.height()
}

#[wasm_bindgen]
pub fn decrypt_message(base64_image: &str) -> String {
    let image_bytes = base64::decode(base64_image).unwrap();
    let img = image::load_from_memory(&image_bytes).unwrap();
    
    let mut secret_message = String::new();
    
    let rgb = img.to_rgb8().into_raw();
    let mut i = 0;
    
    while i < rgb.len() {
        let binary_value_part1 = &format!("{:08b}", rgb[i])[6..8];
        let binary_value_part2 = &format!("{:08b}", rgb[i + 1])[6..8];
        let binary_value_part3 = &format!("{:08b}", rgb[i + 2])[6..8];
        let binary_value_part4 = &format!("{:08b}", rgb[i + 3])[6..8];
        
        i += 4;
        
        let binary_value = format!("{}{}{}{}", binary_value_part4, binary_value_part3, binary_value_part2, binary_value_part1);
        let integer_value = u8::from_str_radix(&binary_value, 2).unwrap();
        let ascii_char = char::from(integer_value);
        secret_message.push(ascii_char);
    }
    
    secret_message
}


pub trait ImgIterator {
    fn new(width: u32, height: u32) -> Self where Self: Sized;
    fn next(&mut self) -> Option<(u32, u32, usize)>;
}

pub struct DefaultIterator {
    x: u32,
    x_len: u32,
    y: u32,
    y_len: u32,
    rbg: usize,
}

impl ImgIterator for DefaultIterator {
    fn new(width: u32, height: u32) -> Self {
        DefaultIterator {
            x: 0,
            x_len: width,
            y: 0,
            y_len: height,
            rbg: 0,
        }
    }

    fn next(&mut self) -> Option<(u32, u32, usize)> {
        if self.x == self.x_len && self.y == self.y_len && self.rbg == 2 {
            return None;
        }
        self.x = (self.x + 1) % self.x_len;
        self.y = (self.y + 1) % self.y_len;
        self.rbg = (self.rbg + 1) % 3;
        Some((self.x, self.y, self.rbg))
    }
}

pub trait CharEncoding {
    fn get_encoding(&self, char: u8) -> Option<Vec<u8>>;
    fn decode_all(&self, encoded: Vec<u8>) -> Option<Vec<u8>>;
}

#[wasm_bindgen]
pub struct ASCIIEncoding;

impl CharEncoding for ASCIIEncoding {
    fn get_encoding(&self, char: u8) -> Option<Vec<u8>> {
        Some(vec![char])
    }

    fn decode_all(&self, encoded: Vec<u8>) -> Option<Vec<u8>> {
        Some(encoded)
    }
}

#[wasm_bindgen]
pub fn lsb_hide_msg(
    msg: &[u8],
    encoding: &ASCIIEncoding,
    width: u32,
    height: u32,
    output: &mut [u8],
) {
    // Check if the output buffer is large enough
    let required_output_size = width * height * 3; // 3 bytes per pixel (RGB)
    if output.len() < required_output_size as usize {
        // If the output buffer is not large enough, panic or return an error
        panic!("Output buffer is too small!");
    }

    let mut iterator = DefaultIterator::new(width, height);
    let mut cur_char_index = 0;
    let mut encoded_bytes_count = 0;

    for c in msg {
        let encodeds = match encoding.get_encoding(*c) {
            Some(result_encoding) => result_encoding,
            None => continue,
        };

        // We create a vec containing the bits we have to "or" with image's pixels
        let mut bytes_to_hide = Vec::with_capacity(4 * encodeds.len());

        for encoded in &encodeds {
            for i in 0..4 {
                bytes_to_hide.push(((encoded >> (2 * i)) & 0x3) as u8);
            }
        }
        encoded_bytes_count += encodeds.len();

        let mut bits_put = 0;
        for bytes in bytes_to_hide {
            let next_pixel = iterator.next();
            if next_pixel.is_none() {
                // Handle the case where the message doesn't fit inside the image
                panic!("Message doesn't fit inside the image!");
            }
            let (x, y, color) = next_pixel.unwrap();
            let pixel_index = ((y * width + x) * 3 + color as u32) as usize;
            output[pixel_index] &= !0b11; // Clear the 2 least significant bits
            output[pixel_index] |= bytes; // Set the 2 least significant bits to bytes
            bits_put += 2;
        }
        cur_char_index += 1;
    }
}

#[wasm_bindgen]
pub fn lsb_retrieve_msg(
    mut length: i32,
    encoding: &ASCIIEncoding,
    width: u32,
    height: u32,
    input: &[u8],
) -> Vec<u8> {
    let mut iterator = DefaultIterator::new(width, height);
    let mut cur_char: u8 = 0;
    let mut cd = 0;
    let mut encoded = Vec::with_capacity(length as usize);

    for _ in 0..4 * length {
        let (x, y, c) = match iterator.next() {
            None => break,
            Some(t) => t,
        };
        let pixel_index = ((y * width + x) * 3 + c as u32) as usize;
        let val = input[pixel_index] & 0b11;
        cur_char |= (val << (2 * cd));
        cd += 1;
        if cd == 4 {
            encoded.push(cur_char);
            length -= 1;
            if length <= 0 {
                break;
            }
            cd = 0;
            cur_char = 0;
        }
    }

    match encoding.decode_all(encoded) {
        Some(result) => result,
        None => Vec::new(), // Return an empty Vec if decoding fails
    }
}

#[wasm_bindgen]
pub fn encrypt_and_hide_message_in_image(image_path: &str, secret_message: &str) -> String {
    // Load the original image
    let img = image::open(image_path).unwrap();

    // Convert the image to RGB format
    let img_rgb = img.to_rgb8();

    // Encode the message as bytes
    let encoded_message = secret_message.as_bytes();

    // Define the width and height of the image
    let width = img_rgb.width();
    let height = img_rgb.height();

    // Create a buffer to hold the output image data
    let mut output_image_bytes = Vec::new();

    // Define the character encoding method (ASCII encoding in this case)
    let encoding = ASCIIEncoding;

    // Hide the message in the image using LSB steganography
    lsb_hide_msg(encoded_message, &encoding, width, height, &mut output_image_bytes);

    // Convert the output image data to a DynamicImage
    let output_image = DynamicImage::ImageRgb8(RgbImage::from_raw(width, height, output_image_bytes).unwrap());

    // Convert the output image to PNG format and write it to a cursor
    let mut output_cursor = Cursor::new(Vec::new());
    output_image.write_to(&mut output_cursor, image::ImageOutputFormat::Png).unwrap();

    // Retrieve the output image bytes from the cursor
    let output_image_bytes = output_cursor.into_inner();

    // Encode the PNG image bytes as base64
    let encoded_output_image = encode(&output_image_bytes);

    // Return the base64 encoded image as a string
    encoded_output_image
}



#[wasm_bindgen]
pub fn retrieve_message_from_image(encoded_image: &str) -> String {
    // Decode the base64 encoded image bytes
    let decoded_image_bytes = decode(encoded_image).unwrap();

    // Load the image from the decoded bytes
    let img = image::load_from_memory(&decoded_image_bytes).unwrap();

    // Convert the image to RGB format
    let img_rgb = img.to_rgb8();

    // Define the width and height of the image
    let width = img_rgb.width();
    let height = img_rgb.height();

    // Create a buffer to hold the input image data
    let input_image_data = img_rgb.into_raw();

    // Define the character encoding method (ASCII encoding in this case)
    let encoding = ASCIIEncoding;

    // Retrieve the message hidden in the image using LSB steganography
    let decoded_message_bytes = lsb_retrieve_msg(256, &encoding, width, height, &input_image_data);

    // Convert the decoded message bytes to a string
    let decoded_message = str::from_utf8(&decoded_message_bytes).unwrap();

    // Return the decoded message
    decoded_message.to_string()
}