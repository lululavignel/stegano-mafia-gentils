use image::{GenericImageView, RgbImage};
use num::pow::Pow;

use crate::lsb::{CharEncoding, ImgIterator};






fn rho_p(i:u32,j:u32)-> u32{
    return 0
} 

fn rho_m(i:u32,j:u32)-> u32{
    return 0
}

///
/// This function calculate the vector y such as H'*y=msg, and where d(data,y) is minimal (using haming distance (when rhos is set to 1.0))
/// H' is composed of h_matrix repeated multiple time, starting at [0,0] and translated by one verticaly, and by h_matrix.len() horizontaly
/// 
/// data is a vec of 0 and 1 and is the cover image.\
/// msg is a vec of 0 and 1 and is the message to embed. \
/// h_matrix is the matrix that will be used to construct H'. It is only and 1-dim vector, because all column are represented as an integer \
/// where the most significant bit is the bit inside last line. And so the 2d matrix  vec\!\[vec\!\[1,1\],vec\!\[0,1\]\]  is reprensented by the 1d matrix : vec\!\[3,2\]\
/// rhos is a vec of float corresponding to the weight of modifing each bits
/// 
/// Returns the stego y and the wheigt of the transformation
/// 
/// See http://dde.binghamton.edu/filler/pdf/fill10spie-syndrome-trellis-codes.pdf for a detailled explanation of the algorithm.
/// 
/// 
/// 
fn viterbi(data: &Vec<u8>, msg:&Vec<u8>,h_matrix:&Vec<usize>,h:u32,rhos:&Vec<f64>) -> (Vec<u8>, f64){
    //weight initialization 
    let mut weight = vec![f64::INFINITY;2_usize.pow(h)];
    weight[0]=0.;
    let mut nweight = vec![f64::INFINITY;2_usize.pow(h)];
    //the sub matrix count is equal to msg len
    let sub_matrix_count=msg.len();
    // w = width
    let w = h_matrix.len();
    //path  array
    let mut path=vec![vec![0;2_usize.pow(h)];data.len()];
    let exponent = 2_usize.pow(h);
    let mut index=0; let mut indexmsg =0;

    //forward part of the algorithm
    for _i in 0..sub_matrix_count{
       
        for j in 0..w as usize{
           
            //we iterate over all posibles bits
            // to calculate the results of all
            let remaining = sub_matrix_count-_i; 
            let mut mask = exponent-1;
            //the last line we "cut" the matrix h_mat
            if remaining<h as usize{
                //to do so, we apply a mask 
                mask >>=h as usize -remaining ;
                //println!("up to : {up_to} , msk : {mask}");
            }
            for k in 0..exponent{
                // the index inside of weight array represents the state of the current block
                
                //first, we calculate the weight of setting the bit to zero (e.g not adding the h*[...,data[index],...]<=> h*[...,0,...])
                //and so not modifying the state of the block
                let w0=weight[k] +data[index] as f64*rhos[index];
                //now we calculate the weight of setting the bit to one and thus we modify the state of the block
                //wich is equivalent to XOR the current state to the value of h_matrix[j]
                let w1=weight[k^(h_matrix[j]&mask) ] + (1-data[index])as f64 *rhos[index];
                
                //println!("w0 : {w0} , w1 : {w1}");
                //we save the weight of the path
                //and we put the value of the bit that minimize the distance
                if w1<w0{
                    path[index][k]=1;
                    nweight[k]=w1;
                } 
                else {
                    path[index][k]=0;
                    nweight[k]=w0;
                }
               
            }
            //println!("weight: {:?}",weight);
            //path[index].reverse();

            index+=1;
            weight.clone_from(&nweight);
            //println!("nweight : {:?}", nweight);
        }
        // we "cut in half" the trellis, keeping only the state that give a correct 
        // bit for the message when multiplying by h 
        for j in 0..2_usize.pow(h-1){
            weight[j]=weight[2*j+msg[indexmsg]as usize];
            
        }
        // set to infinity the last half of the trellis, "cutting it"
        for reset_index in 2_usize.pow(h-1)..2_usize.pow(h){
            weight[reset_index]=f64::INFINITY;
        }
       
        indexmsg+=1;
    }
    //backward part
    let embedding_cost=weight[0];
    indexmsg-=1; // no overflow
    let mut state= msg[indexmsg] as usize;
    //vec "y" such as H*y=m
    let mut out = vec![0;data.len()];
    /* 
    for i in 0..path.len()/2{
        //println!("1 : {:?} 2 : {:?}", path[2*i],path[2*i+1]);
    }
    //println!("path============");
    for i in 0..path[0].len(){
        //println!("-------------------------------");
        for j in 0..path.len(){
            //print!("{} , ", path[j][i]);
        }
    }
    //println!("");*/
    for _i in (0..sub_matrix_count).rev(){
        //println!("aa : {i}");
        
        for j in (0..w).rev(){
            index-=1;
            //println!("state : {state}");
            //getting the current bit
            out[index]=path[index][state];
            //println!("bb : {}",(out[index] as usize*h_matrix[j as usize] )as usize);
            //cutting the matrix for the last bits
            let mut  true_mult =h_matrix[j];
            if sub_matrix_count -_i<h as usize {
                true_mult&= 2_usize.pow((sub_matrix_count -_i)as u32)-1;
            }
            //computing the last state
            //(adding [0...out[index...0] to the state (that goes from 0 to (2^h)-1))
            state^=(out[index] as usize * true_mult)as usize;
        }
        //println!("state : {state}");
        if indexmsg >0{
            indexmsg-=1;
            state+=state+msg[indexmsg] as usize;
        }
        //println!("---");
    }
    //backward
   return (out,embedding_cost);
}
///
/// h_matrix format:
///             --->|->|->|
///                 |  |  |
///                \!/\!/\!/
///                 '  '  '
/// compute H*y and thus is the decode fonction to error-correction code steganography
/// (assuming H is generated in the same H is in this method)
/// 
fn get_error_code_msg(data: &Vec<u8>,h_matrix:&Vec<Vec<u8>>,msg_len:usize)-> Vec<u8>{
    let mut msg = Vec::<u8>::new();
    let mut index=0;
    let mat_width = h_matrix.len();
    let mat_height= h_matrix[0].len();
    let sub_matrix_count=msg_len;
    for i in 0..msg_len{
        //println!("====i : {i}");
        
        let mut i_min = mat_height-1;
        if i > i_min{
            i_min=0;
        }
        else {
            i_min=i_min-i;
        }
        let mut j_min=0;
        if i>=mat_height{
            j_min=mat_width*((i+1)-mat_height);
        }
        
        let mut j_max=(i+1)*mat_width;
        //let mut i_max=0;
        if j_max>= mat_width*sub_matrix_count{
            j_max=mat_width*sub_matrix_count;
        }
        let mut cur_msg=0;

        //println!("j_min : {j_min} , j_max : {j_max}");
        for cur_j in j_min..j_max{
            let cur_calc=data[cur_j]&h_matrix[cur_j%mat_width][mat_height -1 -(i_min + (cur_j-j_min)/mat_width)];
            cur_msg^=cur_calc;
            //print!("h_m : {} , data : {aa} , cc : {cur_calc} , ",h_matrix[cur_j%mat_width][mat_height -1 -(i_min + (cur_j-j_min)/mat_width)]);
            //println!("cm : {cur_msg}");
        }
        //println!("");
        msg.push(cur_msg);
    }
    return msg;
    
}

