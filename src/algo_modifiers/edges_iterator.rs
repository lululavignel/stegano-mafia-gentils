use image::GrayImage;
use imageproc::edges::canny;
use md5::digest::typenum::False;
use crate::lsb;

////DES erreus => implem à finir :(

pub struct  SubEdgesIterator{
    pos: usize,
    available : Vec<(u32,u32,usize)>,
    gray_edges : [GrayImage;3],
    gray:[GrayImage;3]
}
pub struct  EdgesIterator<const IS_EDGE: bool>{
    it:SubEdgesIterator,
    inside_edges: bool,
}
//pub struct NoEdgesIterator{
//   edges: EdgesIterator<false>
//}

impl lsb::ImgIterator for SubEdgesIterator{
    fn new(img:&image::RgbImage, inside_edges:bool)-> Self where Self: Sized {
        
        let mut mask: u8=0xFF;
        let nbr_bits=2;
        let (x_size,y_size)= img.dimensions();
        //one gray img for each color (rgb) to use alreafy existinng functions
        let mut gray_imgs :[GrayImage;3]=[
            GrayImage::new(x_size, y_size),
            GrayImage::new(x_size, y_size),
            GrayImage::new(x_size, y_size),
        ];
        
        //mask creation
        for _ in 0..nbr_bits{
            mask<<=1;
            mask&= 0xFE;
    
        }
        //image pretreatment
        // ie separating in gray img + applying the mask
        for x in 0..x_size{
            for y in 0..y_size{
                let cur_pixel=img.get_pixel(x, y);
                //applying the mask to all pixels, and appending to the bytes vector
                for rgb_index in 0..cur_pixel.0.len(){
                    gray_imgs[rgb_index].put_pixel(x, y, image::Luma([cur_pixel[rgb_index]]));
                    let cur_gray_pixel= gray_imgs[rgb_index].get_pixel_mut(x, y);
                    cur_gray_pixel[0]= cur_pixel.0[rgb_index]&mask;
                    //let cur_gray_pixel= gray_imgs[rgb_index].get_pixel_mut(x, y);
                    //cur_gray_pixel[0]= cur_pixel.0[rgb_index];
                }
            }
        }
        let low_thresold=5.;
        let high_thresold=10.;
        //computing the edges 
        let gray_edges : [GrayImage;3]=[
            canny(&gray_imgs[0],low_thresold,high_thresold),
            canny(&gray_imgs[1],low_thresold,high_thresold),
            canny(&gray_imgs[2],low_thresold,high_thresold),
        ]; 
        //now, we put in a vec the position of all 
        //white pixel (i.e pixel where there is an edge)
        let mut available = Vec::new();
        for index_img in 0.. gray_imgs.len(){
            let (x_size,y_size)=gray_imgs[index_img].dimensions();
            for i in 0..x_size{
                for j in 0..y_size{
                    //if we chose inside edges => then it is true => we add only inside edges, i.e. when pixel!=0
                    //else, only when not inside edges => when pixel ==0
                    if  inside_edges ==(gray_imgs[index_img].get_pixel(i, j).0[0]!=0){
                        available.push((i,j,index_img));
                    }
                }
            }
        }

        //for i in gray_edges[0].pixels();
        println!("Info: number of bits that can be hidden: {}", &available.len()*2);
        let edges_iterator = SubEdgesIterator{pos:0,available,gray_edges,gray:gray_imgs};
        
        return  edges_iterator;



    }
    fn next(&mut self)-> Option<(u32,u32,usize)> {
        if self.pos==self.available.len(){
            print!("Len: {}",self.available.len());
            return None
        }
        self.pos+=1;
        return Some(self.available[self.pos-1]);
    }
}

impl lsb::ImgIterator for EdgesIterator<true> {
    fn new(img:&image::RgbImage)-> Self where Self: Sized {
        return EdgesIterator{it:SubEdgesIterator::new(img, true)};
    }
    fn next(&mut self)-> Option<(u32, u32, usize)> {
       return  self.it.next();
    }
}
impl lsb::ImgIterator for EdgesIterator<false> {
    fn new(img:&image::RgbImage)-> Self where Self: Sized {
        return EdgesIterator{it:SubEdgesIterator::new(img, false)};
    }
    fn next(&mut self)-> Option<(u32, u32, usize)> {
       return  self.it.next();
    }
}

#[cfg(test)]
mod test_edges_iterator{
    

    

    use crate::lsb::ImgIterator;

    use super::*;
    use image::io::Reader as ImageReader;
    
    //use show_image::{ImageView, ImageInfo, create_window};
    
    #[test]
    
   
    fn test_cany(){
        let image = ImageReader::open("./tests/imag.png").unwrap().decode().unwrap().to_rgb8();
        let edges_it = EdgesIterator::<true>::new(&image);
        crate::annalyser::diff_img_create(&image,1).save("./tests/imag-color-diff.png").unwrap();
        for i in 0..edges_it.it.gray_edges.len(){
            edges_it.it.gray_edges[i].save(format!("./tests/imag-gray-edges{}.png",i )).unwrap();
            edges_it.it.gray[i].save(format!("./tests/imag-gray{}.png",i )).unwrap();
        }
        //println!("{:? }",edges_it.it.available);
           
    }
    #[test]
    fn test_edges_iterator(){
       
        let mut image = ImageReader::open("./tests/celeste-3.png").unwrap().decode().unwrap().to_rgb8();
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
        qui trouve la réponse en un temps raisonnable).");
        let encoding =Box::new(lsb::ASCIIEncoding);
        lsb::lsb_hide_msg::<EdgesIterator<true>>(&msg,encoding.as_ref(),&mut image);
        let it = EdgesIterator::<true>::new(&mut image);
        
        it.it.gray[0].save("./img/photo/traces/celeste-3-edges-0.png").unwrap();
        it.it.gray[1].save("./img/photo/traces/celeste-3-edges-1.png").unwrap();
        it.it.gray[2].save("./img/photo/traces/celeste-3-edges-2.png").unwrap();
        it.it.gray_edges[0].save("./img/photo/traces/celeste-3-edges-edges-0.png").unwrap();
        it.it.gray_edges[1].save("./img/photo/traces/celeste-3-edges-edges-1.png").unwrap();
        it.it.gray_edges[2].save("./img/photo/traces/celeste-3-edges-edges-2.png").unwrap();
       
        image.save("./img/photo/secret/celeste-3-edges.png").unwrap();
        println!("{:?}",lsb::lsb_retrieve_msg::<EdgesIterator<true>>(msg.len() as i32,encoding.as_ref(),&mut image));
        
    }
}