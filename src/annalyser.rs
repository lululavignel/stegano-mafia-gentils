

use image::{RgbImage, ImageBuffer, Rgb};
#[allow(dead_code)]
pub fn stats_last_bits(img : & RgbImage,bytes: i32)->Vec<u32>{
    let array_size= i32::pow(2,bytes as u32) as usize;
    let mut bits_count= vec![0;array_size];
    let mut mask :u8 =0;
    for _ in 0..bytes{
        mask=(mask<<1)|1;
    } 
    for (_,_,pixel) in img.enumerate_pixels(){
        for val in pixel.0{
            bits_count[(val&mask)as usize]+=1
        }
    }
    return bits_count;

}
#[allow(dead_code)]
pub fn diff_img_create(img : & RgbImage, bytes : i32)-> RgbImage{
    if (bytes<0) & (bytes>=8){
        panic!("bytes must be positive and strictly inferior to 9");
    }
    let mut msk = 0x1;
    for _ in 1..bytes{
        msk|=msk<<1;
    }
    let shift = (8-bytes) as u8;
    let (x,y)=img.dimensions();
    let mut new_img: RgbImage = ImageBuffer::new(x-1, y-1);
    for i in 1..x-1{
        for j in 1..y-1{
            
            let img_cur_pixel=img.get_pixel(i, j);
            let mut diffs :[[Rgb<u8>;2];2]=[[Rgb::<u8>{0:[0,0,0]};2];2];
            for di in 0..2{
                for dj in 0..2 {
                    if di!=0 &&dj!=0{
                        diffs[di][dj]=distance(
                            img_cur_pixel,img.get_pixel(i-(di as u32),j-(dj as u32)),msk,shift );
                    }
                }
            }
            new_img.put_pixel(i, j, diffs[1][1]);

            //let curs_pixel= ((-1..1),(1..1)).map(dx,dy).m;

        }
    }
    return new_img;
}
#[allow(dead_code)]
fn distance(p1: &Rgb<u8>, p2: &Rgb<u8>,msk:u8,shift:u8)-> Rgb<u8>{
    let mut difference;
    let mut rgb=Rgb::<u8>{0:[0;3]};
    for i in 0..3{
        let cur_color_diff=(p1.0[i] as i32)-(p2.0[i] as i32);
        difference= cur_color_diff*cur_color_diff;
        if difference > 0xFF{
            difference=0xFF;
        }
        difference&=msk as i32;
        rgb.0[i]=(difference as u8)<<shift;
        
    }
    return rgb;
}
#[allow(dead_code)]
fn only_lsb(img: &RgbImage) -> RgbImage{
    let (x,y)=img.dimensions();
    let mut new_img: RgbImage = ImageBuffer::new(x, y);
    for (x,y,pixel) in img.enumerate_pixels(){
        let new_pixel=new_img.get_pixel_mut(x, y);
        new_pixel.0[0]=(pixel.0[0]&0x7)<<5;
        new_pixel.0[1]=(pixel.0[1]&0x7)<<5;
        new_pixel.0[2]=(pixel.0[2]&0x7)<<5;
        
    }
    return new_img;
}
#[cfg(test)]
mod test_annalyzer{
    

    use std::{fs::File, io::Read, path::Path};

    use crate::{lsb::{ASCIIEncoding, CharEncoding}, position_by_hash::HashIterator};

    use crate::lsb;

    use super::*;
    use image::io::Reader as ImageReader;
    use lsb::lsb_hide_msg;
    #[test]
    fn test_key_io(){
        let image1 = ImageReader::open("./unit_tests/in/annalyser/celeste-3-0.1-sha2-n-c.png").unwrap().decode().unwrap().to_rgb8();
        let image2 = ImageReader::open("./unit_tests/in/annalyser/celeste-3-0.5-sha2-n-c.png").unwrap().decode().unwrap().to_rgb8();
        let image3 = ImageReader::open("./unit_tests/in/annalyser/celeste-3-1.0-sha2-n-c.png").unwrap().decode().unwrap().to_rgb8();
        
        let res1 = stats_last_bits(&image1,2);
        let res2 = stats_last_bits(&image2,2);
        let res3 = stats_last_bits(&image3,2);
        println!("original:              {:?}", res1);
        println!("modified:              {:?}", res2);
        println!("modified without key:  {:?}", res3);
    }

