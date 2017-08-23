extern crate binaryen_sys;

use std::ffi::CStr;
use binaryen_sys as ffi;

#[derive(Clone, Copy)]
pub enum ValueType {
    I32,
    I64,
    F32,
    F64,
}

impl ValueType {
    fn raw_type(self) -> ffi::BinaryenType {
        unsafe {
            match self {
                ValueType::I32 => ffi::BinaryenInt32(),
                ValueType::I64 => ffi::BinaryenInt64(),
                ValueType::F32 => ffi::BinaryenFloat32(),
                ValueType::F64 => ffi::BinaryenFloat64(),
            }
        }
    }
}

pub struct FuncType {
    inner: ffi::BinaryenFunctionTypeRef,
}

pub struct Func {
    inner: ffi::BinaryenFunctionRef,
}

pub struct FuncBuilder {

}

pub struct ModuleBuilder {
    inner: ffi::BinaryenModuleRef,
}

impl ModuleBuilder {
    pub fn new() -> ModuleBuilder {
        ModuleBuilder { inner: unsafe { ffi::BinaryenModuleCreate() } }
    }

    pub fn add_func_type<'a>(
        &'a mut self,
        name: &'a CStr,
        result_ty: ValueType,
        args_ty: &[ValueType],
    ) -> FuncType {
        let raw_result_ty = result_ty.raw_type();
        let mut raw_args_ty = args_ty.iter().cloned().map(ValueType::raw_type).collect::<Vec<_>>();
        let func_ty = unsafe {
            ffi::BinaryenAddFunctionType(
                self.inner,
                name.as_ptr(),
                raw_result_ty,
                raw_args_ty.as_mut_ptr(),
                raw_args_ty.len() as u32,
            )
        };
        FuncType { inner: func_ty }
    }

    pub fn add_func() -> FuncBuilder {

    }
}

impl Drop for ModuleBuilder {
    fn drop(&mut self) {
        unsafe {
            ffi::BinaryenModuleDispose(self.inner);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        let module = ModuleBuilder::new();
    }
}
