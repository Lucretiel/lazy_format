/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

// Copyright 2019 Nathan West

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

/// Low level constructor for lazy format instances. Create a lazy formatter
/// with a custom closure as its
/// [`Display`](https://doc.rust-lang.org/std/fmt/trait.Display.html)
/// implementation, for complete control over formatting behavior at write time.
///
/// [`make_lazy_format!`] is the low-level constructor for lazy format instances.
/// It is completely customizable, insofar as it allows you to create a custom
/// [`Display::fmt`](https://doc.rust-lang.org/core/fmt/trait.Display.html#tymethod.fmt)
/// implementation at the call site.
///
/// [`make_lazy_format!`] takes a closure specification as an argument, and creates
/// a [`Display`](https://doc.rust-lang.org/std/fmt/trait.Display.html) struct
/// that captures the local environment in a closure and uses it as the
/// formatting function.
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

/// Lazily format something. Essentially the same as
/// [`format!`](https://doc.rust-lang.org/std/macro.format.html), except that
/// instead of formatting its arguments to a string, it captures them in an opaque
/// struct, which can be formatted later. This allows you to build up formatting
/// operations without any intermediary allocations or extra formatting calls. Also
/// supports lazy conditional and looping constructs.
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
/// # Basic example:
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
/// # Conditional formatting
///
/// `lazy_format!` supports conditional formatting with `match`- or `if`-
/// style syntax. When doing a conditional format, add the formatting pattern
/// and arguments directly into the `match` arms or `if` blocks, rather than
/// code; this allows conditional formatting to still be captured in a single
/// static type.
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
///         | 3 => ("Three"),
///         4 | 5 => ("Four or five"),
///         value if value % 2 == 0 => ("A large even number: {}", value),
///         value => ("An unrecognized number: {v}", v = value),
///     })
/// }
///
/// assert_eq!(get_number(0).to_string(), "Zero");
/// assert_eq!(get_number(1).to_string(), "One");
/// assert_eq!(get_number(2).to_string(), "Two");
/// assert_eq!(get_number(3).to_string(), "Three");
/// assert_eq!(get_number(4).to_string(), "Four or five");
/// assert_eq!(get_number(5).to_string(), "Four or five");
/// assert_eq!(get_number(6).to_string(), "A large even number: 6");
/// assert_eq!(get_number(7).to_string(), "An unrecognized number: 7");
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
///
/// `if`-style lazy formatting also support `if let` expressions:
///
/// ```
/// use std::fmt::Display;
/// use lazy_format::lazy_format;
///
/// fn describe_optional_number(value: Option<isize>) -> impl Display {
///     lazy_format!(
///         if let Some(10) = value => ("It's ten!")
///         else if let Some(3) | Some(4) = value => ("It's three or four!")
///         else if let | Some(0) = value => ("It's zero!")
///         else if let Some(x) = value => ("It's some other value: {}", x)
///         else => ("It's not a number!")
///     )
/// }
///
/// assert_eq!(describe_optional_number(Some(10)).to_string(), "It's ten!");
/// assert_eq!(describe_optional_number(Some(3)).to_string(), "It's three or four!");
/// assert_eq!(describe_optional_number(Some(4)).to_string(), "It's three or four!");
/// assert_eq!(describe_optional_number(Some(0)).to_string(), "It's zero!");
/// assert_eq!(describe_optional_number(Some(5)).to_string(), "It's some other value: 5");
/// assert_eq!(describe_optional_number(None).to_string(), "It's not a number!");
/// ```
///
/// # Looping formatting
///
/// `lazy_format!` supports formatting elements in a collection with a loop.
/// There are a few supported syntaxes:
///
/// ```
/// use std::fmt::Display;
/// use lazy_format::lazy_format;
///
/// let list = vec![1i32, 2, 3, 4];
/// let list_ref = &list;
///
/// // Format each element in the iterable without additional arguments to `format_args`
/// let simple_semicolons = lazy_format!("{v}; " for v in list_ref.iter().map(|x| x - 1));
/// assert_eq!(simple_semicolons.to_string(), "0; 1; 2; 3; ");
///
/// // Perform a full format with additional arguments on each element in the iterable.
/// let header = "Value";
/// let full_format = lazy_format!(("{}: {}; ", header, v) for v in list_ref);
/// assert_eq!(full_format.to_string(), "Value: 1; Value: 2; Value: 3; Value: 4; ");
/// ```
///
/// Note that these looping formatters are not suitable for doing something like
/// a comma separated list, since they'll apply the formatting to all elements.
/// For a lazy string joining library, which only inserts separators between
/// elements in a list, check out [joinery](/joinery).
#[macro_export]
macro_rules! lazy_format {
    // Trivial formatter: just write the pattern
    ($pattern:literal) => {
        $crate::lazy_format!($pattern,)
    };

    // Basic lazy format: collect $args and format via `$pattern` when writing
    // to a destination
    ($pattern:literal, $($args:tt)*) => {
        $crate::make_lazy_format!(f => write!(f, $pattern, $($args)*))
    };

    // Conditional lazy format: evaluate a match expression and format based on
    // the matching arm
    (match ($condition:expr) {
        $(
            $(|)? $match_pattern:pat
            $(| $trailing_pattern:pat)*
            $(if $guard:expr)?
            => ($pattern:literal $($args:tt)*)
        ),* $(,)?
    }) => {
        $crate::make_lazy_format!(f => match ($condition) {
            $(
                $match_pattern
                $(| $trailing_pattern)*
                $(if $guard)?
                => write!(f, $pattern $($args)*),
            )*
        })
    };

    // Conditional pattern lazy format: evaluate

    // Conditional lazy format: evaluate an if / else if / else expression and
    // format based on the successful branch
    (
        if $(let $(|)? $match:pat $(| $trailing_match:pat)* = )? $condition:expr
            => ($pattern:literal $($args:tt)*)
        $(else if $(let $(|)? $elseif_match:pat $(| $elseif_trailing_match:pat)* = )? $elseif_condition:expr
            => ($elseif_pattern:literal $($elseif_args:tt)*))*
        else $(=>)? ($else_pattern:literal $($else_args:tt)*)
    ) => {
        $crate::make_lazy_format!(f => if $(let $match $(| $trailing_match)* = )? $condition {
            write!(f, $pattern $($args)*)
        } $(else if $(let $elseif_match $(| $elseif_trailing_match)* = )? $elseif_condition {
            write!(f, $elseif_pattern $($elseif_args)*)
        })* else {
            write!(f, $else_pattern $($else_args)*)
        })
    };

    // Looping formatter: format each `$item` in `$collection` with `$pattern`
    ($pattern:literal for $item:ident in $collection:expr) => {
        $crate::lazy_format!(($pattern, $item = $item) for $item in $collection)
    };

    // Looping formatter: format each `$item` in `$collection` with the format
    // arguments
    (($pattern:literal $($args:tt)*) for $item:ident in $collection:expr) => {
        $crate::make_lazy_format!(f =>
            ::core::iter::IntoIterator::into_iter($collection)
                .try_for_each(move |$item| write!(f, $pattern $($args)*))
        )
    };
}

