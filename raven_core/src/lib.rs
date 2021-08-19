macro_rules! err {
    ($($arg:tt)*) => {
        ::std::boxed::Box::<dyn ::std::error::Error>::from(format!($($arg)*))
    }
}

mod assets;
mod import;
