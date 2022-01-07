/// Gets a Rollbar exception object representing the provided `std::errors::Error`.
///
/// This method is used to allow Rollbar to automatically capture information about
/// the type of exception which was raised, as well as its message and description.
///
/// It is intended to be called, primarily, by the trace!() macro and generally should
/// not be called by an end user themselves.
#[allow(dead_code)]
pub fn get_exception<T>(err: &T) -> crate::types::Exception
    where T: std::error::Error
{
    crate::types::Exception {
        class: std::any::type_name::<T>().to_owned(),
        message: Some(err.to_string()),
        description: err.source().map_or_else(|| Some(format!("{:#?}", err)), |s| Some(format!("{:#?}", s))),
    }
}

/// Generates a new unique identifier which may be used to identify a particular
/// event for de-duplication purposes.
/// 
/// This method is use internally by Rollbar to generate a unique identifier for
/// events before they queued for sending to Rollbar, ensuring that transports which
/// attempt to retry requests will not result in duplicate entries.
pub (in crate) fn new_uuid() -> String {
    rollbar_rust::Uuid::new().to_string()
}

/// Gathers the current thread's backtrace and returns it for use in a Rollbar
/// trace event.
/// 
/// This method is used internally by Rollbar to gather the current thread's
/// backtrace and is not intended to be called directly by consumers of this
/// crate.
pub fn get_backtrace_frames() -> Vec<crate::types::Frame> {
    let backtrace = backtrace::Backtrace::new();
    let mut frames: Vec<crate::types::Frame> = backtrace.frames().iter()
        .flat_map(|frames| frames.symbols())
        .map(|symbol| crate::types::Frame {
            filename: symbol.filename().map_or_else(|| "".to_owned(), |f| format!("{}", f.display())),
            lineno: symbol.lineno().map(|l| l as i32),
            colno: symbol.colno().map(|c| c as i32),
            method: symbol.name().map(|n| format!("{}", n)),
            ..Default::default()
        }).collect();

    // Remove the last frame, which is this function.
    frames.truncate(frames.len().saturating_sub(1));

    frames
}