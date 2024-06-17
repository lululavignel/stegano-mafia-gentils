

use std::{fs::File, io::Write};

use clap::{Arg, App};
use image::{io::Reader as ImageReader, ImageBuffer, Rgb, RgbImage};
use statrs::distribution::{Binomial, ChiSquared, ContinuousCDF, Discrete};

use crate::{rs::{default_compute_rs, defautl_dyn_rs}, shannon_entropy::{compute_shanon_entropy, entropy_and_randomization_after_first_measure, init_base_shanon_iterator}};

//use crate::test_procedure::test_test_proc::run_test;
const I_O_NAME: &str="input image> <output";
const IM_TRANS: &str="image operation";
const TRANS_LSB: &str="lsb";
const TRANS_DELTA: &str="img-delta";
const RUN_ALL_TESTS_FOLDER: &str="test folder";
pub(crate) fn __main_cli() {
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
                .max_values(3));
    
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
        let mut file =File::options().write(true).open("output").unwrap();
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