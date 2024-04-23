use crate::debug;

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// Wrapper for a Promise. Can be polled to fill in its value.
pub struct PollableValue<T: 'static + std::marker::Send + Clone> {
    pub value: Option<T>,
    promise: poll_promise::Promise<Option<T>>,
}

impl<T: std::marker::Send + 'static + Clone> PollableValue<T> {

    /// default_value: value to be used before poll is complete
    /// 
    /// promise: a Promise that will be polled
    pub fn new(default_value: Option<T>, promise: poll_promise::Promise<Option<T>>) -> PollableValue<T> {
        Self {
            value: default_value,
            promise,
        }
    }

    /// Polls the promise and returns it's value if ready
    ///
    /// returns: value if ready or default exists, none otherwise
    pub fn poll(&mut self) -> Option<T> {
        if let Some(result) = self.promise.ready() {
            self.value = Some(<std::option::Option<T> as Clone>::clone(&result).expect("bad value in PollableValue"));
        } 
        self.value.clone()
    }
}