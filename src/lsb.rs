use image::RgbImage;

pub trait ImgIterator {
    fn new(img:&RgbImage,key: Option<&[u8]>)-> Self where Self: Sized;
    fn next(&mut self)-> Option<(u32,u32,usize)>;
}
#[derive(Default)]
pub struct  DefaultIterator{
    x:u32,
    x_len:u32,
    y:u32,
    y_len:u32,
    rbg:usize,
}
///
/// 
/// 
/// à modifieerrrrrrr
/// 
/// 
/// 

impl ImgIterator for DefaultIterator {
   fn new(img:&RgbImage,_key:Option<&[u8]>)-> Self where Self: Sized {
        let (x_len,y_len)=img.dimensions();
        return DefaultIterator { x: 0, x_len, y: 0, y_len, rbg: 0 }
        
   }
   fn next(&mut self)->Option<(u32,u32,usize)> {
        
        let res =Some((self.x,self.y,self.rbg));
        self.rbg=(self.rbg+1)%3;
        if self.rbg==0{

            self.x=(self.x+1)%self.x_len;
            if self.x==0{
                self.y=(self.y+1)%self.y_len;
            }
        }
        if (self.x==0) && (self.y==0) && (self.rbg==0){
            
            return None;
        }
      
        return res;
   }
}

///
/// 
/// Describe a character encoding
/// 
/// 
pub trait CharEncoding {
    ///
    /// Return the encoding of a char as a Option<Vec<u8>>
    /// (the encoded char may have a )
    /// 
    fn get_enconding(&self,char :u8) -> Option<Vec<u8>>;
    ///
    /// Decode an u8 Vector
    /// 
    fn decode_all(&self,encoded:Vec<u8>) -> Option<Vec<u8>>;
}
///
/// 
/// Default char encoding (i.e ASCIIEncoding)
/// Does not modify the bytes.
/// 
/// 
pub struct  ASCIIEncoding;
impl  CharEncoding for ASCIIEncoding{
    fn get_enconding(&self,char :u8) -> Option<Vec<u8>>{
        let mut vec= Vec::with_capacity(1);
        vec.push(char);
        return  Some(vec);
    }

    fn decode_all(&self, encoded:Vec<u8>) -> Option<Vec<u8>> {
        return Some(encoded);
    }
}
///
/// 
/// Hide the given message inside the image inside the 2 least significant bits, according to the ImgIterator, using the given CharEncoding.
/// 
/// 
pub fn lsb_hide_msg<T:ImgIterator>(msg :&Vec<u8> , enconding: & dyn CharEncoding, img_key: Option<&[u8]>,img : &mut RgbImage){
    let msk : u8 = !0b11;
    let mut iterator = T::new(img,img_key);
    let mut encoded_bytes_count=0;
    for c in msg{
        let encodeds = match enconding.get_enconding(*c){
            Some(result_encoding) => result_encoding,
            None => continue,
        };

        //we create a vec containing the bits we have to "or" with image's pixels
        let mut bytes_to_hide= Vec::with_capacity(4*encodeds.len());

        for encoded in &encodeds{ //we iterate over all bytes
            for i in 0..4{ //we cut all bytes in 4
                //we keep the 2 leasts significant bits first
                //then bits 3 & 4 
                //...
                //and we move these bits to the 2 lsb (so the firsts are not moved, 3&4 are moved 2 bits to the right....)
                bytes_to_hide.push(((encoded>>2*i)&0x3) as u8);
            }
        }
        encoded_bytes_count+=encodeds.len();
        //println!("{:?}  ",cs );
        let mut bits_put=0;
        for bytes in bytes_to_hide{
            let next_pixel=iterator.next();
            if next_pixel==None{
                println!("WARNING: String doesn't fit inside the image!!!");
                println!("Stopped after putting {} bits, at character : {}",bits_put,*c as char);
                println!("Exiting");

                return;
            }
            let (x,y,color)=next_pixel.unwrap();
            img.get_pixel_mut(x, y).0[color]&=msk;
            img.get_pixel_mut(x, y).0[color]|=bytes;
            bits_put+=2;
        }
        
    }
    println!("encoded {} bytes...",encoded_bytes_count);
}

///
/// 
/// Find the message hiden inside the image inside the 2 least significant bits, according to the ImgIterator, using the given CharEncoding.
/// 
/// 
pub fn lsb_retrieve_msg<T:ImgIterator>(mut length :i32 , enconding: & dyn CharEncoding, img_key: Option<&[u8]>,img : &mut RgbImage) -> Vec<u8>{
    let mut iterator = T::new(img,img_key);
    let mut cur_char:u8=0;
    let mut cd=0;
    let mut encoded= Vec::with_capacity(length as usize);
    // bytes stored in the 2 lsb => 4 pixels to store a bytes
    // we have to iterate over 4 * the assumed size of the message
    for _ in 0..4*length{
        let (x,y,c) = match iterator.next()  {
            None => break,
            Some(t) => t,
        };
        let val = img.get_pixel(x, y).0[c];
        cur_char|= (val&0b11)<<(2*cd);
        //print!("{}, ",(val&0b11));
        cd +=1;
        if cd==4{
            //println!("|{}", cur_char);
            encoded.push(cur_char);
            length-=1;
            if length<=0{
                break;
            }
            cd=0;
            cur_char=0;
        }
    }
    
    let result: Vec<u8> = enconding.decode_all(encoded).unwrap();
    return result;
    //return String::from_utf8(result).unwrap();
}
