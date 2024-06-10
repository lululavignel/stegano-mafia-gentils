
use std::{array, usize};

use image::{GenericImageView, RgbImage};
use imageproc::drawing::Canvas;

pub struct Co_oc_mat{
    mat: Vec<Vec<Vec<Vec<i32>>>>,
    t:i32,
    delta_i:i32,
    delta_j:i32,
    histo : Option<Vec<(usize,i32,i32,i32,i32)>>, 
}

fn compact(t:i32,d1:i32,d2:i32,d3:i32)->(i32,i32,i32){
    let (mut d1,mut d2,mut d3)=(d1,d2,d3);
    if d1>t{
        d1=t;
    }
    if d1< -t{
        d1=-t;
    }
    if d2>t{
        d1=t;
    }
    if d2< -t{
        d2=-t;
    }   
    if d3>t{
        d3=t;
    }
    if d3< -t{
        d3=-t;
    }
    return (d1,d2,d3);
}

fn get_co_oc_mat(mat: &Co_oc_mat,rgb:usize,d1:i32,d2:i32,d3:i32)-> i32{
    let (d1,d2,d3) = compact(mat.t,d1,d2,d3);
    return mat.mat[rgb as usize][(d1+mat.t) as usize][(d2+mat.t) as usize][(d3+mat.t) as usize];
}
fn incr_co_oc_mat(mat: &mut Co_oc_mat,rgb:usize,d1:i32,d2:i32,d3:i32){
    let (d1,d2,d3) = compact(mat.t,d1,d2,d3);
    mat.mat[rgb as usize][(d1+mat.t) as usize][(d2+mat.t) as usize][(d3+mat.t) as usize]+=1;
    match mat.histo.as_mut(){
        Some(t) => t.push((rgb,d1,d2,d3,1)),
        None =>(),
    }
}
fn decr_co_oc_mat(mat: &mut Co_oc_mat,rgb:usize,d1:i32,d2:i32,d3:i32){
    let (d1,d2,d3) = compact(mat.t,d1,d2,d3);
    mat.mat[rgb as usize][(d1+mat.t) as usize][(d2+mat.t) as usize][(d3+mat.t) as usize]-=1;
    match mat.histo.as_mut(){
        Some(t) => t.push((rgb,d1,d2,d3,-1)),
        None =>(),
    }
}
fn reset_co_oc_mat(mat : &mut Co_oc_mat){
    let histo = mat.histo.as_mut().unwrap();
    while histo.len()>0{
        let (rgb, d1,d2,d3, _op)=histo.pop().unwrap();
        let (d1,d2,d3) = compact(mat.t,d1,d2,d3);
        mat.mat[rgb as usize][(d1+mat.t) as usize][(d2+mat.t) as usize][(d3+mat.t) as usize]=0; 
    }
}
pub fn compute_rhos( img : &mut RgbImage){
    let (x_dim,y_dim)=img.dimensions();
    let rho = Vec::<f64>::with_capacity((x_dim*y_dim*3) as usize);
}

pub fn compute_co_oc_mat3(t:usize,delta_i:i32,delta_j:i32,matrix:&RgbImage) -> Co_oc_mat{
    let mat = vec![vec![vec![vec![0;2*t+1];2*t+1];2*t+1];3];
    let mut co_oc_mat=Co_oc_mat{mat,t:t as i32,delta_i,delta_j,histo:Some(Vec::new())};
    let (x_dim,y_dim) = matrix.dimensions();
    let (x_min,x_max) = match delta_i {
        0=> (0,x_dim),
        _=> match delta_i>0{
            true=>(0,x_dim -2- (delta_i as u32)),
            false=>(2+(-delta_i) as u32 ,x_dim),
        },   
    };
    let (y_min,y_max) = match delta_i {
        0=> (0,y_dim),
        _=> match delta_j>0{
            true=>(0,y_dim -2- (delta_j as u32)),
            false=>(2+(-delta_j) as u32 ,y_dim),
        },   
    };
    let d_i: i32 = match delta_i {
        0=>0,
        _=> match delta_i>0{
            true => 1,
            false => -1,
        }
    };
    let d_j: i32 = match delta_j {
        0=>0,
        _=> match delta_j>0{
            true => 1,
            false => -1,
        }
    };

    for i in x_min..x_max{
        for j in y_min..y_max{
            let pixel_0 = matrix.get_pixel(i, j).0;
            let pixel_1 = matrix.get_pixel((i as i32 +d_i) as u32, (j as i32 +d_j)as u32).0;
            let pixel_2 = matrix.get_pixel((i as i32 +2*d_i) as u32, (j as i32 +2*d_j)as u32).0;
            let pixel_3 = matrix.get_pixel((i as i32 +3*d_i) as u32, (j as i32 +3*d_j)as u32).0;
            for rgb in 0..3{
                let d1 = pixel_0[rgb] as i32 -pixel_1[rgb] as i32;
                let d2 = pixel_1[rgb] as i32 -pixel_2[rgb] as i32;
                let d3 = pixel_2[rgb] as i32 -pixel_3[rgb] as i32;
                incr_co_oc_mat(&mut co_oc_mat,rgb,d1,d2,d3);
            }
        }
    }
    return co_oc_mat;
}
pub fn delta_pixel(mat: &Co_oc_mat,i:usize,j:usize,d_i:i32,d_j:i32){
    let ds = [[0;3];3];
}

