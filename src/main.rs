pub mod utilities;
pub mod algo_modifiers;
pub mod stega_key_static;
pub mod lsb;
pub mod sigma_lsb;
pub mod pvd;
pub mod annalyser;
pub mod viterbi;
pub mod hugo;
pub mod main_annalyser;
pub mod rs;
pub mod shannon_entropy;
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




use image::{ImageBuffer, Rgb};
use statrs::distribution::{Binomial, ChiSquared, ContinuousCDF, Discrete};

use crate::{rs::{default_compute_rs, defautl_dyn_rs}, shannon_entropy::{compute_shanon_entropy, entropy_and_randomization_after_first_measure, init_base_shanon_iterator}};

//use crate::test_procedure::test_test_proc::run_test;





const IM_TRANS: &str="image operation";
const TRANS_LSB: &str="lsb";
const TRANS_DELTA: &str="img-delta";
const RUN_ALL_TESTS_FOLDER: &str="test folder";
const OUTPUT_FILE: &str = "output-file";
pub(crate) fn main() {
    let mut  app = App::new("Stegano-visuals-tools")
        .version("0.0.3")
        .author("Stegano-police")
        .about("Tools to find hiden messages inside image")
        .usage("\tsteg [-t image-op-type] [-i input-image output-image] \n\t
                \tsteg [-d directory percentage-of-max-capacity]\n\t")
                //steg -r [-l] [-i input-image output-text] [-s message-length] <-e encoding> <-g iterator> <-c key>")
        .arg(Arg::with_name(I_O_NAME)
                 .short('i')
                 .long("io")
                 .help("input image name and output name")
                 .takes_value(true)
                 .min_values(2)
                 .max_values(2))
        .arg(Arg::with_name(IM_TRANS)
                 .short('t')
                 .long("image-transfo")
                 .takes_value(true)
                 .help("Choose the image transformation used")
                 .possible_values(&[TRANS_LSB,TRANS_DELTA])
                 .min_values(1)
                 .max_values(1))
        .arg(Arg::with_name(RUN_ALL_TESTS_FOLDER)
                .short('d')
                .long("test-file")
                .takes_value(true)
                .help("Test an file. Chose an algo. if shanon give mask. The percentage given is used in logs (./output).")
                .min_values(2)
                .max_values(3))
        .arg(Arg::with_name(OUTPUT_FILE)
            .short('o')
            .long("output")
            .takes_value(true)
            .help("Specify the output file for results"));
    
    let matches = app.clone().get_matches();
    let img_in;
    let img_out_filename;
    
    if matches.is_present(RUN_ALL_TESTS_FOLDER) {
        let mut opts = matches.values_of(RUN_ALL_TESTS_FOLDER).unwrap();
        let img_name=  opts.next().unwrap();
        let algo = opts.next().unwrap();
        let mask = opts.next().or(Some("2")).unwrap().parse::<u8>().unwrap();
        println!("dir name : {}",img_name);
        let img = ImageReader::open(img_name).unwrap().decode().unwrap().to_rgb8();

        let sha_fx = | | -> Option<f64>{let mut it=init_base_shanon_iterator(&img,0,0 ,img.width(),img.height(),mask);
                let res=compute_shanon_entropy(&img,&mut it);
                Some(res.0)};
        let rand_sha_fx = | | -> Option<f64>{
            let mut it=init_base_shanon_iterator(&img,0,0 ,img.width(),img.height(),mask);
            let resmin=compute_shanon_entropy(&img,&mut it);
            let mut itmax=init_base_shanon_iterator(&img,0,0 ,img.width(),img.height(),255);
            let resmax=compute_shanon_entropy(&img,&mut itmax);
            let res=entropy_and_randomization_after_first_measure(&img,resmax,resmin,0.1,mask);
            Some(res)};
        let result =match algo{
            "rs" => default_compute_rs(&img),
            "dynrs" => defautl_dyn_rs(&img),
            "shanon" => sha_fx(),
            "randshanon" => rand_sha_fx(),
            _=> None,
        };
        let output_file = matches.value_of(OUTPUT_FILE).unwrap_or("./output");
        let mut file =File::options().create(true).truncate(true).write(true).open(output_file).unwrap();
        if result.is_some(){
            let str = format!("{}",result.unwrap());
            file.write(str.as_bytes());
        }
        
        //run_test(dir_name.to_string(),percentage);
 
    }
    else if matches.is_present(IM_TRANS) {
        let mut images_name=match matches.values_of(I_O_NAME) {
            Some(t) => {t},
            None => {panic!("Error, mising arguments for image input or output")},
        };
        img_in = ImageReader::open(images_name.next().unwrap()).unwrap().decode().unwrap().to_rgb8();
        img_out_filename= match images_name.next() {
            Some(t) => {t},
            None => {panic!("Error : Mising the image output name")},
        };

        let trans_type= match matches.value_of(IM_TRANS){
            Some(t) => {t},
            None => {panic!("Error, mising arguments for image transformation")},
        };
        match trans_type {
            TRANS_DELTA =>  {diff_img_create(&img_in, 2).save(img_out_filename).unwrap()}
            TRANS_LSB =>    {only_lsb(&img_in).save(img_out_filename).unwrap()}
            _ => {panic!("Error : bad tranformation operation")}
        }
        
    }
    else {
        app.print_help().unwrap();
    }

}

fn khi_squared_analysis_windowed(img : & RgbImage, bytes_to_analyse : u32,windows_size :i32)-> f64{ //TODO rgb *3 ?

    //Pour le test de conformité, degres_liberte = nb_categories-1
    let pixel_count=(windows_size*windows_size) as usize;
    let degres_liberte: f64 = (pixel_count*bytes_to_analyse as usize ) as f64;
    let binom = Binomial::new(0.5, (pixel_count*bytes_to_analyse as usize )as u64).unwrap();
    

    //Initialiser le vecteur attendu pour le test (répartition homogène des pixels)
    let (length, height) = img.dimensions();
    let nb_pixels: u32 = 3 * (length-(1+windows_size as u32/2)) * (height-(1+windows_size as u32/2));
    //let repartition_pixels = nb_pixels as f64 /categories as f64;

   
    let mut frequence_pixel: Vec<u32> = vec![0; pixel_count*2+1]; //TODO probabilités d'obtenir chaque lettre
    //println!("nombre de pixels attendus pour chaque catégorie : {}",frequence_pixel_attendue[0]);

    //Générer le vecteur de la répartition des pixels de l'image
    
    for i in windows_size/2..length as i32-(windows_size+1)/2{
        for j in windows_size/2..height as i32-(windows_size+1)/2{
            
            let mut bit_count :[usize;3]=[0,0,0];

            for d_i  in -windows_size/2..(windows_size+1)/2{
                for d_j in -windows_size/2..(windows_size+1)/2{
                    //println!("di : {} dj: {}",d_i,d_j);
                    let pixel = img.get_pixel((i+d_i) as u32, (j+d_j) as u32);
                    let mut index=0;
                    for val in pixel.0{
                        bit_count[index]+= (val & 0x1) as usize;
                        bit_count[index]+= ((val>>1) & 0x1) as usize;
                        index+=1;
                        //println!("val: {} , {}",val, bit_count);
                    }
                }   
            }
            for count in bit_count{
                frequence_pixel[count]+=1;
            }
        }   
    }

    //Calcul pour comparaison entre le vecteur attendu et observé
    let mut sum_khi = 0.0;
    let mut sum_got=0.0;
    let mut sum_expected=0.0;
    for i in 0..pixel_count*2 +1{
        let expected_freq=nb_pixels as f64 *binom.pmf(i as u64);
        let diff = frequence_pixel[i as usize] as f64  - expected_freq;
        sum_expected+= expected_freq;
        sum_got+= frequence_pixel[i as usize] as f64;
        println!("diff : {} , nbr of pixels : {}, expected : {}",diff, frequence_pixel[i as usize],expected_freq);

        let squared_diff =diff *diff;
        let result = squared_diff / expected_freq;
        println!("{:.4}", result);
        sum_khi += result;
        println!("i:  {:?}     sum_khi:{:?}", i, sum_khi);
    }
    println!("pixel get : {} , expected: {}",sum_got,sum_expected);
    //Calcul de l'indice de confiance graĉe à khi carré
    let khi_squared = ChiSquared::new(degres_liberte).unwrap();
    //let khi_result_pdf = Continuous::pdf(&khi_squared, sum_khi); //Formule : 1 / (2^(k / 2) * Γ(k / 2)) * x^((k / 2) - 1) * e^(-x / 2)   (source : documentation de statrs::distribution::ChiSquared)
    //let khi_result_pdf = khi_squared.pdf(sum_khi);
    //let khi_result_cdf = ContinuousCDF::cdf(&khi_squared, sum_khi); //Formule : (1 / Γ(k / 2)) * γ(k / 2, x / 2) where k is the degrees of freedom, Γ is the gamma function, and γ is the lower incomplete gamma function     (source : documentation de statrs::distribution::ChiSquared)
    let khi_result_cdf = khi_squared.cdf(sum_khi/((nb_pixels as f64).log2()*(nb_pixels as f64).log2()));
    //println!("khi pdf:  {:.8}", khi_result_pdf);
    println!("khi cdf:  {:.8}", khi_result_cdf);

    return khi_result_cdf;
}


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
fn only_lsb(img: &RgbImage) -> RgbImage{
    let (x,y)=img.dimensions();
    let mut new_img: RgbImage = ImageBuffer::new(x, y);
    for (x,y,pixel) in img.enumerate_pixels(){
        let new_pixel=new_img.get_pixel_mut(x, y);
        new_pixel.0[0]=(pixel.0[0]&0x3)<<6;
        new_pixel.0[1]=(pixel.0[1]&0x3)<<6;
        new_pixel.0[2]=(pixel.0[2]&0x3)<<6;
        
    }
    return new_img;
}
#[cfg(test)]
mod test_annalyzer{
    

    use super::*;
    use image::io::Reader as ImageReader;
    #[test]
    fn test_key_io(){
        let image1 = ImageReader::open("./photo.jpg").unwrap().decode().unwrap().to_rgb8();
        let image2 = ImageReader::open("./steg_with_key.jpg").unwrap().decode().unwrap().to_rgb8();
        let image3 = ImageReader::open("./steg.jpg").unwrap().decode().unwrap().to_rgb8();
        
        let res1 = stats_last_bits(&image1,2);
        let res2 = stats_last_bits(&image2,2);
        let res3 = stats_last_bits(&image3,2);
        println!("original:              {:?}", res1);
        println!("modified:              {:?}", res2);
        println!("modified without key:  {:?}", res3);
    }

    #[test]
    fn test_only_lsb(){
        let image= ImageReader::open("./tests/results/celeste-3-n-n-c.png").unwrap().decode().unwrap().to_rgb8();
        only_lsb(&image).save("./tests/results/celeste-3-n-n-c-lsb_only.png").unwrap();
        /*test_only_lsb_with_name("celeste-3-1.0-n-n-c.png");
        test_only_lsb_with_name("celeste-3-0.5-n-n-c.png");
        test_only_lsb_with_name("celeste-3-0.3-n-n-c.png");
        test_only_lsb_with_name("celeste-3-0.2-n-n-c.png");
        test_only_lsb_with_name("celeste-3-0.1-n-n-c.png");*/

        test_only_lsb_with_name("celeste-3-1.0-sha2-n-c.png");
        test_only_lsb_with_name("celeste-3-0.5-sha2-n-c.png");
        test_only_lsb_with_name("celeste-3-0.3-sha2-n-c.png");
        test_only_lsb_with_name("celeste-3-0.2-sha2-n-c.png");
        test_only_lsb_with_name("celeste-3-0.1-sha2-n-c.png");
    }
    fn test_only_lsb_with_name(cur_str:&str){
        let image= ImageReader::open("./tests/results/".to_owned()+cur_str).unwrap().decode().unwrap().to_rgb8();
        diff_img_create(&image, 2).save("./img/photo/trnsfrm/".to_owned()+ cur_str).unwrap();
    }
    #[test]
    fn try_img_delta(){
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
        let cur_str: &str="vile-foret-hash512.png";
        let image= ImageReader::open("./img/photo/secret/".to_owned()+cur_str).unwrap().decode().unwrap().to_rgb8();
        diff_img_create(&image, 2).save("./img/photo/trnsfrm/".to_owned()+ cur_str).unwrap();
        let cur_str="hash-celeste-3.png";
        let image= ImageReader::open("./img/photo/secret/".to_owned()+cur_str).unwrap().decode().unwrap().to_rgb8();
        diff_img_create(&image, 2).save("./img/photo/trnsfrm/sec-".to_owned()+ cur_str).unwrap();
        let image= ImageReader::open("a.png").unwrap().decode().unwrap().to_rgb8();
        diff_img_create(&image, 2).save("a2.png").unwrap();
        
        
    }
    #[test]
    fn khi_annalyser(){
        let cur_str="images/christmas.jpg";
        let image= ImageReader::open("./".to_owned()+cur_str).unwrap().decode().unwrap().to_rgb8();
        let result = khi_squared_analysis_windowed(&image, 2,3); 
        println!("result : {}",result);
        /*
        cur_str= "images/IMG_20231017_164521.png";
        image= ImageReader::open("./".to_owned()+cur_str).unwrap().decode().unwrap().to_rgb8();
        result = khi_squared_analysis_windowed(&image, 2,3);  
        println!("result : {}",result);
        */

        // cur_str= "images/cc.png";
        // image= ImageReader::open("./".to_owned()+cur_str).unwrap().decode().unwrap().to_rgb8();
        // result = khi_squared_analysis_windowed(&image, 2,3);  
        // println!("result : {}",result);

        // cur_str= "images/0_1698750559306.png";
        // image= ImageReader::open("./".to_owned()+cur_str).unwrap().decode().unwrap().to_rgb8();
        // result = khi_squared_analysis_windowed(&image, 2,11);  
        // println!("result : {}",result);

        /*
        cur_str= "images/earth-0.5-sha2-p-c.png";
        image= ImageReader::open("./".to_owned()+cur_str).unwrap().decode().unwrap().to_rgb8();
        result = khi_squared_analysis_windowed(&image, 2,10);  
        println!("result : {}",result);
        */
    }
}

fn ___main() {
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
