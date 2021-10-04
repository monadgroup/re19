#[macro_export]
macro_rules! cstr {
    ($s:expr) => {
        concat!($s, "\0").as_ptr() as *const ::std::os::raw::c_char
    };
}
