use std::{fs::{self, File}, collections::HashMap};
use image::{RgbImage, Rgb};
use rand::prelude::*;
use num::Unsigned;
use std::io::prelude::*;

use crate::lsb::CharEncoding;

///
/// 
/// Struct used to indicate each letter frequency, in a given language.
/// 
pub struct LanguageFrenquency{
    frequency : [(char,f64);26],
}
///
/// 
/// French letters frequency
/// 
const FRENCH : LanguageFrenquency= LanguageFrenquency{
    frequency :[
        ('e',0.1735),
        ('a',0.0820),
        ('s',0.0793),
        ('i',0.0753),
        ('n',0.0717),
        ('t',0.0699),
        ('r',0.0665),
        ('l',0.0591),
        ('u',0.0573),
        ('o',0.0553),
        ('d',0.0401),
        ('c',0.0333),
        ('m',0.0297),
        ('p',0.0292),
        ('v',0.0139),
        ('g',0.0109),
        ('f',0.0108),
        ('q',0.0104),
        ('h',0.0093),
        ('b',0.0092),
        ('x',0.0047),
        ('j',0.0034),
        ('y',0.0021),
        ('z',0.0010),
        ('k',0.0006),
        ('w',0.0003),
        
        ]
};

///
/// 
/// Entry for a key S2409 key.
/// Associates values to a char 
/// 
#[derive(PartialEq)]
#[derive(Debug)]
struct S2409KeyEntry<T:Unsigned>{
    c:char,
    values: Vec<T>,
}

///
/// 
/// S2409Key. It associate each char inside the map to multiple u16.
/// The key_entry_map associate each char inside the map to its position inside the
/// map (and so its key_entry position inside the key_entry field).
/// 
/// 
#[derive(PartialEq)]
#[derive(Debug)]
pub struct S2409Key{
    key_entry : Vec<S2409KeyEntry<u16>>,
    map: Vec<char>,
    key_entry_map : HashMap<char,usize>,
}

impl CharEncoding for S2409Key {
    fn get_enconding(&self,char :u8) -> Option<Vec<u8>> {
        if self.key_entry_map.get(& (char as char)) == None {
            return None;
        }
        let cur_key_entry: &S2409KeyEntry<u16>= self.key_entry.get(*self.key_entry_map.get(& (   char as char)).unwrap()).unwrap();
        //selecting a random value from each encoding posible from key entry 
        let chiffered_c :u16= cur_key_entry.values[random::<usize>()%cur_key_entry.values.len()];
        let mut vec = Vec::<u8>::with_capacity(2);
        vec.push((chiffered_c&0xFF)as u8);
        vec.push(((chiffered_c>>8))as u8);
        return Some(vec);

    }

    fn decode_all(&self,encoded:Vec<u8>) -> Option<Vec<u8>> {
        let mut msg = Vec::with_capacity(encoded.len()/2);
        for i in 0.. encoded.len()/2{
            let mut cur_encoded=0;
            cur_encoded|= encoded[2*i]as u16;
            cur_encoded|= (encoded[2*i+1]as u16)<<8;
            msg.push(self.map[cur_encoded as usize] as u8);
                
        }
        return Some(msg);
    }
}


///
/// 
/// Generate a S2409 key for the given LanguageFrenquency
/// 
/// 
pub fn key_gen(lf: LanguageFrenquency) -> S2409Key{
    let u16_count= f64::powf(2.,16.);
    const TAB_SIZE :usize =usize::pow(2,16);
    let mut key = S2409Key{key_entry : Vec::new(),map: vec!['\0';TAB_SIZE],key_entry_map: HashMap::new()} ;
    key.map.reserve(TAB_SIZE);
    
    let mut key_bytes_count:[f64;26]=[0.0;26];
    for i in 0..26{
        key_bytes_count[i]=FRENCH.frequency[i].1*u16_count;
    }
    let mut count: usize =0;
    
    let mut byte_list :[u16;TAB_SIZE]= [0;TAB_SIZE];
    for i in 0..TAB_SIZE{
        byte_list[i]=i as u16;
    }
    ficher_yates_shuffles(&mut byte_list);
    let mut curr_char=0;
    for mut f in key_bytes_count{
        let mut  key_entry: S2409KeyEntry<u16> =S2409KeyEntry{
                c:lf.frequency[curr_char].0,
                values:Vec::new(),
                };
        key.key_entry_map.insert(key_entry.c, curr_char);
        curr_char+=1;
        while f>0.51{
            key_entry.values.push(byte_list[count]);
            key.map[byte_list[count as usize] as usize]=key_entry.c;
            count+=1;
            f-=1.;
        }
        key.key_entry.push(key_entry);
    }
    //let tab:[u16;65535];


    return key;

}

