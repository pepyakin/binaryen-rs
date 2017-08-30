use std::ffi::CString;
use std::os::raw::c_char;

pub struct Name(CString);

impl Name {
    pub(crate) fn as_ptr(&self) -> *const c_char {
        self.0.as_ptr()
    }
}

impl From<String> for Name {
    fn from(s: String) -> Name {
        Name(CString::new(s).unwrap())
    }
}

impl<'a> From<&'a str> for Name {
    fn from(s: &str) -> Name {
        Name(CString::new(s).unwrap())
    }
}

impl From<CString> for Name {
    fn from(s: CString) -> Name {
        Name(s)
    }
}
