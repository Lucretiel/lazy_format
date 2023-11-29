/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

// Copyright 2019-2022 Nathan West

#![no_std]

/*!
[`lazy_format!`] is a [`format!`]-style macro which lazily formats its
arguments. That is, rather than immediatly formatting them into a [`String`]
(which is what [`format!`] does), it captures its arguments and returns an
opaque struct with a [`Display`] implementation, so that the actual formatting
can happen directly into its final destination buffer (such as a file or
string).

```
use std::fmt::Display;

use lazy_format::prelude::*;

// NOTE: This is profoundly insecure and you should never actually
// render HTML without escape guards, code injection prevention, etc.
fn html_tag<'a>(tag: &'a str, content: impl Display + 'a) -> impl Display + 'a {
    lazy_format!("<{tag}>{content}</{tag}>")
}

let result = html_tag("div", html_tag("p", "Hello, World!")).to_string();
assert_eq!(result, "<div><p>Hello, World!</p></div>");
```

[`format!`]: https://doc.rust-lang.org/std/macro.format.html
[`Display`]: https://doc.rust-lang.org/std/fmt/trait.Display.html
[`String`]: https://doc.rust-lang.org/std/string/struct.String.html
*/

/**
Smarter write macro. Encodes some common patterns, such as writing an empty
string being a no-op. Used in the more complex lazy-format operations, like
conditionals, where writing only strings or empty strings, is common.
*/
#[macro_export]
#[doc(hidden)]
macro_rules! write {
    ($dest:expr, "" $(,)? ) => { ::core::fmt::Result::Ok(()) };

    ($dest:expr, $pattern:literal $(,)? ) => {{
        enum Style {
            Empty,
            Plain,
            Format,
        }

        match {
            // We want this to be const so that we can guarantee it's computed
            // at compile time. Need to put the `STYLE` global in a nested
            // scope because otherwise it leaks out
            const STYLE: Style = match $pattern.as_bytes().split_first() {
                ::core::option::Option::None => Style::Empty,
                ::core::option::Option::Some((&(b'}' | b'{'), _)) => Style::Format,
                ::core::option::Option::Some((_, mut s)) => loop {
                    s = match s.split_first() {
                        None => break Style::Plain,
                        Some((&(b'}' | b'{'), _)) => break Style::Format,
                        Some((_, s)) => s,
                    };
                }
            };

            STYLE
        } {
            Style::Empty => ::core::fmt::Result::Ok(()),
            Style::Plain => ::core::fmt::Write::write_str($dest, $pattern),
            Style::Format => ::core::fmt::Write::write_fmt($dest, ::core::format_args!($pattern)),
        }
    }};

    ($dest:expr, $pattern:literal, $($args:tt)+ ) => {
        ::core::fmt::Write::write_fmt($dest, ::core::format_args!($pattern, $($args)+))
    };
}

/**
Helper macro for common formatting shortcuts. In a few places in lazy_format,
it's permitted to write either `lazy_format!(if cond => "foo")` or
`lazy_format!(if cond => ("value: {}", value))`. This macro takes care of
handling both cases.
*/
#[macro_export]
#[doc(hidden)]
macro_rules! write_tt {
    ($dest:expr, $pattern:literal) => { $crate::write!($dest, $pattern) };
    ($dest:expr, ($pattern:literal $($args:tt)*)) => { $crate::write!($dest, $pattern $($args)*) };
}

/// Test that an empty format string succeeds unconditionally.
#[test]
fn test_write_tt_empty_pattern() {
    use core::fmt;

    struct BadDest;

    impl fmt::Write for BadDest {
        fn write_str(&mut self, _s: &str) -> fmt::Result {
            Err(fmt::Error)
        }
    }

    let x = 10;

    write_tt!(&mut BadDest, "").unwrap();
    write_tt!(&mut BadDest, ("")).unwrap();
    write_tt!(&mut BadDest, ("",)).unwrap();

    write_tt!(&mut BadDest, "Plain String").unwrap_err();
    write_tt!(&mut BadDest, "Formatted String: {x}").unwrap_err();
}