/// Lazily format something by eagerly evaluate the arguments ahead of time,
/// then storing them and formatting them at write time.
///
/// This macro is essentially the same as
/// [`format!`](https://doc.rust-lang.org/std/macro.format.html), except that
/// instead of formatting its arguments to a string, it evaluates them and
/// captures them in an opaque struct, which can be formatted later. This allows
/// you to build up formatting operations without any extra allocations or
/// formatting calls.
///
/// The return value of this macro is left deliberately unspecified and
/// undocumented. The most important thing about it is its
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
/// and therefore have lifetime issues. However, this also limits
/// [`semi_lazy_format!`] to only supporting plain formatting, rather than the
/// conditional and looping structures supported by [`lazy_format!`].
///
///
/// ```
/// use std::fmt::Display;
/// use std::mem::{size_of_val, size_of};
/// use lazy_format::semi_lazy_format;
///
/// fn get_formatted() -> impl Display {
///     let x: isize = 512;
///     let y: isize = 512;
///     semi_lazy_format!("value: {v}, again: {v}", v = (x + y))
/// }
///
/// let result = get_formatted();
///
/// // At this point, addition expression has been performed. `result`
/// // captures only the result of the expression, rather than the two
/// // operands.
/// assert_eq!(size_of_val(&result), size_of::<isize>());
///
/// let result_str = result.to_string();
/// assert_eq!(result_str, "value: 1024, again: 1024");
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
    // Important: the `name = value` variants of this macro definition must
    // come first, because "name = value" is a valid rust expr
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

    // The trick here is to use a hygenic identifier. The `value` name used in
    // this step of the evaluation is considered distinct from the other ones.
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
/// in /tests) allows us to get better compiler diagnostics. Note that we can't
/// actually test the evaluated result of these expressions, because we don't
/// have a viable `write!` target (since we're in no_std so there are no
/// strings)
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
