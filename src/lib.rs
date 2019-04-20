#![no_std]

//! lazy_format is a collection of [`format!`]-style macros which lazily
//! formatting their arguments its arguments. That is, rather than immediatly
//! formatting them into a [`String`](https://doc.rust-lang.org/std/string/struct.String.html)
//! (which is what [`format!`](https://doc.rust-lang.org/std/macro.format.html))
//! does, it captures its arguments and returns an opaque struct with a
//! [`Display`](https://doc.rust-lang.org/std/fmt/trait.Display.html)
//! implementation, so that the actual formatting can happen directly into its
//! final destination buffer (such as a file or string).
//!
//! ```
//! use std::fmt::Display;
//!
//! use lazy_format::prelude::*;
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
//!
//! This library contains two lazy formatting macros:
//!
//! - [`lazy_format!`], a completely lazy formatter. It captures the expressions
//! passed as arguments in a closure, and doesn't evaluate them until the
//! instance is *actually* written to a destination (like a string or file).
//! This means that
//! - [`semi_lazy_format`] is partially lazy. It fully evaluates all of its
//! arguments when it is invoked, but it stores them

/// Lazily format something
///
/// This macro is essentially the same as
/// [`format!`](https://doc.rust-lang.org/std/macro.format.html), except that
/// instead of formatting its arguments to a string, it captures them in an opaque
/// struct, which can be formatted later. This allows you to build up formatting
/// operations without any intermediary allocations or extra formatting calls.
///
/// The return value of this macro is left deliberately unspecified and
/// undocumented. The most important this about it is its `Display`
/// implementation, which executes the deferred formatting operation. It
/// also provides a `Debug` implementation, which simply prints the
/// `lazy_format!(...)` call without evaluating any of its arguments, as well
/// as `Clone` and `Copy` if those traits are available in the captured values.
///
/// Note that this macro is completely lazy; it captures the expressions to
/// be formatted in the struct and doesn't evaluate them until the struct is
/// actually written to a `String` or `File` or or other writable destination.
/// This means that the argument expression will be evaluated *every* time the
/// instance is written, which may not be what you want. See [`semi_lazy_format`]
/// for a macro which eagerly evaluates its arguments but lazily does the final
/// formatting.
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
/// // formatted into a String.
/// let result_str = result.to_string();
/// assert_eq!(result_str, "Hello, World!");
/// ```
#[macro_export]
macro_rules! lazy_format {
    ($pattern:literal $($args:tt)*) => {{
        #[derive(Clone, Copy)]
        struct LazyFormat<F: Fn(&mut ::core::fmt::Formatter) -> ::core::fmt::Result>(F);

        // TODO: customize Debug impl for semi_lazy_format to include value
        impl<F: Fn(&mut ::core::fmt::Formatter) -> ::core::fmt::Result> ::core::fmt::Debug for LazyFormat<F> {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                f.write_str(concat!("lazy_format!(", stringify!( $pattern $($args)* ), ")"))
            }
        }

        impl<F: Fn(&mut ::core::fmt::Formatter) -> ::core::fmt::Result> ::core::fmt::Display for LazyFormat<F> {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                (self.0)(f)
            }
        }

        LazyFormat(#[inline] move |f: &mut ::core::fmt::Formatter| -> ::core::fmt::Result {
            write!(f, $pattern $($args)*)
        })
    }};
}

