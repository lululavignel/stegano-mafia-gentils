use std::{collections::HashMap, fs::File, io::Write};

use image::io::Reader as ImageReader;
use crate::shannon_entropy::{compute_shanon_entropy, entropy_and_randomization, entropy_and_randomization_after_first_measure, init_base_shanon_iterator};


pub enum SteganographyAlgorithm{
    LSB,
    SigmaLSB,
    None,
}
pub enum Steganalysislgorithm{
    KhiSquaredBase255,
    KhiSquaredBase1,
    KhiSquaredBase3,
    KhiSquaredBase252,
    ShanonEntropy255,
    ShanonEntropy3,
    ShanonEntropyRandom3,
}
pub struct ImgSteganographyInfos{
    is_steganographied: bool,
    percentage: f64,
    method: SteganographyAlgorithm,
    cyphered: bool,
}
pub struct ImgSteganalysisInfos{
    steganographied_estimation: Vec<f64>,
    method: Steganalysislgorithm,
    cyphered: bool,
}

fn defaut_test(img_names: Vec<String>,infos:ImgSteganographyInfos,verbose:bool,logs: &mut Option<Vec<File>>,main_log_name:&str){
    let mut log_file = File::create(main_log_name).unwrap();
    let mut map = HashMap::new();
    for img_name in img_names{
        let result = default_test_proc(&img_name, &infos, logs, verbose);
        map.insert(img_name, result);
    }                                  
    let mut stat_se3 =(0.,10000.,-10000.,0);//total, min, max, count
    let mut stat_se255 =(0.,10000.,-10000.,0);//total, min, max, count
    let write_se_x = |stat :ImgSteganalysisInfos, stat_se_x:&mut (f64, f64, f64, i32) | {
                    stat_se_x.0+=stat.steganographied_estimation[2];
                    if stat_se_x.1>stat.steganographied_estimation[2] {stat_se_x.1=stat.steganographied_estimation[2];}
                    if stat_se_x.2<stat.steganographied_estimation[2] {stat_se_x.2=stat.steganographied_estimation[2];}
                    stat_se_x.3+=1};
    let mut stat_ser3 =(0.,10000.,-10000.,0);
    let write_ser_x = |stat :ImgSteganalysisInfos, stat_ser_x:&mut (f64, f64, f64, i32) | {
                    stat_ser_x.0+=stat.steganographied_estimation[0];
                    if stat_ser_x.1>stat.steganographied_estimation[0] {stat_ser_x.1=stat.steganographied_estimation[0];}
                    if stat_ser_x.2<stat.steganographied_estimation[0] {stat_ser_x.2=stat.steganographied_estimation[0];}
                    stat_ser_x.3+=1
    };
    for (_name,stats) in  map.into_iter(){
        for stat in stats{
            match stat.method {
                Steganalysislgorithm::KhiSquaredBase255     =>(),
                Steganalysislgorithm::KhiSquaredBase1       =>(),
                Steganalysislgorithm::KhiSquaredBase3       =>(),
                Steganalysislgorithm::KhiSquaredBase252     =>(),
                Steganalysislgorithm::ShanonEntropy255      =>write_se_x(stat,&mut stat_se255),
                Steganalysislgorithm::ShanonEntropy3        =>write_se_x(stat,&mut stat_se3),
                Steganalysislgorithm::ShanonEntropyRandom3  =>write_ser_x(stat,&mut stat_ser3),
            };
        }
    }

    log_file.write_all(format!("stats ShanonEntropy3 --- min : {} max : {} average : {}\n",
                    stat_se3.1,stat_se3.2,stat_se3.0/stat_se3.3 as f64).as_bytes()).unwrap();

    log_file.write_all(format!("stats ShanonEntropy255 --- min : {} max : {} average : {}\n",
                    stat_se255.1,stat_se255.2,stat_se255.0/stat_se255.3 as f64).as_bytes()).unwrap();

    log_file.write_all(format!("stats ShanonEntropyRandom3 --- min : {} max : {} average : {}\n",
                    stat_ser3.1,stat_ser3.2,stat_ser3.0/stat_ser3.3 as f64).as_bytes()).unwrap();
}

