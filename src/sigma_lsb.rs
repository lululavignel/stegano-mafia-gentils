use image::RgbImage;
use sha2::Digest;
use crate::{lsb::ImgIterator, position_by_hash::HashIterator};
use crate::utilities::hash_rng::{*};
use crate::utilities::encryption::{decrypt_aes128_ecb, sigma_encrypt_aes128};


fn  prune_sigma_str(message: &mut String,start_char:u8, end_char:u8,_block_len:usize)-> Vec<u8>{
    let mut message = message.replace(
            &['(', ')', ',', '\"', '.', ';', ':', '\'',
                    '!','&','#','$','%','*','+','-','/','\\',
                    '@','{','}','[',']'][..], "");
    if !message.is_ascii(){
        panic!("Fatal error : Trying to prune a non ascii String");
    }
    message.make_ascii_lowercase();
    if start_char!=0{

    }
    if end_char!=0{

    }
    return message.into_bytes();
}

fn  verify_sigma_str(message: &Vec<u8>,start_char:u8, end_char:u8,block_len:usize)-> bool{
    let mut index = 0;
    for cur_char in message {
        if start_char!=0 && index%block_len==0{
            if *cur_char!=start_char{
                return false;
            }
        }
        else if end_char!=0 && (index+1)%block_len==0 {
            if *cur_char!=end_char{
                return false;
            }
        }
            //     alphanumerical                      lowercasse letter            space
        else if (48<=*cur_char && *cur_char<=57) || (97<=*cur_char && *cur_char<=122) ||*cur_char==32{
        }
        else{
            return false
        }
        index+=1;
        
    }
    return true;
}

pub fn create_permutation<T:Digest>(rng:  &mut HashIterator::<T>,x_len:u32,_y_len:u32,perm_size:usize)-> Option<Vec<u32>>{
    let mut permutation = Vec::<u32>::with_capacity(perm_size);
    for _ in 0..perm_size{
        let next_pixel= match rng.next(){
            None => return None,
            Some(t) => t,
        };
        let (x,y,rgb)=next_pixel;
        permutation.push(3*(x+y*x_len)+rgb as u32);
    }
    return  Some(permutation);
}


///
/// 
/// permutation : set of pixel indexes
/// 
/// 
pub fn find_best_perm<T:Digest>(msg: &[u8],permutation: &Vec<u32>,
     img: &mut RgbImage,rng:  &mut SimpleHashRng<T>,mask:u8,max_iter:u32)
            ->Option<Vec<usize>>{
    let mut pixels = Vec::<u8>::with_capacity((&permutation).len());
    let mut masked_msg=Vec::<u8>::with_capacity(msg.len()*4);
    //println!("msglen : {}", msg.len());
    for char in msg{
        masked_msg.push(char&mask);
        masked_msg.push(char>>2&mask);
        masked_msg.push(char>>4&mask);
        masked_msg.push(char>>6&mask);
    }

    let (x_len,_)= img.dimensions();
    for position in permutation{
        let true_x=(position/3)%x_len;
        let true_y=(position/3)/x_len;
        
        pixels.push(mask&img.get_pixel(true_x, true_y).0[(position%3) as usize]);
    }
    let mut perm_order = Vec::<usize>::with_capacity((&permutation).len());
    for i in 0..permutation.len(){
        perm_order.push(i);
    }
    let mut best_match_count:i32=-1;
    let mut best_match_perm=None;
    for iter in 0..max_iter{
        
        let mut current_match=0;
        for index in 0..masked_msg.len(){
            if masked_msg[index]==pixels[perm_order[index]]&mask{
                current_match+=1;
            }  
        }
        if current_match>best_match_count{
            if current_match==masked_msg.len()as i32{
                return Some(perm_order);
            }
            best_match_count=current_match;
            best_match_perm=Some(perm_order.clone());
        }
        //todo!("implementer yattzle shuffle :(");
        if iter != max_iter-1{
            sigma_ficher_yates_shuffles::<T,usize>(&mut perm_order,rng);
        }
    }
    return best_match_perm;

}   
///
/// 
/// 
/// pixels : set of pixels indexes
/// permutation : the developped perm
/// 

