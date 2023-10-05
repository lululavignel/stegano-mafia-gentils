fn main() {
    let img = image::open("cc.png").unwrap().to_rgb8();
    let rgb: Vec<u8> = img.into_raw();
    let mut i = 0;
    while i < 800 {
        let binary_value_part1 = &format!("{:08b}", rgb[i])[6..8];
        let binary_value_part2 = &format!("{:08b}", rgb[i+1])[6..8];
        let binary_value_part3 = &format!("{:08b}", rgb[i+2])[6..8];
        let binary_value_part4 = &format!("{:08b}", rgb[i+3])[6..8];
        i = i + 4;
        let binary_value = format!("{}{}{}{}", binary_value_part4, binary_value_part3, binary_value_part2, binary_value_part1);
        println!("{}", binary_value);
        // Convert binary to an integer (u8)
        let integer_value = u8::from_str_radix(&binary_value, 2).unwrap();

        println!("{}", integer_value);

        let ascii_char = char::from(integer_value);
        println!("Binary: {} - ASCII: {}", binary_value, ascii_char);
    }
}
