use std::fs::File;
use std::io::{Write, Read};
use std::path::Path;
use std::fmt::Write as Write_m;
use openssl::rand::rand_bytes;
use openssl::symm::{Cipher, decrypt, encrypt};

/// 
/// Encrypt a plaintext stored as a Vec<u8>, using the key given, with an initialisation vector equal to :
/// b"\x00\x01\x02\x03\x04\x05\x06\x07\x00\x01\x02\x03\x04\x05\x06\x07" in cbc mode. Return 
/// the ciphertext as a Vec<u8>.
/// *Panic if the key is incorect.
/// 
/// 
/// 

pub fn encrypt_aes128(plaintext:&Vec<u8>,key:&[u8]) -> Vec<u8>{
    let iv: &[u8; 16] = b"\x00\x01\x02\x03\x04\x05\x06\x07\x00\x01\x02\x03\x04\x05\x06\x07";
    let  ciphertext = encrypt(
        Cipher::aes_128_cbc(),
        key,
        Some(iv),
        &plaintext).unwrap();
    return ciphertext;
}

///
/// 
/// Decrypt a ciphertext stored as a Vec<u8>, using the key given, with an initialisation vector equal to :
/// b"\x00\x01\x02\x03\x04\x05\x06\x07\x00\x01\x02\x03\x04\x05\x06\x07" in cbc mode. Return 
/// the ciphertext as a Vec<u8>.
/// # Panics 
/// If the key is incorect.
/// 
/// 
/// 
///   
pub fn decrypt_aes128(ciphertext:&Vec<u8>, key:&[u8]) -> Vec<u8> {
    //println!("ciphertext len : {}", ciphertext.len());
    let iv = b"\x00\x01\x02\x03\x04\x05\x06\x07\x00\x01\x02\x03\x04\x05\x06\x07";
    let  plaintext = decrypt(
        Cipher::aes_128_cbc(),
        key,
        Some(iv),
        &ciphertext).unwrap();

    return  plaintext;
    
}
pub fn decrypt_aes128_ecb(ciphertext:&[u8], key:&[u8]) -> Vec<u8> {
    
    let iv = b"\x00\x01\x02\x03\x04\x05\x06\x07\x00\x01\x02\x03\x04\x05\x06\x07";
    let  plaintext = decrypt(
        Cipher::aes_128_ecb(),
        key,
        Some(iv),
        &ciphertext).unwrap_or(vec!['!' as u8; 16]);
    return  plaintext;
    
}
pub fn sigma_encrypt_aes128_old(plaintext:&Vec<u8>,key:&[u8])->Vec<u8>{
    let iv: &[u8; 16] = b"\x00\x01\x02\x03\x04\x05\x06\x07\x00\x01\x02\x03\x04\x05\x06\x07";
    let mut ciphertexts= Vec::<u8>::with_capacity(plaintext.len()+16);
    ciphertexts.append(
            &mut encrypt(
                Cipher::aes_128_ofb(),
                key,
                Some(iv),
                &plaintext).unwrap()
    );
        //println!("Len : {} ",ciphertexts.len());
    
    //let  ciphertext = 
    return ciphertexts;
}

pub fn sigma_encrypt_aes128(plaintext:&Vec<u8>,key:&[u8])->Vec<u8>{
    let iv: &[u8; 16] = b"\x00\x01\x02\x03\x04\x05\x06\x07\x00\x01\x02\x03\x04\x05\x06\x07";
    let mut ciphertexts= Vec::<u8>::with_capacity(plaintext.len()+16);
     for i in 0..plaintext.len()/15{
        //println!("euuuuhhh : {}",&plaintext[15*i..15*(i+1)].len());
        ciphertexts.append(
            &mut encrypt(
                //Cipher::aes_128_ofb(),
                Cipher::aes_128_ecb(),
                key,
                Some(iv),
                &plaintext[15*i..15*(i+1)]).unwrap()
        );
        //println!("Len : {} ",ciphertexts.len());
    }
    //let  ciphertext = 
    return ciphertexts;
}
pub fn prepare_sigma_decrypt_aes128(number_of_blocks:usize,key:&[u8])-> Vec<Vec<u8>>{
    let iv = b"\x00\x01\x02\x03\x04\x05\x06\x07\x00\x01\x02\x03\x04\x05\x06\x07";
    let  mut cyphered_ivs= Vec::<Vec<u8>>::with_capacity(number_of_blocks);
    let mut last_iv=iv.as_slice();
    let  mut ciphertext;
    for _ in 0..number_of_blocks {
        ciphertext = encrypt(
            Cipher::aes_128_cbc(),
            key,
            None,
            last_iv).unwrap();
        last_iv=ciphertext.as_slice();
        cyphered_ivs.push(ciphertext.clone());         
    }
    return cyphered_ivs;
}
pub fn sigma_decrypt_aes128(ciphertext:&[u8],iv:&Vec<u8>) -> Vec<u8> {
    let mut plaintext= Vec::<u8>::with_capacity(16);
    println!("cipertext: {:?}",ciphertext);
    for i in 0..ciphertext.len(){
        plaintext.push(ciphertext[i]^iv[i]);
    }
    println!("!plaintext: {:?}", plaintext);
    return  plaintext;
    
}

