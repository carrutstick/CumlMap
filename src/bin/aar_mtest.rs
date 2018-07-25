#![feature(alloc_system)]
extern crate alloc_system;
extern crate cuml_map;

#[global_allocator]
static GLOBAL: alloc_system::System = alloc_system::System;

use cuml_map::CumlMap;
use cuml_map::AARCumlTree;

fn main() {
    let mut t = AARCumlTree::with_capacity(10);
    t.insert(4, 2);
    t.insert(5, 2);
    t.insert(8, 2);
    t.insert(9, 2);
    t.insert(10, 2);
    t.insert(6, 2);
    t.insert(7, 2);
}
