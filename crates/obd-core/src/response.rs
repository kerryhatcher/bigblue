//! Parsing of raw adapter text into payload bytes.
//!
//! Assumes the adapter was initialised with headers off and spaces off
//! (`ATH0`, `ATS0`), so a Mode 01 reply to PID `0C` looks like `410C1AF8`.

use crate::{Error, Result};

/// Adapter status strings that indicate failure rather than data.
const ERROR_TOKENS: &[&str] = &[
    "NO DATA",
    "UNABLE TO CONNECT",
    "BUS INIT",
    "BUS ERROR",
    "CAN ERROR",
    "DATA ERROR",
    "BUFFER FULL",
    "STOPPED",
    "ERROR",
    "?",
];

/// Extract the data bytes of a Mode 01 response for `pid`.
///
/// Returns the bytes that follow the `41 <pid>` echo.
pub fn mode01_payload(text: &str, pid: u8) -> Result<Vec<u8>> {
    let upper = text.to_ascii_uppercase();

    for token in ERROR_TOKENS {
        if upper.contains(token) {
            return Err(Error::Adapter((*token).to_string()));
        }
    }

    let expected_prefix = format!("41{pid:02X}");

    // Adapters end lines with \r and the prompt `>` may share a line with data.
    for line in upper.split(['\r', '\n']) {
        let compact: String = line
            .chars()
            .filter(|c| !c.is_whitespace() && *c != '>')
            .collect();
        if compact.is_empty() {
            continue;
        }
        if let Some(rest) = compact.strip_prefix(&expected_prefix) {
            return decode_hex(rest);
        }
    }

    Err(Error::UnexpectedResponse(text.to_string()))
}

/// Decode an even-length hex string into bytes.
pub fn decode_hex(s: &str) -> Result<Vec<u8>> {
    if s.len() % 2 != 0 || !s.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(Error::InvalidHex(s.to_string()));
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|_| Error::InvalidHex(s.to_string())))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_rpm_reply() {
        assert_eq!(
            mode01_payload("410C1AF8\r\r>", 0x0C).unwrap(),
            vec![0x1A, 0xF8]
        );
    }

    #[test]
    fn parses_spaced_reply() {
        assert_eq!(mode01_payload("41 0D 3C\r\n>", 0x0D).unwrap(), vec![0x3C]);
    }

    #[test]
    fn reports_no_data() {
        assert!(matches!(
            mode01_payload("NO DATA\r>", 0x0C),
            Err(Error::Adapter(_))
        ));
    }

    #[test]
    fn rejects_wrong_pid() {
        assert!(matches!(
            mode01_payload("410D3C\r>", 0x0C),
            Err(Error::UnexpectedResponse(_))
        ));
    }
}