pub fn write_best_perm(msg: &[u8],pixels:& Vec<u32>,permutation: Vec<usize>,img: &mut RgbImage,mask:u8){
    let (x_len,_)= img.dimensions();
    
    let mut masked_msg=Vec::<u8>::with_capacity(msg.len()*4);
    for char in msg{
        masked_msg.push(char&mask);
        masked_msg.push((char>>2)&mask);
        masked_msg.push((char>>4)&mask);
        masked_msg.push((char>>6)&mask);
    }
    
    for index in 0..masked_msg.len(){
        
        let position =pixels[permutation[index]as usize];
        let true_x=(position/3)%x_len;
        let true_y=(position/3)/x_len;
        let rgb=(position%3) as usize;
        let cur_pixel = img.get_pixel_mut(true_x, true_y);
        cur_pixel.0[rgb]&=!mask;
        cur_pixel.0[rgb]|=masked_msg[index]&mask;
    }
    
}

pub fn sigma_lsb_hide_msg<T:Digest>(msg :Vec<u8> ,key:&[u8],key_img : Option<&[u8]>,max_iter:u32,img : &mut RgbImage){
    let msk : u8 = 0b11;
    let mut iterator = HashIterator::<T>::new(&img,key_img);
    let mut str_msg=String::from_utf8(msg).unwrap();
    let pruned_msg = prune_sigma_str(& mut str_msg, 0, 0, 16);
    let ciphertext = sigma_encrypt_aes128(&pruned_msg, key);
    let perm_count = ciphertext.len()/16;
    let mut permutations: Vec<Vec<u32>> = Vec::<Vec<u32>>::with_capacity(perm_count);
    let (x_len,y_len)= img.dimensions();
    //println!("jpp : {} {} {} {}",x_len,y_len,x_len*y_len,perm_count);
    for _ in 0..perm_count{
        let cur_perm = match create_permutation(&mut iterator, x_len, y_len, 16*4*2){
            None => panic!("Error : image to small to hide data"),
            Some(t) => t,
        };
        permutations.push(cur_perm);
    }
    //println!("{:?}",permutations[0]);
    for i in 0..perm_count{
        //println!("{:?}",permutations[i]);
        let mut rng: SimpleHashRng<T> = SimpleHashRng::<T>::new(img_to_bytes_masked(&img,2),key_img);
        let cur_best_perm = find_best_perm::<T>(&ciphertext.as_slice()[(i)*16..(i+1)*16],&permutations[i],
        img, &mut rng,0b11,max_iter).unwrap();
        write_best_perm(&ciphertext.as_slice()[(i)*16..(i+1)*16], &permutations[i], cur_best_perm, img, msk);
    }


}



pub fn is_a_valid_perm(key: &[u8],msg_len:usize,pixels:& Vec<u32>,permutation: & Vec<usize>,img: &mut RgbImage,mask:u8)-> Option<Vec<u8>>{
    let (x_len,_)= img.dimensions();
    let mut msg = Vec::<u8>::with_capacity(msg_len);
    let mut cur_char = 0;
    let mut cd=0;
    //println!("\n reading in :");
    for index in 0..4*msg_len{
        
        let position =pixels[permutation[index]as usize];
        let x=(position/3)%x_len;
        let y=(position/3)/x_len;
        let rgb=(position%3) as usize;
        
        let val = img.get_pixel(x, y).0[rgb];
        cur_char|= (val&mask)<<(2*cd);
        //print!("pos : {}  value :  {} , ",pixels[permutation[index]as usize],val &0b11);
        
        cd+=1;
        if cd==4{
            //println!("|{}", cur_char as char);
            msg.push(cur_char);
            cd=0;
            cur_char=0;
        }
    }  
    
    let plaintext =decrypt_aes128_ecb(&msg, key);
    if verify_sigma_str(&plaintext, 0 ,0, 16){
        return Some(plaintext); 
    }
    None
        
}

