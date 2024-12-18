#![allow(unused_imports, dead_code)]
use std::f32::consts;

mod arrayi32;
use arrayi32::*;

mod arrayu32;
use arrayu32::*;

mod arrayisize;
use arrayisize::*;

mod arrayusize;
use arrayusize::*;

mod arraybool;
use arraybool::*;

mod arrayf32;
use arrayf32::*;

mod utils;
use utils::*;

fn main() {
    //let temp = vec![1, 2, 3, 4, 5,]
    //    .into_iter()
    //    .map(|i| Some(i))
    //    .collect();
    //let temp = vec![
    //    Some(1),
    //    Some(15),
    //    None,
    //    None,
    //    Some(5),
    //    Some(25),
    //    Some(1),
    //    None,
    //];

    //let array = ArrayU32::from_vec(temp);
    //dbg!(&array);
    //let iter = array.iter();
    //println!("{}", array.is_null(3));

    //for val in iter {
    //    println!("{val:?}");
    //}

    let temp = [3.14, -3.1, 0.01, -8.9, 42.0, consts::E, consts::PI];
    let array = Into::<ArrayF32>::into(&temp);

    dbg!(array);
}
