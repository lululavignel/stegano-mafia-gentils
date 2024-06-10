use hash_rng::{*};
use sha2::{Digest, digest::generic_array::GenericArray};
use image::RgbImage;

use crate::{lsb::ImgIterator, utilities::hash_rng};

pub struct  HashIterator<T:Digest>{
    pos: usize,
    permutation : Vec::<usize>,
    hash:GenericArray<u8, T::OutputSize>,
    visited_map : RgbImage,
    rng : SimpleHashRng<T>,
}
impl<T:Digest> ImgIterator for  HashIterator<T>{
    fn new(img:&RgbImage,key:Option<&[u8]>)-> Self where Self: Sized {
        let (x_size,y_size) = img.dimensions();
        let visited_map =RgbImage::new(x_size,y_size);
        let hash =get_hash_from_img::<T>(3, img); 
        let mut img_bytes= img_to_bytes_masked(&img,3);
        if key.is_some(){
            let key_bytes= key.unwrap();
            let mut min_len=key_bytes.len();
            if min_len > img_bytes.len(){
                min_len=img_bytes.len();
            }
            for index in 0..min_len{
                img_bytes[index]^=key_bytes[index];
            }
        }
        let rng: SimpleHashRng<T> = SimpleHashRng::<T>::new(img_bytes,None);
        
        let mut  permutation = Vec::<usize>::with_capacity((x_size*y_size*3) as usize);
        for i in 0..(x_size*y_size*3){
            permutation.push(i as usize);
        }
        //si_ficher_yates_shuffles(&mut permutation,&mut rng);
        return HashIterator::<T> { pos: 0, permutation,hash,visited_map,rng };
    }

    fn next(&mut self)-> Option<(u32, u32, usize)> {
        let len: usize = self.hash.len();
        if len>=self.permutation.len(){
            return None;
        }
        if self.pos+2==self.permutation.len(){
            return None;
        }
        let (x_len,_y_len)= self.visited_map.dimensions();
        partial_ficher_yates_shuffles(&mut self.permutation, self.pos, self.pos+1, &mut self.rng);
        let pixel_index = self.permutation[self.pos] as u32;
        //println!("pix: {}",pixel_index);
        self.pos+=1;
        let x_pos=(pixel_index/3)%x_len;
        let y_pos=(pixel_index/3)/x_len;
        let color=(pixel_index%3) as usize;
       
        return Some((x_pos,y_pos,color));
    }
}

///
/// Calculate the hash from an image, after applying a mask,
/// seting the last nbr_bits to 0. The nbr_bits must be between 0 and 7. 
/// Use the hashing algorithm T (must implements the trait Digest).
/// 
fn get_hash_from_img<T:Digest>(nbr_bits: u32 , img : & RgbImage) -> GenericArray<u8, T::OutputSize>   {
    let mut hasher = T::new();
    let mut bytes= Vec::<u8>::new();
    
    if nbr_bits>7{
        panic!("Number of bits too important")
    }
    //creating the mask
    let mask=!((1<<nbr_bits)-1);
   

    //iterate over all pixels
    let (x_size,y_size)= img.dimensions();
    for x in 0..x_size{
        for y in 0..y_size{
            let cur_pixel=img.get_pixel(x, y);
            //applying the mask to all pixels, and appending to the bytes vector
            for color in cur_pixel.0{
                let masked_color=color & mask;
                bytes.push(masked_color);
            }
        }
    }
    hasher.update(bytes);
    return hasher.finalize();
    

}
///
/// 
/// Calculate a new hash, using the genericArray given in argument
/// The new hash will remplace the old hash given in argument
/// T is the hash algorithm
/// 
/// 
pub fn reset_hash<T:Digest>(hash:  &mut GenericArray<u8,T::OutputSize>){
    let mut hasher = T::new();
    hasher.update(hash.clone());
    let new_hash=hasher.finalize();
    hash.copy_from_slice(&new_hash[0..new_hash.len()]);
}

///
/// 
/// Return the next pixel to visit, using the hash, and the current next position to visit inside the hash.
/// It consumes 5 bytes of the hash. If there is not enough bytes remaining, reset the hash using reset_hash.
/// 
/// 
/// 


