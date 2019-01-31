#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub mod passes;

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    use std::ptr;

    #[test]
    fn test_fuzz() {
        let vec: Vec<u8> = vec![0, 1, 2, 3, 4, 5];
        unsafe {
            let module = translateToFuzz(vec.as_ptr() as *const i8, vec.len(), true);
            let result = BinaryenModuleValidate(module);
            assert!(result != 0);
        }
    }

    #[test]
    fn test_sanity() {
        // see https://github.com/WebAssembly/binaryen/blob/master/test/example/c-api-hello-world.c
        unsafe {
            let module = BinaryenModuleCreate();
            let mut params = [BinaryenInt32(), BinaryenInt32()];

            let func_type_name = CString::new("iii").unwrap();
            let iii = BinaryenAddFunctionType(
                module,
                func_type_name.as_ptr(),
                BinaryenInt32(),
                &mut params[0] as *mut BinaryenType, // TODO: Is this safe?
                2,
            );

            let x = BinaryenGetLocal(module, 0, BinaryenInt32());
            let y = BinaryenGetLocal(module, 1, BinaryenInt32());
            let add = BinaryenBinary(module, BinaryenAddInt32(), x, y);

            let func_name = CString::new("adder").unwrap();
            let _ = BinaryenAddFunction(
                module,
                func_name.as_ptr(),
                iii,
                ptr::null::<BinaryenType>() as *mut BinaryenType,
                0,
                add,
            );

            BinaryenModulePrint(module);
            BinaryenModuleDispose(module);
        }
    }
}
