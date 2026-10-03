//! Independently written from the protocol facts in docs/HARDWARE_PROTOCOL.md.
use std::fmt;
use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Model {
    Original,
    Rgb,
    Mini,
    Pro,
}

impl Model {
    pub fn from_usb(vid: u16, pid: u16) -> Option<Self> {
        match (vid, pid) {
            (0x04d8, 0xeb52) => Some(Self::Rgb),
            (0x0483, 0xa3c4) => Some(Self::Mini),
            (0x0483, 0xa3c5) => Some(Self::Pro),
            _ => None,
        }
    }

    pub fn knobs(self) -> u8 {
        if self == Self::Pro {
            5
        } else {
            4
        }
    }

    pub fn analog_count(self) -> u8 {
        if self == Self::Pro {
            9
        } else {
            4
        }
    }

    pub fn maximum(self) -> u8 {
        match self {
            Self::Original | Self::Rgb => 100,
            Self::Mini | Self::Pro => 255,
        }
    }
}

impl fmt::Display for Model {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Original => "Original/Maple (experimental)",
            Self::Rgb => "RGB",
            Self::Mini => "Mini",
            Self::Pro => "Pro",
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ControlEvent {
    /// Zero-based wire index; raw position, not measured OS audio volume.
    Analog {
        index: u8,
        raw: u8,
        maximum: u8,
    },
    Button {
        index: u8,
        pressed: bool,
    },
}

impl ControlEvent {
    pub fn display(&self, model: Model) -> String {
        match *self {
            Self::Analog {
                index,
                raw,
                maximum,
            } => {
                let (kind, number) = if index < model.knobs() {
                    ("KNOB", index + 1)
                } else {
                    ("SLIDER", index - model.knobs() + 1)
                };
                let percent = (u16::from(raw) * 100 + u16::from(maximum) / 2) / u16::from(maximum);
                format!("{kind}_{number} = {percent} (raw={raw}/{maximum})")
            }
            Self::Button { index, pressed } => format!(
                "KNOB_{}_PRESS = {}",
                index + 1,
                if pressed { "TRUE" } else { "FALSE" }
            ),
        }
    }
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ParseError {
    #[error("HID report must have 3..=64 bytes, received {0}")]
    Length(usize),
    #[error("unknown HID event opcode {0:#04x}")]
    Opcode(u8),
    #[error("control index {0} is outside this model's controls")]
    Index(u8),
    #[error("invalid control value {0}")]
    Value(u8),
    #[error("Original/Maple uses serial, not HID")]
    Transport,
    #[error("unrecognized Original/Maple line")]
    SerialLine,
    #[error("serial line exceeds 64 bytes; discarded until newline")]
    LineTooLong,
    #[error("serial input ended in a partial line")]
    TruncatedLine,
}

/// HIDAPI returns unnumbered reports without a leading report-ID byte.
/// No speculative zero-byte stripping: that can turn corrupt data into events.
pub fn parse_hid(model: Model, report: &[u8]) -> Result<ControlEvent, ParseError> {
    if model == Model::Original {
        return Err(ParseError::Transport);
    }
    if !(3..=64).contains(&report.len()) {
        return Err(ParseError::Length(report.len()));
    }
    let (opcode, index, value) = (report[0], report[1], report[2]);
    match opcode {
        1 => {
            if index >= model.analog_count() {
                return Err(ParseError::Index(index));
            }
            if value > model.maximum() {
                return Err(ParseError::Value(value));
            }
            Ok(ControlEvent::Analog {
                index,
                raw: value,
                maximum: model.maximum(),
            })
        }
        2 => {
            if index >= model.knobs() {
                return Err(ParseError::Index(index));
            }
            if value > 1 {
                return Err(ParseError::Value(value));
            }
            Ok(ControlEvent::Button {
                index,
                pressed: value == 1,
            })
        }
        _ => Err(ParseError::Opcode(opcode)),
    }
}

/// HIDAPI write includes report ID 0 followed by 64 payload bytes.
/// Only used on the three allowlisted HID models; never on Original serial.
pub fn initialization_report() -> [u8; 65] {
    let mut report = [0; 65];
    report[1] = 1;
    report
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SerialMessage {
    Control(ControlEvent),
    Heartbeat,
}

/// This grammar and active-low button interpretation still need stock-firmware validation.
pub fn parse_original_line(line: &[u8]) -> Result<SerialMessage, ParseError> {
    let line = line.strip_suffix(b"\r").unwrap_or(line);
    if line == b"pong" {
        return Ok(SerialMessage::Heartbeat);
    }
    if line.len() < 4 || !(b'0'..=b'3').contains(&line[1]) {
        return Err(ParseError::SerialLine);
    }
    let index = line[1] - b'0';
    match (line[0], line[2]) {
        (b'v', b'x') => {
            let digits = &line[3..];
            if digits.len() > 3 || !digits.iter().all(u8::is_ascii_digit) {
                return Err(ParseError::SerialLine);
            }
            let value = digits
                .iter()
                .fold(0u16, |v, d| v * 10 + u16::from(d - b'0'));
            if value > 100 {
                return Err(ParseError::SerialLine);
            }
            Ok(SerialMessage::Control(ControlEvent::Analog {
                index,
                raw: value as u8,
                maximum: 100,
            }))
        }
        (b'b', b' ') if line.len() == 4 && (line[3] == b'0' || line[3] == b'1') => {
            Ok(SerialMessage::Control(ControlEvent::Button {
                index,
                pressed: line[3] == b'0',
            }))
        }
        _ => Err(ParseError::SerialLine),
    }
}

/// Incremental bounded framer: reads may split or concatenate lines arbitrarily.
#[derive(Default)]
pub struct OriginalDecoder {
    pending: Vec<u8>,
    discarding: bool,
}

impl OriginalDecoder {
    pub fn finish(&self) -> Result<(), ParseError> {
        if self.pending.is_empty() && !self.discarding {
            Ok(())
        } else {
            Err(ParseError::TruncatedLine)
        }
    }

    pub fn feed(&mut self, bytes: &[u8]) -> Vec<Result<SerialMessage, ParseError>> {
        let mut messages = Vec::new();
        for &byte in bytes {
            if byte == b'\n' {
                if !self.discarding {
                    messages.push(parse_original_line(&self.pending));
                }
                self.pending.clear();
                self.discarding = false;
            } else if !self.discarding {
                if self.pending.len() == 64 {
                    self.pending.clear();
                    self.discarding = true;
                    messages.push(Err(ParseError::LineTooLong));
                } else {
                    self.pending.push(byte);
                }
            }
        }
        messages
    }
}