fn get_next_pixel<T:Digest>(hash:  &mut GenericArray<u8,T::OutputSize>,pos: &mut usize, visited_map: &mut RgbImage) -> (u32,u32,usize){
    let len = hash.len();
    let (x_len,y_len)= visited_map.dimensions();
    let mut tab:[u32;5]=[0;5];
    for i in  0..tab.len(){
        if len==*pos{
            reset_hash::<T>(hash);
            //println!("{:?}",hash);
            *pos=0;       
        }
        tab[i]=hash[*pos] as u32;
        *pos+=1;
    }
    
    let mut x_pos= ((tab[0]<<8)+tab[1])%x_len;
    let mut  y_pos=((tab[2]<<8)+tab[3])%y_len;
    let mut color =(tab[4]%3) as usize;
    //println!("x: {} , y: {} , c: {}",x_pos,y_pos,color);

    while  visited_map.get_pixel(x_pos, y_pos).0[color]==1{
        color+=1;
        if color==3{
            color=0;
            x_pos+=1;
            if x_pos==x_len{
                x_pos=0;
                y_pos+=1;
                if y_pos==y_len{
                    y_pos=0;
                }
            }
        }
    }
    visited_map.get_pixel_mut(x_pos, y_pos).0[color]=1; 
    if x_pos>0 && y_pos>0 &&visited_map.get_pixel(x_pos-1, y_pos-1).0[color]==1{
        //println!("Diag en :x: {} , y: {} , c: {}",x_pos,y_pos,color);
    }
    return (x_pos,y_pos,color);

}

///
/// 
/// Hide the given string inside the RgbImage, using the specified hash
/// 
/// 
pub fn add_mes_with_hash_pattern<T:Digest>(msg :&str ,img : &mut RgbImage){

    let msk : u8 = !0b11;
    let (x_size,y_size)= img.dimensions();
    let mut pos = 0;
    let mut hash= get_hash_from_img::<T>(2, img);
    let mut visited_map= RgbImage::new(x_size,y_size);
    for c in msg.as_bytes(){
        let mut cs : [u8;4]=[0;4];
        //print!("{}  ",(*c as char) );
        //print!("{}  ",c );
        cs[0]= c&0x3;
        cs[1]= (c>>2)&0x3;
        cs[2]= (c>>4)&0x3;
        cs[3]= (c>>6)&0x3;
        //println!("{:?}  ",cs );
        for bits in cs{
            let (x,y,color)=get_next_pixel::<T>(&mut hash, &mut pos, &mut visited_map);
            img.get_pixel_mut(x, y).0[color]&=msk;
            img.get_pixel_mut(x, y).0[color]|=bits;
            
        }
    }
}

///
/// 
///Try to retrieve a message potentialy hiden in the image, using hash algorithm 
/// 
/// 
pub fn get_mes_with_hash_pattern<T:Digest>(mut length :i32 ,img : &mut RgbImage)->String{
    let (x_size,y_size)= img.dimensions();
    let mut pos = 0;
    let mut hash= get_hash_from_img::<T>(2, img);
    let mut visited_map= RgbImage::new(x_size,y_size);
    let mut cur_char:u8=0;
    let mut mes = String::new();
    let mut cd=0;
    for _ in 0..4*length{
        let (x,y,c) = get_next_pixel::<T>(&mut hash, &mut pos, &mut visited_map);
        let val = img.get_pixel(x, y).0[c];
        cur_char|= (val&0b11)<<(2*cd);
        //print!("{}, ",(val&0b11));
        cd +=1;
        if cd==4{
            //println!("|{}", cur_char);
            mes.push(cur_char as char);
            cd=0;
            cur_char=0;
        }
    }
    length-=1;
    if length<=0{
        return mes;
    }
    return mes;
}
#[cfg(test)]
mod test_pos_hash{
    use image::io::Reader as ImageReader;
    use sha2::Sha512;
    use super::*;
    #[test]
    fn test_key_hash(){
        let mut image = ImageReader::open("./img/photo/vile-foret.png").unwrap().decode().unwrap().to_rgb8();
        let mut msg = String::from("L'objectif du défi est d'élire un utilisateur suite à la résolution de ce dernier, afin de lui permettre
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
        msg.push_str(&msg.clone());
        msg.push_str(&msg.clone());
        msg.push_str(&msg.clone());
        msg.push_str(&msg.clone());
        add_mes_with_hash_pattern::<Sha512>(&msg,&mut image);
        image.save("./img/photo/secret/vile-foret-hash512.png").unwrap();
        println!("{}",get_mes_with_hash_pattern::<Sha512>(10000,&mut image));
       

    }
}
    