///
/// 
/// Read a file, and if its format is correct, return the S2409 key described by the file.
/// Doesn't support the "'" char
/// 
/// 
pub fn key_read(filename:&String) -> S2409Key{
    let contents = fs::read_to_string(filename)
        .expect("Should have been able to read the file");
    const TAB_SIZE :usize =usize::pow(2,16);
    let mut key = S2409Key{key_entry : Vec::new(), map: vec!['\0';TAB_SIZE], key_entry_map: HashMap::new()};
    
    let lines = contents.lines();
    for line in lines{
        let mut values = line.split(',');
        let mut key_entry: S2409KeyEntry<u16> =S2409KeyEntry{
            c:values.next().unwrap().as_bytes()[0]as char,
            values:Vec::new(),
            };
        key.key_entry_map.insert(key_entry.c, key.key_entry.len());
        for value in values{
            
            if !value.is_empty(){
                let nbr = u16::from_str_radix(value,10).unwrap();
            //print!("{:?}",tab);
           
                key_entry.values.push(nbr);
                key.map[nbr as usize]= key_entry.c;
            }
            
        }
        key.key_entry.push(key_entry);
    }
    return key;
}

///
/// 
/// Write the S2409 key inside a file
/// 
/// 
pub fn key_write(key: &S2409Key, filename :&String){
    let mut file = File::create(filename).unwrap();
    let mut buf = String::new();
    for entry in &key.key_entry{
        buf.push(entry.c);
        buf.push(',');
        for values in &entry.values{
            buf+= &format!("{},",values) ;
        }
        buf.push('\n');
    }
    
    file.write_all(buf.as_bytes()).unwrap();

}


///
/// DEPRECATED
/// Add the message given in the string inside the image, using the S2409key
/// Each character that is not inside the key will be skipped
/// 
#[allow(dead_code)]
fn add_mes_with_key(msg :&str ,key: &S2409Key,img : &mut RgbImage){
    let mut x=0;
    let mut y=0;
    let mut cd=0;
    let msk : u8 = !0b11;
    let (x_size,y_size)= img.dimensions();
    let mut mes_len_hided=0;

    //iterating over the message to be hidded
    for c in msg.as_bytes(){
        // if the char isn't inside the key => we skip it
        if key.key_entry_map.get(& (*c as char)) == None{
            continue;
        }

        // getting the position of the char inside the key entries
        let cur_key_entry= key.key_entry.get(*key.key_entry_map.get(& (*c as char)).unwrap()).unwrap();
        //selecting a random value from each encoding posible from key entry 
        let chiffered_c :u16= cur_key_entry.values[random::<usize>()%cur_key_entry.values.len()];
        //spliting the encoded value into 8 u8 to be hiden inside the image
        let mut cs : [u8;8]=[0;8];
        for i in 0..cs.len(){
            //the first value inside the array will have
            //the two right most value
            //the next the two after....
            //the last the two left  most 
            cs[i]=((chiffered_c>>2*i)&0x3) as u8;
        }
        //hiding the bytes
        for i in 0..8{
            //getting current pixelt
            let mut  curr_pixel: Rgb<u8> = img.get_pixel(x,y).clone();
            curr_pixel.0[cd]=(curr_pixel.0[cd]&msk)|cs[i];
            img.put_pixel(x,y,curr_pixel);
            cd+=1;
            if cd==3{//R,G,B have been visited=> going to next pixel
                cd=0;
                x+=1;
                if x==x_size{ 
                    x=0;
                    y+=1;
                    if y==y_size{
                        println!("WARNING : IMAGE'S SIZE IS TOO SMALL. ONLY {} CHAR HAVE BEEN HIDDEN INSIDE THE IMAGE",mes_len_hided);
                        return;
                    }
                }
            }
        }
        mes_len_hided+=1;
    }
}
pub fn read_mes_with_key(img : & RgbImage, key: &S2409Key, mut length: i32 )->String{
    let mut mes= String::with_capacity(1000);
    let mut cd=0;
    
    let mut cur_char :u16=0;

    for (_,_,pixel) in img.enumerate_pixels(){
        for val in pixel.0{
            cur_char|= ((val&0b11)as u16)<<(2*cd);
            print!("{}, ",(val&0b11));
            cd +=1;
            if cd==8{
                //println!("|{}", cur_char);
                mes.push(key.map[cur_char as usize]);
                cd=0;
                cur_char=0;
            }
        }
        length-=1;
        if length<=0{
            return mes;
        }
    }
    return mes;

}

