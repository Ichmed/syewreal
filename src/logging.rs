use std::error::Error;

use serde::Serialize;


#[cfg(not(feature = "panic_on_error"))]
#[inline]
/// prints an error to the JS console or panics if the `panic_on_error` feature is enabled
pub fn handle_error(error: impl Error) {
    print_error(error)
}

#[cfg(feature = "panic_on_error")]
#[inline]
/// prints an error to the JS console or panics if the `panic_on_error` feature is enabled
pub fn handle_error(error: impl Error) {
    panic_error(error)
}


/// prints an error to the JS console
#[inline]
#[allow(dead_code)]
pub fn print_error(error: impl Error) {
    web_sys::console::error_1(&error.to_string().into());
}

/// always panics with a given error
#[inline]
#[allow(dead_code)]
pub fn panic_error(error: impl Error) -> ! {
    panic!("fatal error: {}", error.to_string());
}

pub enum Operation {
    Update,
    Receive
}


#[cfg(feature = "log_traffic")]
/// prints an object to the JS console if the `log_traffic` feature is enabled
pub fn print_traffic(direction: Operation, obj: &impl Serialize) {
    use js_sys::JSON::parse;

    let obj = match parse(&serde_json::to_string(&obj).unwrap().as_str()) {
        Ok(value) => value,
        Err(value) => value
    };
    let direction = match direction {
        Operation::Update => "Sent Update",
        Operation::Receive => "Received"
    };
    web_sys::console::log_2(&direction.into(), &obj);

    
}
#[cfg(not(feature = "log_traffic"))]
#[inline]
/// prints an object to the JS console if the `log_traffic` feature is enabled
pub fn print_traffic(_direction: Operation, _obj: &impl Serialize) {
    ()
}
