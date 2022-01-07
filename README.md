# rollbar-rs
**Send error messages to Rollbar from your Rust applications**

This package provides a [Rollbar](https://rollbar.com) client for Rust applications, allowing
you to report errors and keep track of crashes in your apps. Its primary goal is providing an
extremely easy to use API.

## Example
```rust
#[macro_use] extern crate rollbar_rs;

fn main() {
    rollbar_rs::set_token("....");
    rollbar_rs::set_code_version(env!("CARGO_PKG_VERSION"));
    rollbar_rs::set_environment("production");

    // Handle panics and report them as critical errors
    rollbar_rs::handle_panics!(Critical);

    rollbar!(Debug message = "Starting Up");

    match std::fs::File::open("/tmp/foo.txt") {
        Ok(file) => {
            // Do something with the file
        },
        Err(err) => {
            rollbar!(error = err, custom = { file = "/tmp/foo.txt" });
        }
    };
}
```
