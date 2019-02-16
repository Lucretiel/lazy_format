#![no_std]

//! [`lazy_format!`] is a macro which lazily formats its arguments. That is, rather
//! than immediatly formatting them into a
//! [`String`](https://doc.rust-lang.org/std/string/struct.String.html)
//! (which is what [`format!`](https://doc.rust-lang.org/std/macro.format.html))
//! does, it captures its arguments and returns an opaque struct with a
//! [`Display`](https://doc.rust-lang.org/std/fmt/trait.Display.html)
//! implementation, so that the actual formatting can happen directly into its
//! final destination buffer (such as a file or string).
//!
//! ```
//! use std::fmt::Display;
//!
//! use lazy_format::lazy_format;
//!
//! // NOTE: This is obviously profoundly unsafe and you should never actually
//! // render HTML without escape guards, code injection prevention, etc.
//! fn html_tag<'a>(tag: &'a str, content: impl Display + 'a) -> impl Display + 'a {
//!     lazy_format!("<{tag}>{content}</{tag}>", tag=tag, content=content)
//! }
//!
//! let result = html_tag("div", html_tag("p", "Hello, World!")).to_string();
//! assert_eq!(result, "<div><p>Hello, World!</p></div>");
//! ```

use core::fmt::{self, Debug, Display, Formatter};

/// Lazily format something
///
/// This macro is essentially the same as
/// [`format!`](https://doc.rust-lang.org/std/macro.format.html), except that
/// instead of formatting its arguments to a string, it captures them in an opaque
/// struct, which can be formatted later. This allows you to build up formatting
/// operations without any intermediary allocations or extra formatting calls.
/// See the [module-level documentation](/lazy_format) for details.
///
/// The return type of this macro is `impl Display`.
#[macro_export]
macro_rules! lazy_format {
    // TODO: test that this fails with non string literals
    ($pattern:literal) => {
        (::lazy_format::require_static_str($pattern))
    };
    ($pattern:literal, $($args:tt)*) => {
        (::lazy_format::make_lazy_format(#[inline] move |f: &mut ::core::fmt::Formatter| -> ::core::fmt::Result {
            write!(f, $pattern, $($args)*)
        }))
    };
}

// TODO: pub const fn
#[doc(hidden)]
#[inline(always)]
pub fn require_static_str(s: &'static str) -> impl Display + 'static {
    s
}

#[doc(hidden)]
#[inline(always)]
pub fn make_lazy_format<F: Fn(&mut Formatter) -> fmt::Result>(f: F) -> impl Display {
    LazyFormat(f)
}

#[derive(Clone, PartialEq, Eq)]
struct LazyFormat<F: Fn(&mut Formatter) -> fmt::Result>(F);

impl<F: Fn(&mut Formatter) -> fmt::Result> Debug for LazyFormat<F> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str("LazyFormat(<closure>)")
    }
}

impl<F: Fn(&mut Formatter) -> fmt::Result> Display for LazyFormat<F> {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        (self.0)(f)
    }
}

pub mod prelude {
    pub use crate::lazy_format;
}
