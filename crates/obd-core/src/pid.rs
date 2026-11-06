//! Mode 01 PID decoding (SAE J1979 standard PIDs).
//!
//! Only a starter set is defined. Add entries to [`PIDS`] as needed.
//! Ford-specific enhanced PIDs (Mode 21/22) are not covered here.

use crate::{Error, Result};

/// A decoded, human-readable value.
#[derive(Debug, Clone, PartialEq)]
pub struct Reading {
    pub pid: u8,
    pub name: &'static str,
    pub value: f64,
    pub unit: &'static str,
}

/// Static definition for one Mode 01 PID.
#[derive(Debug)]
pub struct PidDef {
    pub code: u8,
    pub name: &'static str,
    pub unit: &'static str,
    /// Number of payload bytes the decoder reads.
    pub len: usize,
    pub decode: fn(&[u8]) -> f64,
}

fn coolant_temp(d: &[u8]) -> f64 {
    f64::from(d[0]) - 40.0
}
fn manifold_pressure(d: &[u8]) -> f64 {
    f64::from(d[0])
}
fn engine_rpm(d: &[u8]) -> f64 {
    (f64::from(d[0]) * 256.0 + f64::from(d[1])) / 4.0
}
fn vehicle_speed(d: &[u8]) -> f64 {
    f64::from(d[0])
}
fn percent(d: &[u8]) -> f64 {
    f64::from(d[0]) * 100.0 / 255.0
}

pub const PIDS: &[PidDef] = &[
    PidDef {
        code: 0x05,
        name: "Engine coolant temperature",
        unit: "°C",
        len: 1,
        decode: coolant_temp,
    },
    PidDef {
        code: 0x0B,
        name: "Intake manifold pressure",
        unit: "kPa",
        len: 1,
        decode: manifold_pressure,
    },
    PidDef {
        code: 0x0C,
        name: "Engine RPM",
        unit: "rpm",
        len: 2,
        decode: engine_rpm,
    },
    PidDef {
        code: 0x0D,
        name: "Vehicle speed",
        unit: "km/h",
        len: 1,
        decode: vehicle_speed,
    },
    PidDef {
        code: 0x11,
        name: "Throttle position",
        unit: "%",
        len: 1,
        decode: percent,
    },
    PidDef {
        code: 0x2F,
        name: "Fuel tank level",
        unit: "%",
        len: 1,
        decode: percent,
    },
];

/// Look up the definition for a PID code.
pub fn lookup(code: u8) -> Option<&'static PidDef> {
    PIDS.iter().find(|p| p.code == code)
}

/// Decode a payload for `code` into a [`Reading`].
pub fn decode(code: u8, payload: &[u8]) -> Result<Reading> {
    let def = lookup(code).ok_or(Error::UnsupportedPid(code))?;
    if payload.len() < def.len {
        return Err(Error::ShortPayload {
            need: def.len,
            got: payload.len(),
        });
    }
    Ok(Reading {
        pid: code,
        name: def.name,
        value: (def.decode)(payload),
        unit: def.unit,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_rpm() {
        let r = decode(0x0C, &[0x1A, 0xF8]).unwrap();
        assert!((r.value - 1726.0).abs() < 1e-9);
        assert_eq!(r.unit, "rpm");
    }

    #[test]
    fn decodes_coolant_temp() {
        let r = decode(0x05, &[0x7B]).unwrap(); // 123 - 40 = 83
        assert!((r.value - 83.0).abs() < 1e-9);
    }

    #[test]
    fn rejects_short_payload() {
        assert!(matches!(
            decode(0x0C, &[0x1A]),
            Err(Error::ShortPayload { need: 2, got: 1 })
        ));
    }
}