///
/// Quickly generate a key for aes 128 encryption scheme.
/// Use cryptographicaly strong pseudo-random.
/// 

pub fn keygen_aes128()-> [u8;16]{
    let mut key :[u8;16] =[0;16];
    rand_bytes(&mut key).unwrap();
    return key;
}

///
/// Write an aes 128 key in a file. Only used for tests.
/// # Panics 
/// On IO error
/// 
/// 
pub fn key_write_aes128(key:&[u8],filename:&str){
    let path = Path::new(&filename);
    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't create {}: {}", filename, why),
        Ok(file) => file,
    };
    //let mut key_hex= [0;32];
    let mut key_hex =std::string::String::new();
    for byte in key{
        std::write!(&mut key_hex, "{:0>2X}",byte).unwrap();

    }
    file.write(key_hex.as_bytes()).unwrap();
}
///
/// Read an aes 128 key in a file, stored in the same format used in [`key_write_aes128`] Only used for tests.
/// # Panics 
/// On IO error
/// 
/// 
pub fn key_read_aes128(filename:&str) -> [u8;16]{
    let mut key = [0;16];
    let path = Path::new(&filename);
   
    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", filename, why),
        Ok(file) => file,
    };
    let mut buffer= [0;32];
    let mut read_bytes=1;
    while read_bytes!=0 {
        read_bytes=file.read(&mut buffer).unwrap();    
    }
    for i in 0..16{
        let mut s_buffer =[0;2];
        s_buffer[0]=buffer[2*i];
        s_buffer[1]=buffer[2*i+1];
        let str_buff=std::str::from_utf8(&s_buffer).unwrap();
        key[i]=u8::from_str_radix(&str_buff, 16).unwrap();
    }
    return key;
}

pub fn general_key_read(filename:&str) -> Vec<u8>{
    let mut key =Vec::<u8>::new();
    let path = Path::new(&filename);
   
    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", filename, why),
        Ok(file) => file,
    };
    let mut buffer= Vec::new();
    let read_bytes=1;
    while read_bytes!=0 && read_bytes<buffer.len(){
        file.read_to_end(&mut buffer).unwrap();
           
    }
    for i in 0..read_bytes/2{
        let mut s_buffer =[0;2];
        s_buffer[0]=buffer[2*i];
        s_buffer[1]=buffer[2*i+1];
        let str_buff=std::str::from_utf8(&s_buffer).unwrap();
        key[i]=u8::from_str_radix(&str_buff, 16).unwrap();
    }
    return key;
}


#[cfg(test)]
mod test_encryption{
    
    use super::*;
    #[test]
    fn test_encryption(){
        for _ in 0..10{
            let key = keygen_aes128();
            println!("key generated");
            let message :Vec<u8>= vec!(1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20
                ,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20
                ,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19);
            let ciphertext=encrypt_aes128(&message, &key);
            println!("message encrypted");
            key_write_aes128(&key, "aes_key");
            println!("key save");
            let key2 = key_read_aes128("aes_key");
            println!("key red");
            let decrypt = decrypt_aes128(&ciphertext, &key2);
            assert_eq!(message,decrypt)
        }
    }
    #[test]
    fn test_sigma_encryption(){
        let key = keygen_aes128();
        println!("key generated");
        let message :Vec<u8>= vec!(1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20
            ,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20
            ,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,1,2,3,4,5);
        println!("msg len: {}" ,message.len());
        let ciphertext=sigma_encrypt_aes128_old(&message, &key);
        let blocks_count = ciphertext.len()/16;
        let ivs= prepare_sigma_decrypt_aes128(blocks_count, &key);
        let mut recovered_message=Vec::<u8>::new();
        for i in 0..blocks_count{
            let cur_ciphertext = &ciphertext[16*i..16*(i+1)];
            let iv = &ivs[i];
            let mut cur_block=sigma_decrypt_aes128(cur_ciphertext, iv);
            recovered_message.append(&mut cur_block);
        }
        println!("message: {:?}",message);
        println!("recovered message: {:?}",recovered_message);
        assert_eq!(message,recovered_message);

    }
}