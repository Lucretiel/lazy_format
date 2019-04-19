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
//! let result = html_tag("div", html_tag("p", "Hello, World!"));
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
/// The return type of this macro is `impl Display + Debug`. When Rust's
/// `impl trait` feature is more powerful, this will also include traits that
/// are conditional on the caputred variables, like `Clone`.
///
/// # Example:
///
/// ```
/// use std::fmt::Display;
/// use lazy_format::lazy_format;
///
/// fn get_hello() -> String {
///     String::from("Hello")
/// }
///
/// fn get_world() -> String {
///     String::from("World")
/// }
///
/// fn hello_world() -> impl Display {
///     lazy_format!("{}, {w}!", get_hello(), w = get_world())
/// }
///
/// let result = hello_world();
///
/// // get_hello and get_world aren't called until the object is
/// // formatted into a String
/// let result_str = result;
/// assert_eq!(result_str, "Hello, World!");
/// ```
#[macro_export]
macro_rules! lazy_format {
    ($pattern:literal) => {
        $crate::make_lazy_format(#[inline] move |f: &mut ::core::fmt::Formatter| -> ::core::fmt::Result {
            write!(f, $pattern)
        }) };
    ($pattern:literal, $($args:tt)*) => {
        $crate::make_lazy_format(#[inline] move |f: &mut ::core::fmt::Formatter| -> ::core::fmt::Result {
            write!(f, $pattern, $($args)*)
        })
    };
}

#[macro_export]
macro_rules! semi_lazy_format {

    ($pattern:literal, $($args:tt)*) => {
        $crate::semi_lazy_format_impl!($pattern, $($args)*)
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! semi_lazy_format_impl {
    (
        $({ $evaluated_value:ident $($fmt_name:ident)?})*
    @ $pattern:literal) => {
        $crate::lazy_format!($pattern $(, $($fmt_name =)? $evaluated_value)*)
    };

    (
        $({ $evaluated_value:ident $($fmt_name:ident)?})*
    @ $pattern:literal,) => {
        $crate::lazy_format!($pattern $(, $($fmt_name =)? $evaluated_value)*)
    };

    (
        $({ $evaluated_value:ident $($fmt_name:ident)?})*
    @ $pattern:literal, $name:ident = $value:expr) => {{
        let value = $value;
        $crate::semi_lazy_format_impl!(
            $({ $evaluated_value $($fmt_name)? })*
            { value $name }
        @ $pattern)
    }};

    (
        $({ $evaluated_value:ident $($fmt_name:ident)?})*
    @ $pattern:literal, $value:expr) => {{
        let value = $value;
        $crate::semi_lazy_format_impl!(
            $({ $evaluated_value $($fmt_name)? })*
            { value }
        @ $pattern)
    }};

/*
    ({
        $({ $evaluated_value:ident $($fmt_name:ident)?})*
    } @ $pattern:literal, $name:ident = $value:expr, $($tail:tt)*) => {{
        let value = $value;
        $crate::semi_lazy_format_impl!({
            $({ $evaluated_value $($fmt_name)? })*
            { value $name }
        } @ $pattern, $($tail)*)
    }};

    ({
        $({ $evaluated_value:ident $($fmt_name:ident)?})*
    } @ $pattern:literal, $value:expr, $($tail:tt)*) => {{
        let value = $value;
        $crate::semi_lazy_format_impl!({
            $({ $evaluated_value $($fmt_name)? })*
            { value }
        } @ $pattern, $($tail)*)
    }};
    */
}

#[doc(hidden)]
#[inline(always)]
pub fn make_lazy_format<F: Fn(&mut Formatter) -> fmt::Result>(f: F) -> impl Display + Debug {
    LazyFormat(f)
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct LazyFormat<F: Fn(&mut Formatter) -> fmt::Result>(F);

impl<F: Fn(&mut Formatter) -> fmt::Result> Debug for LazyFormat<F> {
    #[inline]
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

#[cfg(test)]
mod semi_lazy_format_syntax_tests {
    use crate::semi_lazy_format;
    #[test]
    fn test_no_args() {
        let result = semi_lazy_format!("Hello, World!");
    }

    #[test]
    fn test_trailing_comma() {
        let result = semi_lazy_format!("Hello, World!",);
    }

    #[test]
    fn test_one_arg() {
        let result = semi_lazy_format!("{}!", "Hello, World");
    }

    #[test]
    fn test_one_named_arg() {
        let result = semi_lazy_format!("{text}!", text = "Hello, World");
    }

    /*
    #[test]
    fn test_two_args() {
        let result = semi_lazy_format!("{}, {}!", "Hello", "World");
    }

    #[test]
    fn test_two_named_args() {
        let result = semi_lazy_format!("{h}, {w}!", h="Hello", w="World");
    }

    #[test]
    fn test_mixed_args() {
        let result = semi_lazy_format!("{}, {w}!", "Hello", w="World");
    }
    */
}
