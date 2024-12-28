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

mod union;
use union::*;

mod utils;
use utils::*;

fn main() {
    let elems = ["one", "1", "1.00", "", "-14", "false", "null", "Bublé"];

    let mut builder = union::UnionBuilder::new();

    elems.into_iter().for_each(|val| builder.parse_push(val));

    let max = -(u32::MAX as isize) + 1;

    builder.parse_push(max.to_string());

    let un = Union::from_builder(builder);

    dbg!(un);

    //dbg!(un.get(8));
}