fn compute_c_d1_d2_d3(i:i32,j:i32,d_i:i32,d_j:i32,matrix:&mut RgbImage) -> [(i32, i32, i32); 3]{
    let mut results: [(i32, i32, i32); 3]= [(0_i32,0_i32,0_i32);3];
    let pixel_0 = matrix.get_pixel(i as u32, j as u32).0;
    let pixel_1 = matrix.get_pixel((i as i32 +d_i) as u32, (j as i32 +d_j)as u32).0;
    let pixel_2 = matrix.get_pixel((i as i32 +2*d_i) as u32, (j as i32 +2*d_j)as u32).0;
    let pixel_3 = matrix.get_pixel((i as i32 +3*d_i) as u32, (j as i32 +3*d_j)as u32).0;
    for rgb in 0..3{
        let d1 = pixel_0[rgb] as i32 -pixel_1[rgb] as i32;
        let d2 = pixel_1[rgb] as i32 -pixel_2[rgb] as i32;
        let d3 = pixel_2[rgb] as i32 -pixel_3[rgb] as i32;
        results[rgb]=(d1,d2,d3);
    }
    return results;

}
fn optimal_def_weight(d1:i32,d2:i32,d3:i32)-> f64{
    let gamma =4.;
    let sigma =10.;
    let distance = ((d1*d1+d2*d2+d3*d3)as f64).powf(0.5);
    return 1./((distance+sigma).powf(gamma));
    
}
pub fn delta_pixel_i_guess(mat_neut: &Co_oc_mat,mat_pos: &Co_oc_mat,mat_neg: &Co_oc_mat,img: &mut RgbImage,i:usize,j:usize){
    let (mut delta_i,mut delta_j)=(mat_neut.delta_i,mat_neut.delta_j);
    let dim= img.dimensions();
    let dim_x=dim.0 as i32;
    let dim_y=dim.1 as i32;
    //                       [[(d1,d2,d3),rgb],delta_mult]
    let mut neutrals_values  = [[(0,0,0);3];3];
    let mut positives_values = [[(0,0,0);3];3];
    let mut negatives_values = [[(0,0,0);3];3];
    //asserting that it is in bound
    if i as i32-3*delta_i>=0 && i as i32+3*delta_i<dim_x
            && j as i32 -3*delta_j>=0 && j as i32+3*delta_j<dim_y{
        neutrals_values[0]=compute_c_d1_d2_d3(i as i32,j as i32,delta_i,delta_j,img);
        neutrals_values[1]=compute_c_d1_d2_d3(i as i32 -   delta_i,j as i32 -  delta_j,delta_i,delta_j,img);
        neutrals_values[2]=compute_c_d1_d2_d3(i as i32 - 2*delta_i,j as i32 -2*delta_j,delta_i,delta_j,img);
        let pixel = img.get_pixel_mut(i as u32, j as u32);
        pixel.0[0]+=1;
        pixel.0[1]+=1;
        pixel.0[2]+=1;
        positives_values[0]=compute_c_d1_d2_d3(i as i32,j as i32,delta_i,delta_j,img);
        positives_values[1]=compute_c_d1_d2_d3(i as i32 -   delta_i,j as i32 -  delta_j,delta_i,delta_j,img);
        positives_values[2]=compute_c_d1_d2_d3(i as i32 - 2*delta_i,j as i32 -2*delta_j,delta_i,delta_j,img);
        let pixel = img.get_pixel_mut(i as u32, j as u32);
        pixel.0[0]-=2;
        pixel.0[1]-=2;
        pixel.0[2]-=2;
        negatives_values[0]=compute_c_d1_d2_d3(i as i32,j as i32,delta_i,delta_j,img);
        negatives_values[1]=compute_c_d1_d2_d3(i as i32 -   delta_i,j as i32 -  delta_j,delta_i,delta_j,img);
        negatives_values[2]=compute_c_d1_d2_d3(i as i32 - 2*delta_i,j as i32 -2*delta_j,delta_i,delta_j,img);
        let pixel = img.get_pixel_mut(i as u32, j as u32);
        pixel.0[0]+=1;
        pixel.0[1]+=1;
        pixel.0[2]+=1;
    }

}

pub fn delta_pixels(mat: &Co_oc_mat,i:usize,j:usize){
    let (mut delta_p,mut delta_m)=(0,0);


    for d_i in -1..=1{
        for d_j in -1..=1{
            if d_i==0&&d_j==0{
                continue;
            }
            //let cur_val = 
        }
    }

}