/// Lazily format something, but eagerly evaluate the arguments ahead of time.
///
/// This macro is essentially the same as
/// [`format!`](https://doc.rust-lang.org/std/macro.format.html), except that
/// instead of formatting its arguments to a string, it evaluates them and
/// captures them in an opaque struct, which can be formatted later. This allows
/// you to build up formatting operations without any extra allocations or
/// formatting calls.
///
/// The return value of this macro is left deliberately unspecified and
/// undocumented. The most important this about it is its `Display`
/// implementation, which executes the deferred formatting operation. It
/// also provides `Clone` and `Copy` if those traits are implementated in all
/// of the evaluated formatting arguments.
///
/// Unlike [`lazy_format`], this macro is partially lazy, in that it evaluates
/// its argument expressions and stores the values to be formatted later. This
/// is often more convenient, especially if the formatted values are simple
/// data types like integers and `&str`. This can also make using the value
/// easier, as its less likely to capture locally scoped variable or references
/// and therefore have lifetime issues.
///
///
/// ```
/// use std::fmt::Display;
/// use std::mem::{size_of_val, size_of};
/// use lazy_format::semi_lazy_format;
///
///
/// fn get_value() -> usize {
///     1024
/// }
///
/// fn get_formatted() -> impl Display {
///     semi_lazy_format!("value: {v}, again: {v}", v = get_value())
/// }
///
/// let result = get_formatted();
///
/// // At this point, get_value was called, and `result` stores just the
/// // usize return value. No allocations have been performed yet.
/// let result_str = result.to_string();
/// assert_eq!(result_str, "value: 1024, again: 1024");
/// assert_eq!(size_of_val(&result), size_of::<usize>());
/// ```
#[macro_export]
macro_rules! semi_lazy_format {
    // TODO: Debug implementation with values
    ($pattern:literal $($args:tt)*) => {
        $crate::semi_lazy_format_impl!($pattern $($args)*)
    };
}

/// Implementation macro semi_lazy_format
#[doc(hidden)]
#[macro_export]
macro_rules! semi_lazy_format_impl {
    ($({ $evaluated_value:ident $($fmt_name:ident)?})* $pattern:literal) => {
        $crate::lazy_format!($pattern $(, $($fmt_name =)? $evaluated_value)*)
    };

    ($({ $evaluated_value:ident $($fmt_name:ident)?})* $pattern:literal,) => {
        $crate::semi_lazy_format_impl!($({ $evaluated_value $($fmt_name)? })* $pattern)
    };

    // Important: the name = value variants must come first, because "name = value"
    // is a valid rust expr
    ($({ $evaluated_value:ident $($fmt_name:ident)?})* $pattern:literal, $name:ident = $value:expr) => {
        $crate::semi_lazy_format_impl!($({ $evaluated_value $($fmt_name)? })* $pattern, $name = $value,)
    };

    ($({ $evaluated_value:ident $($fmt_name:ident)?})* $pattern:literal, $name:ident = $value:expr, $($tail:tt)*) => {{
        let value = $value;
        $crate::semi_lazy_format_impl!(
            $({ $evaluated_value $($fmt_name)? })*
            { value $name }
        $pattern, $($tail)*)
    }};

    ($({ $evaluated_value:ident $($fmt_name:ident)?})* $pattern:literal, $value:expr) => {
        $crate::semi_lazy_format_impl!($({ $evaluated_value $($fmt_name)? })* $pattern, $value, )
    };

    ($({ $evaluated_value:ident $($fmt_name:ident)?})* $pattern:literal, $value:expr, $($tail:tt)*) => {{
        let value = $value;
        $crate::semi_lazy_format_impl!(
            $({ $evaluated_value $($fmt_name)? })*
            { value }
        $pattern, $($tail)*)
    }};
}

pub mod prelude {
    pub use crate::{lazy_format, semi_lazy_format};
}

/// The syntax of semi_lazy_format is fairly complicated; these tests are
/// provided to ensure there are no failures. Putting them here (rather than
/// in /tests) allows us to get better compiler diagnostics.
#[cfg(test)]
mod semi_lazy_format_syntax_tests {
    #[test]
    fn test_no_args() {
        let _result = semi_lazy_format!("Hello, World!");
    }

    #[test]
    fn test_trailing_comma() {
        let _result = semi_lazy_format!("Hello, World!",);
    }

    #[test]
    fn test_one_arg() {
        let _result = semi_lazy_format!("{}!", "Hello, World");
    }

    #[test]
    fn test_two_args() {
        let _result = semi_lazy_format!("{}, {}!", "Hello", "World");
    }

    #[test]
    fn test_one_named_arg() {
        let _result = semi_lazy_format!("{text}!", text="Hello, World");
    }

    #[test]
    fn test_two_named_args() {
        let _result = semi_lazy_format!("{h}, {w}!", h="Hello", w="World");
    }

    #[test]
    fn test_mixed_args() {
        let _result = semi_lazy_format!("{}, {w}!", "Hello", w="World");
    }

    #[test]
    fn test_many_args() {
        let _result = semi_lazy_format!("{} {} {} {a} {b} {} {b} {a} {}", 1, 2, 3, 4, 5, a=10, b=20);
    }
}