///
/// 
/// Algorithm executing a random permuttation on the array. (see wikipedia for algorithm explanation/proof)
/// It has a O(n) complexity.
/// 
/// 
pub fn ficher_yates_shuffles<T:Copy>(tab:&mut [T]){
    let len: usize =tab.len();
    for i in 0..(len-1){
        let rdm =( rand::random::<usize>()%(len-1-i))+i;
        let temp: T = tab[i];
        tab[i]=tab[rdm];
        tab[rdm]=temp;
    }

}

#[cfg(test)]
mod test_key{
    

    

    use super::*;
    use image::io::Reader as ImageReader;
    #[test]
    fn test_key_io(){

        
        let key = key_gen(FRENCH);
        let filename=String::from("key");
        key_write(&key,&filename );
        let readed_key = key_read(&filename);
       
        assert_eq!(key,readed_key);
    }
    #[test]
    fn test_key_steg(){
        let mut image = ImageReader::open("./unit_tests/in/default_img/celeste-3.png").unwrap().decode().unwrap().to_rgb8();
        let msg = String::from("L'objectif du défi est d'élire un utilisateur suite à la résolution de ce dernier, afin de lui permettre
        d'écrire le prochain bloc de la blockchain en échange d'une récompense monétaire. Ce processus est donc
        appelé le minage de la blockchain, c'est-à-dire un processus qui valide les blocs de données contenant
        les transactions de donnée avant d'être ajoutés à celle-ci. Cependant, on ne peut pas juste demander
        à chaque machine de tirer un nombre au hasard : chaque machine est potentiellement un attaquant, il
        peut donc choisir le nombre qu'il veut et toujours gagner.
        Pour résoudre ce problème d'élection, la solution du « proof of work » est utilisé. Cette méthode
        pose un « défi » que les utilisateurs tentent de résoudre. Ce défi doit respecter les contraintes suivantes :
        — Vérifier qu'une solution est correcte doit être très rapide
        — Trouver une solution ne peut se faire que de manière aléatoire (pas d'algorithme déterministe
        qui trouve la réponse en un temps raisonnable).
        — Les problèmes peuvent être générés de manière simple (en utilisant le dernier bloc crée, on
        obtient un nouveau problème)
        Cette méthode est très rependue dans les blockchains de crypto-monnaie, c'est pourquoi nous y
        avons pensé en premier
        Partant du principe que le Proof of work fonctionne grâce un phénomène de masse, nous nous
        sommes plutôt dirigés vers une autre méthode, qui est celle des « Smart contracts ».
        2.3.3 Gagner la monnaie en jouant
        Le principe des smart contracts est assez simple : SI la condition est remplie ALORS le résultat est
        exécuté et le contrat est enregistré dans la blockchain. Du point de vue de notre jeu, on peut considérer
        que si le mini-jeu défi est réussi, alors le joueur est récompensé avec la monnaie, et cet événement est
        enregistré dans la blockchain. C'est de cette manière que le joueur peut gagner des pièces.
        Ce genre de système est plus utilisé dans les « crypto games », qui ressemblent à ce que nous
        voulons produire. Cette solution reste théorique, et nous n'avons pas encore discuté avec nos tuteurs
        techniques de si elle est envisageable.");
        let key: S2409Key= key_gen(FRENCH);
        add_mes_with_key(&msg, &key,&mut image);
        println!("{}",read_mes_with_key(&image,&key,10000));
        key_write(&key, &String::from("key_from_test"));
        image.save("steg_with_key.jpg").unwrap();
        //let  _image = ImageReader::open("./photo.jpg").unwrap().decode().unwrap().to_rgb8();
        //add_mes(&msg, &mut image);
        //image.save("steg.jpg").unwrap();

    }
}
