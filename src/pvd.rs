use std::cmp::{max, min};

use image::{Rgb, RgbImage};

use crate::lsb::ImgIterator;

pub struct BitStream{
    cur_byte : usize,
    cur_bit : usize,
    bytes : Vec<u8>
}

pub fn end_of_byte_stream(stream: &BitStream) -> bool{
    return stream.cur_byte==stream.bytes.len()-1 && stream.cur_bit==8;

}


pub fn next_bits(stream: &mut BitStream,bit_count : usize) -> Option<u8>{
    let remaining_bits_for_this_byte= 8-stream.cur_bit; // 7 because we start counting to zero
    if stream.cur_byte+1 == stream.bytes.len() && remaining_bits_for_this_byte > bit_count{
        stream.cur_byte= stream.bytes.len() -1;
        stream.cur_bit=8;
        return None;
    } 

    if remaining_bits_for_this_byte>bit_count{
        let result = (stream.bytes[stream.cur_byte] >> stream.cur_bit )& ((2 as u16).pow(bit_count as u32) -1) as u8;
        stream.cur_bit+=bit_count;
        if stream.cur_bit==8{
            stream.cur_bit=0;
            stream.cur_byte+=1;
        }
        return Some(result);
    }
    
    let mut result = (stream.bytes[stream.cur_byte] >> stream.cur_bit )& ((1<<remaining_bits_for_this_byte) -1) as u8;
    //result<<= bit_count - remaining_bits_for_this_byte;
    stream.cur_byte+=1;
    result|= (stream.bytes[stream.cur_byte] & ((1<<(bit_count - remaining_bits_for_this_byte)) -1) as u8)<< (remaining_bits_for_this_byte);
    stream.cur_bit=bit_count- remaining_bits_for_this_byte;
    return Some(result);
}


pub fn write_bit_stream(stream: &mut BitStream,data: u8,bit_count : usize ) {
    let mut data= data;
    let remaining_bits_for_this_byte= 8-stream.cur_bit as u32;
    if remaining_bits_for_this_byte>=bit_count as u32{ 
        stream.bytes[stream.cur_byte]= (stream.bytes[stream.cur_byte]& ((1<<stream.cur_bit) -1))
                                    |   ((data & (( 1<< (bit_count as u32) )-1)) << stream.cur_bit );
        stream.cur_bit+=bit_count;
        //println!("_oui : {}", stream.bytes[stream.cur_byte]);
        if stream.cur_bit==8{
            stream.cur_bit=0;
            stream.cur_byte+=1;
        }
        return;
    }
    //01101111
    //11000011
    //11000111
    //println!("a: {} , b: {} ",remaining_bits_for_this_byte, stream.cur_bit );
    stream.bytes[stream.cur_byte]= stream.bytes[stream.cur_byte]&((1<<stream.cur_bit) -1) ;
    //println!("non : {}", stream.bytes[stream.cur_byte]);
    //println!("mais gros {}",(data & ((1<<remaining_bits_for_this_byte) -1) as u8) << stream.cur_bit);
    stream.bytes[stream.cur_byte]|=   (data & ((1<<remaining_bits_for_this_byte) -1) as u8) << stream.cur_bit ;
    data >>= remaining_bits_for_this_byte;
    //println!("oui : {}", stream.bytes[stream.cur_byte]);
    //println!("mais groooooos {} {} {}",data,(data & (1<<(bit_count as u32 -remaining_bits_for_this_byte)-1) as u8),(1<<(bit_count as u32 -remaining_bits_for_this_byte))-1);
    stream.cur_byte+=1;
    stream.cur_bit=0;
    stream.bytes[stream.cur_byte]=  data & ((1<<(bit_count as u32 -remaining_bits_for_this_byte))-1) as u8;
 
    stream.cur_bit=bit_count as usize -remaining_bits_for_this_byte as usize;



}