///
/// Transform the matrix format needed by @viterbi to one that can be used with get_error_code_msg
/// 
/// 
/// 
fn uncompress_matrix(data : &Vec<usize>,h:usize) -> Vec<Vec<u8>>{
    let mut matrix = vec![vec![0;h];data.len()];
    for j in 0..data.len(){
        let mut mask =1;
        for i in 0..h{
            if data[j]&mask!=0{
                matrix[j][i]=1;
            }
            mask*=2;
        }
    }

    return matrix;
}

fn get_img_msg_bit_vec<T:ImgIterator>(msg:&Vec<u8>,img : &mut RgbImage,iterator:&mut T, enconding: & dyn CharEncoding)-> (Vec<u8>,Vec<u8>){
   
    let mut encoded_bytes_count=0;
    let (img_w,img_h) = img.dimensions();
    let mut bytes_to_hide= Vec::with_capacity(8*msg.len());
    println!("oui");
    for c in msg{
        let encodeds = match enconding.get_enconding(*c){
            Some(result_encoding) => result_encoding,
            None => continue,
        };

        //we create a vec containing the bits we have to "or" with image's pixels
        
        for encoded in &encodeds{ //we iterate over all bytes
            for i in 0..8{ //we cut all bytes in 8
                //we keep the 2 leasts significant bits first
                //then bits 3 & 4 
                //...
                //and we move these bits to the 2 lsb (so the firsts are not moved, 3&4 are moved 2 bits to the right....)
                bytes_to_hide.push(((encoded>>i)&0x1) as u8);
            }
        }
        
        encoded_bytes_count+=encodeds.len();
        
    }
    //println!("....");
    let mut img_bits = Vec::<u8>::with_capacity((img_h*img_w*3) as usize);
    let mut  next_pixel =iterator.next();
    while  next_pixel.is_some(){
        let (x,y,rgb)= next_pixel.unwrap();
        img_bits.push(img.get_pixel(x, y).0[rgb]&0x1);
        next_pixel=iterator.next();
    }
    return (img_bits,bytes_to_hide);
}
///
/// Hide a message inside an image using 
/// 
/// 
pub fn viterbi_hide_msg<T:ImgIterator>(msg :&Vec<u8> , enconding: & dyn CharEncoding, img_key: Option<&[u8]>,h_mat: Option<&Vec<usize>>, h :u32, alpha: Option<usize>, hide_len:bool,img : &mut RgbImage) {
    let mut msg=msg.clone();
    if hide_len{
        let len = msg.len();
        msg.insert(0, ((len)&0xFF) as u8);
        msg.insert(1, ((len>>8)&0xFF) as u8);
        msg.insert(2, ((len>>16)&0xFF) as u8);
        msg.insert(3, ((len>>24)&0xFF) as u8);
    }
    let dim = img.dimensions();
   
    let mut iterator = T::new(img,img_key);
    let h_matrix;
    let temp_mat;
    if h_mat.is_some(){
        h_matrix=h_mat.unwrap();
    }
    else{
        let alpha = alpha.unwrap_or(2);
        temp_mat=default_matrix(h as usize, alpha);
        h_matrix=&temp_mat;
    }
    let img_cap = (dim.0*dim.1*3)as usize/(8*h_matrix.len() );
    if msg.len()>img_cap{
        println!("Warning : msg to long to fit, cutted to {} bytes",img_cap-1-h_matrix.len());
        msg.truncate(img_cap-1-h_matrix.len());
    }
    let (img_bits, bytes_to_hide) = get_img_msg_bit_vec(&msg, img, &mut iterator, enconding);
    let (stegano,weight) = viterbi(&img_bits, &bytes_to_hide, h_matrix, h, &vec![1.0;img_bits.len()]);
        
    println!("encoded {} bits...",bytes_to_hide.len());
    println!("weight of the modifcation : {}", weight);
    let mut index=0;
    let mut iterator = T::new(img,img_key);
    let mut next_pixel =iterator.next();
    while  next_pixel.is_some()&&index<stegano.len(){
        let (x,y,rgb)= next_pixel.unwrap();
        let pixel =img.get_pixel_mut(x, y);
        pixel.0[rgb]=(pixel.0[rgb]&(!0x1))|stegano[index];
        next_pixel=iterator.next();
        index+=1;
    }
}

