use std::collections::HashMap;
use image::RgbImage;
use rand::{distributions::{Distribution, Uniform}, Rng};


pub struct PixelMaskInfo{
    x:u32,
    y:u32,
    rgb:usize,
    mask:u8,
}

pub trait ShanonIterator {
    fn next(&mut self)-> Option<Vec<PixelMaskInfo>>;
    fn has_next(&mut self) -> bool;
    fn next_bytes_vec(& mut self) -> Option<Vec<u8>>;
}

pub struct BaseShanonIterator<'a>{
    img: &'a RgbImage,
    cur_x:u32,
    cur_y:u32,

    min_x:u32,
    _min_y:u32,
    max_x:u32,
    max_y:u32,
    
    mask:u8,
}
pub fn init_base_shanon_iterator (img:&RgbImage,min_x:u32,min_y:u32,max_x:u32,max_y:u32,mask:u8)-> BaseShanonIterator {
    BaseShanonIterator{
        img,
        cur_x:min_x,
        cur_y:min_y,

        min_x,
        _min_y:min_y,
        max_x,
        max_y,
        
        mask,
    }
}
impl ShanonIterator for BaseShanonIterator<'_> {
    

    fn next(&mut self)-> Option<Vec<PixelMaskInfo>> {
        if !self.has_next(){
            return None;
        }
        let mut pixels = Vec::<PixelMaskInfo>::new();
        for i in 0..3{
            pixels.push(PixelMaskInfo{
                x:self.cur_x,
                y:self.cur_y,
                rgb:i,
                mask:self.mask,
            })
        }
        if self.cur_x==self.max_x-1{
            self.cur_x=self.min_x;
            if self.cur_y==self.max_y-1{
                self.cur_x=self.max_x;
            }
            self.cur_y+=1;
        }
        else{
            self.cur_x+=1;
        }
        
        //println!("{},{} {}",self.cur_x,self.cur_y, self.max_y);
        return  Some(pixels);


    }
    fn next_bytes_vec(& mut self) -> Option<Vec<u8>>{
        let data = match self.next(){
            Some(t) => t,
            None => return None,
        };
        let mut output =Vec::<u8>::with_capacity(3);
        for i in 0..3{
            let pixel =self.img.get_pixel(data[i].x,data[i].y);
            //println!("pix {} : {}", i,pixel.0[i]);
            output.push(  pixel.0[i]& (self.mask));
        }
        return Some(output);
    }

    fn has_next(&mut self) -> bool {
        //println!("{} {} {} {} ",self.cur_x,self.cur_y, self.max_x,self.max_y);
        return (self.cur_x!= self.max_x) || (self.cur_y!=self.max_y);
    }

}


pub fn compute_shanon_entropy(img:&RgbImage,iterator:&mut dyn ShanonIterator) ->(f64,f64,f64){
    let mut map_values=HashMap::<Vec<u8>,u32>::new();
    let mut total_data_count=0;
    while iterator.has_next(){
        let data = iterator.next_bytes_vec().unwrap();
        //println!("data: {:?}",data);
        if map_values.contains_key(&data){
            let new_value=map_values.get(&data).unwrap()+1;
            map_values.insert(data, new_value);
        }
        else{
            map_values.insert(data, 1);
        }
        total_data_count+=1;
    }
    let mut entropy=0.0;
    for (_key,val) in map_values.iter(){
        let proba = (*val as f64)/(total_data_count as f64);
        entropy-= proba*proba.log2();
    }
    
    return (entropy, (map_values.len() as f64).log2(), entropy/(map_values.len() as f64).log2());


}

