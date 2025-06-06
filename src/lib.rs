//! [![Crates.io](https://img.shields.io/crates/v/cucumber-thirtyfour-worlder?logo=rust)](https://crates.io/crates/cucumber-thirtyfour-worlder)
//! [![License](https://img.shields.io/crates/l/cucumber-thirtyfour-worlder?logo=mit)](https://github.com/mondeja/cucumber-thirtyfour-worlder/blob/master/LICENSE)
//! [![Tests](https://img.shields.io/github/actions/workflow/status/mondeja/cucumber-thirtyfour-worlder/ci.yml?label=tests&logo=github)](https://github.com/mondeja/cucumber-thirtyfour-worlder/actions)
//! [![macro docs.rs](https://img.shields.io/docsrs/cucumber-thirtyfour-worlder?logo=docs.rs)](https://docs.rs/cucumber-thirtyfour-worlder)
//! [![reference docs.rs](https://img.shields.io/docsrs/cucumber-thirtyfour-worlder?logo=docs.rs&label=world%20reference)][appworld-reference]
//! [![Crates.io downloads](https://img.shields.io/crates/d/cucumber-thirtyfour-worlder)](https://crates.io/crates/cucumber-thirtyfour-worlder)
//!
//! Do you need to reuse a bunch of logic between different projects testing
//! apps with [cucumber-rs] and [thirtyfour]? This crate is for you.
//!
//! Provides a [`cucumber::World`] builder that can be used to create an
//! `AppWorld` for thirtyfour tests, allowing to inject environment variables to
//! parametrize them.
//!
//! - `BROWSER`: browser to use. Supported are `firefox`, `chrome`, and `edge`.
//! - `HEADLESS`: by default, tests are executed in headless mode. Set this
//!   to `false` to run them in a visible browser.
//! - `WINDOW_SIZE`: size of the browser window. The default is `1920x1080`.
//! - `HOST_URL`: base URL of the application under test. The default is
//!   `http://localhost:8080`.
//! - `DRIVER_URL`: the URL of the `WebDriver` server. The default is
//!   `http://localhost:4444`.
//!
//! # Usage
//!
//! Create a crate and add the following dependencies to your `Cargo.toml`.
//!
//! ```toml
//! [dependencies]
//! cucumber = "0.21"
//! thirtyfour = "0.35"
//! cucumber-thirtyfour-worlder = "0.1"
//! ```
//!
//! Inside, create your [`AppWorld`][appworld-reference] struct and pass
//! it the [`#[worlder]`][worlder] attribute.
//!
//! ```rust,ignore
//! use cucumber_thirtyfour_worlder::worlder;
//!
//! #[worlder]
//! pub struct AppWorld;
//! ```
//!
//! > See the reference of the created world [here][appworld-reference].
//!
//! Then, create a crate for tests and run the world as you would do with
//! [`cucumber::World`] directly.
//!
//! ```rust,ignore
//! // tests/desktop.rs
//! use your_crate::AppWorld;
//! use cucumber::World;
//!
//! #[tokio::main]
//! async fn main() {
//!     AppWorld::cucumber()
//!         .fail_on_skipped()
//!         .run_and_exit("./features/desktop")
//!         .await
//! }
//! ```
//!
//! Start a webdriver server before running the tests.
//!
//! ```sh
//! chromedriver --port=4444
//! # or `geckodriver --port=4444` (for Firefox)
//! # or `msedgedriver --port=4444` (for MsEdge)
//! ```
//!
//! And run your tests passing a browser in the `BROWSER` environment variable.
//!
//! ```sh
//! BROWSER=chrome cargo test --package your-crate --test desktop -- --fail-fast
//! ```
//!
//! Where `desktop` is the name of your test file and `your-crate` is the name of
//! the crate that contains the [`AppWorld`][appworld-reference] struct.
//!
//! # Known issues
//!
//! ## Additional configuration for cargo-machete
//!
//! The [`cargo-machete`][cargo-machete] tool don't know that you're not using
//! `cucumber` and `thirtyfour`, so it could complain about missing dependencies.
//! To fix this, add the following to your _Cargo.toml_.
//!
//! ```toml
//! [package.metadata.cargo-machete]
//! ignored = ["thirtyfour", "cucumber"]
//! ```
//!
//! [cucumber-rs]: https://cucumber-rs.github.io/cucumber/main/
//! [thirtyfour]: https://docs.rs/thirtyfour/latest/thirtyfour/
//! [`cucumber::World`]: https://docs.rs/cucumber/latest/cucumber/trait.World.html
//! [appworld-reference]: https://docs.rs/cucumber-thirtyfour-worlder-docref/latest/cucumber_thirtyfour_worlder_docref/struct.AppWorld.html
//! [worlder]: https://docs.rs/cucumber-thirtyfour-worlder/latest/cucumber_thirtyfour_worlder/attr.worlder.html
//! [cargo-machete]: https://github.com/bnjbvr/cargo-machete

