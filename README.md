# bigblue

A tool for interacting with my F150.

Reads live data from a Ford F-150 over the OBD-II port using an ELM327-compatible
USB adapter (tested target: OHP FORScan ELM327 OBD2 to USB, MS-CAN/HS-CAN).

## Layout

| Crate | Purpose |
| --- | --- |
| `crates/obd-core` | Adapter-independent: `Transport` trait, Mode 01 PID decoding, response parsing |
| `crates/obd-elm327` | Serial driver for ELM327-compatible adapters (AT commands, prompt framing) |
| `crates/obd-cli` | `obd` binary: list ports, read PIDs, send raw commands |

Dependency direction: `obd-cli` → `obd-elm327` → `obd-core`.

## Build

```sh
cargo build --workspace
cargo test --workspace
```

No system libraries are required. `serialport` is built without its `libudev`
feature, so port enumeration uses sysfs on Linux.

## Usage

Plug the adapter into a USB port, then:

```sh
cargo run -p obd-cli -- ports                        # find the adapter, e.g. /dev/ttyUSB0
export OBD_PORT=/dev/ttyUSB0                         # or pass --port each time

cargo run -p obd-cli -- read 0C 0D 05                # RPM, speed, coolant temp
cargo run -p obd-cli -- raw ATI                      # adapter identification
cargo run -p obd-cli -- --protocol 6 read 0C         # force ISO 15765-4 CAN 11-bit / 500 kbaud
```

The ignition should be on (engine running is not required for most PIDs).

## Notes

- Standard Mode 01 PIDs only so far. Ford enhanced PIDs (Mode 21/22) and
  module addressing for MS-CAN modules (PCM, ABS, etc.) are not implemented yet.
- ELM327 clones vary in quality. Start with `raw ATI` to confirm the link before
  reading PIDs.
