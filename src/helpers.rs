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