///
/// 
/// Function that retrieve the message hided using sigma lsb 
/// 
/// 
fn sigma_lsb_retrieve_msg<T:Digest>(key:&[u8],key_img : Option<&[u8]>,len: u32,img : &mut RgbImage,max_count:u32)->Option<String> {
    let mask : u8 = 0b11;
    let (x_len,y_len)= img.dimensions();
    let mut iterator = HashIterator::<T>::new(&img,key_img);
    let perm_count=len as usize/16;
    let mut permutations = Vec::<Vec<u32>>::with_capacity(perm_count);
    let mut message=Vec::<u8>::new();
    for _ in 0..perm_count{
        let cur_perm = match create_permutation(&mut iterator, x_len, y_len, 16*4*2){
            None => panic!("Error : image to small to hide data"),
            Some(t) => t,
        };
        permutations.push(cur_perm);
    }
    for i in 0..perm_count{
        let permutation= &permutations[i];
        let mut perm_order = Vec::<usize>::with_capacity((&permutation).len());
        for i in 0..permutation.len(){
            perm_order.push(i);
        }
        //println!("{:?}",permutations[i]);
        let mut msg: Option<Vec<u8>> =is_a_valid_perm(key, 16, &permutation, & perm_order, img, mask);
        let mut count=1;
        //println!("oui euh   {:?}",msg);
        //println!("\n permutation2\n\n");
        let mut rng: SimpleHashRng<T> = SimpleHashRng::<T>::new(img_to_bytes_masked(&img,2),key_img);
    
        while msg.is_none() &&  count<max_count{   
            sigma_ficher_yates_shuffles::<T,usize>(&mut perm_order, &mut  rng);
            msg =is_a_valid_perm(key, 16, &permutation, & perm_order, img, mask);
            count+=1; 
        }
        //println!("count: {}",count);
        if msg.is_some(){
            message.append(&mut msg.unwrap());
            println!("Found part : {} after {} iterations",perm_count, count);
        }
        else {
            message.append(& mut vec!['?' as u8 ; 16]);
            println!("Lost part : {} ",perm_count);
        }
        
    }
    return Some(String::from_utf8(message).unwrap());

}

#[cfg(test)]
mod test_sigma{
    use image::io::Reader as ImageReader;
    use sha2::Sha512;
    use crate::utilities::encryption::keygen_aes128;

    use super::*;


    #[test]
    fn test_sigma_string(){
        let mut msg = String::from("oui - b@fonjour ceci");
        let pruned =prune_sigma_str(&mut msg, 0, 0, 16);
       
        assert!( verify_sigma_str(&pruned,0,0,16));
        
    }
    #[test]
    fn testa_sigma_encryption(){
        let mut msg = String::from("oui - b@fonjour ceci est effectivement un message omg");
        let pruned_msg = prune_sigma_str(&mut msg, 0, 0, 16);
        let key = keygen_aes128();
        let ciphertext =sigma_encrypt_aes128(&pruned_msg, &key);
        let mut plaintext = Vec::<u8>::new();
        for i in 0..ciphertext.len()/16{
            plaintext.append(&mut decrypt_aes128_ecb(&ciphertext[i*16..(i+1)*16],&key));
        }
        let txt = String::from_utf8(plaintext).unwrap();
        
        assert!(!msg.contains(&txt));
        let txt_bytes = txt.as_bytes();
        for i in 0..txt.len(){
            assert!(pruned_msg[i]==txt_bytes[i]);
        }
        

    }
    #[test]
    fn test_sigma_lsb(){
        let mut image = ImageReader::open("./unit_tests/in/default_img/maddy-pfff.png").unwrap().decode().unwrap().to_rgb8();
        let msg = String::from("oui bonjour ceci est un test je suppose ouaaaaahhh yen a des mots brefffffff fr fr");
        //let mut msg = String::from("oui bonjour ceci");
        //msg.push_str(&msg.clone());
        //msg.push_str(&msg.clone());
        
        let key = keygen_aes128();
        sigma_lsb_hide_msg::<Sha512>(msg.clone().into_bytes(),&key,None ,10000,&mut image);
        image.save("./unit_tests/out/sigma/madeline-pfff.png").unwrap();
        let rd_msg = sigma_lsb_retrieve_msg::<Sha512>(&key, None ,msg.len() as u32, &mut image, 10000).unwrap();
        println!("FIN!!!!!");

        println!("{}", rd_msg);
        println!("{}",rd_msg.len());
        assert!(msg.contains(&rd_msg));
        //println!("{}",get_mes_with_hash_pattern::<Sha512>(10000,&mut image));
       

    }
}