    #[test]
    fn test_only_lsb(){
        //let image= ImageReader::open("./unit_tests/in/modified_img/celeste-3-n-n-c.png").unwrap().decode().unwrap().to_rgb8();
        //only_lsb(&image).save("./tests/results/celeste-3-n-n-c-lsb_only.png").unwrap();
        /*test_only_lsb_with_name("celeste-3-1.0-n-n-c.png");
        test_only_lsb_with_name("celeste-3-0.5-n-n-c.png");
        test_only_lsb_with_name("celeste-3-0.3-n-n-c.png");
        test_only_lsb_with_name("celeste-3-0.2-n-n-c.png");
        test_only_lsb_with_name("celeste-3-0.1-n-n-c.png");*/
        /* 
        ./target/release/steg -w -l -i ./tests/celeste-3.png ./tests/results/celeste-3-0.5-sha2-p-c.png -t long.txt -g sha256 -c aes_key -p 0.50
        ./target/release/steg -w -l -i ./tests/celeste-3.png ./tests/results/celeste-3-1.0-sha2-p-c.png -t long.txt -g sha256 -c aes_key -p 0.999
        ./target/release/steg -w -l -i ./tests/earth.png ./tests/results/earth-1.0-sha2-p-c.png -t verylong.txt -g sha256 -c aes_key -p 0.999
        */
        test_only_lsb_with_name("celeste-3-1.0-sha2-n-c.png",0.99);
        test_only_lsb_with_name("celeste-3-0.5-sha2-n-c.png",0.5);
        test_only_lsb_with_name("celeste-3-0.3-sha2-n-c.png",0.3);
        test_only_lsb_with_name("celeste-3-0.2-sha2-n-c.png",0.2);
        test_only_lsb_with_name("celeste-3-0.1-sha2-n-c.png",0.1);
    }
    fn test_only_lsb_with_name(cur_str:&str,percentage:f32){
        let full_name="./unit_tests/in/annalyser/".to_owned()+ cur_str;
        let mut image;
        if !Path::exists(Path::new(&full_name)){
            let mut text_file = File::open("./unit_tests/in/verylong.txt").unwrap();
            let mut buffer =Vec::<u8>::new();
            text_file.read_to_end(&mut buffer).unwrap();
            
            image= ImageReader::open("./unit_tests/in/default_img/celeste-3.png").unwrap().decode().unwrap().to_rgb8();
            let dimensions = image.dimensions();
            let total_bytes = ((dimensions.0*dimensions.1*3*2)/8) as f32;
            let used_bytes = (total_bytes*percentage) as usize;
            let encoding: Box<dyn CharEncoding> = Box::new(ASCIIEncoding);
            buffer.truncate(used_bytes);
            lsb_hide_msg::<HashIterator<sha2::Sha256>>(&buffer, encoding.as_ref() ,None,&mut image);
            image.save(&full_name).unwrap();
        }
        else{
            image= ImageReader::open(&full_name).unwrap().decode().unwrap().to_rgb8();
        
        }
        let full_name="./unit_tests/out/annalyser/".to_owned()+ cur_str;
        diff_img_create(&image, 2).save(&full_name).unwrap();
    }
    #[test]
    fn try_img_delta(){
        let  image = ImageReader::open("./unit_tests/in/default_img/maddy-pfff.png").unwrap().decode().unwrap().to_rgb8();
        only_lsb(&image).save("./unit_tests/out/annalyser/lsb-maddy-pfff.png").unwrap();
        diff_img_create(&image, 3).save("./unit_tests/out/annalyser/delta/none-maddy-pfff.png").unwrap();
        
        let  image = ImageReader::open("./unit_tests/out/pvd/earth.png").unwrap().decode().unwrap().to_rgb8();
        only_lsb(&image).save("./unit_tests/out/annalyser/earth.png").unwrap();
        diff_img_create(&image, 3).save("./unit_tests/out/earth.png").unwrap();
        //image.save("./unit_tests/out/pvd/madeline-pfff.png").unwrap();
        /*
        let image= ImageReader::open("./img/photo/celeste-.png").unwrap().decode().unwrap().to_rgb8();
        diff_img_create(&image, 2).save("./img/photo/trnsfrm/celeste-_2.png").unwrap();
        let cur_str="origami.png";
        let image= ImageReader::open("./img/photo/".to_owned()+cur_str).unwrap().decode().unwrap().to_rgb8();
        diff_img_create(&image, 2).save("./img/photo/trnsfrm/".to_owned()+ cur_str).unwrap();
        let cur_str="celeste-3.png";
        let image= ImageReader::open("./img/photo/".to_owned()+cur_str).unwrap().decode().unwrap().to_rgb8();
        diff_img_create(&image, 2).save("./img/photo/trnsfrm/".to_owned()+ cur_str).unwrap();
        let cur_str="steg_with_key.png";
        let image= ImageReader::open(cur_str).unwrap().decode().unwrap().to_rgb8();
        diff_img_create(&image, 2).save("./img/photo/trnsfrm/".to_owned()+ cur_str).unwrap();
        let cur_str="cc.png";
        let image= ImageReader::open(cur_str).unwrap().decode().unwrap().to_rgb8();
        diff_img_create(&image, 2).save("./img/photo/trnsfrm/".to_owned()+ cur_str).unwrap();
        */
        /*let cur_str: &str="vile-foret-hash512.png";
        let image= ImageReader::open("./unit_tests/in/modified_img/".to_owned()+cur_str).unwrap().decode().unwrap().to_rgb8();
        diff_img_create(&image, 2).save("./img/photo/trnsfrm/".to_owned()+ cur_str).unwrap();
        let cur_str="hash-celeste-3.png";
        let image= ImageReader::open("./img/photo/secret/".to_owned()+cur_str).unwrap().decode().unwrap().to_rgb8();
        diff_img_create(&image, 2).save("./img/photo/trnsfrm/sec-".to_owned()+ cur_str).unwrap();
        let image= ImageReader::open("a.png").unwrap().decode().unwrap().to_rgb8();
        diff_img_create(&image, 2).save("a2.png").unwrap();
        */
        
    }

}