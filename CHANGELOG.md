# Changelog

## 2025-05-27 - [0.1.5]

### Enhancements

- Don't render unused code with `#[worlder]` macro.
- Explicitly set window size with [`set_window_rect`] when running Firefox.

[`set_window_rect`]: https://docs.rs/thirtyfour/0.35.0/thirtyfour/session/handle/struct.SessionHandle.html#method.set_window_rect

## 2025-05-26 - [0.1.4]

### New features

- Add `check_concurrency_cli_option_when_firefox` argument to `#[worlder]`
  macro.
- Add `cucumber` argument to `#[worlder]` macro to allow passing an in-scope
  `cucumber` crate.
- Add `thirtyfour` argument to `#[worlder]` macro to allow passing an in-scope
  `thirtyfour` crate.

### Enhancements

- Stop polluting the scope of the `#[worlder]` macro calling site.

## 2025-05-25 - [0.1.3]

### Bug fixes

- Fix error checking concurrency when using geckodriver.

## 2025-05-24 - [0.1.2]

### Bug fixes

- Allow to document the generated `AppWorld`.
- Don't print to stdout.

## 2025-05-24 - 0.1.0

Initial release

[0.1.5]: https://github.com/mondeja/cucumber-thirtyfour-worlder/compare/v0.1.4...v0.1.5
[0.1.4]: https://github.com/mondeja/cucumber-thirtyfour-worlder/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/mondeja/cucumber-thirtyfour-worlder/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/mondeja/cucumber-thirtyfour-worlder/compare/v0.1.0...v0.1.2