///
/// 
/// based on https://www.hindawi.com/journals/scn/2017/1924618/
/// 
/// 
fn compute_pixel_grid_3by2(img: &mut RgbImage,stream: &mut BitStream,bit_count : &mut Box<u32>,x_pos : u32, y_pos: u32, rgb_color: usize) -> Option<[u8;4]>{
    let mut gs = Vec::<&Rgb<u8>>::with_capacity(4);
    gs.push(img.get_pixel(x_pos, y_pos));
    gs.push(img.get_pixel(x_pos+2, y_pos));
    let gu_pixel=img.get_pixel(x_pos+1, y_pos);
    let gb_pixel=img.get_pixel(x_pos+1, y_pos+1);
    gs.push(img.get_pixel(x_pos, y_pos+1));
    gs.push(img.get_pixel(x_pos+2, y_pos+1));
    
    let gu=gu_pixel.0[rgb_color];
    let gb=gb_pixel.0[rgb_color];
    let mut gs_prime : [u8;4] = [0;4];
    let mut d_us :[i16;4] = [0;4];
    let mut d_bs :[i16;4] = [0;4];
    let mut ls : [u16;4] = [0;4];
    let mut us : [u8;4] = [0;4];
    let mut ns : [u8;4] = [0;4];
    let mut bs : [u8;4] = [0;4];
    

    for index in 0..gs.len(){
        d_us[index]=gs[index].0[rgb_color] as i16 - gu as i16;
        d_bs[index]=gs[index].0[rgb_color] as i16 - gb as i16; 
    }

    for  i in 0..4{
        if d_us[i]>0 && d_bs[i]>0{
            //print!("1 ");
            ls[i]= max(gu as u16 +1 ,gb as u16 +1);
            us[i]=255;
        }
        else if d_us[i]<=0 && d_bs[i]<=0{
            //print!("2 ");
            ls[i]=0;
            us[i]=min(gu,gb);   
        }
        else if d_us[i]>0 && d_bs[i]<=0 {
            //print!("3 ");
            ls[i]=gu as u16 +1;
            us[i]=gb;           
        }
        else{
            ls[i]=gb as u16 +1;
            us[i]=gu;
        }
        //print!("bb : {}  {}  {}   ,",us[i],ls[i],i16::abs(us[i] as i16 -ls[i] as i16+1));
        //println!("aaa: {}",f64::floor(f64::log2(i16::abs(us[i] as i16 -ls[i] as i16+1) as f64)));
        ns[i] = min(f64::floor(f64::log2(i16::abs(us[i] as i16 -ls[i] as i16+1) as f64)) as u8,3);
        //println!("ns: {}",ns[i]);
        let next =next_bits(stream, ns[i] as usize);
        if next.is_none(){
            return None;
        }

        bs[i]=next.unwrap();
        //println!("bs in: {}",bs[i]);
        **bit_count=bit_count.checked_add(ns[i] as u32).unwrap();
        let mut min=255;
        let mut argmin=255;
        //println!("ns: {}", ns[i]);
        for e in ls[i]..(us[i]as u16+1) {
            if e > 255{
                break;
            }
            if (i16::abs(e as i16 -gu as i16)% (2 as i16 ).pow(ns[i] as u32) )as u8 == bs[i]{
                let cur_arg = i16::abs(e as i16 - gs[i].0[rgb_color] as i16) as u8;
                if cur_arg<min{
                    argmin=e;
                    min=cur_arg;
                }
            }

        }
        gs_prime[i]=argmin as u8;
        //println!("aaa {}",gs_prime[i] as i32 -gs[i].0[rgb_color] as i32);
    }
    return Some(gs_prime);
}

