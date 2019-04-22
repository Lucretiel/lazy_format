#![cfg(test)]

use std::cell::Cell;

// We put all the tests in a separate test crate, to ensure that macro imports
// work correctly, and also to give us access to std

#[derive(Debug, Clone, Default, Eq, PartialEq)]
struct ValueEmitter {
    count: Cell<usize>,
}

impl ValueEmitter {
    fn new() -> Self {
        ValueEmitter {
            count: Cell::new(0),
        }
    }

    fn next(&self) -> String {
        self.count.set(self.count.get() + 1);
        return format!("{}", self.count.get());
    }

    fn count(&self) -> usize {
        self.count.get()
    }
}

mod lazy_format {
    use std::fmt::Display;
    use std::fmt::Write;

    use crate::ValueEmitter;
    use lazy_format::lazy_format;

    #[test]
    fn basic_format() {
        let mut dest = String::new();
        write!(
            &mut dest,
            "{}",
            lazy_format!("{}, {value}", 123, value = "Hello, World")
        )
        .unwrap();
        assert_eq!(dest, "123, Hello, World");
    }

    #[test]
    fn no_args_format() {
        let result = lazy_format!("Hello, World!").to_string();
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn ensure_lazy() {
        let emitter = &ValueEmitter::new();

        let lazily_formatted = lazy_format!("{}, {} ({})", "Hello", "World", emitter.next());
        let lazy2 = lazy_format!("{} {} {}", emitter.next(), emitter.next(), emitter.next());

        // At this point, the cell should not have been incremented at all
        assert_eq!(emitter.count(), 0);

        //
        let mut dest = String::new();
        write!(&mut dest, "{}\n{}", lazily_formatted, lazy2).unwrap();
        assert_eq!(dest, "Hello, World (1)\n2 3 4");
        assert_eq!(emitter.count(), 4);

        // Because the formatting is lazy, emitter.next() will continue to be called
        // every time the lazy_format values are formatted
        dest.clear();
        write!(&mut dest, "{}\n{}", lazily_formatted, lazy2).unwrap();
        assert_eq!(dest, "Hello, World (5)\n6 7 8");
        assert_eq!(emitter.count(), 8);
    }

    #[test]
    fn test_recursive() {
        let emitter = &ValueEmitter::new();

        let lazy1 = lazy_format!("{}, {}", emitter.next(), emitter.next());
        let lazy2 = lazy_format!("({lazy}), ({lazy})", lazy = lazy1);
        let lazy3 = lazy_format!("({lazy}), ({lazy})", lazy = lazy2);

        assert_eq!(emitter.count(), 0);

        assert_eq!(lazy3.to_string(), "((1, 2), (3, 4)), ((5, 6), (7, 8))");
        assert_eq!(emitter.count(), 8);

        assert_eq!(
            lazy3.to_string(),
            "((9, 10), (11, 12)), ((13, 14), (15, 16))"
        );
        assert_eq!(emitter.count(), 16);
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
        // This function tests that the return value of lazy_format (and specifically
        // of make_lazy_format) fulfills the lifetime bound
        fn double_str<'a>(s: &'a str) -> impl Display + 'a {
            lazy_format!("{}, {}", s, s)
        }

        let content = "Hello".to_string();
        let result = double_str(content.as_str()).to_string();
        assert_eq!(result, "Hello, Hello");
    }
}

mod semi_lazy_format {
    use std::fmt::Display;

    use lazy_format::semi_lazy_format;

    use crate::ValueEmitter;

    #[test]
    fn no_args() {
        let result = semi_lazy_format!("Hello, World!").to_string();
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn one_arg() {
        let result = semi_lazy_format!("{}!", "Hello, World").to_string();
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn two_args() {
        let result = semi_lazy_format!("{}, {}!", "Hello", "World").to_string();
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn named_args() {
        let result = semi_lazy_format!("{h}, {w}!", h = "Hello", w = "World").to_string();
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn reverse_named_args() {
        let result = semi_lazy_format!("{h}, {w}!", w = "World", h = "Hello").to_string();
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_evaluate_once() {
        let emitter = ValueEmitter::new();

        let formatted = semi_lazy_format!(
            "{} {b} {a} {b} {}",
            emitter.next(),
            emitter.next(),
            a = emitter.next(),
            b = emitter.next(),
        );

        // At this point, the emitted should have been advanced
        assert_eq!(emitter.count(), 4);

        let result1 = formatted.to_string();
        assert_eq!(result1, "1 4 3 4 2");

        let result2 = formatted.to_string();
        assert_eq!(result2, "1 4 3 4 2");

        // The emitter should not have been touched
        assert_eq!(emitter.count(), 4);
    }

    /// Test that lazy_format drops lifetimes by evaluating the arguments
    /// immediately.
    #[test]
    fn test_dropped_lifetime() {
        fn get_value<'a>(value: &'a str) -> impl Display {
            semi_lazy_format!("{} {a}", String::from(value), a = String::from(value))
        }

        assert_eq!(get_value("HELLO").to_string(), "HELLO HELLO");
    }
}
