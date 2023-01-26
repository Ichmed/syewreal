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

pub enum Direction {
    Send,
    Receive
}


#[cfg(feature = "log_traffic")]
/// prints an object to the JS console if the `log_traffic` feature is enabled
pub fn print_traffic(direction: Direction, obj: &impl Serialize) {
    let obj = serde_json::to_string(&obj);
    let direction = match direction {
        Direction::Send =>    "Sent    ",
        Direction::Receive => "Received"
    };
    web_sys::console::log_1(&format!("{} {:?}", direction, obj).into());

    
}
#[cfg(not(feature = "log_traffic"))]
#[inline]
/// prints an object to the JS console if the `log_traffic` feature is enabled
pub fn print_traffic(_direction: Direction, _obj: &impl Serialize) {
    ()
}
