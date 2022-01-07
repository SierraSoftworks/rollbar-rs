/// Reports an event to Rollbar using the default client.
///
/// This macro will generate and submit an event to Rollbar using the default client.
/// It supports messages and errors, and allows you to specify any of the supported
/// reporting fields.
/// 
/// # Examples
/// ## Messages
/// ```rust
/// use rollbar_rs::*;
/// 
/// rollbar!(Debug message = "This is an example message", { foo: "bar" }, context = "project#index");
/// ```
/// 
/// ## Errors
/// ```rust
/// use rollbar_rs::*;
/// 
/// let err = std::io::Error::new(std::io::ErrorKind::Other, "Some error");
/// rollbar!(Critical error = err, context = "project#index");
/// ```
/// 
/// ## Custom Fields
/// You can also specify custom fields which are included in your event by setting
/// the `custom` field. A `map!` macro is provided to simplify the generation of 
/// the appropriate data structure.
/// ```rust
/// use rollbar_rs::*;
/// 
/// rollbar!(Info message = "This is an example with custom fields.", { foo: "bar" }, custom = map!{ owner: "Bob" });
/// ```
#[macro_export]
macro_rules! rollbar {
    (message = $msg:expr $(, { $($extra_key:ident: $extra_val:expr),+ })? $(,$key:ident = $val:expr)*) => {
        $crate::report($crate::rollbar_format!(message = $msg $(, { $($extra_key: $extra_val),+ })? $(, $key = $val)*));
    };

    (error = $err:expr $(,$key:ident = $val:expr)*) => {
        $crate::report($crate::rollbar_format!(error = $err $(, $key = $val)*));
    };
    
    ($level:ident message = $msg:expr $(, { $($extra_key:ident: $extra_val:expr),+ })? $(,$key:ident = $val:expr)*) => {
        $crate::report($crate::rollbar_format!($level message = $msg $(, { $($extra_key: $extra_val),+ })? $(, $key = $val)*));
    };

    ($level:ident error = $err:expr $(,$key:ident = $val:expr)*) => {
        $crate::report($crate::rollbar_format!($level error = $err $(, $key = $val)*));
    };
}

/// Generates a Rollbar data payload which can be submitted to the Rollbar API.
/// 
/// This macro is intended to be used to generate the reporting payload
/// that your application uses to send events to Rollbar when working with
/// a custom client. Under normal conditions, you will use the [`rollbar!`]
/// macro instead.
/// 
/// # Examples
/// ## Messages
/// ```rust
/// use rollbar_rs::*;
/// 
/// let client = Client::with_default_transport(Configuration::default()).unwrap();
/// client.report(rollbar_format!(Debug message = "This is an example message", { foo: "bar" }, context = "project#index"));
/// ```
/// 
/// ## Errors
/// ```rust
/// use rollbar_rs::*;
/// 
/// let err = std::io::Error::new(std::io::ErrorKind::Other, "Some error");
/// let client = rollbar_rs::Client::with_default_transport(rollbar_rs::Configuration::default()).unwrap();
/// client.report(rollbar_format!(Critical error = err, context = "project#index"));
/// ```
/// 
/// ## Custom Fields
/// You can also specify custom fields which are included in your event by setting
/// the `custom` field. A `map!` macro is provided to simplify the generation of 
/// the appropriate data structure.
/// ```rust
/// use rollbar_rs::*;
/// 
/// let client = rollbar_rs::Client::with_default_transport(rollbar_rs::Configuration::default()).unwrap();
/// client.report(rollbar_format!(Info message = "This is an example with custom fields.", { foo: "bar" }, custom = map!{ owner: "Bob" }));
/// ```
#[macro_export]
macro_rules! rollbar_format {
    (message = $msg:expr $(, { $($extra_key:ident: $extra_val:expr),+ })? $(,$key:ident = $val:expr)*) => {
        $crate::types::Data {
            body: $crate::types::Body::MessageBody {
                telemetry: None,
                message: $crate::types::Message {
                    body: $msg.into(),
                    extra: $crate::map!{$($($extra_key : $extra_val),+)?},
                }
            },
            notifier: Some($crate::types::Notifier {
                name: Some("SierraSoftworks/rollbar-rs".into()),
                version: Some($crate::VERSION.into()),
            }),
            $($key: Some($val.into()),)*
            ..Default::default()
        }
    };

    (error = $err:expr $(,$key:ident = $val:expr)*) => {
        {
            let mut frames = $crate::helpers::get_backtrace_frames();
            let line = line!() - 3;

            frames.push($crate::types::Frame {
                filename: file!().to_string(),
                lineno: Some(line as i32),
                ..Default::default()
            });

            $crate::types::Data {
                body: $crate::types::Body::TraceBody {
                    telemetry: None,
                    trace: $crate::types::Trace {
                        exception: $crate::helpers::get_exception(&$err),
                        frames: frames,
                    }
                },
                notifier: Some($crate::types::Notifier {
                    name: Some("SierraSoftworks/rollbar-rs".into()),
                    version: Some($crate::VERSION.into()),
                }),
                $($key: Some($val.into()),)*
                ..Default::default()
            }
        }
    };

    ($level:ident message = $msg:expr $(, { $($extra_key:ident: $extra_val:expr),+ })? $(,$key:ident = $val:expr)*) => {
        {
            let mut data = $crate::rollbar_format!(message = $msg $(, { $($extra_key: $extra_val),+ })? $(,$key = $val)*);
            data.level = Some($crate::Level::$level);
            data
        }
    };

    ($level:ident error = $err:expr $(,$key:ident = $val:expr)*) => {
        {
            let mut data = $crate::rollbar_format!(error = $err $(,$key = $val)*);
            data.level = Some($crate::Level::$level);
            data
        }
    };
}

