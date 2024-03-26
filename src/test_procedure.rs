
use std::{collections::HashMap, fs::File, io::Write};

use image::io::Reader as ImageReader;
use crate::shannon_entropy::{compute_shanon_entropy, entropy_and_randomization_after_first_measure, init_base_shanon_iterator};


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
    ShanonEntropy2,
    ShanonEntropyRandom3,
    ShanonEntropyRandom2,
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

fn defaut_test(img_names: Vec<String>,infos:ImgSteganographyInfos,verbose:bool,logs: &mut Option<Vec<File>>,data: & Option<Vec<String>>,main_log_name:&str){
    let mut log_file = File::create(main_log_name).unwrap();
    let mut map = HashMap::new();
    for img_name in img_names{
        let result = default_test_proc(&img_name, &infos, logs,data, verbose);
        //a changer, la lecture devrait renvoyer les données si déjà fait
        if result.is_some(){
            map.insert(img_name, result.unwrap());
        }
        
    }                                  
    let mut stat_se3 =(0.,10000.,-10000.,0);//total, min, max, count
    let mut stat_se255 =(0.,10000.,-10000.,0);//total, min, max, count
    let mut stat_se2 =(0.,10000.,-10000.,0);//total, min, max, count
    let write_se_x = |stat :ImgSteganalysisInfos, stat_se_x:&mut (f64, f64, f64, i32) | {
                    stat_se_x.0+=stat.steganographied_estimation[2];
                    if stat_se_x.1>stat.steganographied_estimation[2] {stat_se_x.1=stat.steganographied_estimation[2];}
                    if stat_se_x.2<stat.steganographied_estimation[2] {stat_se_x.2=stat.steganographied_estimation[2];}
                    stat_se_x.3+=1};
    let mut stat_ser3 =(0.,10000.,-10000.,0);
    let mut stat_ser2 =(0.,10000.,-10000.,0);
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
                Steganalysislgorithm::ShanonEntropy2        =>write_se_x(stat,&mut stat_se2),
                Steganalysislgorithm::ShanonEntropy3        =>write_se_x(stat,&mut stat_se3),
                Steganalysislgorithm::ShanonEntropyRandom3  =>write_ser_x(stat,&mut stat_ser3),
                Steganalysislgorithm::ShanonEntropyRandom2  =>write_ser_x(stat,&mut stat_ser2),
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

///
/// This function executes all tests on a single img, and return the results as a list of ImgSteganalysisInfos
/// In case of verbose activated, also show some info on the cli
/// 

pub fn default_test_proc(img_name:&str,known_data:&ImgSteganographyInfos,log_file:&mut Option<Vec<File>>,old_logs:& Option<Vec<String>>,verbose:bool) -> Option<Vec<ImgSteganalysisInfos>>{
    if verbose{
        println!("==============================================================\n==============================================================\nAnalysing {} ...",img_name);
    }
    //opening the img
    let mut analysis = Vec::<ImgSteganalysisInfos>::new();
    let image = ImageReader::open(img_name).unwrap().decode().unwrap().to_rgb8();
    let binding = Vec::<String>::new();
    let old_logs_content= match old_logs {
        Some(t)=>t,
        None  => &binding,
    };
    if old_logs.is_none()  {
        let mut already_done = true;
        for content in old_logs_content{
            if !content.contains(&img_name){
                already_done=false;
                break;
            }
           
        }
        if already_done{
            return None;
        }
    }
    
    //computing the whole image entropy with a 0xFF mask
    //
    //
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

    
    //computing the whole image entropy with a 0x03 mask
    //
    //
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

    //computing the whole image entropy with a 0x02 mask
    //
    //
    let mut iterator = init_base_shanon_iterator(&image,0,0,image.width(),image.height(),2);
    let results2 = compute_shanon_entropy(&image,&mut iterator);
    let a =vec!(results2.0,results2.1,results2.2);
    analysis.push(ImgSteganalysisInfos{
            steganographied_estimation:a,
            method:Steganalysislgorithm::ShanonEntropy3,
            cyphered:known_data.cyphered});
    if verbose {println!("Computed shanon entropy with a 0x02 mask : {:?}", results2);}
    if log_file.is_some(){
        log_file.as_mut().unwrap()[2].write_all(&format!("{img_name};{};{};{}\n",results2.0,results2.1,results2.2).as_bytes()).unwrap();
    }

    //computing the whole image entropy with the method where we try to modify the image
    //
    //
    let rdm_result = entropy_and_randomization_after_first_measure(&image,results255,results3,0.1,0x03);
    //let rdm_result = entropy_and_randomization(&image,0.1,2);
    analysis.push(ImgSteganalysisInfos{
        steganographied_estimation:vec![rdm_result],
        method:Steganalysislgorithm::ShanonEntropyRandom3,
        cyphered:known_data.cyphered});
    if log_file.is_some(){
        log_file.as_mut().unwrap()[3].write_all(&format!("{img_name};{}\n",rdm_result).as_bytes()).unwrap();
    }


    //computing the whole image entropy with the method where we try to modify the image
    //
    //
    let rdm_result = entropy_and_randomization_after_first_measure(&image,results255,results2,0.1,2);
    //let rdm_result = entropy_and_randomization(&image,0.1,2);
    analysis.push(ImgSteganalysisInfos{
        steganographied_estimation:vec![rdm_result],
        method:Steganalysislgorithm::ShanonEntropyRandom3,
        cyphered:known_data.cyphered});
    if log_file.is_some(){
        log_file.as_mut().unwrap()[4].write_all(&format!("{img_name};{}\n",rdm_result).as_bytes()).unwrap();
    }
    return Some(analysis); 

}

#[cfg(test)]
mod test_test_proc{
    use super::*;
    use std::{env, fs, io::Read, path::Path, thread};
    #[test]
    ///
    /// Note : I don't know why, but using a 0x02 seem more effective than 0x03??
    
    fn test_default_test(){
        let home_dir  =env::home_dir().unwrap();
        let home= home_dir.as_path().to_str().unwrap();
        let dir_name = home_dir.file_name().unwrap().to_str().unwrap();
        println!("{home}/Images/steg/base_img");
     
        let mut working_dirs = Vec::new();
        working_dirs.push((format!("{home}/Images/steg/base_img"),-1.));
        for i in 1..10{
            working_dirs.push((format!("{home}/Images/steg/s256-c-p-0.{i}"),(i as f64)/10.));
        }

        working_dirs.push((format!("{home}/Images/steg/s256-c-p-0.99"),0.99));
        let mut handles= Vec::new();
        for working_dir in working_dirs{
            let path = working_dir.0.clone();
            let percentage = working_dir.1;
            println!("{}",path);
            handles.push(thread::spawn(move || {run_test(path,percentage);}));
        }

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
        let files = vec!(
                format!("{path}/logs/SE0xFF"),
                format!("{path}/logs/SE0x03"),
                format!("{path}/logs/SE0x02"),
                format!("{path}/logs/SER0x03"),
                format!("{path}/logs/SER0x02")
        );
        for file in files{
            if ! Path::new(&file).exists(){
                File::create(&file).unwrap();
            }
            logs.push(File::options().read(true).write(true).open(file).unwrap());
        }
        /*
        logs.push(File::options().read(true).write(true).open(format!("{path}/logs/SE0x03")).unwrap());
        logs.push(File::options().read(true).write(true).open(format!("{path}/logs/SE0x02")).unwrap());
        logs.push(File::options().read(true).write(true).open(format!("{path}/logs/SER0x03")).unwrap());
        logs.push(File::options().read(true).write(true).open(format!("{path}/logs/SER0x02")).unwrap());
        */
        let mut data =Vec::new();
        let mut binding :&str;
       
        for mut log in &logs{
            let mut buffer = Vec::<u8>::new();
            log.read_to_end(&mut buffer).unwrap();
            data.push(String::from_utf8(buffer).unwrap());
           
            
        }
        defaut_test(img_names, infos, true, &mut Some(logs),  &Some(data), &format!("{path}/logs/log"));
    }
}