pub extern crate binaryen_sys;

#[cfg(test)]
extern crate rand;

#[cfg(test)]
extern crate wabt;

pub use binaryen_sys as ffi;

use std::rc::Rc;
use std::os::raw::c_char;
use std::{ptr, slice};
use std::sync::{Once, Mutex};
use std::ffi::CString;

pub mod tools;


/// Codegen configuration.
/// 
/// Use `set_global_codegen_config`.
pub struct CodegenConfig {
    /// 0, 1, 2 correspond to -O0, -Os, -Oz
    pub shrink_level: u32,
    /// 0, 1, 2 correspond to -O0, -O1, -O2, etc.
    pub optimization_level: u32,
    /// If set, the names section is emitted.
    pub debug_info: bool,
}

/// Set the global code generation configuration.
/// 
/// This can be used to set parameters before running `optimize` function. 
/// However, this can influence behavior of running binaryen passes in general (for example, 
/// `auto_drop` is implemented via a pass).
pub fn set_global_codegen_config(codegen_config: &CodegenConfig) {
    static mut MUTEX: Option<Mutex<()>> = None;
    static INIT: Once = Once::new();

    // Initialize the mutex only once.
    INIT.call_once(|| {
        unsafe {
            // This is safe since we are in `call_once`, and it will execute this closure only once.
            // If the second invocation happens before the closure returned then the invocation will be blocked until 
            // the closure returns.
            MUTEX = Some(Mutex::new(()));
        }
    });

    unsafe {
        let _guard = MUTEX
            .as_ref()
            .expect("should be initialized in call_once block above")
            .lock()
            .unwrap();

        ffi::BinaryenSetOptimizeLevel(codegen_config.optimization_level as i32);
        ffi::BinaryenSetShrinkLevel(codegen_config.shrink_level as i32);
        ffi::BinaryenSetDebugInfo(codegen_config.debug_info as i32);
    }
}

fn is_valid_pass(pass: &str) -> bool {
    let cstr = CString::new(pass).unwrap();
    unsafe { ffi::BinaryenShimPassFound(cstr.as_ptr()) }
}

struct InnerModule {
    raw: ffi::BinaryenModuleRef,
}

impl Drop for InnerModule {
    fn drop(&mut self) {
        unsafe { ffi::BinaryenModuleDispose(self.raw) }
    }
}

/// Modules contain lists of functions, imports, exports, function types.
pub struct Module {
    inner: Rc<InnerModule>,
}

impl Default for Module {
    fn default() -> Module {
        Module::new()
    }
}

impl Module {
    /// Create a new empty Module.
    pub fn new() -> Module {
        unsafe { 
            let raw = ffi::BinaryenModuleCreate();
            Module::from_raw(raw)
        }
    }

    /// Deserialize a module from binary form.
    pub fn read(wasm_buf: &[u8]) -> Result<Module, ()> {
        unsafe {
            let raw = ffi::BinaryenModuleSafeRead(wasm_buf.as_ptr() as *mut c_char, wasm_buf.len());
            if raw.is_null() {
               return Err(())
            }
            Ok(Module::from_raw(raw))
        }
    }

    pub unsafe fn from_raw(raw: ffi::BinaryenModuleRef) -> Module {
        Module {
            inner: Rc::new(InnerModule { raw }),
        }
    }

    /// Run the standard optimization passes on the module.
    /// 
    /// It will take into account code generation configuration set by `set_global_codegen_config`.
    pub fn optimize(&self) {
        unsafe { ffi::BinaryenModuleOptimize(self.inner.raw) }
    }

    /// Run a specified set of optimization passes on the module.
    ///
    /// This WILL NOT take into account code generation configuration set by `set_global_codegen_config`.
    pub fn run_optimization_passes<B: AsRef<str>, I: IntoIterator<Item = B>>(
        &self,
        passes: I,
    ) -> Result<(), ()> {
        let mut cstr_vec: Vec<_> = vec![];

        for pass in passes {
            if !is_valid_pass(pass.as_ref()) {
                return Err(());
            }

            cstr_vec.push(CString::new(pass.as_ref()).unwrap());
        }

        // NOTE: BinaryenModuleRunPasses expectes a mutable ptr
        let mut ptr_vec: Vec<_> = cstr_vec
            .iter()
            .map(|pass| pass.as_ptr())
            .collect();

        unsafe { ffi::BinaryenModuleRunPasses(self.inner.raw, ptr_vec.as_mut_ptr(), ptr_vec.len() as u32) };
        Ok(())
    }

    /// Validate a module, printing errors to stdout on problems.
    pub fn is_valid(&self) -> bool {
        unsafe { ffi::BinaryenModuleValidate(self.inner.raw) == 1 }
    }

    /// Serialize a module into binary form.
    ///
    /// # Examples
    ///
    /// ```
    /// # use binaryen::Module;
    /// let module = Module::new();
    /// let wasm = module.write();
    /// ```
    pub fn write(&self) -> Vec<u8> {
        unsafe {
            let write_result = ffi::BinaryenModuleAllocateAndWrite(self.inner.raw, ptr::null());

            // Create a slice from the resulting array and then copy it in vector.
            let binary_slice = if write_result.binaryBytes == 0 {
                &[]
            } else {
                slice::from_raw_parts(
                    write_result.binary as *const u8, 
                    write_result.binaryBytes
                )
            };
            let binary_buf = binary_slice.to_vec();

            // This will free buffers in the write_result.
            ffi::BinaryenShimDisposeBinaryenModuleAllocateAndWriteResult(write_result);

            binary_buf
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let module = Module::default();
        assert!(module.is_valid());
    }

    #[test]
    fn test_new() {
        let module = Module::new();
        assert!(module.is_valid());
    }

    #[test]
    fn test_optimization_passes() {
        const CODE: &'static str = 
        r#"
            (module
                (table 1 1 anyfunc)
            
                (type $return_i32 (func (result i32)))
                (func $test (; 0 ;) (result i32)
                    (call_indirect (type $return_i32)
                        (unreachable)
                    )
                )
            )
        "#;
        let code = wabt::wat2wasm(CODE).unwrap();
        let module = Module::read(&code).unwrap();

        assert!(module.is_valid());

        module.run_optimization_passes(&["vacuum", "untee"]).expect("passes succeeded");

        assert!(module.is_valid());
    }

    #[test]
    fn test_invalid_optimization_passes() {
        let module = Module::new();
        assert!(module.run_optimization_passes(&["invalid"]).is_err());
    }
}
