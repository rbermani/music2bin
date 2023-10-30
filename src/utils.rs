#[macro_export]
macro_rules! function {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        &name[..name.len() - 3]
    }};
}

#[cfg(windows)]
pub const NL: &str = "\r\n";
#[cfg(not(windows))]
pub const NL: &str = "\n";

pub mod into_option {
    pub trait IntoOption<T> {
        fn into_option(self) -> Option<Vec<T>>;
    }

    impl<T> IntoOption<T> for Vec<T> {
        fn into_option(self) -> Option<Vec<T>> {
            if self.is_empty() {
                None
            } else {
                Some(self)
            }
        }
    }
}