///
/// 
/// based on https://www.hindawi.com/journals/scn/2017/1924618/
/// 
/// 
fn retrieve_pixel_grid_3by2(img: &mut RgbImage,stream: &mut BitStream,x_pos : u32, y_pos: u32, rgb_color: usize)-> u32{
    let mut bit_count:u32=0;
    let mut gs = Vec::<&Rgb<u8>>::with_capacity(4);
    gs.push(img.get_pixel(x_pos, y_pos));
    gs.push(img.get_pixel(x_pos+2, y_pos));
    let gu_pixel=img.get_pixel(x_pos+1, y_pos);
    let gb_pixel=img.get_pixel(x_pos+1, y_pos+1);
    gs.push(img.get_pixel(x_pos, y_pos+1));
    gs.push(img.get_pixel(x_pos+2, y_pos+1));

    let gu=gu_pixel.0[rgb_color];
    let gb=gb_pixel.0[rgb_color];
    let mut d_us :[i16;4] = [0;4];
    let mut d_bs :[i16;4] = [0;4];
    let mut ls : [u16;4] = [0;4];
    let mut us : [u8;4] = [0;4];
    let mut ns : [u8;4] = [0;4];
    let mut bs : [u8;4] = [0;4];
    

    for index in 0..gs.len(){
        d_us[index]=gs[index].0[rgb_color] as i16 - gu as i16;
        d_bs[index]=gs[index].0[rgb_color] as i16 - gb as i16; 
    }

    for  i in 0..4{
        if d_us[i]>0 && d_bs[i]>0{
            ls[i]= max(gu as u16 +1 ,gb as u16 +1);
            us[i]=255;
        }
        else if d_us[i]<=0 && d_bs[i]<=0{
            ls[i]=0;
            us[i]=min(gu,gb);   
        }
        else if d_us[i]>0 && d_bs[i]<=0 {
            ls[i]=gu as u16 +1;
            us[i]=gb;           
        }
        else{
            ls[i]=gb as u16 +1;
            us[i]=gu;
        }
        ns[i] = min(f64::floor(f64::log2(i16::abs(us[i] as i16 -ls[i] as i16+1) as f64)) as u8,3);
    
        bs[i]=i16::abs(gu as i16 - gs[i].0[rgb_color] as i16) as u8 %(2 as u8).pow(ns[i] as u32) ;
        bit_count+=ns[i] as u32;
        //println!("bits_count: {} ", bit_count);
        //println!("bs out: {}",bs[i]);
        write_bit_stream(stream, bs[i], ns[i] as usize);
    }  
    return bit_count as u32; 
    
}
///
/// 
/// 
/// 
/// 
/// 
/// 
/// 
/// 
pub fn pvd_3by2_hide_msg<T:ImgIterator>(msg :&Vec<u8> , img_key: Option<&[u8]>,img : &mut RgbImage){
    let mut bit_count= Box::<u32>::new(0);
    let img_dim = img.dimensions();
    let mut img_for_iterator = RgbImage::new(img_dim.0/3, img_dim.1/2);
    for x in 0..img_dim.0/3{
        for y in 0..img_dim.1/2{
            img_for_iterator.put_pixel(x, y,img.get_pixel(x*3+1, y*2).clone());
        }
    }
    let mut iterator = T::new(&mut img_for_iterator,img_key);
   
    let mut cloned_bytes=msg.clone();
    cloned_bytes.push(0);
    cloned_bytes.push(0);
    let mut stream = BitStream{cur_bit:0,cur_byte:0,bytes:cloned_bytes};
    while !end_of_byte_stream(&stream){
        let next_pixel=iterator.next();
        if next_pixel==None{
            println!("WARNING: String doesn't fit inside the image!!!");
            //println!("Stopped after putting {} bits, at character : {}",bits_put,*c as char);
            println!("Exiting");

            break;
        }
        let (x,y,color)=next_pixel.unwrap();
        //println!("pos : {} {} {}",x,y,color);
        let modified_pixels =compute_pixel_grid_3by2(img, &mut stream,&mut bit_count, x*3, y*2, color);
        if modified_pixels.is_none(){
            break;
        }
        let pix=modified_pixels.unwrap();
        img.get_pixel_mut(x*3   , y*2   ).0[color]=pix[0];
        img.get_pixel_mut(x*3 +2, y*2   ).0[color]=pix[1];
        img.get_pixel_mut(x*3   , y*2 +1).0[color]=pix[2];
        img.get_pixel_mut(x*3 +2, y*2 +1).0[color]=pix[3];
    }
    println!("encoded {} bits...",bit_count);
}