///
/// Iterate over all pixels. Each pixel has a probability p to be reroled.
/// Only modify the number of LSB  given in mask
/// 
pub fn randomize_lsb(img_in: &RgbImage, p:f32,mask:u8) ->RgbImage{
    let mut img_out= img_in.clone();
    let mut rng = rand::thread_rng();
    let u = Uniform::<f32>::from(0.0..1.0);
    let mask_bits= (1 as u8)<<mask-1;
    for rgb_pixel in img_out.pixels_mut(){
        for i in 0..3{
            if u.sample(&mut rng)>1.0-p{
                let new_pixel : u8 = rng.gen();
                rgb_pixel.0[i]&=!mask_bits;
                rgb_pixel.0[i]|=mask_bits&new_pixel;
            }
        }
        
    }

    return  img_out;
}
pub fn entropy_and_randomization(img_in: &RgbImage, p:f32,mask_bits:u8) -> f64{
    //let mask_bits = (1<<mask-1) as u8;
    let mut iterator1 = init_base_shanon_iterator(&img_in,0,0,img_in.width(),img_in.height(),255);
    let a =compute_shanon_entropy(&img_in,&mut iterator1);
    let mut iterator2 = init_base_shanon_iterator(&img_in,0,0,img_in.width(),img_in.height(),mask_bits);
    let b=compute_shanon_entropy(&img_in,&mut iterator2);
    return entropy_and_randomization_after_first_measure(img_in,a,b,p,mask_bits);


}
pub fn entropy_and_randomization_after_first_measure(img_in: &RgbImage,values_mask_max: (f64, f64, f64),values_mask_min: (f64, f64, f64), p:f32,mask_bits:u8) -> f64{
    //let mask_bits = (1<<mask-1) as u8;
    let image =randomize_lsb(& img_in, p, mask_bits);
    let image_dim= img_in.dimensions();
    let image_size=image_dim.0 * image_dim.1;
    //println!("jpp: {:?} ; {:?}", a,b);
    let mut iterator1 = init_base_shanon_iterator(&image,0,0,img_in.width(),img_in.height(),255);
    let values_mask_max2 =compute_shanon_entropy(&img_in,&mut iterator1);
    let mut iterator2 = init_base_shanon_iterator(&image,0,0,img_in.width(),img_in.height(),mask_bits);
    let values_mask_min2=compute_shanon_entropy(&img_in,&mut iterator2);
    println!("===ausec===");
    println!("max1 : {:?}   min1 : {:?} \n max2 : {:?} min2 : {:?}",values_mask_max,values_mask_min,values_mask_max2,values_mask_min2);
    return  f64::abs( (((values_mask_max2.0/values_mask_min2.0)/(values_mask_max.0/values_mask_min.0))-1.)*100.)
            *f64::log2(image_size as f64 )/f64::log2(1920.*1080.);
    
}

#[cfg(test)]
mod test_entropy{
    use std::fs;

    use image::io::Reader as ImageReader;
    use super::*;

    fn compute_base_shanon_iterator(img_name: &str){
        println!("{img_name}");
        let image = ImageReader::open(img_name).unwrap().decode().unwrap().to_rgb8();
        let mut iterator1 = init_base_shanon_iterator(&image,0,0,image.width(),image.height(),255);
        let a =compute_shanon_entropy(&image,&mut iterator1);
        println!("Computed shanon entropy with a 0xFF mask : {:?}", a);
        let mut iterator2 = init_base_shanon_iterator(&image,0,0,image.width(),image.height(),3);
        let b=compute_shanon_entropy(&image,&mut iterator2);
        println!("Computed shanon entropy with a 0x03 mask : {:?}",b );
        println!("Normalized shanon entropy with a 0x03 % : {:?}",(1.-b.2)*100. );
        let mut iterator3 = init_base_shanon_iterator(&image,0,0,image.width(),image.height(),252);
        println!("Computed shanon entropy with a 0xFC mask : {:?}", compute_shanon_entropy(&image,&mut iterator3));
        println!("0xFF/Ox03 : {}",a.0/b.0);
        
        
    }
    ///
    /// 
    /// Calculate  d(a.0/b.0)/d(t)
    /// with a.0 entropy of the img with a mask of 0xFF, and b.0 the one with a 0x03 mask.
    /// In other words calculate the entropy of an image. Then, randomize the value of some pixel
    /// And calculate the entropy once again.
    /// Then we look a the percentage 
    /// 
    /// 
    /// 
    
