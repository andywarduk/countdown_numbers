#![warn(missing_docs)]

//! This library wrappers num_format to format numbers according to the system locale

use lazy_static::lazy_static;
use num_format::{Locale, SystemLocale, ToFormattedString};

lazy_static! {
    static ref SYSTEM_LOCALE: Option<SystemLocale> = SystemLocale::default().ok();
}

/// Trait applied to numeric types to add the num_format() function
pub trait NumFormat: Sized {
    #[doc(hidden)]
    fn num_format(&self) -> String;
}

macro_rules! gen_impl {
    ($type:ty) => {
        impl NumFormat for $type {
            fn num_format(&self) -> String {
                match &*SYSTEM_LOCALE {
                    Some(locale) => self.to_formatted_string(locale),
                    None => self.to_formatted_string(&Locale::en),
                }
            }
        }
    };
}

gen_impl!(usize);
gen_impl!(u32);
gen_impl!(u16);
gen_impl!(u8);