pub fn viterbi_retrieve_msg<T:ImgIterator>(length :Option<i32> , enconding: & dyn CharEncoding, img_key: Option<&[u8]>,h_mat: Option<&Vec<usize>>, h :u32, alpha: Option<usize>, img : &mut RgbImage) -> Vec<u8>{
    let mut iterator = T::new(img,img_key);
    let h_matrix;
    let temp_mat;
    if h_mat.is_some(){
        h_matrix=h_mat.unwrap();
    }
    else{
        let alpha = alpha.unwrap_or(2);
        temp_mat=default_matrix(h as usize, alpha);
        h_matrix=&temp_mat;
    }
    let h = uncompress_matrix(&h_matrix, h as usize);
    let (img_bits, _bytes_to_hide) = get_img_msg_bit_vec(&vec![0_u8], img, &mut iterator, enconding);
    let t_length=match length {
        Some(t)=> t, 
        //retrieving the len inside the img...
        None => | v : Vec<u8>| -> i32 {let mut l:i64 = 0;
                                    for i in 0..(32/8){ 
                                        for j in  0..8{
                                            l|= ((v[(8*i+j) as usize]&1) as i64)<<i*8+j;
                                        }
                                    }
                                    println!("found len : {l}");
                                    return l as i32+4;}
            (get_error_code_msg(&img_bits, &h, 32)),
    };
    let msg_bits = get_error_code_msg(&img_bits, &h, 8*t_length as usize);
    let mut msg = Vec::with_capacity(msg_bits.len()/8);
    let index_min= match length {
        Some(_)=>0,
        None=>4,
    };
    for i in index_min..(msg_bits.len()/8){
        let mut cur_byte=0; 
        for j in  0..8{
            cur_byte|= (msg_bits[8*i+j]&1)<<j;
        }
        msg.push(cur_byte);
        //println!("cur b : {cur_byte}");
    }
    return msg;
    //return String::from_utf8(result).unwrap();
}

