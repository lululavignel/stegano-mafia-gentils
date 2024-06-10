pub mod utilities;
pub mod algo_modifiers;
pub mod stega_key_static;
pub mod lsb;
pub mod sigma_lsb;
pub mod pvd;
pub mod annalyser;
pub mod viterbi;
pub mod hugo;
//pub mod edges_iterator;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::num::ParseIntError;
use std::process::Command;
use std::time::SystemTime;

use algo_modifiers::histo_transformation;
use algo_modifiers::position_by_hash;
use image::io::Reader as ImageReader;
use clap::{Arg,App};
use image::RgbImage;
use lsb::ImgIterator;
use lsb::{ASCIIEncoding,CharEncoding,DefaultIterator,lsb_hide_msg,lsb_retrieve_msg};
use position_by_hash::{*};

use pvd::pvd_3by2_hide_msg;
use pvd::pvd_3by2_retrieve_msg;
use show_image::exit;
use stega_key_static::key_read;

use utilities::encryption::decrypt_aes128;
use utilities::encryption::encrypt_aes128;
use utilities::encryption::key_read_aes128;

use histo_transformation::histo_unzip;
use histo_transformation::histo_zip;
use utilities::encryption::general_key_read;
use viterbi::viterbi_hide_msg;

//use imageproc::stats;


const I_O_NAME: &str="input image> <output";
const HIDE: &str="hiding algorithm";
const FIND: &str="finding algorithm";
const LSB: &str="LSB algorithm";
const PVD: &str="PVD algorithm";
const VITERBI: &str="Viterbi algorithm";
const IMGIT:&str="LSB image iterator";
const CHAR_ENCOD: &str="LSB char encoding";
const TEXT_IN: &str="text message";
const TEXT_LEN: &str="text length";
const CIPHER: &str="AES key file";
const IMG_PERCENT: &str = "percentage of image";
const IMG_KEY: &str = "permutation key";
const ZIP: &str = "zip number (8-12)";
const H_MAT: &str ="matrix";
const HEIGHT: &str ="matrix-height";
const ALPHA: &str ="matrix-length";



