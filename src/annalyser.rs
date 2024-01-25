

use std::ops::Div;

use image::{RgbImage, ImageBuffer, Rgb};
use num::traits::Pow;
use statrs::distribution::*;

/**
 * Calculer les fréquences des valeurs des derniers bits de chaque composante de couleur dans une image.
 * @param img: &RgbImage: Une référence à une image avec un format de couleur RGB. Chaque pixel de cette image est représenté par trois valeurs (rouge, vert, bleu).
 * @param bytes: i32: Le nombre de derniers bits à considérer dans chaque composante de couleur du pixel.
 */
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
    let mut msk: u8 = 0x1;
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

/**
 * Test du Khi Carré pour une image donnée
 * @param img: &RgbImage: Une référence à une image avec un format de couleur RGB. Chaque pixel de cette image est représenté par trois valeurs (rouge, vert, bleu).
 * @param bytes: i32: Le nombre de derniers bits à considérer dans chaque composante de couleur du pixel. Il détermine aussi le degré de liberté dans ce test.
 */ 
fn khi_squared_analysis(img : & RgbImage, bytes_to_analyse : u32)-> f64{ //TODO rgb *3 ?

    //Pour le test de conformité, degres_liberte = nb_categories-1
    let degres_liberte: f64 = 3.0; //TODO : à adapter
    let categories = 2u32.pow(bytes_to_analyse);

    //Initialiser le vecteur attendu pour le test (répartition homogène des pixels)
    let (largeur, hauteur) = img.dimensions();
    let nb_pixels: u32 = largeur * hauteur;
    let repartition_pixels = nb_pixels/categories;

    //TODO vecteur de taille "categories"
    //let frequence_pixel_attendue: Vec<u32> = vec![repartition_pixels, repartition_pixels, repartition_pixels, repartition_pixels]; 
    let frequence_pixel_attendue: Vec<u32> = vec![repartition_pixels*3; categories as usize]; //TODO probabilités d'obtenir chaque lettre
    println!("nombre de pixels attendus pour chaque catégorie : {}",frequence_pixel_attendue[0]);

    //Générer le vecteur de la répartition des pixels de l'image
    let frequence_pixel_observee = stats_last_bits(img, bytes_to_analyse as i32);
    for p in 0..categories{
        println!("vecteur observee {}: {}", p, frequence_pixel_observee[p as usize]);
    }

    //Calcul pour comparaison entre le vecteur attendu et observé
    let mut sum_khi = 0.0;
    for i in 0..categories {
        let diff = frequence_pixel_observee[i as usize] as i64 - frequence_pixel_attendue[i as usize] as i64;
        let squared_diff = (diff as i64).pow(2);
        let result = squared_diff as f64 / frequence_pixel_attendue[i as usize] as f64;
        println!("{:.4}", result);
        sum_khi += result;
        println!("i:  {:?}     sum_khi:{:?}", i, sum_khi);
    }
    
    //Calcul de l'indice de confiance graĉe à khi carré
    let khi_squared = ChiSquared::new(degres_liberte).unwrap();
    //let khi_result_pdf = Continuous::pdf(&khi_squared, sum_khi); //Formule : 1 / (2^(k / 2) * Γ(k / 2)) * x^((k / 2) - 1) * e^(-x / 2)   (source : documentation de statrs::distribution::ChiSquared)
    let khi_result_pdf = khi_squared.pdf(sum_khi);
    //let khi_result_cdf = ContinuousCDF::cdf(&khi_squared, sum_khi); //Formule : (1 / Γ(k / 2)) * γ(k / 2, x / 2) where k is the degrees of freedom, Γ is the gamma function, and γ is the lower incomplete gamma function     (source : documentation de statrs::distribution::ChiSquared)
    let khi_result_cdf = khi_squared.cdf(sum_khi);
    println!("khi pdf:  {:.8}", khi_result_pdf);
    println!("khi cdf:  {:.8}", khi_result_cdf);

    return khi_result_pdf;
}


///// Modules de test ///////////////////////////////////////////////////////////////////////////
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
        let cur_str="vile-foret-hash512.png";
        let image= ImageReader::open("./img/photo/secret/".to_owned()+cur_str).unwrap().decode().unwrap().to_rgb8();
        diff_img_create(&image, 2).save("./img/photo/trnsfrm/".to_owned()+ cur_str).unwrap();
        let cur_str="maddy-pfff.png";
        let image= ImageReader::open("./img/photo/secret/".to_owned()+cur_str).unwrap().decode().unwrap().to_rgb8();
        diff_img_create(&image, 2).save("./img/photo/trnsfrm/sec-".to_owned()+ cur_str).unwrap();
        
    }

    #[test]
    fn khi_annalyser(){
        let cur_str="images/a.png";
        let image= ImageReader::open("./".to_owned()+cur_str).unwrap().decode().unwrap().to_rgb8();
        let result = khi_squared_analysis(&image, 1);
    }

}