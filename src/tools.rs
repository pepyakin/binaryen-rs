use Module;
use ffi;
use std::os::raw::c_void;

/// Convert some random array of bytes to a Module.
pub fn translate_to_fuzz(seed: &[u8]) -> Module {
    if seed.len() == 0 {
        return Module::new();
    }

    unsafe {
        let raw_module = ffi::translateToFuzz(
            seed.as_ptr() as *const c_void, 
            seed.len()
        );
        Module::from_raw(raw_module)
    }
}

#[cfg(test)]
mod tests {
    use super::translate_to_fuzz;
    use rand::{self, Rng};

    #[test]
    fn test_translate_to_fuzz() {
        let mut seed = vec![0; 1000];
        for _ in 0..1000 {
            let mut rng = rand::thread_rng();
            rng.fill_bytes(&mut seed);
            let module = translate_to_fuzz(&seed);
            
            assert!(module.is_valid());
        }
    }
}