fn main() {
    let  mut app = App::new("Stegano-tools")
        .version("0.4.0")
        .author("Stegano-mafia-méchants")
        .about("Tools to hide/retrieve text inside rgb8 pictures.")
        .usage("\tsteg -w [-l|v|d] [-i input-image output-image] [-t message-file] <-s message-length|-p percentage-of-image> <-e encoding> <-g iterator> <-c key> <--img-key key-filename> <-z zip value>\n\t\
                steg -r [-l|v|d] [-i input-image output-text] [-s message-length] <-e encoding> <-g iterator> <-c key> <--img-key key-filename> <-z zip value>")
        .arg(Arg::with_name(I_O_NAME)
                 .short('i')
                 .long("io")
                 .help("input image name and output name (image or text, depending of the action)")
                 .takes_value(true)
                 .min_values(2)
                 .max_values(2))
        .arg(Arg::with_name(HIDE)
                 .short('w')
                 .long("hiding-algorithm")
                 .takes_value(false)
                 .help("Choose the algorithm to use to hide data. Multiple alogrithm can be used at the same time")
                 //.possible_values(&["lsb","hash","key"])
                 .conflicts_with(FIND))
        .arg(Arg::with_name(FIND)
                 .short('r')
                 .long("retrieving-algorithm")
                 .takes_value(false)
                 .help("Choose the algorithm to use to retrieve data. Multiple alogrithm can be used at the same time")
                 //.possible_values(&["lsb","hash","key"])
                 .conflicts_with(HIDE))
        .arg(Arg::with_name(LSB)
                 .short('l')
                 .long("lsb")
                 .takes_value(false)
                 .help("Choose the LSB algorithm. Extension of the classic LSB algorithm can be provided using options"))
        .arg(Arg::with_name(PVD)
                 .short('d')
                 .long("pvd")
                 .takes_value(false)
                 .help("Choose the PVD algorithm. Extension of the classic PVD algorithm can be provided using options"))
        .arg(Arg::with_name(VITERBI)
                 .short('v')
                 .long("viterbi")
                 .takes_value(false)
                 .help("Choose the Viterbi algorithm. Extension of the classic viterbi algorithm can be provided using options.\
                        Has additionnal option allowing to pass a arbitrary matrix using: <--mat mat --h_mat matrix-height>\
                        or select an optimal matrix of a given size proividing : <--h_mat matrix-height --alpha alpha>. See help viterbi for more detailed informations."))
        .arg(Arg::with_name(IMGIT)
                 .short('g')
                 .long("img-iterator")
                 .takes_value(true)
                 .help("Choose the iterator algorithm")
                 .possible_values(&["md5","sha256","sha512","sha224","sha384" ,"sha3-512"])
                 .multiple(false))
        .arg(Arg::with_name(CHAR_ENCOD)
                 .short('e')
                 .long("using-encoding ")
                 .takes_value(true)
                 .help("Choose the LSB algorithm. Extension of the classic LSB algorithm can be provided using arguments")
                 .possible_values(&["s2409key"])
                 .multiple(true))
        .arg(Arg::with_name(IMG_KEY)
                 .long("img-key")
                 .takes_value(true)
                 .help("Name of the text file containing the key used to secure the order of the modified pixels")
                 .multiple(false))
        .arg(Arg::with_name(TEXT_IN)
                .short('t')
                .long("text-file")
                .takes_value(true)
                .max_values(1)
                .help("Name of the text file to hide inside the image. Requires hiding a message.")
                .requires(HIDE))
        .arg(Arg::with_name(TEXT_LEN)
                .short('s')
                .long("text-len")
                .takes_value(true)
                .max_values(1)
                .conflicts_with(IMG_PERCENT)
                .help("The number of characters to read or to write inside the image. If it is superior to the length of the text, or to the number of char that can be hidden inside the image, skip it "))
        .arg(Arg::with_name(IMG_PERCENT)
                .short('p')
                .long("img-per")
                .takes_value(true)
                .max_values(1)
                .conflicts_with(TEXT_LEN)
                .help("The percentage of the image max capacity that will be used to hide the message.\
                 For exemple, if the image has a size of 100*100, it has a capacity of 100*100*3*2 bits, e.g. 7500 bytes,\
                  and if we use 50% of the capacity we have 3750 available bytes. The value entered must be >0 and <1 "))
        .arg(Arg::with_name(CIPHER)
                .short('c')
                .long("AES")
                .takes_value(true)
                .max_values(1)
                .help("Enable encrypting/decrypting messages using the key given"))       
        .arg(Arg::with_name(ZIP)
            .short('z')
            .long("zip")
            .takes_value(true)
            .max_values(1)
            .help("Zip the image before any atempt to read it or write it"))
        .arg(Arg::with_name(H_MAT)
            .long("h-mat")
            .takes_value(true)
            .max_values(1)
            .help("Matrix used to hide/recover the message. Reserved to viterbi algorithm. 2d Matrix given as a 1d array, with a <<,>> between numbers. It is only and 1-dim vector, because all column are represented as an integer  where the most significant bit is the bit inside last line. Format : 3,1,2"))
        .arg(Arg::with_name(HEIGHT)
            .long("height")
            .takes_value(true)
            .max_values(1)
            .help("The height of the matrix"))
        .arg(Arg::with_name(ALPHA)
            .long("alpha")
            .takes_value(true)
            .max_values(1)
            .help("The lenght of the matrix. The stego max capacity is divided by this number, but usually the greater the number the lesser bits are modified"));       
    
    let matches = app.clone().get_matches();
    let before = SystemTime::now();
    if matches.is_present(FIND) {
        read_message(& matches);
    }
    else if matches.is_present(HIDE) {
        write_message(& matches);
    }
    
    else {
        app.print_help().unwrap();
        return;
    }
    let now = SystemTime::now().duration_since(before).expect("get millis error");
    println!("program took {} ms to run",now.as_millis());
    

}
///
/// 
/// Case where the user ask to read message from an image
/// 
/// 
fn read_message(matches: &clap::ArgMatches){
   
    if ! matches.is_present(LSB)
        && ! matches.is_present(PVD)
        && ! matches.is_present(VITERBI){ 
       eprintln!("ERROR: NO ALGORITHM GIVEN");
       exit(10);
    }
    println!("Searching a message inside an image...");
    let msg =get_steg_msg(matches);
    //image.save(image_filename).unwrap();

    let mut imagename = matches.values_of(I_O_NAME).unwrap();
    imagename.next();
    let text_filename= imagename.next().unwrap();
    let mut file = match File::create(text_filename) {
        Err(why) => panic!("couldn't open {}: {}", text_filename, why),
        Ok(file) => file,
    };
    file.write_all(msg.as_bytes()).unwrap();
}

