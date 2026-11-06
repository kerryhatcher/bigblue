use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    /// The underlying link (serial port, etc.) failed.
    #[error("transport error: {0}")]
    Transport(String),

    /// The adapter answered with an error token such as `NO DATA` or `UNABLE TO CONNECT`.
    #[error("adapter reported: {0}")]
    Adapter(String),

    /// The response did not contain the expected service/PID echo.
    #[error("unexpected response: {0:?}")]
    UnexpectedResponse(String),

    /// The response contained something that is not hex where hex was expected.
    #[error("invalid hex in response: {0:?}")]
    InvalidHex(String),

    /// No PID definition exists for this code in the decoder table.
    #[error("unsupported PID {0:#04X}")]
    UnsupportedPid(u8),

    /// The payload was shorter than the PID's definition requires.
    #[error("payload too short: need {need} bytes, got {got}")]
    ShortPayload { need: usize, got: usize },
}

pub type Result<T> = std::result::Result<T, Error>;
