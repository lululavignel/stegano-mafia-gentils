
use image::RgbImage;
use sha2::{Digest, digest::generic_array::GenericArray};
use crate::position_by_hash::reset_hash;


//use encryption::{decrypt_aes128_ecb, sigma_encrypt_aes128};
pub trait HashRng<T:Digest>{
    fn new(data:Vec<u8>,key : Option<&[u8]>)-> Self where Self: Sized;
    fn random(&mut self,max:u32)-> u32;
}
pub struct  SimpleHashRng<T:Digest>{
    pos: usize,
    hash:GenericArray<u8, T::OutputSize>,
}
impl<T:Digest> HashRng<T> for  SimpleHashRng<T>{
    fn new(mut data:Vec<u8>,key : Option<&[u8]>)-> Self where Self: Sized {
        let mut hasher = T::new();
        if key.is_some(){
            let key_bytes= key.unwrap();
            for i in 0..key_bytes.len(){
                data[i]^=key_bytes[i]
            }
        }
        hasher.update(data);
        let hash= hasher.finalize();
        return SimpleHashRng::<T>{pos:0,hash} ;
    }
    fn random(&mut self,max:u32)-> u32 {
        let len: usize = self.hash.len();
        let mut tab =[0;2];
        for i in 0..2{
            if len==self.pos{
                reset_hash::<T>(&mut self.hash);
                //println!("{:?}",hash);
                self.pos=0;       
            }
            tab[i]=self.hash[self.pos] as u32;
            self.pos+=1;
        }
        
        return ((tab[0]<<8)+tab[1])%max;
    }
}

pub fn img_to_bytes_masked(img:&RgbImage,nbr_bits:u8)-> Vec<u8>{
    if nbr_bits>7{
        panic!("Number of bits too important")
    }
    //creating the mask
    let mask=!((1<<nbr_bits)-1);
    let mut bytes= Vec::<u8>::new();
    let (x_size,y_size)= img.dimensions();
    for x in 0..x_size{
        for y in 0..y_size{
            let cur_pixel=img.get_pixel(x, y);
            //applying the mask to all pixels, and appending to the bytes vector
            for color in cur_pixel.0{
                let masked_color=color & mask;
                bytes.push(masked_color);
            }
        }
    }
    //println!("{:?}",&bytes[0..10]);
    return bytes;
}


///
/// 
/// Algorithm executing a random permuttation on the array. (see wikipedia for algorithm explanation/proof)
/// It has a O(n) complexity.
/// 
/// 
pub fn sigma_ficher_yates_shuffles<T:Digest,S:Copy>(tab:&mut [S],rng:  &mut dyn HashRng<T>){
    let len: usize =tab.len();
    for i in 0..(len-1){
        let rdm=rng.random((len-1-i) as u32)as usize+i;
        let temp: S = tab[i];
        tab[i]=tab[rdm];
        tab[rdm]=temp;
    }
}
pub fn partial_ficher_yates_shuffles<T:Digest,S:Copy>(tab:&mut [S],start:usize,end:usize, rng:  &mut dyn HashRng<T>) {
    let len: usize =tab.len();
    if start>end ||end >len{
        panic!("Error : unvalid values : start = {} , end = {} len = {} ",start,end,len);
    }
    for i in start..(end){
        let rdm=rng.random((len-1-i) as u32)as usize+i;
        let temp: S = tab[i];
        tab[i]=tab[rdm];
        //println!("rdm: {}",rdm);
        tab[rdm]=temp;
    }
}

