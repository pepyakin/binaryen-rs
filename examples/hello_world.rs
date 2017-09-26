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