#[cfg(test)]
mod tests;

use proc_macro2::{TokenStream, TokenTree};
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
};

/// Attribute macro to build [`cucumber::World`] struct for the app to test.
///
/// Accept the next named arguments:
///
/// - `check_concurrency_cli_option_when_firefox` (*bool*, default `true`): when enabled,
///   the implementation will check if the `--concurrency` or `-c` CLI option is set
///   to `1` invoking cucumber tests when using Firefox. Multiple sessions in parallel
///   are not allowed by geckodriver and this limitation is easy to forget, hence this
///   convenient argument.
///
/// See the reference of the created world [here][appworld-reference].
///
/// [`cucumber::World`]: https://docs.rs/cucumber/latest/cucumber/trait.World.html
/// [appworld-reference]: https://docs.rs/cucumber-thirtyfour-worlder-docref/latest/cucumber_thirtyfour_worlder_docref/struct.AppWorld.html
#[proc_macro_attribute]
pub fn worlder(
    args: proc_macro::TokenStream,
    stream: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    assert!(
        !stream.is_empty(),
        "#[worlder] macro requires a struct to be passed"
    );

    let args = parse_macro_input!(args as WorlderArgs);
    let (check_concurrency_cli_option_when_firefox, check_concurrency_cli_option_when_firefox_fn) =
        if args.check_concurrency_cli_option_when_firefox {
            (
                quote!(Self::__check_firefox_concurrency_cli_option()),
                build_check_concurrency_cli_option_when_firefox_fn(),
            )
        } else {
            (TokenStream::new(), TokenStream::new())
        };
    let cucumber = args.cucumber;
    let thirtyfour = args.thirtyfour;

    let mut before_struct = TokenStream::new();
    let original_struct = TokenStream::from(stream.clone());

    let mut token_stream_iter = original_struct.into_iter();
    let length = token_stream_iter.clone().count();

    // Allow whatever before the struct definition.
    // This is needed if we want to add attributes to the struct,
    // like documenting it.
    let mut maybe_item_1 = None;
    for _ in 0..length {
        let item = token_stream_iter.next();
        if let Some(item) = item {
            let item_clone = item.clone();
            if let TokenTree::Ident(ident) = { item_clone } {
                let ident_str = ident.to_string();
                if &ident_str == "pub" || ident_str.starts_with("pub(") || &ident_str == "struct" {
                    maybe_item_1 = Some(TokenTree::Ident(ident));
                    break;
                }
            }
            before_struct.extend(std::iter::once(item));
        }
    }
    let maybe_item_2 = token_stream_iter.next();
    let maybe_item_3 = token_stream_iter.next();
    let maybe_item_4 = token_stream_iter.next();

    let mut struct_idents = vec![];

    let item_1_string = match maybe_item_1 {
        Some(TokenTree::Ident(ident)) => {
            struct_idents.push(TokenTree::from(ident.clone()));
            ident.to_string()
        }
        _ => String::new(),
    };
    let item_2_string = match maybe_item_2 {
        Some(TokenTree::Ident(ident)) => {
            struct_idents.push(TokenTree::Ident(ident.clone()));
            ident.to_string()
        }
        _ => String::new(),
    };
    let item_3_string = match maybe_item_3 {
        Some(TokenTree::Ident(ident)) => {
            struct_idents.push(TokenTree::Ident(ident.clone()));
            ident.to_string()
        }
        Some(TokenTree::Punct(punct)) => punct.to_string(),
        _ => String::new(),
    };
    let item_4_string = match maybe_item_4 {
        Some(TokenTree::Punct(punct)) => punct.to_string(),
        _ => String::new(),
    };

    let item_1_str = item_1_string.as_str();
    let item_2_str = item_2_string.as_str();
    let item_3_str = item_3_string.as_str();
    let item_4_str = item_4_string.as_str();

    let (valid, with_vis) = if (item_1_str == "pub" || item_1_str.starts_with("pub("))
        && item_2_str == "struct"
        && item_4_str == ";"
    {
        (true, true)
    } else if item_1_str == "struct" && item_3_str == ";" {
        (true, false)
    } else {
        (false, false)
    };

    assert!(
        valid,
        "#[worlder] macro requires a token stream like `pub struct AppWorld;` or `struct AppWorld;`"
    );

    let (vis_ident, struct_ident, struct_name_ident) = if with_vis {
        (
            struct_idents[0].clone(),
            struct_idents[1].clone(),
            struct_idents[2].clone(),
        )
    } else {
        (
            TokenTree::Ident(proc_macro2::Ident::new("", proc_macro2::Span::call_site())),
            struct_idents[0].clone(),
            struct_idents[1].clone(),
        )
    };

    let ret = quote! {
        #before_struct
        #[derive(Debug, #cucumber::World)]
        #[world(init = Self::new)]
        #vis_ident #struct_ident #struct_name_ident {
            driver: #thirtyfour::WebDriver,
            driver_url: String,
            host_url: String,
            headless: bool,
            window_size: (u32, u32),
        }

        impl #struct_name_ident {
            #[doc(hidden)]
            pub async fn new() -> Self {
                Self::__build_driver().await
            }

            #[doc = "Get the driver of the world."]
            #[must_use]
            pub fn driver(&self) -> &#thirtyfour::WebDriver {
                &self.driver
            }

            #[doc = "Get the driver URL of the world."]
            #[doc = ""]
            #[doc = "It's defined by the `DRIVER_URL` environment variable, which defaults to `\"http://localhost:4444\"`."]
            #[must_use]
            pub fn driver_url(&self) -> &str {
                &self.driver_url
            }

            #[doc = "Get the host URL of the world."]
            #[doc = ""]
            #[doc = "It's defined by the `HOST_URL` environment variable, which defaults to `\"http://localhost:8080\"`."]
            #[must_use]
            pub fn host_url(&self) -> &str {
                &self.host_url
            }

            #[doc = "Get the headless mode of the world."]
            #[doc = ""]
            #[doc = "It's defined by the `HEADLESS` environment variable, which defaults to `true`."]
            #[must_use]
            pub fn headless(&self) -> bool {
                self.headless
            }

            #[doc = "Get the window size of the world."]
            #[doc = ""]
            #[doc = "It's defined by the `WINDOW_SIZE` environment variable, which defaults to `\"1920x1080\"`."]
            #[must_use]
            pub fn window_size(&self) -> (u32, u32) {
                self.window_size
            }

            #[doc = "Navigate to the given path inside the host."]
            pub async fn goto_path(&self, path: &str) -> Result<&Self, #thirtyfour::error::WebDriverError> {
                let url = format!("{}{}", self.host_url(), path);
                if let Err(err) = self.driver().goto(&url).await {
                    Err(err)
                } else {
                    Ok(self)
                }
            }

            async fn __build_driver() -> Self {
                let browser = Self::__discover_browser();
                let driver_url = Self::__discover_driver_url();
                let host_url = Self::__discover_host_url();
                let headless = Self::__discover_headless();
                let (window_width, window_height) = Self::__discover_window_size();

                let driver = if &browser == "chrome" {
                    let mut caps = #thirtyfour::DesiredCapabilities::chrome();
                    let window_size_opt = format!(
                        "--window-size={window_width},{window_height}",
                    );
                    let mut opts = vec!["--no-sandbox", &window_size_opt];
                    if headless {
                        opts.push("--headless");
                    }
                    <#thirtyfour::ChromeCapabilities
                        as
                    #thirtyfour::BrowserCapabilitiesHelper>::insert_browser_option(
                        &mut caps, "args", opts
                    )
                        .unwrap_or_else(|err| {
                            panic!("Failed to set Chrome options: {err}");
                        });
                    #thirtyfour::WebDriver::new(&driver_url, caps)
                        .await
                        .unwrap_or_else(|err| {
                            panic!(
                                "Failed to create WebDriver for Chrome: {err}. \
                                Make sure that chromedriver server is running at {driver_url}",
                            )
                        })
                } else if &browser == "firefox" {
                    #check_concurrency_cli_option_when_firefox;
                    let mut caps = #thirtyfour::DesiredCapabilities::firefox();
                    if headless {
                        caps.set_headless().unwrap_or_else(|err| {
                            panic!("Failed to set Firefox headless mode: {err}");
                        });
                    }
                    let driver = #thirtyfour::WebDriver::new(&driver_url, caps).await.unwrap_or_else(|err| {
                        panic!(
                            "Failed to create WebDriver for Firefox: {err}. \
                            Make sure that geckodriver server is running at {driver_url}",
                        )
                    });
                    // Firefox loads the window dimensions of the last session,
                    // so we need to set the window size explicitly.
                    driver.set_window_rect(0, 0, window_width, window_height)
                        .await
                        .expect("Failed to set window size to {width}x{height}");
                    driver
                } else if &browser == "edge" {
                    let mut caps = #thirtyfour::DesiredCapabilities::edge();
                    let window_size_opt = format!(
                        "--window-size={window_width},{window_height}",
                    );
                    let mut opts = vec!["--no-sandbox", &window_size_opt];
                    if headless {
                        opts.push("--headless");
                    }
                    <#thirtyfour::EdgeCapabilities
                        as
                    #thirtyfour::BrowserCapabilitiesHelper>::insert_browser_option(&mut caps, "args", opts)
                        .unwrap_or_else(|err| {
                            panic!("Failed to set Edge options: {err}");
                        });
                    #thirtyfour::WebDriver::new(&driver_url, caps).await.unwrap_or_else(|err| {
                        panic!(
                            "Failed to create WebDriver for Edge: {err}. \
                            Make sure that edgedriver server is running at {driver_url}",
                        )
                    })
                } else {
                    panic!(
                        "Unsupported browser. BROWSER environment variable is: \
                        {browser}. Supported browsers are: \"chrome\", \"firefox\" \
                        and \"edge\"."
                    );
                };

                Self {
                    driver,
                    driver_url,
                    host_url,
                    headless,
                    window_size: (window_width, window_height),
                }
            }

            fn __discover_browser() -> String {
                std::env::var("BROWSER").unwrap_or_else(|_| {
                    panic!(
                        "BROWSER environment variable is not set. \
                         Supported browsers are: \"chrome\", \"firefox\" \
                         and \"edge\"."
                    )
                })
            }

            fn __discover_driver_url() -> String {
                std::env::var("DRIVER_URL").unwrap_or("http://localhost:4444".to_string())
            }

            fn __discover_host_url() -> String {
                std::env::var("HOST_URL").unwrap_or("http://localhost:8080".to_string())
            }

            fn __discover_headless() -> bool {
                std::env::var("HEADLESS").unwrap_or("true".to_string()) == "true"
            }

            fn __discover_window_size() -> (u32, u32) {
                let window_size = std::env::var("WINDOW_SIZE").unwrap_or("1920x1080".to_string());
                let mut parts = window_size.split('x');
                let width = parts.next().unwrap_or_else(|| {
                    panic!(
                        "Invalid WINDOW_SIZE environment variable format. \
                        Expected format: WIDTHxHEIGHT"
                    );
                }).parse::<u32>().unwrap_or_else(|_| {
                    panic!(
                        "Invalid WINDOW_SIZE environment variable format. \
                        Expected format: WIDTHxHEIGHT"
                    );
                });
                let height = parts.next().unwrap_or_else(|| {
                    panic!(
                        "Invalid WINDOW_SIZE environment variable format. \
                        Expected format: WIDTHxHEIGHT"
                    );
                }).parse::<u32>().unwrap_or_else(|_| {
                    panic!(
                        "Invalid WINDOW_SIZE environment variable format. \
                        Expected format: WIDTHxHEIGHT"
                    );
                });
                (width, height)
            }

            #check_concurrency_cli_option_when_firefox_fn
        }
    };

    proc_macro::TokenStream::from(ret)
}

