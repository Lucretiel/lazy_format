// We put all the tests in a separate test crate, to ensure that macro imports
// work correctly

use std::cell::Cell;
use std::fmt::Display;
use std::fmt::Write;

use lazy_format::lazy_format;

#[test]
fn basic_format() {
    let mut dest = String::new();
    write!(&mut dest, "{}", lazy_format!("{}, {}", 123, "Hello, World")).unwrap();
    assert_eq!(dest, "123, Hello, World")
}

#[test]
fn no_args_format() {
    let result = lazy_format!("Hello, World!").to_string();
    assert_eq!(result, "Hello, World!")
}

#[test]
fn ensure_lazy() {
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
    // This function tests that the return value of lazy_format (and specifically
    // of make_lazy_format) fulfills the lifetime bound
    fn double_str<'a>(s: &'a str) -> impl Display + 'a {
        lazy_format!("{}, {}", s, s)
    }

    let content = "Hello".to_string();
    let result = double_str(content.as_str()).to_string();
    assert_eq!(result, "Hello, Hello");
}