fn default_matrix(h:usize,alpha: usize) -> Vec<usize>{
    return match (h,alpha) {
        (7,2) => vec![71,109],
        (7,3) => vec![95,101,121],
        (7,4) => vec![81,95,107,121],
        (7,5) => vec![75,95,97,105,117],
        (7,6) => vec![73,83,95,103,109,123],
        (7,7) => vec![69,77,93,107,111,115,121],
        (7,8) => vec![69,79,81,89,93,99,107,119],
        (7,9) => vec![69,79,81,89,93,99,107,119,125],

        (10,3) => vec![621, 659, 919],
        (10,4) => vec![601, 703, 893, 955],
        (10,5) => vec![635, 725, 775, 929, 975],
        (10,6) => vec![73, 659, 747, 793, 959, 973],
        (10,7) => vec![589, 601, 679, 751, 831, 851, 989],
        (10,8) => vec![589, 631, 689, 713, 821, 899, 971, 1013],
        (10,9) => vec![531, 589, 603, 699, 735, 789, 857, 903, 1017],

        _=> vec![3,2],
    }
}

#[cfg(test)]
mod test_hugo{
  

    use crate::{lsb::{ASCIIEncoding, DefaultIterator}, utilities::encryption::keygen_aes128, HashIterator};
    use image::io::Reader as ImageReader;
    use sha2::Sha512;
  

    use super::*;
    use rand::prelude::*;
    fn rdm_u8_vec(max: u8,count:u32)-> Vec<u8>{
        let mut vec = Vec::<u8>::with_capacity(count as usize);
        for _ in 0..count{
            vec.push(random::<u8>()%max);
        }
        return vec;
    }
    fn rdm_usize_vec(max: usize,count:u32)-> Vec<usize>{
        let mut vec = Vec::<usize>::with_capacity(count as usize);
        for _ in 0..count{
            vec.push(random::<usize>()%max);
        }
        return vec;
    }
   
