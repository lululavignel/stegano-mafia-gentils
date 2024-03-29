use std::{env, fs::{self, File}, io::Read, path::Path, thread};

mod test_procedure;
use test_procedure::{*};
fn main (){
    test_default_test();
}

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