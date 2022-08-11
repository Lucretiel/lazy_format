/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

// Copyright 2019-2022 Nathan West

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
        format!("{}", self.count.get())
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
    fn no_args_with_curlies() {
        let result = lazy_format!("{{ braces }}").to_string();
        assert_eq!(result, "{ braces }")
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
        fn double_str(s: &str) -> impl Display + '_ {
            lazy_format!("{}, {}", s, s)
        }

        let content = "Hello".to_string();
        let result = double_str(content.as_str()).to_string();
        assert_eq!(result, "Hello, Hello");
    }

    #[test]
    fn test_if_let() {
        fn describe_optional(value: Option<isize>) -> impl Display {
            lazy_format!(
                if let Some(3) | Some(4) = value => "Got a 3 or a 4"
                else if let Some(x) = value => "Got a value: {x}"
                else => "Got nothing"
            )
        }

        assert_eq!(describe_optional(Some(3)).to_string(), "Got a 3 or a 4");
        assert_eq!(describe_optional(Some(4)).to_string(), "Got a 3 or a 4");
        assert_eq!(describe_optional(Some(10)).to_string(), "Got a value: 10");
        assert_eq!(describe_optional(None).to_string(), "Got nothing")
    }

    #[test]
    fn test_if_else() {
        let value = 10;

        // This mostly exists to use "expand macro recursively" to check that
        // it correctly generates optimized write calls in these branches. A
        // future version of this test can actually test this by using a Write
        // adapter.
        let result = lazy_format!(
            if value == 10 => "ten"
            else if value > 10 => ("value: {}", value)
            else => ""
        );

        assert_eq!(result.to_string(), "ten");
    }

    #[test]
    fn test_loop_underscore() {
        let result = lazy_format!("Ab" for _ in 0..5);
        assert_eq!(result.to_string(), "AbAbAbAbAb");
    }

    #[test]
    fn test_contextual_loop() {
        let value = 10;
        let pairs = [('a', 'b'), ('c', 'd')];
        let result = lazy_format!("{value} {left} {right}, " for &(left, right) in &pairs);
        assert_eq!(result.to_string(), "10 a b, 10 c d, ")
    }

    /// Test that the for loop version of lazy_format still works when the
    /// iterator type still has a try_for_each method, for some reason.
    #[test]
    fn test_bad_iterator() {
        #[derive(Copy, Clone)]
        struct Collection<'a> {
            slice: &'a [i32],
        }

        impl<'a> Iterator for Collection<'a> {
            type Item = &'a i32;

            fn next(&mut self) -> Option<Self::Item> {
                self.slice.split_first().map(|(head, tail)| {
                    self.slice = tail;
                    head
                })
            }
        }

        impl<'a> Collection<'a> {
            pub fn new(slice: &'a [i32]) -> Self {
                Self { slice }
            }

            #[allow(dead_code)]
            pub fn try_for_each<T>(&mut self, _body: impl FnMut(&'a i32) -> T) -> T {
                panic!("This shouldn't be called")
            }
        }

        let collection = Collection::new(&[1, 2, 3, 4]);
        let output = lazy_format!("{item} " for item in collection);
        assert_eq!(output.to_string(), "1 2 3 4 ");
    }
}