    #[test]
    fn test_viterbi(){
        let mat = vec![3,2,1];
        let u_mat = uncompress_matrix(&mat, 2);
        assert_eq!(u_mat,vec![vec![1,1,],vec![0,1],vec![1,0]]);
        //let msg = rdm_u8_vec(2, 4);
        //let img = rdm_u8_vec(2, 8);
        //let img = vec![0,0,0,0,0,0,0,0];
        let msg = vec![0,1,1,1];

        let img = vec![1,0,1,1,0,0,0,1];
        let expected_img = vec![0,0,1,1,1,0,0,1];
        
        //let t_mat = vec![vec![1,1,0],vec![0,1,0],vec![1,0,0]];
        let mat = vec![3,2];
        let t_mat = vec![vec![1,1],vec![0,1]];
        
        println!("-MSG : {:?}",msg);
        println!("-IMG : {:?}",img);

        let (y,weight)= viterbi(&img.clone(), &msg.clone(), &mat, 2, &vec![1.;100]);
        let yc= y.clone();
        println!("-WEIGHT : {weight} , -Y : {:?}",yc);
        let msg2 = get_error_code_msg(&y.clone(), &t_mat.clone(), 4);
        println!("MSG-OUT : {:?}",msg2);
       
        assert_eq!(msg,msg2);
        
        let img = rdm_u8_vec(2, 1024);
        let h_mat = rdm_usize_vec(128, 32);
        let msg = rdm_u8_vec(2, 32);
        let y = viterbi(&img, &msg, &h_mat, 7, &vec![1.;1024]);
        let y2 = viterbi(&y.0, &msg, &h_mat, 7, &vec![1.;1024]);
        assert_eq!(y.0,y2.0);
        let mat =uncompress_matrix(&h_mat,7);
        println!("compressed mat : {:?}",h_mat);
        println!("mat :");
        for i in 0..mat.len(){
            print!("{:?}",mat[i]);
            let mut s = 0;
            for j in 0..7{
                s<<=1;
                s+=mat[i][6-j];
            }
            println!("   | {s}");
        }
        let msg2 = get_error_code_msg(&y.0,& mat,32);
        println!("MSG-IN : {:?}",msg);
        println!("MSG-OUT : {:?}",msg2);
        println!("weight : {}",y.1);
        assert_eq!(msg,msg2);

    }
    #[test]
    fn test_viterbi_steg(){
        let mut image = ImageReader::open("./unit_tests/in/default_img/maddy-pfff.png").unwrap().decode().unwrap().to_rgb8();
        let msg = String::from("oui bonjour ceci est un test je suppose ouaaaaahhh yen a des mots brefffffff fr fr fr fr fr fr fr fr fr fr fr fr fr fr fr fr fr fr fr fr fr fr fr fr fr fr fr fr fr fr fr fr fr fr fr fr fr fr");
        let key_img_rng = keygen_aes128();
        let h_mat = rdm_usize_vec(128, 32);
        let encoding= Box::new(ASCIIEncoding);
        let h=10;
        //when giving the len
        viterbi_hide_msg::<DefaultIterator>(&msg.clone().into_bytes(), encoding.as_ref(),Some(&key_img_rng),None,h,Some(9), false,&mut image);
        image.save("./unit_tests/out/sigma/madeline-pfff.png").unwrap();
        let mut  rd_msg = viterbi_retrieve_msg::<DefaultIterator>(Some((msg.len() )as i32),encoding.as_ref(),Some(&key_img_rng),  
            None,h,Some(9),&mut image);
        println!("FIN!!!!!");
        rd_msg.truncate(msg.len());
        let mes_retrieved=String::from_utf8(rd_msg).unwrap();
        println!("resultat algo viterbi : {}",mes_retrieved);
        assert_eq!(msg,mes_retrieved);
        
        //without giving the len
        viterbi_hide_msg::<DefaultIterator>(&msg.clone().into_bytes(), encoding.as_ref(),Some(&key_img_rng),None,h,Some(9), true,&mut image);
        image.save("./unit_tests/out/sigma/madeline-pfff.png").unwrap();
        let mut  rd_msg = viterbi_retrieve_msg::<DefaultIterator>(None,encoding.as_ref(),Some(&key_img_rng),  
            None,h,Some(9),&mut image);
        println!("FIN!!!!!");
        rd_msg.truncate(msg.len());
        let mes_retrieved=String::from_utf8(rd_msg).unwrap();
        println!("resultat algo viterbi : {}",mes_retrieved);
        assert_eq!(msg,mes_retrieved);
        image.save("./unit_tests/out/viterbi/madeline-pfff.png").unwrap();
    }
}