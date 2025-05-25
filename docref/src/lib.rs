use cucumber_thirtyfour_worlder::worlder;

/// Example of a struct for a world derived with the [`#[worlder]`][worlder] macro.
///
/// It should not be used directly, because you will not be able to specify the versions
/// of `thirtyfour` and `cucumber` to use in your tests.
///
/// Instead, derive your own world struct using the [`#[worlder]`][worlder] macro with:
///
/// ```rust,ignore
/// use cucumber_thirtyfour_worlder::worlder;
///
/// #[worlder]
/// pub struct AppWorld;
/// ```
///
/// [worlder]: https://docs.rs/cucumber-thirtyfour-worlder/latest/cucumber_thirtyfour_worlder/attr.worlder.html
#[worlder]
pub struct AppWorld;
