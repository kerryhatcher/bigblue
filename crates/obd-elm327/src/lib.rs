//! Driver for ELM327-compatible USB/serial OBD-II adapters.
//!
//! The adapter speaks ASCII over a serial port: commands end with `\r`, and
//! replies end with the `>` prompt. Lines starting with `AT` configure the
//! adapter itself; hex strings such as `010C` are sent to the vehicle.

use std::io::{ErrorKind, Read, Write};
use std::thread;
use std::time::{Duration, Instant};

use obd_core::{Error, Result, Transport};
use serialport::SerialPort;

/// Default baud rate for ELM327 clones. Many use 38400; some use 115200.
pub const DEFAULT_BAUD: u32 = 38_400;

const PROMPT: u8 = b'>';
const READ_TIMEOUT: Duration = Duration::from_millis(100);
const COMMAND_TIMEOUT: Duration = Duration::from_secs(10);

/// An open connection to an ELM327-compatible adapter.
pub struct Elm327 {
    port: Box<dyn SerialPort>,
}

impl Elm327 {
    /// Open the serial device at `path`, e.g. `/dev/ttyUSB0` or `COM3`.
    pub fn open(path: &str, baud: u32) -> Result<Self> {
        let port = serialport::new(path, baud)
            .timeout(READ_TIMEOUT)
            .open()
            .map_err(|e| Error::Transport(format!("{path}: {e}")))?;
        Ok(Self { port })
    }

    /// Reset the adapter and configure it for clean, machine-parsable output.
    ///
    /// Turns off echo, line feeds, and spaces; disables headers; and selects
    /// automatic protocol detection. Call once after [`Elm327::open`].
    pub fn init(&mut self) -> Result<()> {
        // ATZ takes ~1s to reboot the chip and replies with a banner.
        self.command_with_timeout("ATZ", Duration::from_secs(3))?;
        for cmd in ["ATE0", "ATL0", "ATS0", "ATH0", "ATSP0"] {
            self.command(cmd)?;
        }
        Ok(())
    }

    /// Force a specific protocol, e.g. `"6"` for ISO 15765-4 CAN 11-bit / 500 kbaud,
    /// which is what most Ford HS-CAN Mode 01 traffic uses.
    pub fn set_protocol(&mut self, protocol: &str) -> Result<()> {
        self.command(&format!("ATSP{protocol}"))?;
        Ok(())
    }

    fn command_with_timeout(&mut self, cmd: &str, timeout: Duration) -> Result<String> {
        self.port
            .clear(serialport::ClearBuffer::Input)
            .map_err(|e| Error::Transport(e.to_string()))?;

        let mut frame = Vec::with_capacity(cmd.len() + 1);
        frame.extend_from_slice(cmd.as_bytes());
        frame.push(b'\r');
        self.port.write_all(&frame).map_err(io_err)?;
        self.port.flush().map_err(io_err)?;

        let deadline = Instant::now() + timeout;
        let mut buf = Vec::with_capacity(128);
        let mut chunk = [0u8; 64];

        loop {
            match self.port.read(&mut chunk) {
                Ok(0) => {}
                Ok(n) => {
                    buf.extend_from_slice(&chunk[..n]);
                    if buf.contains(&PROMPT) {
                        break;
                    }
                }
                Err(e) if e.kind() == ErrorKind::TimedOut => {}
                Err(e) => return Err(io_err(e)),
            }
            if Instant::now() >= deadline {
                // Adapter did not finish; return what we have so far.
                // A missing prompt usually means the vehicle is silent.
                if buf.is_empty() {
                    return Err(Error::Transport(format!(
                        "timeout waiting for reply to {cmd}"
                    )));
                }
                break;
            }
            thread::sleep(Duration::from_millis(5));
        }

        let text = String::from_utf8_lossy(&buf)
            .trim_end_matches(|c: char| c == '>' || c.is_whitespace())
            .to_string();
        Ok(text)
    }
}

impl Transport for Elm327 {
    fn command(&mut self, cmd: &str) -> Result<String> {
        self.command_with_timeout(cmd, COMMAND_TIMEOUT)
    }
}

/// List serial ports the OS can see. ELM327 clones usually appear as
/// `/dev/ttyUSB*` (CH340/CP210x/FTDI chips) or `/dev/ttyACM*`.
pub fn available_ports() -> Result<Vec<String>> {
    let ports = serialport::available_ports().map_err(|e| Error::Transport(e.to_string()))?;
    Ok(ports.into_iter().map(|p| p.port_name).collect())
}

fn io_err(e: std::io::Error) -> Error {
    Error::Transport(e.to_string())
}