#[test]
fn test_write_string_pattern() {
    use core::fmt;

    struct WeirdDest;

    impl fmt::Write for WeirdDest {
        fn write_str(&mut self, _s: &str) -> fmt::Result {
            Ok(())
        }

        fn write_fmt(&mut self, _args: fmt::Arguments<'_>) -> fmt::Result {
            Err(fmt::Error)
        }
    }

    let x = 10;

    write_tt!(&mut WeirdDest, "Plain String").unwrap();
    write_tt!(&mut WeirdDest, "Formatted String: {x}").unwrap_err();
}

/**
Low level constructor for lazy format instances. Create a lazy formatter with a
custom closure as its [`Display`] implementation, for complete control over
formatting behavior at write time.

[`make_lazy_format!`] is the low-level constructor for lazy format instances. It
is completely customizable, insofar as it allows you to create a custom
[`Display::fmt`] implementation at the call site.

[`make_lazy_format!`] takes a closure as an argument, and creates a [`Display`]
struct that captures the local environment in a closure and uses it as the
formatting function.

# Example:

```
use std::fmt::Display;
use lazy_format::make_lazy_format;

let data = vec![1, 2, 3, 4, 5];

let comma_separated = make_lazy_format!(|f| {
    let mut iter = data.iter();
    match iter.next() {
        None => Ok(()),
        Some(first) => {
            write!(f, "{}", first)?;
            iter.try_for_each(|value| write!(f, ", {}", value))
        }
    }
});

let result = comma_separated.to_string();

assert_eq!(result, "1, 2, 3, 4, 5");
```

[`Display`]: https://doc.rust-lang.org/std/fmt/trait.Display.html
[`Display::fmt`]: https://doc.rust-lang.org/core/fmt/trait.Display.html#tymethod.fmt
*/
#[macro_export]
macro_rules! make_lazy_format {
    (|$fmt:ident| $write:expr) => {{
        #[derive(Clone, Copy)]
        struct LazyFormat<F: Fn(&mut ::core::fmt::Formatter) -> ::core::fmt::Result>(F);

        // TODO: customize Debug impl for semi_lazy_format to include value
        impl<F: Fn(&mut ::core::fmt::Formatter) -> ::core::fmt::Result> ::core::fmt::Debug
            for LazyFormat<F>
        {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                f.write_str(concat!(
                    "make_lazy_format!(",
                    stringify!(|$fmt| $write),
                    ")"
                ))
            }
        }

        impl<F: Fn(&mut ::core::fmt::Formatter) -> ::core::fmt::Result> ::core::fmt::Display
            for LazyFormat<F>
        {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                (self.0)(f)
            }
        }

        LazyFormat(move |$fmt: &mut ::core::fmt::Formatter| -> ::core::fmt::Result { $write })
    }};
}

