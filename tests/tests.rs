/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

// Copyright 2019 Nathan West

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

    #[test]
    fn test_if_let() {
        fn describe_optional(value: Option<isize>) -> impl Display {
            lazy_format!(
                if let Some(3) | Some(4) = value => ("Got a 3 or a 4")
                else if let | Some(x) = value => ("Got a value: {}", x)
                else => ("Got nothing")
            )
        }

        assert_eq!(describe_optional(Some(3)).to_string(), "Got a 3 or a 4");
        assert_eq!(describe_optional(Some(4)).to_string(), "Got a 3 or a 4");
        assert_eq!(describe_optional(Some(10)).to_string(), "Got a value: 10");
        assert_eq!(describe_optional(None).to_string(), "Got nothing")
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
    fn test_positional_and_named() {
        let result = semi_lazy_format!("{h}, {}!", "World", h = "Hello").to_string();
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

    #[test]
    fn test_shadowed_name() {
        // Ensure that the keys used in formats don't act as shadowing variable
        // names

        let a: isize = 10;
        let b: isize = 20;
        let ab: isize = 0;

        let result = semi_lazy_format!("{a} {b} {ab}", ab = a + b, b = ab, a = ab);
        assert_eq!(result.to_string(), "0 0 30");
    }
}

#[cfg(feature = "horrorshow")]
mod horrorshow {
    use horrorshow::prelude::*;
    use horrorshow::{html, owned_html};
    use lazy_format::lazy_format;
    use std::string::ToString;

    #[test]
    fn test_horrorshow() {
        let data = "Hello & Goodbye".to_string();
        let content = lazy_format!("Content in angles: <{}>", data);

        let html = owned_html! {
            div {
                h1: content;
            }
        };

        let mut html_string = String::new();
        html.write_to_string(&mut html_string).unwrap();

        assert_eq!(
            html_string,
            "<div><h1>Content in angles: &lt;Hello &amp; Goodbye&gt;</h1></div>"
        );
    }

    #[test]
    fn test_in_render() {
        struct DemoRender<T: Render> {
            slug: &'static str,
            question: &'static str,
            answer: T,
        };

        impl<T: Render> Render for DemoRender<T> {
            fn render<'a>(&self, tmpl: &mut TemplateBuffer<'a>) {
                let link = lazy_format!("#{}", self.slug);

                tmpl << html! {
                    dt(class="faq-question", id=self.slug) {
                        strong: self.question;
                        a(class="hoverlink", href=link) {
                            i(class="fas fas-link")
                        }
                    }
                    dd(class="faq-answer"): &self.answer;
                }
            }
        }

        impl<T: Render> RenderMut for DemoRender<T> {
            fn render_mut<'a>(&mut self, tmpl: &mut TemplateBuffer<'a>) {
                self.render(tmpl)
            }
        }

        impl<T: Render> RenderOnce for DemoRender<T> {
            fn render_once<'a>(self, tmpl: &mut TemplateBuffer<'a>)
            where
                Self: Sized,
            {
                self.render(tmpl)
            }
        }

        let x = DemoRender {
            slug: "a-b-c",
            question: "A B C",
            answer: "Hello, World!",
        };

        let r = owned_html! {
            div {
                :x;
            }
        };
    }
}
