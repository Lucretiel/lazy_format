#![no_std]

//! lazy_format is a collection of
//! [`format!`](https://doc.rust-lang.org/std/macro.format.html)-style macros which lazily
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
//! This means that the expressions are evaluated *every* time the instance is
//! written somewhere.
//! - [`semi_lazy_format!`] is partially lazy. It fully evaluates all of its
//! arguments when it is invoked, but it stores them inside the returned instance
//! and formats them when the instance is written.

/// Low level constructor for lazy format instances
///
/// [`make_lazy_format!`] is the low-level constructor for lazy format instances.
/// It is completely customizable, insofar as it allows you to create a custom
/// [`Display::fmt`] implementation at the call site.
///
/// [`make_lazy_format!`] takes a closure specification as an argument, and creates
/// a [`Display`] struct that captures the local environment in a closure and uses
/// it as the formatting function.
///
/// # Example:
///
/// ```
/// use std::fmt::Display;
/// use lazy_format::make_lazy_format;
///
/// let data = vec![1, 2, 3, 4, 5];
///
/// let comma_separated = make_lazy_format!(f => {
///     let mut iter = data.iter();
///     match iter.next() {
///         None => Ok(()),
///         Some(first) => {
///             write!(f, "{}", first)?;
///             iter.try_for_each(|value| write!(f, ", {}", value))
///         }
///     }
/// });
///
/// let result = comma_separated.to_string();
///
/// assert_eq!(result, "1, 2, 3, 4, 5");
/// ```
#[macro_export]
macro_rules! make_lazy_format {
    ($fmt:ident => $write:expr) => {{
        #[derive(Clone, Copy)]
        struct LazyFormat<F: Fn(&mut ::core::fmt::Formatter) -> ::core::fmt::Result>(F);

        // TODO: customize Debug impl for semi_lazy_format to include value
        impl<F: Fn(&mut ::core::fmt::Formatter) -> ::core::fmt::Result> ::core::fmt::Debug for LazyFormat<F> {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                f.write_str(concat!("make_lazy_format!(", stringify!( $fmt => $write ), ")"))
            }
        }

        impl<F: Fn(&mut ::core::fmt::Formatter) -> ::core::fmt::Result> ::core::fmt::Display for LazyFormat<F> {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                (self.0)(f)
            }
        }

        LazyFormat(#[inline] move |$fmt: &mut ::core::fmt::Formatter| -> ::core::fmt::Result {
            $write
        })
    }}
}

