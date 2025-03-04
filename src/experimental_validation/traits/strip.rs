use std::path::PathBuf;

/// Sexy.
pub trait Strip {
    type Output;

    fn strip(self) -> Self::Output;
}

// TODO: Add generics
#[macro_export]
macro_rules! impl_strip_self {
    ($t:ty) => {
        impl Strip for $t {
            type Output = Self;

            fn strip(self) -> Self {
                self
            }
        }
    };
}

impl_strip_self!(PathBuf);