fn default_test_proc(img_name:&str,known_data:&ImgSteganographyInfos,log_file:&mut Option<Vec<File>>,verbose:bool) -> Vec<ImgSteganalysisInfos>{
    if verbose{
        println!("==============================================================\n==============================================================\nAnalysing {} ...",img_name);
    }
    //opening the img
    let mut analysis = Vec::<ImgSteganalysisInfos>::new();
    let image = ImageReader::open(img_name).unwrap().decode().unwrap().to_rgb8();

    //computing the whole image entropy with a 0xFF mask
    let mut iterator = init_base_shanon_iterator(&image,0,0,image.width(),image.height(),255);
    let results255 = compute_shanon_entropy(&image,&mut iterator);
    let a =vec!(results255.0,results255.1,results255.2);
    analysis.push(ImgSteganalysisInfos{
            steganographied_estimation:a,
            method:Steganalysislgorithm::ShanonEntropy255,
            cyphered:known_data.cyphered});
    if verbose {println!("Computed shanon entropy with a 0xFF mask : {:?}", results255);}
    if log_file.is_some(){
        log_file.as_mut().unwrap()[0].write_all(&format!("{img_name};{};{};{}\n",results255.0,results255.1,results255.2).as_bytes()).unwrap();
    }


    //computing the whole image entropy with a 0xFF mask
    let mut iterator = init_base_shanon_iterator(&image,0,0,image.width(),image.height(),3);
    let results3 = compute_shanon_entropy(&image,&mut iterator);
    let a =vec!(results3.0,results3.1,results3.2);
    analysis.push(ImgSteganalysisInfos{
            steganographied_estimation:a,
            method:Steganalysislgorithm::ShanonEntropy3,
            cyphered:known_data.cyphered});
    if verbose {println!("Computed shanon entropy with a 0x03 mask : {:?}", results3);}
    if log_file.is_some(){
        log_file.as_mut().unwrap()[1].write_all(&format!("{img_name};{};{};{}\n",results3.0,results3.1,results3.2).as_bytes()).unwrap();
    }


    let rdm_result = entropy_and_randomization_after_first_measure(&image,results255,results3,0.1,2);
    //let rdm_result = entropy_and_randomization(&image,0.1,2);
    analysis.push(ImgSteganalysisInfos{
        steganographied_estimation:vec![rdm_result],
        method:Steganalysislgorithm::ShanonEntropyRandom3,
        cyphered:known_data.cyphered});
    if log_file.is_some(){
        log_file.as_mut().unwrap()[2].write_all(&format!("{img_name};{}\n",rdm_result).as_bytes()).unwrap();
    }
    return analysis; 

}

#[cfg(test)]
mod test_test_proc{
    use super::*;
    use std::{env, fs, thread};
    #[test]
    ///
    /// Note : I don't know why, but using a 0x02 seem more effective than 0x03??
    
    fn test_default_test(){
        let home_dir  =env::home_dir().unwrap();
        let home= home_dir.as_path().to_str().unwrap();
        let dir_name = home_dir.file_name().unwrap().to_str().unwrap();
        println!("{home}/Images/steg/base_img");
        let working_dir1=format!("{home}/Images/steg/base_img");
        let working_dir2=format!("{home}/Images/steg/s256-c-p-0.1");
        let working_dir3=format!("{home}/Images/steg/s256-c-p-0.2");
        let working_dir4=format!("{home}/Images/steg/s256-c-p-0.5");
        let working_dir5=format!("{home}/Images/steg/s256-c-p-0.8");
        let working_dir6=format!("{home}/Images/steg/s256-c-p-0.99");
        let mut handles= Vec::new();
        handles.push(thread::spawn(|| {run_test(working_dir1, -1.);}));
        handles.push(thread::spawn(|| {run_test(working_dir2, 0.1);}));
        handles.push(thread::spawn(|| {run_test(working_dir3, 0.2);}));
        handles.push(thread::spawn(|| {run_test(working_dir4, 0.5);}));
        handles.push(thread::spawn(|| {run_test(working_dir5, 0.8);}));
        handles.push(thread::spawn(|| {run_test(working_dir6, 0.99);}));
        for handle in handles{
            handle.join().unwrap();
        }
    }
    fn run_test(path:String, percentage:f64){
        let mut img_names= Vec::new();
        for file in fs::read_dir(path.clone()).unwrap() {
            let file =file.unwrap();
            if file.path().is_file(){
                let filename =file.file_name().into_string().unwrap();
                img_names.push(format!("{path}/{filename}"));
            }   
        }
        let infos= ImgSteganographyInfos{
            is_steganographied:percentage>0.,
            percentage:percentage,
            method:SteganographyAlgorithm::None,
            cyphered:false,
        };
        let mut logs = Vec::new();

        logs.push(File::create(format!("{path}/logs/SE0xFF")).unwrap());
        logs.push(File::create(format!("{path}/logs/SE0x03")).unwrap());
        logs.push(File::create(format!("{path}/logs/SER0x02")).unwrap());
        defaut_test(img_names, infos, true, &mut Some(logs), &format!("{path}/logs/log"));
    }
}