#![allow(unused_imports, dead_code)]
use std::alloc::{self, Layout};
use std::f64::consts;
use std::ptr::{self, NonNull};

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

mod arrayf64;
use arrayf64::*;

mod arraytext;
use arraytext::*;

mod utils;
use utils::*;

fn main() {
    let elems = [
        String::from("It turns"),
        String::from("All"),
        String::from("Your good"),
        String::from("Feelings into bad feelings."),
        String::from("Its a"),
        String::from("Nightmare💀!"),
    ];

    let array = Into::<ArrayText>::into(elems);
    dbg!(array);
}