/// Lazily format something.
///
/// This macro is essentially the same as
/// [`format!`](https://doc.rust-lang.org/std/macro.format.html), except that
/// instead of formatting its arguments to a string, it captures them in an opaque
/// struct, which can be formatted later. This allows you to build up formatting
/// operations without any intermediary allocations or extra formatting calls.
///
/// The return value of this macro is left deliberately unspecified and
/// undocumented. The most important this about it is its
/// [`Display`](https://doc.rust-lang.org/std/fmt/trait.Display.html)
/// implementation, which executes the deferred formatting operation. It
/// also provides a [`Debug`](https://doc.rust-lang.org/std/fmt/trait.Debug.html)
/// implementation, which simply prints the [`lazy_format!`]`(...)` call without
/// evaluating any of its arguments, as well as [`Clone`] and [`Copy`] if those
/// traits are available in the captured context.
///
/// Note that this macro is completely lazy; it captures the expressions to
/// be formatted in the struct and doesn't evaluate them until the struct is
/// actually written to a
/// [`String`](https://doc.rust-lang.org/std/string/struct.String.html) or
/// [`File`](https://doc.rust-lang.org/std/fs/struct.File.html) or or other
/// writable destination. This means that the argument expression will be
/// evaluated *every* time the instance is written, which may not be what you
/// want. See [`semi_lazy_format!`] for a macro which eagerly evaluates its
/// arguments but lazily does the final formatting.
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
///
/// `lazy_format!` supports conditional formatting with `match` or `if` like syntax.
/// When doing a conditional format, add the formatting pattern and arguments
/// directly into the `match` arms or `if` blocks, rather than code.
///
/// # `match` conditional example:
///
/// ```
/// use std::fmt::Display;
/// use lazy_format::lazy_format;
///
/// fn get_number(num: usize) -> impl Display {
///     // Note that the parenthesis in the match conditional are required,
///     // due to limitations in Rust's macro parsing (can't follow an
///     // expression with `{}`)
///     lazy_format!(match (num) {
///         0 => ("Zero"),
///         1 => ("One"),
///         2 => ("Two"),
///         3 => ("Three"),
///         value if value % 2 == 0 => ("A large even number: {}", value),
///         value => ("An unrecognized number: {v}", v = value),
///     })
/// }
///
/// assert_eq!(get_number(0).to_string(), "Zero");
/// assert_eq!(get_number(1).to_string(), "One");
/// assert_eq!(get_number(2).to_string(), "Two");
/// assert_eq!(get_number(3).to_string(), "Three");
/// assert_eq!(get_number(4).to_string(), "A large even number: 4");
/// assert_eq!(get_number(5).to_string(), "An unrecognized number: 5");
/// ```
///
/// # `if` conditional example:
///
/// ```
/// use std::fmt::Display;
/// use lazy_format::lazy_format;
///
/// fn describe_number(value: isize) -> impl Display {
///     lazy_format!(
///         if value < 0 => ("A negative number: {}", value)
///         else if value % 3 == 0 => ("A number divisible by 3: {}", value)
///         else if value % 2 == 1 => ("An odd number: {}", value)
///         else ("Some other kind of number")
///     )
/// }
///
/// assert_eq!(describe_number(-2).to_string(), "A negative number: -2");
/// assert_eq!(describe_number(-1).to_string(), "A negative number: -1");
/// assert_eq!(describe_number(0).to_string(), "A number divisible by 3: 0");
/// assert_eq!(describe_number(1).to_string(), "An odd number: 1");
/// assert_eq!(describe_number(2).to_string(), "Some other kind of number");
/// assert_eq!(describe_number(3).to_string(), "A number divisible by 3: 3");
/// ```
#[macro_export]
macro_rules! lazy_format {
    ($pattern:literal $($args:tt)*) => {
        $crate::make_lazy_format!(f => write!(f, $pattern $($args)*))
    };

    (match ($condition:expr) {
        $($match_pattern:pat $(if $guard:expr)? => ($pattern:literal $($args:tt)*)),* $(,)?
    }) => {
        $crate::make_lazy_format!(f => match ($condition) {
            $($match_pattern $(if $guard)? => write!(f, $pattern $($args)*),)*
        })
    };

    (
        if $condition:expr => ($pattern:literal $($args:tt)*)
        $(else if $elseif_condition:expr => ($elseif_pattern:literal $($elseif_args:tt)*))*
        else $(=>)? ($else_pattern:literal $($else_args:tt)*)
    ) => {
        $crate::make_lazy_format!(f => if ($condition) {
            write!(f, $pattern $($args)*)
        } $(else if ($elseif_condition) {
            write!(f, $elseif_pattern $($elseif_args)*)
        })* else {
            write!(f, $else_pattern $($else_args)*)
        })
    };
    
    (for $element:ident in $collection:expr: ($pattern:literal $($args:tt)*)) => {
        $crate::make_lazy_format!(f => ($collection).try_for_each(move |$element| write!(f, $pattern $($args)*)))
    }
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
/// undocumented. The most important this about it is its
/// [`Display`](https://doc.rust-lang.org/std/fmt/trait.Display.html)
/// implementation, which executes the deferred formatting operation. It
/// also provides [`Clone`] and [`Copy`] if those traits are implementated in all
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
    // Important: the name = value variants must come first, because "name = value"
    // is a valid rust expr
    ($({ $evaluated_name:ident })* $pattern:literal $(, $name:ident = $value:expr)* $(,)?) => {{
        $(let $name = $value;)*

        $crate::lazy_format!(
            $pattern,
            $($evaluated_name,)*
            $($name = $name,)*
        )
    }};

    ($({ $evaluated_value:ident })* $pattern:literal, $value:expr) => {
        $crate::semi_lazy_format_impl!($({ $evaluated_value })* $pattern, $value, )
    };

    ($({ $evaluated_value:ident })* $pattern:literal, $value:expr, $($tail:tt)*) => {{
        let value = $value;
        $crate::semi_lazy_format_impl!(
            $({ $evaluated_value })*
            { value }
            $pattern,
            $($tail)*
        )
    }};
}

pub mod prelude {
    pub use crate::{lazy_format, make_lazy_format, semi_lazy_format};
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
        let _result = semi_lazy_format!("{text}!", text = "Hello, World");
    }

    #[test]
    fn test_two_named_args() {
        let _result = semi_lazy_format!("{h}, {w}!", h = "Hello", w = "World");
    }

    #[test]
    fn test_mixed_args() {
        let _result = semi_lazy_format!("{}, {w}!", "Hello", w = "World");
    }

    #[test]
    fn test_many_args() {
        let _result = semi_lazy_format!(
            "{} {} {} {a} {b} {} {b} {a} {}",
            1,
            2,
            3,
            4,
            5,
            a = 10,
            b = 20
        );
    }
}