/// Constructs a generic Rollbar object with the provided keys.
///
/// This macro is intended to be used with the [`rollbar!`] and
/// [`rollbar_format!`] macros to generate fields like `client`,
/// `server`, and `custom`.
/// 
/// # Examples
/// ```rust
/// use rollbar_rs::*;
/// 
/// rollbar!(message = "Example with custom data", custom = map!{ foo: "bar" });
/// ```
#[macro_export]
macro_rules! map {
    {$($key:ident : $val:expr),*} => {
        {
            #[allow(unused_mut)]
            let mut extra: std::collections::HashMap<String, serde_json::Value> = std::collections::HashMap::new();
            $(
                extra.insert(stringify!($key).to_string(), serde_json::json!($val));
            )*

            extra
        }
    };
}

/// Configures Rollbar to handle any panics which occur within your
/// application, reporting them as exceptions at the specified level.
#[macro_export]
macro_rules! handle_panics {
    ($($key:ident = $val:expr),*) => {
        handle_panics!(Critical $(,$key = $val)*)
    };

    ($level:ident $(,$key:ident = $val:expr)*) => {
        ::std::panic::set_hook(::std::boxed::Box::new(move |panic_info| {
            let payload = panic_info.payload();
            let message = match payload.downcast_ref::<&str>() {
                Some(s) => s,
                None => match payload.downcast_ref::<String>() {
                    Some(s) => s,
                    None => "Panic",
                }
            };

            let frames = if let Some(location) = panic_info.location() {
                vec![
                    $crate::types::Frame {
                        filename: location.file().into(),
                        lineno: Some(location.line()).map(|l| l as i32),
                        colno: Some(location.column()).map(|c| c as i32),
                        ..Default::default()
                    },
                ]
            } else {
                vec![]
            };

            $crate::report($crate::types::Data {
                body: $crate::types::Body::TraceBody {
                    telemetry: None,
                    trace: $crate::types::Trace {
                        exception: $crate::types::Exception {
                            class: "<panic>".into(),
                            message: Some(message.into()),
                            description: Some(message.into()),
                            ..Default::default()
                        },
                        frames: frames,
                    }
                },
                level: Some($crate::Level::$level),
                notifier: Some($crate::types::Notifier {
                    name: Some("SierraSoftworks/rollbar-rs".into()),
                    version: Some($crate::VERSION.into()),
                }),
                $($key: Some($val.into()),)*
                ..Default::default()
            });
        }));
    };
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_report() {
        rollbar!(message = "Hello, world!");
        rollbar!(Debug message= "Hello, world!", environment = "production", context = "test", custom = map!{foo: "bar"});
    }

    #[test]
    fn test_handle_panics() {
        handle_panics!();
        let _ = ::std::panic::take_hook();

        handle_panics!(Critical);
        let _ = ::std::panic::take_hook();
    }

    #[test]
    fn generate_message_report() {
        let msg = rollbar_format!(Debug message = "Hello, world!", { foo: "bar" }, environment = "testing");
        assert_eq!(msg.level, Some(Level::Debug));
        assert_eq!(msg.environment, Some("testing".to_owned()));

        match msg.body {
            crate::types::Body::MessageBody { message, .. } => {
                assert_eq!(message.body, "Hello, world!");
                assert_eq!(message.extra, map!{foo: "bar"});
            },
            _ => panic!("Expected message body")
        }
    }

    #[test]
    fn generate_error_report() {
        let err = crate::errors::user("This is a test error.", "Try not crashing.");
        let data = rollbar_format!(error = err, title = "Example Exception", environment = "testing");
        assert_eq!(data.environment, Some("testing".to_owned()));

        match data.body {
            crate::types::Body::TraceBody { trace, .. } => {
                assert_eq!(trace.exception.class, "rollbar_rs::errors::Error");
                assert_ne!(trace.exception.message, None);
                assert_ne!(trace.exception.description, None);

                assert!(trace.frames.len() > 0, "the trace should have at least one frame");
                assert_eq!(trace.frames[trace.frames.len()-1].filename, file!().to_string());
            },
            _ => panic!("Unexpected trace type")
        }
    }

    #[test]
    fn generate_extra()  {
        let extra = map!(
            foo: "bar",
            baz: "qux"
        );

        assert_eq!(extra.len(), 2);
        assert_eq!(extra["foo"], "bar".to_owned());
        assert_eq!(extra["baz"], "qux".to_owned());
    }
}