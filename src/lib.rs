#![cfg_attr(not(test), no_std)]

//! lazy_format is a macro which lazily formats its arguments. That is, rather
//! than immediatly formatting them into a `String` (which is what `format!`)
//! does, it captures its arguments and returns an opaque struct with a `Display`
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
/// This macro is essentially the same as `format!`, except that instead of
/// formatting its arguments to a string, it captures them in an opaque struct
/// ([`LazyFormat`]), which can be formatted later. This allows you to build
/// up formatting operations without any intermediary allocations or extra
/// formatting calls. See the module-level documentation for details.
#[macro_export]
macro_rules! lazy_format {
    ($pattern:literal) => {
        $crate::LazyFormat::new(#[inline] move |f: &mut ::core::fmt::Formatter| -> ::core::fmt::Result {
            // TODO: replace this with f.write_str, once we have a way to ensure
            // that pattern is a string literal.
            write!(f, $pattern)
        })
    };
    ($pattern:literal, $($args:tt)*) => {
        $crate::LazyFormat::new(#[inline] move |f: &mut ::core::fmt::Formatter| -> ::core::fmt::Result {
            write!(f, $pattern, $($args)*)
        })
    };
}

/// Struct containing the captured information from a [`lazy_format`] invocation.
///
/// This struct provides a `Display` implementation, which actually executes
/// the formatting which was lazily requested by [`lazy_format`].
#[derive(Clone, PartialEq, Eq)]
pub struct LazyFormat<F: Fn(&mut Formatter) -> fmt::Result>(F);

impl<F: Fn(&mut Formatter) -> fmt::Result> LazyFormat<F> {
    pub fn new(f: F) -> Self {
        LazyFormat(f)
    }
}

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

#[cfg(test)]
mod tests {
    use crate::lazy_format;
    use core::fmt::Write;

    #[test]
    fn basic_format() {
        let mut dest = String::new();
        write!(&mut dest, "{}", lazy_format!("{}, {}", 123, "Hello, World")).unwrap();
        assert_eq!(dest, "123, Hello, World")
    }

    #[test]
    fn ensure_lazy() {
        use std::cell::Cell;

        let call_count = Cell::new(0);

        let get_value = || {
            call_count.set(call_count.get() + 1);
            return 10;
        };

        let lazily_formatted = lazy_format!("{}, {} ({})", "Hello", "World", get_value());
        let lazy2 = lazy_format!("{} {} {}", get_value(), get_value(), get_value());

        // At this point, the cell should not have been incremented at all
        assert_eq!(call_count, Cell::new(0));

        let mut dest = String::new();

        write!(&mut dest, "{}\n{}", lazily_formatted, lazy2).unwrap();
        assert_eq!(dest, "Hello, World (10)\n10 10 10");
        assert_eq!(call_count, Cell::new(4));
    }

    #[test]
    fn test_recursive() {
        use std::cell::Cell;
        let call_count = Cell::new(0);

        let get_value = || {
            call_count.set(call_count.get() + 1);
            return call_count.get();
        };

        let lazy1 = lazy_format!("{}, {}", get_value(), get_value());
        let lazy2 = lazy_format!("({lazy}), ({lazy})", lazy = lazy1);
        let lazy3 = lazy_format!("({lazy}), ({lazy})", lazy = lazy2);

        assert_eq!(call_count, Cell::new(0));

        let result = lazy3.to_string();

        assert_eq!(result, "((1, 2), (3, 4)), ((5, 6), (7, 8))");
        assert_eq!(call_count, Cell::new(8));
    }

    #[test]
    fn test_return_value() {
        let values = (0..5).map(|value| lazy_format!("'{}'... ", value));

        let mut dest = String::new();

        for value in values {
            write!(&mut dest, "{}", value).unwrap();
        }

        assert_eq!(dest, "'0'... '1'... '2'... '3'... '4'... ")
    }

    #[test]
    fn test_result_value_with_lifetime() {
        let message = "this is a     sentence with\twhitespace".to_string();
        let parts = message
            .as_str()
            .split_whitespace()
            .map(|part| lazy_format!("'{}' ", part));

        let mut dest = String::new();

        for part in parts {
            write!(&mut dest, "{}", part).unwrap();
        }

        assert_eq!(dest, "'this' 'is' 'a' 'sentence' 'with' 'whitespace' ")
    }

    // TODO: performance test, see if this is comparable to a handwritten `Display` impl
}