    #[test]
    fn test_randomize_img(){
        let path = "/home/admin/Images/steg/base_img";
         for file in fs::read_dir(path).unwrap() {
            let filename =file.unwrap().file_name().into_string().unwrap();
            println!("===========For an unmodified image===========");
            println!("base::");
            
            let mut image = ImageReader::open(&format!("{path}/{filename}")).unwrap().decode().unwrap().to_rgb8();
            let mut iterator1 = init_base_shanon_iterator(&image,0,0,image.width(),image.height(),255);
            let a =compute_shanon_entropy(&image,&mut iterator1);
            let mut iterator2 = init_base_shanon_iterator(&image,0,0,image.width(),image.height(),3);
            let b=compute_shanon_entropy(&image,&mut iterator2);
            let mut a_s = Vec::new();
            let mut b_s = Vec::new();
            a_s.push(a);
            b_s.push(b);
            for i in 0..10{
                
                let mut iterator1 = init_base_shanon_iterator(&image,0,0,image.width(),image.height(),255);
                let a =compute_shanon_entropy(&image,&mut iterator1);
                let mut iterator2 = init_base_shanon_iterator(&image,0,0,image.width(),image.height(),3);
                let b=compute_shanon_entropy(&image,&mut iterator2);
                println!("===========For {i} modification ===========");
                println!("Computed shanon entropy with a 0xFF mask : {:?}",a );
                println!("Computed shanon entropy with a 0x03 mask : {:?}",b );
                println!("0xFF/Ox03 : {}",a.0/b.0);
                let a2 =a_s.last().unwrap();
                let b2=b_s.last().unwrap();
                println!("last : {}", ((a.0/b.0)/(a2.0/b2.0)-1.)*100.);
                a_s.push(a);
                b_s.push(b);
                
                image =randomize_lsb(& image, 0.1, 2);
            }
         }
    }
    #[test] 
    fn test_base_shanon_iterator(){
        let max_n=0;
        for file in fs::read_dir("/home/admin/Images/steg/s256-c-p-0.99").unwrap() {
            //println!("{}", (&file).unwrap().path().display());
           
            let a ="/home/admin/Images/steg/s256-c-p-0.99/";
            let b =&file.unwrap().file_name().into_string().unwrap();
            println!("{b}");
            println!("===========For an 0.99 modified image===========");
            compute_base_shanon_iterator(&format!("{a}{b}"));
            println!("===========For an 0.5 modified image===========");
            let a ="/home/admin/Images/steg/s256-c-p-0.5/";
            compute_base_shanon_iterator(&format!("{a}{b}"));
            println!("===========For an 0.1 modified image===========");
            let a ="/home/admin/Images/steg/s256-c-p-0.1/";
            compute_base_shanon_iterator(&format!("{a}{b}"));
            println!("===========For an unmodified image===========");
            let a ="/home/admin/Images/steg/base_img/";
            compute_base_shanon_iterator(&format!("{a}{b}"));
            println!("------");
        }
        for file in fs::read_dir("/home/admin/Images/steg/base_img").unwrap() {
            //println!("{}", (&file).unwrap().path().display());
            
        }

        println!("===========For an unmodified image===========");
        compute_base_shanon_iterator("./unit_tests/in/default_img/celeste-3.png");
        println!("===========For an image with 20% of pixels modified===========");
        compute_base_shanon_iterator("./unit_tests/in/annalyser/celeste-3-0.2-sha2-n-c.png");
        println!("===========For an image with 30% of pixels modified===========");
        compute_base_shanon_iterator("./unit_tests/in/annalyser/celeste-3-0.3-sha2-n-c.png");
        println!("===========For an image with 50% of pixels modified===========");
        compute_base_shanon_iterator("./unit_tests/in/annalyser/celeste-3-0.5-sha2-n-c.png");
        println!("===========For an image with 100% of pixels modified===========");
        compute_base_shanon_iterator("./unit_tests/in/annalyser/celeste-3-1.0-sha2-n-c.png");
        
        
    
    }
}