/**
Lazily format something. Essentially the same as [`format!`], except that
instead of formatting its arguments to a string, it captures them in an opaque
struct, which can be formatted later. This allows you to build up formatting
operations without any intermediary allocations or extra formatting calls. Also
supports lazy conditional and looping constructs.

The return value of this macro is left deliberately unspecified and
undocumented. The most important this about it is its [`Display`]
implementation, which executes the deferred formatting operation. It also
provides a [`Debug`] implementation, which simply prints the
[`lazy_format!`]`(...)` call without evaluating any of its arguments, as well
as [`Clone`] and [`Copy`] if those traits are available in the captured
context.

Note that this macro is completely lazy; it captures the expressions to
be formatted in the struct and doesn't evaluate them until the struct is
actually written to a [`String`] or [`File`] or or other writable destination.
This means that the argument expression will be evaluated *every* time the
instance is written, which may not be what you want; be sure to eagerly perform
any 1-time calculations you want to before calling `lazy_format!`.

# Basic example:

```
use std::fmt::Display;
use lazy_format::lazy_format;

fn get_hello() -> String {
    String::from("Hello")
}

fn get_world() -> String {
    String::from("World")
}

fn hello_world() -> impl Display {
    lazy_format!("{}, {w}!", get_hello(), w = get_world())
}

let result = hello_world();

// get_hello and get_world aren't called until the object is
// formatted into a String.
let result_str = result.to_string();
assert_eq!(result_str, "Hello, World!");
```

Just like with regular formatting, `lazy_format` can automatically, implicitly
capture named parameters:

```
use std::mem::{size_of_val, size_of};
use lazy_format::lazy_format;

let a = 10;
let b = 20;

let result = lazy_format!("{a} {b}");
assert_eq!(size_of_val(&result), size_of::<i32>() * 2);
assert_eq!(result.to_string(), "10 20");
```

# Demonstation of lazy capturing:

```
use std::fmt::Display;
use std::mem::{size_of_val, size_of};
use lazy_format::lazy_format;


fn get_formatted() -> impl Display {
    let a: isize = 10;
    let b: isize = 15;

    lazy_format!("10 + 15: {v}, again: {v}", v = (a + b))
}

let result = get_formatted();

// The result captures 2 isize values (a and b) from get_formatted.
assert_eq!(size_of_val(&result), size_of::<isize>() * 2);

let result_str = result.to_string();
assert_eq!(result_str, "10 + 15: 25, again: 25");
```

# Conditional formatting

`lazy_format!` supports conditional formatting with `match`- or `if`-
style syntax. When doing a conditional format, add the formatting pattern
and arguments directly into the `match` arms or `if` blocks, rather than
code; this allows conditional formatting to still be captured in a single
static type.

## `match` conditional example:

```
use std::fmt::Display;
use lazy_format::lazy_format;

fn get_number(num: usize) -> impl Display {
    // Note that the parenthesis in the match conditional are required,
    // due to limitations in Rust's macro parsing (can't follow an
    // expression with `{}`)
    lazy_format!(match (num) {
        0 => "Zero",
        1 => "One",
        2 => "Two",
        3 => "Three",
        4 | 5 => "Four or five",
        value if value % 2 == 0 => ("A large even number: {}", value),
        value => "An unrecognized number: {value}",
    })
}

assert_eq!(get_number(0).to_string(), "Zero");
assert_eq!(get_number(1).to_string(), "One");
assert_eq!(get_number(2).to_string(), "Two");
assert_eq!(get_number(3).to_string(), "Three");
assert_eq!(get_number(4).to_string(), "Four or five");
assert_eq!(get_number(5).to_string(), "Four or five");
assert_eq!(get_number(6).to_string(), "A large even number: 6");
assert_eq!(get_number(7).to_string(), "An unrecognized number: 7");
```

## `if` conditional example:

```
use std::fmt::Display;
use lazy_format::lazy_format;

fn describe_number(value: isize) -> impl Display {
    lazy_format!(
        if value < 0 => ("A negative number: {}", value)
        else if value % 3 == 0 => ("A number divisible by 3: {}", value)
        else if value % 2 == 1 => ("An odd number: {}", value)
        else => "Some other kind of number"
    )
}

assert_eq!(describe_number(-2).to_string(), "A negative number: -2");
assert_eq!(describe_number(-1).to_string(), "A negative number: -1");
assert_eq!(describe_number(0).to_string(), "A number divisible by 3: 0");
assert_eq!(describe_number(1).to_string(), "An odd number: 1");
assert_eq!(describe_number(2).to_string(), "Some other kind of number");
assert_eq!(describe_number(3).to_string(), "A number divisible by 3: 3");
```

`if` formatters are allowed to exclude the final `else` branch, in which
case `lazy_format` will simply write an empty string:

```
use std::fmt::Display;
use lazy_format::lazy_format;

fn only_evens(value: i32) -> impl Display {
    lazy_format!(if value % 2 == 0 => ("An even number: {}", value))
}

assert_eq!(only_evens(10).to_string(), "An even number: 10");
assert_eq!(only_evens(5).to_string(), "");
```

## `if let` conditional example:

```
use std::fmt::Display;
use lazy_format::lazy_format;

fn describe_optional_number(value: Option<isize>) -> impl Display {
    lazy_format!(
        if let Some(10) = value => "It's ten!"
        else if let Some(3) | Some(4) = value => "It's three or four!"
        else if let | Some(0) = value => "It's zero!"
        else if let Some(x) = value => ("It's some other value: {}", x)
        else => "It's not a number!"
    )
}

assert_eq!(describe_optional_number(Some(10)).to_string(), "It's ten!");
assert_eq!(describe_optional_number(Some(3)).to_string(), "It's three or four!");
assert_eq!(describe_optional_number(Some(4)).to_string(), "It's three or four!");
assert_eq!(describe_optional_number(Some(0)).to_string(), "It's zero!");
assert_eq!(describe_optional_number(Some(5)).to_string(), "It's some other value: 5");
assert_eq!(describe_optional_number(None).to_string(), "It's not a number!");
```

# Looping formatting

`lazy_format!` supports formatting elements in a collection with a loop:

```
use std::fmt::Display;
use lazy_format::lazy_format;

let list = vec![1i32, 2, 3, 4];
let list_ref = &list;

// Format each element in the iterable without additional arguments to `format_args`
let simple_semicolons = lazy_format!("{v}; " for v in list_ref.iter().map(|x| x - 1));
assert_eq!(simple_semicolons.to_string(), "0; 1; 2; 3; ");

// Perform a full format with additional arguments on each element in the iterable.
let header = "Value";
let full_format = lazy_format!(("{}: {}; ", header, v) for v in list_ref);
assert_eq!(full_format.to_string(), "Value: 1; Value: 2; Value: 3; Value: 4; ");
```

Note that these looping formatters are not suitable for doing something like
a comma separated list, since they'll apply the formatting to all elements.
For a lazy string joining library, which only inserts separators between
elements in a list, check out [joinery](/joinery).

[`format!`]: https://doc.rust-lang.org/std/macro.format.html
[`Display`]: https://doc.rust-lang.org/std/fmt/trait.Display.html
[`Debug`]: https://doc.rust-lang.org/std/fmt/trait.Debug.html
[`String`]: https://doc.rust-lang.org/std/string/struct.String.html
[`File`]: https://doc.rust-lang.org/std/fs/struct.File.html
*/
#[macro_export]
macro_rules! lazy_format {
    // Basic lazy format: collect $args and format via `$pattern` when writing
    // to a destination
    ($pattern:literal $(, $($args:tt)*)?) => {
        $crate::make_lazy_format!(|f| $crate::write!(f, $pattern $(, $($args)*)?))
    };

    // Conditional lazy format: evaluate a match expression and format based on
    // the matching arm
    (match ($condition:expr) {
        $($(
            $match_pattern:pat
            $(if $guard:expr)?
            => $output:tt
        ),+ $(,)?)?
    }) => {
        $crate::make_lazy_format!(|f| match $condition {
            $($(
                $match_pattern
                $(if $guard)?
                => $crate::write_tt!(f, $output),
            )+)?
        })
    };

    // Conditional pattern lazy format: evaluate

    // Conditional lazy format: evaluate an if / else if / else expression and
    // format based on the successful branch
    (
        if $(let $match:pat = )? $condition:expr => $output:tt
        $(else if $(let $elseif_match:pat = )? $elseif_condition:expr => $elseif_output:tt)*
        $(else => $else_output:tt)?
    ) => {
        $crate::make_lazy_format!(|f|
            if $(let $match = )? $condition {
                $crate::write_tt!(f, $output)
            }
            $(else if $(let $elseif_match = )? $elseif_condition {
                $crate::write_tt!(f, $elseif_output)
            })*
            $(else if true {
                $crate::write_tt!(f, $else_output)
            })?
            else {
                ::core::fmt::Result::Ok(())
            }
        )
    };

    // Looping formatter: format each `$item` in `$collection` with the format
    // arguments
    ($output:tt for $item:pat in $collection:expr) => {
        $crate::make_lazy_format!(|f| {
            let mut iter = ::core::iter::IntoIterator::into_iter($collection);
            ::core::iter::Iterator::try_for_each(&mut iter, |$item| $crate::write_tt!(f, $output))
        })
    };
}

pub mod prelude {
    pub use crate::{lazy_format, make_lazy_format};
}
