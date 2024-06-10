use image::RgbImage;

use crate::lsb::{lsb_hide_msg, lsb_retrieve_msg, CharEncoding, ImgIterator};



pub fn histo_unzip(img:&mut RgbImage, zip : u8){
    let (x_dim,y_dim) = img.dimensions();
    for x in 0..x_dim{
        for y in 0..y_dim{
            let pixel = img.get_pixel_mut(x, y);
            for color in 0..3{
                let res = pixel.0[color] as u16+(pixel.0[color]/(zip)) as u16;
                if res>255{
                    pixel.0[color]=255;
                }
                else{
                    pixel.0[color]=res as u8;
                }
            }
        }
    }
}
pub fn histo_zip(img:&mut RgbImage, zip : u8){
    let (x_dim,y_dim) = img.dimensions();
    for x in 0..x_dim{
        for y in 0..y_dim{
            let pixel = img.get_pixel_mut(x, y);
            for color in 0..3{
                pixel.0[color]-=   pixel.0[color]/(zip+1);
            }
        }
    }

}
pub fn lsb_retrieve_msg_zip<T:ImgIterator>(length :i32 , enconding: & dyn CharEncoding, img_key: Option<&[u8]>,img : &mut RgbImage) -> Vec<u8>{
    histo_zip(img, 8);
    return lsb_retrieve_msg::<T>(length, enconding, img_key, img);

}
pub fn lsb_hide_msg_zip<T:ImgIterator>(msg :&Vec<u8> , enconding: & dyn CharEncoding, img_key: Option<&[u8]>,img : &mut RgbImage){
    histo_zip(img, 8);
    lsb_hide_msg::<T>(msg, enconding, img_key, img);
    histo_unzip(img, 8);
}
#[cfg(test)]
mod test_histo_transfo{
    

    use image::io::Reader as ImageReader;
  
    use crate::{lsb::ASCIIEncoding, HashIterator};
   
    use super::*;
    #[test]
    fn  test_histo(){
        println!("{}",4/3);
        let mut image = ImageReader::open("./unit_tests/in/default_img/maddy-pfff.png").unwrap().decode().unwrap().to_rgb8();
        let msg = String::from("L'objectif du défi");
       
        let encoding= Box::new(ASCIIEncoding);
        let img_key=Some([0,1,2,3,4,5,6,7,8].as_slice());
        lsb_hide_msg_zip::<HashIterator<sha2::Sha256>>(&msg.as_bytes().to_vec() ,encoding.as_ref(), img_key,&mut image);
        image.save("./unit_tests/out/zip/maddy-pfff.png").unwrap();
        let res =lsb_retrieve_msg_zip::<HashIterator<sha2::Sha256>>(msg.len() as i32 ,encoding.as_ref(),img_key,&mut image);
        unsafe{
            println!("{}",String::from_utf8_unchecked(res));
        }
       

    }
}
    
