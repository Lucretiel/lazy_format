#![cfg_attr(not(test), no_std)]

use core::fmt::{self, Debug, Display, Formatter};

#[macro_export]
macro_rules! lazy_format {
    ($pattern:literal) => {
        $crate::LazyFormat(#[inline] move |f: &mut ::core::fmt::Formatter| -> ::core::fmt::Result {
            // TODO: replace this with f.write_str, once we have a way to ensure
            // that pattern is a string literal.
            write!(f, $pattern)
        })
    };
    ($pattern:literal, $($args:tt)*) => {
        $crate::LazyFormat(#[inline] move |f: &mut ::core::fmt::Formatter| -> ::core::fmt::Result {
            write!(f, $pattern, $($args)*)
        })
    };
}

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
