// Declare the module
mod lsb;

// Re-export the functions if you want to make them available from the root of the crate
pub use lsb::{ASCIIEncoding, CharEncoding, DefaultIterator, lsb_hide_msg, lsb_retrieve_msg};

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

#[wasm_bindgen]
pub enum SteganographyMethod {
    LSB,
    MatrixEmbedding,
    FourierTransform,
}

#[wasm_bindgen]
pub fn hide_message_image(base64_image: &str, method: SteganographyMethod, message: &str) -> Result<String, JsValue> {
    // Decode the base64 image
    let decoded_image = decode(base64_image).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let mut image = image::load_from_memory(&decoded_image).map_err(|e| JsValue::from_str(&e.to_string()))?.to_rgb8();

    match method {
        SteganographyMethod::LSB => {
            let msg_bytes = message.as_bytes().to_vec();
            let encoding = ASCIIEncoding;
            lsb_hide_msg::<DefaultIterator>(&msg_bytes, &encoding, None, &mut image);
        }
        SteganographyMethod::MatrixEmbedding => {
            unimplemented!("Matrix Embedding method is not implemented yet");
        }
        SteganographyMethod::FourierTransform => {
            unimplemented!("Fourier Transform method is not implemented yet");
        }
    };

    // Encode the image back to base64
    let mut buffer = Cursor::new(Vec::new());
    DynamicImage::ImageRgb8(image).write_to(&mut buffer, image::ImageOutputFormat::Png).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let encoded_base64 = base64::encode(buffer.get_ref());

    Ok(encoded_base64)
}