fn get_steg_msg(matches: &clap::ArgMatches)->String{
    let mut imagename = matches.values_of(I_O_NAME).unwrap();
    let mut image = ImageReader::open(imagename.next().unwrap()).unwrap().decode().unwrap().to_rgb8();
    match get_zip(matches){
        None => (),
        Some(v) => {
            println!("ziping the image...");
            histo_zip(&mut image,v);
        }
    }

    let char_count= match matches.value_of(TEXT_LEN){
        None => 1000,
        Some(t) => match t.parse::<i32>(){
            Ok(v) =>v,
            Err(e) => |e: ParseIntError  | -> i32 { println!("Warning, {} is not a number, using default text length instead",e);return 1000} (e),
        }
    };
    let res = get_img_key(matches);
    let img_key = match &res {
        Some (t) => Some(t.as_slice()),
        None =>None,
    };
    let encoding = get_char_encoding_from_cli(&matches);
    let mut res = match matches.value_of(IMGIT){
        None =>lsb_retrieve_msg::<DefaultIterator>(char_count,encoding.as_ref(),img_key,&mut image),
        Some(t) =>match t{
            "md5" =>    match_steg_msg::<HashIterator<md5::Md5>>(matches,char_count,encoding.as_ref(),img_key,&mut image),
            "sha256" => match_steg_msg::<HashIterator<sha2::Sha256>>(matches,char_count,encoding.as_ref(),img_key,&mut image),
            "sha512" => match_steg_msg::<HashIterator<sha2::Sha512>>(matches,char_count,encoding.as_ref(),img_key,&mut image),
            "SHA3-512"=>match_steg_msg::<HashIterator<sha3::Sha3_512>>(matches,char_count,encoding.as_ref(),img_key,&mut image),
            //"edges" =>  lsb_retrieve_msg::<EdgesIterator<true>>(char_count,encoding.as_ref(),&mut image),
            &_ =>       match_steg_msg::<DefaultIterator>(matches,char_count,encoding.as_ref(),img_key,&mut image),
        
        }
        
    };
    let cipher: Option<&str> =  matches.value_of(CIPHER);
    if cipher.is_some(){
        let key_file= cipher.unwrap();
        let key = key_read_aes128(key_file);
        let ciphertext=decrypt_aes128(&res, &key);
        res = ciphertext;
        println!("decrypting...");
        
    }
    return String::from_utf8_lossy(&res).to_string();
    
}

fn match_steg_msg<T:ImgIterator>(matches: &clap::ArgMatches,length :i32 , encoding: & dyn CharEncoding, img_key: Option<&[u8]>,img : &mut RgbImage) -> Vec<u8>{
    if matches.is_present(LSB){
        return lsb_retrieve_msg::<T>(length , encoding, img_key,img);
    }
    else if matches.is_present(PVD){
        return pvd_3by2_retrieve_msg::<T>(img_key, length as usize,img);
    }
    else {
        eprintln!("NO algorithm given :(");
        panic!();
    }
}

