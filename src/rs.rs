use image::{GenericImageView, RgbImage};
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
    return (singular_c as f64/total_f,regular_c as f64/total_f,unusable_c as f64/total_f,total);

}

pub fn compute_rs(img:&RgbImage,mask:&[&[i8]]) ->f64{


    let steganographied_percentage=0.;
    return steganographied_percentage;
}