use std::collections::HashMap;


use image::RgbImage;

#[allow(unused)]
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
#[allow(unused)]
fn init_base_shanon_iterator (img:&RgbImage,min_x:u32,min_y:u32,max_x:u32,max_y:u32,mask:u8)-> BaseShanonIterator {
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

#[allow(unused)]
fn compute_shanon_entropy(img:&RgbImage,iterator:&mut dyn ShanonIterator) ->(f64,f64,f64){
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

//#[cfg(test)]
mod test_entropy{
    use std::fs;

    use image::io::Reader as ImageReader;
    use super::*;

    fn compute_base_shanon_iterator(img_name: &str){
        let image = ImageReader::open(img_name).unwrap().decode().unwrap().to_rgb8();
        let mut iterator1 = init_base_shanon_iterator(&image,0,0,image.width(),image.height(),255);

        println!("Computed shanon entropy with a 0xFF mask : {:?}", compute_shanon_entropy(&image,&mut iterator1));
        let mut iterator2 = init_base_shanon_iterator(&image,0,0,image.width(),image.height(),3);
        println!("Computed shanon entropy with a 0x03 mask : {:?}", compute_shanon_entropy(&image,&mut iterator2));
        let mut iterator3 = init_base_shanon_iterator(&image,0,0,image.width(),image.height(),252);
        println!("Computed shanon entropy with a 0xFC mask : {:?}", compute_shanon_entropy(&image,&mut iterator3));
        
    }

    #[test]
    fn test_base_shanon_iterator(){
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
        
        for file in fs::read_dir("~/Images/steg/").unwrap() {
            //println!("{}", (&file).unwrap().path().display());
            println!("===========For an unmodified image===========");
            compute_base_shanon_iterator(&file.unwrap().file_name().into_string().unwrap());
        }
    
    }
}