fn hide_steg_msg(matches: &clap::ArgMatches){
    let mut imagename = matches.values_of(I_O_NAME).unwrap();
    let mut image = ImageReader::open(imagename.next().unwrap()).unwrap().decode().unwrap().to_rgb8();
    let image_filename= imagename.next().unwrap();
    println!("Opened original image : {} ...",image_filename);
    match get_zip(matches){
        None => (),
        Some(v) => {
            println!("ziping the image...");
            histo_zip(&mut image,v);
        }
    }

    let text_filename: &str= matches.value_of(TEXT_IN).unwrap();
    let mut file = match File::open(text_filename) {
        Err(why) => panic!("couldn't open {}: {}", text_filename, why),
        Ok(file) => file,
    };
    
    let mut buffer: Vec<u8> = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    if matches.is_present(TEXT_LEN){
        match matches.value_of(TEXT_LEN){
            None => (),
            Some(t) => match t.parse::<usize>(){
                Ok(v) =>|v:usize| ->() {println!("v: {}",v);buffer.truncate(v);} (v),
                Err(e) => println!("Warning, {} is not a number, using default text length instead",e),
            }
        };
    }
    else if matches.is_present(IMG_PERCENT){
        let percent_arg=matches.value_of(IMG_PERCENT);
        if percent_arg.is_none(){
            eprintln!("Error : no value provided for the image percent");
            return;
        }
       
        let mut percentage=-1.0;
        match matches.value_of(IMG_PERCENT){
            None => (),
            Some(t) => match t.parse::<f32>(){
                Ok(v) => percentage=v,
                Err(e) => eprintln!("Error, the parameter given is not a float : {}",e),
            }
        };
        if percentage>=1.0 || percentage<=0.{
            eprintln!("Warning, {} is not a number or is not between 0 and 1 (both excluded)",percentage);
            return;
        }
        let dimensions=image.dimensions();
        let total_bytes = ((dimensions.0*dimensions.1*3*2)/8) as f32;
        let used_bytes = (total_bytes*percentage) as usize;
        buffer.truncate(used_bytes);

    }

   println!("number of char to write:{}",buffer.len());
   let dim =image.dimensions();
   println!("number of bytes writable : {}",(dim.0*dim.1*3*2)/8) ;

    

    //let txt = std::str::from_utf8(&buffer[..]).unwrap();
    //println!("Message readed :\n\n{}\n\n", txt);
    let res =  matches.value_of(CIPHER);
    if res.is_some(){
        let key_file= res.unwrap();
        let key = key_read_aes128(key_file);
        let ciphertext=encrypt_aes128(&buffer, &key);
        buffer = ciphertext;
        
    }
    let res =  get_img_key(matches);
    let img_key = match &res {
        Some (t) => Some(t.as_slice()),
        None =>None,
    };
    
    let encoding: Box<dyn CharEncoding> = get_char_encoding_from_cli(&matches);
    println!("Hiding the message.....");
    match matches.value_of(IMGIT){
        None =>lsb_hide_msg::<DefaultIterator>(&buffer , encoding.as_ref(),img_key, &mut image),
        Some(t) =>match t{
            "md5" =>write_img::<HashIterator<md5::Md5>>(matches,&buffer , encoding.as_ref(), img_key,&mut image),
            "sha512" =>write_img::<HashIterator<sha2::Sha512>>(matches,&buffer , encoding.as_ref(),img_key, &mut image),
            "sha256" =>write_img::<HashIterator<sha2::Sha256>>(matches,&buffer , encoding.as_ref(), img_key,&mut image),
            "SHA3-512" =>write_img::<HashIterator<sha3::Sha3_512>>(matches,&buffer , encoding.as_ref(),img_key, &mut image),
            //"edges"  =>lsb_hide_msg::<EdgesIterator<true>>(&txt , encoding.as_ref(), &mut image),
            &_ =>write_img::<DefaultIterator>(matches,&buffer , encoding.as_ref(),img_key, &mut image),
        }
      
    };
    match get_zip(matches){
        None => (),
        Some(v) => {
            println!("ziping the image...");
            histo_unzip(&mut image,v);
        }
    };
    image.save(image_filename).unwrap();
}
fn write_img<T:ImgIterator>(matches: &clap::ArgMatches,msg :&Vec<u8> , enconding: & dyn CharEncoding, img_key: Option<&[u8]>,img : &mut RgbImage){
    if matches.is_present(LSB){
        lsb_hide_msg::<T>(&msg , enconding, img_key,img);
    }
    else if matches.is_present(PVD){
        pvd_3by2_hide_msg::<T>(&msg ,img_key,img);
    }
    else if matches.is_present(VITERBI){
        let hide_len;
        if ! matches.is_present(TEXT_LEN){
            hide_len = true;
        }
        else{
            hide_len = false;
        }
        let def_matrix= vec![0_usize];
        let h_mat =match matches.value_of(H_MAT){
            None=> None,
            //todo
            Some(t) =>Some(&def_matrix),
        };
        let h= match matches.value_of(HEIGHT) {
            None=> match h_mat{
                Some(t) => t.len() as u32,
                None => 2,
            },
            Some(t) => u32::from_str_radix(t, 10).unwrap(),
        };
        let alpha = match matches.value_of(ALPHA) {
            None => Some(h_mat.unwrap_or(&vec![3_usize,1]).len()),
            Some(t) => Some(usize::from_str_radix(t, 10).unwrap()),
        };
        viterbi_hide_msg::<T>(msg, enconding, img_key, h_mat, h, alpha, hide_len, img)
    }
    else {
        eprintln!("NO algorithm given :(");
    }
}