///
/// 
/// 
/// 
/// 
/// 
/// 
pub fn pvd_3by2_retrieve_msg<T:ImgIterator>(img_key: Option<&[u8]>, iteration_count: usize,img : &mut RgbImage) -> Vec<u8>{
    let data = vec!(0 as u8 ;100000000);
    let img_dim = img.dimensions();
    let mut img_for_iterator = RgbImage::new(img_dim.0/3, img_dim.1/2);
    for x in 0..img_dim.0/3{
        for y in 0..img_dim.1/2{
            img_for_iterator.put_pixel(x, y,img.get_pixel(x*3+1, y*2).clone());
        }
    }
    let mut iterator = T::new(&mut img_for_iterator,img_key);
    let mut decoded_bits_count=0;
    let mut stream =BitStream{cur_bit:0,cur_byte:0,bytes:data};
    while !end_of_byte_stream(&stream) && decoded_bits_count<iteration_count{
        let next_pixel=iterator.next();
        if next_pixel==None{
            println!("WARNING: String doesn't fit inside the image!!!");
            println!("Stopped after putting {} bits.",decoded_bits_count);
            println!("Exiting");            
            break;

        }
        let (x,y,color)=next_pixel.unwrap();
        decoded_bits_count+=retrieve_pixel_grid_3by2(img, &mut stream, x*3, y*2, color) as usize;
    }
    if decoded_bits_count%8!=0{
        stream.bytes.truncate(1+((decoded_bits_count-(decoded_bits_count%8))/8) );
    }
    return stream.bytes;
   
}

#[cfg(test)]
mod test_pvd{
    use std::{fs::File, io::Read};

    use image::io::Reader as ImageReader;
    use sha2::Sha512;
    use crate::{utilities::encryption::keygen_aes128, HashIterator};

    use super::*;


    #[test]
    fn test_pvd(){
        let mut image = ImageReader::open("./unit_tests/in/default_img/maddy-pfff.png").unwrap().decode().unwrap().to_rgb8();
        let msg = String::from("oui bonjour ceci est un test je suppose ouaaaaahhh yen a des mots brefffffff fr fr fr fr fr fr fr fr fr fr fr fr fr fr fr fr fr fr fr fr fr fr fr fr fr fr fr fr fr fr fr fr fr fr fr fr fr fr fr");
        let key_img_rng = keygen_aes128();
        pvd_3by2_hide_msg::<HashIterator<Sha512>>(&msg.clone().into_bytes(),Some(&key_img_rng),&mut image);
        image.save("./unit_tests/out/sigma/madeline-pfff.png").unwrap();
        let mut  rd_msg = pvd_3by2_retrieve_msg::<HashIterator<Sha512>>(Some(&key_img_rng), msg.len()*8,&mut image);
        println!("FIN!!!!!");
        rd_msg.truncate(msg.len());
        let mes_retrieved=String::from_utf8(rd_msg).unwrap();
        println!("resultat algo PVD : {}",mes_retrieved);
        assert_eq!(msg,mes_retrieved);
        image.save("./unit_tests/out/pvd/madeline-pfff.png").unwrap();
    }
    #[test]
    fn write_all(){
        let mut image = ImageReader::open("./unit_tests/in/default_img/maddy-pfff.png").unwrap().decode().unwrap().to_rgb8();
        
        let mut file = File::open("./unit_tests/in/verylong.txt").unwrap();

        let mut buffer= Vec::with_capacity(10000000);
        file.read_to_end(&mut buffer).unwrap();
        let key_img_rng = keygen_aes128();
        pvd_3by2_hide_msg::<HashIterator<Sha512>>(&buffer,Some(&key_img_rng),&mut image);
        image.save("./unit_tests/out/pvd/madeline-pfff.png").unwrap();
        let image = ImageReader::open("./tests/earth.png").unwrap().decode().unwrap().to_rgb8();
        image.save("./unit_tests/out/pvd/earth.png").unwrap();
        println!("FIN!!!!!");
       
       

    }
}