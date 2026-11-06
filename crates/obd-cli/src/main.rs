use std::process::ExitCode;

use clap::{Parser, Subcommand};
use obd_core::{Transport, pid, response};
use obd_elm327::{DEFAULT_BAUD, Elm327, available_ports};

#[derive(Parser)]
#[command(
    name = "obd",
    version,
    about = "Read OBD-II data from a vehicle via an ELM327 adapter"
)]
struct Cli {
    /// Serial device for the adapter, e.g. /dev/ttyUSB0 or COM3.
    #[arg(short, long, env = "OBD_PORT", global = true)]
    port: Option<String>,

    /// Serial baud rate.
    #[arg(short, long, default_value_t = DEFAULT_BAUD, global = true)]
    baud: u32,

    /// Force an ELM327 protocol number (e.g. 6 = ISO 15765-4 CAN 11/500).
    #[arg(long, global = true)]
    protocol: Option<String>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// List serial ports visible to the OS.
    Ports,

    /// Read one or more Mode 01 PIDs given as hex, e.g. `obd read 0C 0D 05`.
    Read {
        #[arg(required = true, value_parser = parse_hex_byte)]
        pids: Vec<u8>,
    },

    /// Send a raw ELM327 command or hex request and print the reply.
    Raw {
        /// e.g. `ATI`, `0100`, `0902`.
        command: String,
    },
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(msg) => {
            eprintln!("error: {msg}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<(), String> {
    if let Command::Ports = cli.command {
        for p in available_ports().map_err(|e| e.to_string())? {
            println!("{p}");
        }
        return Ok(());
    }

    let port = cli
        .port
        .ok_or("no port given; pass --port or set OBD_PORT (see `obd ports`)")?;
    let mut adapter = Elm327::open(&port, cli.baud).map_err(|e| e.to_string())?;
    adapter.init().map_err(|e| e.to_string())?;
    if let Some(p) = &cli.protocol {
        adapter.set_protocol(p).map_err(|e| e.to_string())?;
    }

    match cli.command {
        Command::Ports => unreachable!("handled above"),
        Command::Read { pids } => {
            for code in pids {
                let text = adapter
                    .command(&format!("01{code:02X}"))
                    .map_err(|e| e.to_string())?;
                let payload = response::mode01_payload(&text, code).map_err(|e| e.to_string())?;
                let reading = pid::decode(code, &payload).map_err(|e| e.to_string())?;
                println!(
                    "{:<30} {:>10.1} {}",
                    reading.name, reading.value, reading.unit
                );
            }
        }
        Command::Raw { command } => {
            let text = adapter.command(&command).map_err(|e| e.to_string())?;
            println!("{text}");
        }
    }
    Ok(())
}

fn parse_hex_byte(s: &str) -> Result<u8, String> {
    let s = s.trim_start_matches("0x").trim_start_matches("0X");
    u8::from_str_radix(s, 16).map_err(|e| format!("invalid PID {s:?}: {e}"))
}