fn build_check_concurrency_cli_option_when_firefox_fn() -> TokenStream {
    quote! {
        fn __check_firefox_concurrency_cli_option() {
            let lets_panic = || {
                panic!(
                    "The driver geckodriver requires --concurrency or -c \
                    option to be set to 1 because geckodriver does not allows \
                    multiple sessions in parallel. Pass --concurrency=1 or -c 1 \
                    to the test command, like \
                    `cargo test --test <test-name> -- --concurrency=1`."
                )
            };

            let mut reading_arg = false;
            let mut found = false;
            let args = std::env::args();
            for arg in args {
                if arg == "--concurrency" || arg == "-c" {
                    reading_arg = true;
                } else if arg.starts_with("--concurrency=")
                    || arg.starts_with("-c=")
                {
                    let value = arg
                        .split('=')
                        .nth(1)
                        .unwrap_or_else(|| panic!("Invalid argument: {arg}"));
                    let value = value.parse::<u32>();
                    if value.is_ok() && value.unwrap() != 1 {
                        lets_panic();
                    }
                    found = true;
                    break;
                } else if reading_arg {
                    let value = arg.parse::<u32>();
                    if value.is_ok() && value.unwrap() != 1 {
                        lets_panic();
                    }
                    found = true;
                    break;
                }
            }

            if !found {
                lets_panic();
            }
        }
    }
}

