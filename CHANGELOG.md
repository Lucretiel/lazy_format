# Changelog

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning 2.0.0](https://semver.org/spec/v2.0.0.html).

## 2.0.1

### Internal

- Improved detecting of empty format strings

## 2.0.0

What started as a simple collection of cleanup changes eventually spiraled out to the complete implementation of everything I've been looking to do for `lazy_format` 2.0. In particular, the improvements to `if` necessitated minor but technically backwards incompatible macro changes, which presented the opportunity to make the other backwards incompatible changes, like removing deprecated functionality and simplifying macro definitions. In general, if you're only using `lazy_format` and you aren't using `horrorshow`, you shouldn't experience any breakages in the upgrade.

### Added

- `lazy_format` now supports implicit named parameters. Because we use `format_args!` internally, this support came automatically
- `lazy_format`'s `if`, `match`, and `for` syntaxes now allow for unparenthesized format literals, in cases where there's no arguments.

### Changed

- `lazy_format`'s `if` conditional version now allows omitting the `else` branch, in which case it will simply write nothing if none of the other `if` / `else if` branches match.
- `lazy_format`'s `else` branch now requires the `=>` syntax, for consistency (previously it was optional).
- Upgrade to Edition 2021. This means we can use the new `pat` pattern, which correctly handles nested `|` in patterns. This allows for much simpler macro definitions in a way that's fully backwards compatible (aside from the increased MSRV).

### Removed

- **Breaking** Removed `semi_lazy_format`; the complexity of its implementation wasn't worth the added functionality. Instead, callers should manually evaluate any parameters they want to be eagerly evaluated before calling `lazy_format` and then pass them manually.
- **Breaking** Removed `horrorshow` feature, due to potential buggy interactions with transitive dependencies. Prefer instead to create a `display` adapter which connects `Display` to `horrorshow`.
- **Breaking** `semi_lazy_format!` no longer supports the deprecated `fmt => body` syntax; it now must use the `|fmt| body` syntax.

### Fixed

- Fixed potential issue with ambiguous method call in the `for` loop version of `lazy_format`

## 1.10.0

### Added

- `make_lazy_format!` now has a closure-like syntax, using `|f| { ... }` instead of `f => { ... }`. The older syntax is retained for backwards compatibility.

### Fixed

- Fix an issue where formatting without arguments (e.g, `lazy_format!("{}")`) would succeed incorrectly ([#5](https://github.com/Lucretiel/lazy_format/issues/5)).
  - Thanks [@ten0](https://github.com/Ten0) for the report.

## 1.9.0

- Changelog started
