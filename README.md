# `binaryen-rs`

[![Build Status](https://travis-ci.org/pepyakin/binaryen-rs.svg?branch=master)](https://travis-ci.org/pepyakin/binaryen-rs) 
[![crates.io](https://img.shields.io/crates/v/binaryen.svg)](https://crates.io/crates/binaryen)
[![docs.rs](https://docs.rs/binaryen/badge.svg)](https://docs.rs/binaryen/)

[Binaryen](https://github.com/WebAssembly/binaryen) bindings for Rust.

With Binaryen you can create optimized [WebAssembly](http://webassembly.org/) modules.

For example what you can create with Binaryen you can check out [DEMO](https://pepyakin.github.io/emchipten/)*. Yes, this is [CHIP-8](https://en.wikipedia.org/wiki/CHIP-8) roms compiled straight to the WebAssembly. See [emchipten](https://github.com/pepyakin/emchipten) test bed for this project.

(*) Modern browser required

## Example

```rust
extern crate binaryen;

use binaryen::*;

fn main() {
    let module = Module::new();

    let params = &[ValueTy::I32, ValueTy::I32];
    let iii = module.add_fn_type(Some("iii"), params, Ty::I32);

    let x = module.get_local(0, ValueTy::I32);
    let y = module.get_local(1, ValueTy::I32);
    let add = module.binary(BinaryOp::AddI32, x, y);

    let _adder = module.add_fn("adder", &iii, &[], add);

    assert!(module.is_valid());

    module.print();
}
```

This example will print:

```WebAssembly
(module
 (type $iii (func (param i32 i32) (result i32)))
 (memory $0 0)
 (func $adder (type $iii) (param $0 i32) (param $1 i32) (result i32)
  (i32.add
   (get_local $0)
   (get_local $1)
  )
 )
)
```
