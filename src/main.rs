mod annalyser;

fn main() {
    let img = image::open("cc.png").unwrap().to_rgb8();
    let rgb: Vec<u8> = img.into_raw();
    let mut i = 0;
    let mut j = 0;

    while j < 12 {
        let part1 = (rgb[j] & 0b11) << 6;
        let part2 = (rgb[j+1] & 0b11) << 4;
        let part3 = (rgb[j+2] & 0b11) << 2;
        let part4 = (rgb[j+3] & 0b11) << 0;
        println!("{:08b}", rgb[j]);
        println!("part1 : {:08b}", part1);
        println!("{:08b}", rgb[j+1]);
        println!("part2 : {:08b}", part2);
        println!("{:08b}", rgb[j+2]);
        println!("part3 : {:08b}", part3);
        println!("{:08b}", rgb[j+3]);
        println!("part4 : {:08b}", part4);
        j = j + 4;
    }
    
    while i < 12 {
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