struct WorlderArgs {
    check_concurrency_cli_option_when_firefox: bool,
    cucumber: syn::Path,
    thirtyfour: syn::Path,
}

impl Default for WorlderArgs {
    fn default() -> Self {
        Self {
            check_concurrency_cli_option_when_firefox: true,
            cucumber: syn::parse_str::<syn::Path>("::cucumber").unwrap(),
            thirtyfour: syn::parse_str::<syn::Path>("::thirtyfour").unwrap(),
        }
    }
}

impl Parse for WorlderArgs {
    fn parse(input: ParseStream) -> syn::parse::Result<Self> {
        let mut args = WorlderArgs::default();
        while !input.is_empty() {
            let ident: syn::Ident = input.parse()?;
            if ident == "check_concurrency_cli_option_when_firefox" {
                input.parse::<syn::Token![=]>()?;
                let value: syn::LitBool = input.parse()?;
                args.check_concurrency_cli_option_when_firefox = value.value;
            } else if ident == "cucumber" {
                input.parse::<syn::Token![=]>()?;
                args.cucumber = input.parse()?;
            } else if ident == "thirtyfour" {
                input.parse::<syn::Token![=]>()?;
                args.thirtyfour = input.parse()?;
            } else {
                return Err(input.error(format!("Unknown argument: {ident}")));
            }
            if !input.is_empty() {
                input.parse::<syn::Token![,]>()?;
            }
        }
        Ok(args)
    }
}