///
/// 
/// Case where the user ask to read message from an image
/// 
/// 

fn write_message(matches: &clap::ArgMatches){
    if ! matches.is_present(LSB)
        && ! matches.is_present(PVD)
        && ! matches.is_present(VITERBI){ 
       eprintln!("ERROR: NO ALGORITHM GIVEN");
       exit(10);
    }
    println!("Hiding a message inside an image...");
    hide_steg_msg(matches);
}

fn get_char_encoding_from_cli(matches: &clap::ArgMatches) ->  Box<dyn CharEncoding>{
    let encoding= Box::new(ASCIIEncoding);
    let encoding_params =matches.values_of(CHAR_ENCOD);
    if encoding_params.is_none(){
        return  encoding;
    }
    let encoding_params=encoding_params.unwrap();
    for value in encoding_params.clone().into_iter(){
        let mut value: String =String::from(value);
        value.make_ascii_lowercase();
        match &value[..] {
            "s2409key" => return Box::new(key_read(&String::from("key"))),
            //"sha256" => |it :&mut dyn ImgIterator| ->() {it= HashIterator<Sha256>;}(iterator),
            &_=>(),
        } 
    }
    return encoding;
}

fn get_img_key(matches: &clap::ArgMatches)-> Option<Vec<u8>>{
    let res =  matches.value_of(IMG_KEY);
    if res.is_some(){
        let key_file= res.unwrap();
        return Some(general_key_read(key_file));
    }
    return  None;
}
 fn get_zip (matches: &clap::ArgMatches)-> Option<u8>{
    let res =  matches.value_of(ZIP);
    if res.is_some(){
        let zip_str= res.unwrap();
        let zip_value = zip_str.parse::<u8>();
        match zip_value {
            Ok(v) => return Some(v),
            Err(e) => {
                eprintln!("Error : unable to read zip's value :");
                eprintln!("{}",e);
                eprintln!("Exiting");
                exit(9);
            }
        }

    }
    return  None;
 }