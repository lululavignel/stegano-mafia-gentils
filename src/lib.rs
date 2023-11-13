use wasm_bindgen::prelude::*;
use image::DynamicImage;
use base64::{encode, decode};
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

