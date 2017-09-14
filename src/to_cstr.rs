use std::ffi::{CStr, CString};
use std::ptr;
use std::os::raw::c_char;

pub struct Stash<T> {
    pub storage: T,
    pub ptr: *const c_char,
}

impl<T> Stash<T> {
    fn new(storage: T, ptr: *const c_char) -> Stash<T> {
        Stash { storage, ptr }
    }

    pub fn as_ptr(&self) -> *const c_char {
        self.ptr
    }
}

pub trait ToCStr<T> {
    fn to_cstr_stash(self) -> Stash<T>;
}

impl<'a, T: AsRef<CStr>> ToCStr<&'a CStr> for &'a T {
    fn to_cstr_stash(self) -> Stash<&'a CStr> {
        let r = self.as_ref();
        let ptr = r.as_ptr();
        Stash::new(r, ptr)
    }
}

impl<'a, T: AsRef<[u8]>> ToCStr<CString> for &'a T {
    fn to_cstr_stash(self) -> Stash<CString> {
        let vec = self.as_ref().to_vec();
        let cstring = CString::new(vec).unwrap();
        let ptr = cstring.as_ptr();
        Stash::new(cstring, ptr)
    }
}

impl<'a> ToCStr<CString> for &'a str {
    fn to_cstr_stash(self) -> Stash<CString> {
        let cstring = CString::new(self).unwrap();
        let ptr = cstring.as_ptr();
        Stash::new(cstring, ptr)
    }
}

// Good. We were given String and we convert it into CString.
impl ToCStr<CString> for String {
    fn to_cstr_stash(self) -> Stash<CString> {
        let cstring = CString::new(self).unwrap();
        let ptr = cstring.as_ptr();
        Stash::new(cstring, ptr)
    }
}

// Good, there is nothing to do here.
impl ToCStr<CString> for CString {
    fn to_cstr_stash(self) -> Stash<CString> {
        let ptr = self.as_ptr();
        Stash::new(self, ptr)
    }
}

pub fn to_cstr_stash_option<P, T: ToCStr<P>>(name: Option<T>) -> Stash<Option<P>> {
    match name {
        Some(str) => {
            let Stash { storage, ptr } = str.to_cstr_stash();
            Stash::new(Some(storage), ptr)
        }
        None => Stash::new(None, ptr::null()),
    }
}
