# Changelog

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning 2.0.0](https://semver.org/spec/v2.0.0.html).

## 1.10.0

### Added

- `make_lazy_format!` now has a closure-like syntax, using `|f| { ... }` instead of `f => { ... }`. The older syntax is retained for backwards compatibility.

### Fixed

- Fix an issue where formatting without arguments (e.g, `lazy_format!("{}")`) would succeed incorrectly ([#5](https://github.com/Lucretiel/lazy_format/issues/5)).
  - Thanks [@ten0](https://github.com/Ten0) for the report.

## 1.9.0

- Changelog started
