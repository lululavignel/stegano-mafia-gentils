use image::RgbImage;
use polynomen::poly;

use crate::shannon_entropy::randomize_lsb;
pub enum Group{
    Regular,
    Singular,
    Unusable,
}

pub fn discriminant(data : &[i16])-> u32{
    let mut sum=0;
    for i in 0..(data.len()-1){
        sum+= (data[i] as u16).abs_diff(data[i+1] as u16) as u32
    }
    return sum;
}

pub fn apply_mask(data : &mut[i16],pos_mask:&[&[i16]]){
    let mut index=0;
    for line in pos_mask{
        for element in *line{
            match element {
                1  => data[index] += 1 - (data[index]%2)*2,
                -1 => data[index] -= 1 - (data[index]%2)*2,
                0  => (),
                _  => println!("WARNING : unknown mask value : {} at index : {} doing nothing for this one", element,index)
            };
            index+=1;
        }
    }
}

pub fn compute_group(img:&RgbImage,pos_mask:&[&[i16]],x_start:u32,y_start:u32,x_end:u32,y_end:u32,color:usize) -> Group{
    let mut data = Vec::<i16>::new();
    for x in x_start..x_end{
        for y in y_start..y_end{
            data.push(img.get_pixel(x, y).0[color] as i16);
        }
    }
    let base_discr = discriminant(&data);
    apply_mask(&mut data, pos_mask);
    let modified_discr = discriminant(&data);
    if modified_discr>base_discr{
        return  Group::Regular;
    }
    if modified_discr<base_discr{
        return  Group::Singular;
    }
    return Group::Unusable;
}
pub fn group_stats(img:&RgbImage,pos_mask:&[&[i16]])-> (f64,f64,f64,i32){
    let x_mask= pos_mask.len() as u32;
    let y_mask= pos_mask[0].len() as u32;
    let mut total=0;
    let mut singular_c=0;
    let mut regular_c=0;
    let mut unusable_c=0;
    let (x_img_dim ,y_img_dim) = img.dimensions();
    for x in 0..x_img_dim/x_mask{
        for y in 0..y_img_dim/y_mask{
            for color in 0..3{
                let group= compute_group(img,pos_mask,x*x_mask,y*y_mask,(x+1)*x_mask,(y+1)*y_mask,color);
                match group {
                    Group::Regular =>  regular_c+=1,
                    Group::Singular=> singular_c+=1,
                    Group::Unusable=> unusable_c+=1,
                }
                total+=1;
            }
        }
    }
    let total_f = total as f64;
    return (regular_c as f64/total_f,singular_c as f64/total_f,unusable_c as f64/total_f,total);

}

pub fn get_polynomial(img:&RgbImage,mask:&[&[i16]],verbose:bool) -> (f64,f64,f64){
    let positive_mask= mask;
    let mut negative_mask= Vec::<Vec::<i16>>::new();
    let mut line_v;
    for line in 0..positive_mask.len(){
        line_v = Vec::<i16>::with_capacity(positive_mask[line].len());
        for column in 0..positive_mask[line].len(){
            line_v.push(positive_mask[line][column]  * -1);
        }
        negative_mask.push(line_v);
    }
    let mut neg = Vec::<&[i16]>::new();
    for a in &negative_mask{
        neg.push(a);
    }
    let stats_p2 = group_stats(img, mask);
    let neg_stats_p2 = group_stats(img, neg.as_slice());

    let img_1_minus_p2=randomize_lsb(img,2.0,0x01 );
    let stats_1_minus_p2 = group_stats(&img_1_minus_p2, mask);
    let neg_stats_1_minus_p2 = group_stats(&img_1_minus_p2, neg.as_slice());

    let r_m_p_2=  stats_p2.0;
    let s_m_p_2=  stats_p2.1;

    let r_m_1_p_2=  stats_1_minus_p2.0;
    let s_m_1_p_2=  stats_1_minus_p2.1;

    let r_neg_m_p_2=  neg_stats_p2.0;
    let s_neg_m_p_2=  neg_stats_p2.1;

    let r_neg_m_1_p_2=  neg_stats_1_minus_p2.0;
    let s_neg_m_1_p_2=  neg_stats_1_minus_p2.1;


    let d_0 = r_m_p_2 -s_m_p_2;
    let d_1 = r_m_1_p_2 -s_m_1_p_2;

    let d_neg_0 = r_neg_m_p_2 -s_neg_m_p_2;
    let d_neg_1 = r_neg_m_1_p_2 -s_neg_m_1_p_2;

    let a = 2.*(d_1+d_0);
    let b= d_neg_0 - d_neg_1 - d_1 - 3.*d_0;
    let c = d_0- d_neg_0; 
    return  (a,b,c);
}

pub fn get_p_from_pol(a:f64,b:f64,c:f64,verbose:bool)-> Option<f64>{
    
    let delta = b*b  - 4.*a*c ;
    if verbose{
        println!("Polynome : {}X² + {}X + {}",a,b,c);
    }

    if delta<0.{
        return None;
    }
    let x;
    if delta==0.{
        x=-b/(2.*a);
        return Some(x/(x-(1./2.)));
    }

    let x1=(-b+delta.sqrt())/(2.*a);
    let x2=(-b-delta.sqrt())/(2.*a);
   
    let z1=x1/(x1-(1./2.));
    let z2=x2/(x2-(1./2.));
    if verbose{
        println!("X1::: {x1}");
        println!("X2::: {x2}");
        println!("Z1:: {}",z1);
        println!("Z2:: {}",z2);
    }
    if z1<0.{
        return Some(z2);
    }
    if z2<0. && z2>-0.01{
        return Some(0.);
    }
    if z2>0. && z2<z1{
        return Some(z2);
    }
    return Some(z1);  
}

