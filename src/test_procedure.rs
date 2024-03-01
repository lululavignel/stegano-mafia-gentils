use image::io::Reader as ImageReader;
use crate::shannon_entropy::{compute_shanon_entropy, init_base_shanon_iterator};


pub enum SteganographyAlgorithm{
    LSB,
    SigmaLSB,
}
pub enum Steganalysislgorithm{
    KhiSquaredBase255,
    KhiSquaredBase1,
    KhiSquaredBase3,
    KhiSquaredBase252,
    ShanonEntropyBase,
}
pub struct ImgSteganographyInfos{
    is_steganographied: bool,
    percentage: f32,
    method: SteganographyAlgorithm,
    cyphered: bool,
}
pub struct ImgSteganalysisInfos{
    steganographied_estimation: f32,
    method: Steganalysislgorithm,
    cyphered: bool,
}

fn default_test_proc(img_name:&str,known_data:ImgSteganographyInfos,verbose:bool) -> Vec<ImgSteganalysisInfos>{
    if verbose{
        println!("==============================================================\n==============================================================\nAnalysing {} ...",img_name);
    }
    let mut analysis = Vec::<ImgSteganalysisInfos>::new();
    let image = ImageReader::open(img_name).unwrap().decode().unwrap().to_rgb8();
    let mut iterator1 = init_base_shanon_iterator(&image,0,0,image.width(),image.height(),255);
    let results = compute_shanon_entropy(&image,&mut iterator1);

    if verbose {println!("Computed shanon entropy with a 0xFF mask : {:?}", results);}



    return analysis; 

}