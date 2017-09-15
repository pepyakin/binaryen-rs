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

pub trait ToCStr {
    type Storage;
    fn to_cstr_stash(self) -> Stash<Self::Storage>;
}

impl<'a> ToCStr for &'a CStr {
    type Storage = &'a CStr;
    fn to_cstr_stash(self) -> Stash<&'a CStr> {
        let r = self.as_ref();
        let ptr = r.as_ptr();
        Stash::new(r, ptr)
    }
}

impl ToCStr for CString {
    type Storage = CString;
    fn to_cstr_stash(self) -> Stash<CString> {
        let ptr = self.as_ptr();
        Stash::new(self, ptr)
    }
}

impl<'a> ToCStr for &'a str {
    type Storage = CString;
    fn to_cstr_stash(self) -> Stash<CString> {
        let cstring = CString::new(self).unwrap();
        let ptr = cstring.as_ptr();
        Stash::new(cstring, ptr)
    }
}

impl ToCStr for String {
    type Storage = CString;
    fn to_cstr_stash(self) -> Stash<CString> {
        let cstring = CString::new(self).unwrap();
        let ptr = cstring.as_ptr();
        Stash::new(cstring, ptr)
    }
}

pub fn to_cstr_stash_option<T: ToCStr>(name: Option<T>) -> Stash<Option<T::Storage>> {
    match name {
        Some(str) => {
            let Stash { storage, ptr } = str.to_cstr_stash();
            Stash::new(Some(storage), ptr)
        }
        None => Stash::new(None, ptr::null()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn accepts_cstr(x: *const c_char) {
        assert!(!x.is_null());
        unsafe {
            let cstr = CStr::from_ptr(x);
            let str_ = cstr.to_str().unwrap();
            println!("cstr={}, len={}", str_, str_.len());
        }
    }

    fn do_stuff_maybe<T: ToCStr>(name: Option<T>) {
        let stash = to_cstr_stash_option(name);
        if !stash.as_ptr().is_null() {
            accepts_cstr(stash.as_ptr());
        }
    }

    fn do_stuff<T: ToCStr>(name: T) {
        let stash = name.to_cstr_stash();
        accepts_cstr(stash.as_ptr());
    }

    #[test]
    fn test_use_cases() {
        do_stuff_maybe(Some("Some(String)".to_string()));
        do_stuff_maybe(Some("Some(&str)"));
        do_stuff_maybe(None::<&str>);

        do_stuff("String".to_string());
        do_stuff(&*"&String".to_string());

        let formatted_name = format!("format!");
        do_stuff(&*formatted_name);

        do_stuff("&str");


        do_stuff(&*CString::new("&*CString").unwrap());
        do_stuff(&*CString::new("&*CString, as_c_str()").unwrap().as_c_str());
    }
}