pub fn compute_rs(img:&RgbImage,mask:&[&[i16]],verbose:bool) ->Option<f64>{
    let (a,b,c)= get_polynomial(img, mask, verbose);
    let opt_result = get_p_from_pol(a, b, c, verbose);
    return opt_result;
}

pub fn compute_dyn_rs(img:&RgbImage,masks:&Vec::<&[&[i16]]>,verbose:bool)-> Option<f64>{
    let mut q_min =99999.;
    let mut mask_min = masks[0];
    let mut best_pol = (0.,0.,0.);
    for mask in masks{
        let (a,b,c)= get_polynomial(img, mask, verbose);
        //                         X⁰        X¹                 X²          X³
        let pol = poly![2.*b*c,  4.*a*c+2.*b*b  ,  6.*a*b  ,  4.*a*a];

        let roots = pol.real_roots().unwrap();
        if verbose{
            println!("For mask : {:?}, roots are : {:?}",mask,roots);
        }
        for root in roots{
            let q= a*root*root +b*root+c;
            println!("for root {root} : {}", q);
            let abs_q = q.abs();
            if q_min>abs_q{
                q_min=abs_q;
                mask_min=mask;
                best_pol=(a,b,c);
            }
        }

    }
    return get_p_from_pol(best_pol.0, best_pol.1, best_pol.2, verbose);

}

pub fn default_compute_rs(img:&RgbImage) -> Option::<f64>{
    let mask= [[0 as i16 ,1,1,0].as_slice()];
    //let mask= [[0 as i16 ,0,0].as_slice(),[0 as i16 ,1,0].as_slice(),[0 as i16 ,0,0].as_slice()];
    //let mask= [[0 as i16 ,1,0].as_slice()];
    return compute_rs(&img,mask.as_slice(),false);
}
pub fn defautl_dyn_rs(img: &RgbImage) -> Option::<f64>{
    let m1 =[[0 as i16 ,1,1,0].as_slice()];
    let m2 =[[0 as i16 ,1,0,1].as_slice()];
    let m3 =[[0 as i16 ,1,0].as_slice()];
    let m4 =[[0 as i16 ,0,0].as_slice() , [0 as i16 ,1,0].as_slice(),[0 as i16 ,0,0].as_slice()];
    let masks = vec![m1.as_slice(),m2.as_slice(),m3.as_slice(),m4.as_slice()];
    return compute_dyn_rs(img,&masks,false);

}
#[cfg(test)]
mod test_entropy{


    use image::io::Reader as ImageReader;
    use super::*;


    #[test]
    fn test_compute_rs(){
        let values = [0.1,0.2,0.3,0.4,0.5,0.6,0.7,0.8,0.9,0.99];
        for value in values{
            let img_name=format!("/home/admin/Images/steg/s256-c-p-{}/IMG_20230425_115924.png",value);
            //let img_name=format!("/home/admin/Images/steg/qarr-p-{}/the-backrooms-image.png",value);
            
            //let img_name=format!("/home/admin/Images/steg/base_img/image16.png");
            let image = ImageReader::open(img_name).unwrap().decode().unwrap().to_rgb8();

            let mask= [[0 as i16 ,1,0].as_slice()];
            //let mask= [[0 as i16 ,0,0].as_slice(),[0 as i16 ,1,0].as_slice(),[0 as i16 ,0,0].as_slice()];
            let res = compute_rs(&image,mask.as_slice(),true);

            println!("========================\nResult for {}% steganographied : {}",value,res.unwrap());
        }
        
    
    } 
    #[test]
    fn test_dyn_compute_rs(){
        let values = [0.1,0.2,0.3,0.4,0.5,0.6,0.7,0.8,0.9,0.99];
        let m1 =[[0 as i16 ,1,1,0].as_slice()];
        let m2 =[[0 as i16 ,1,0,1].as_slice()];
        let m3 =[[0 as i16 ,1,0].as_slice()];
        let m4 =[[0 as i16 ,0,0].as_slice() , [0 as i16 ,1,0].as_slice(),[0 as i16 ,0,0].as_slice()];
        let masks = vec![m1.as_slice(),m2.as_slice(),m3.as_slice(),m4.as_slice()];
            //let mask= [[0 as i16 ,0,0].as_slice(),[0 as i16 ,1,0].as_slice(),[0 as i16 ,0,0].as_slice()];
        for value in values{
            let img_name=format!("/home/admin/Images/steg/s256-c-p-{}/the-backrooms-image.png",value);
            //let img_name=format!("/home/admin/Images/steg/base_img/image16.png");
            let image = ImageReader::open(img_name).unwrap().decode().unwrap().to_rgb8();

            
            let res = compute_dyn_rs(&image,&masks,true);

            println!("========================\nResult for {}% steganographied : {}",value,res.unwrap());
        }
    }
}
/*/home/admin/Images/steg/base_img/image16.png;2.0009061913359196*/

