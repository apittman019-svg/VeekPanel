//! OS access is isolated here; protocol parsing has no device dependencies.
use crate::protocol::{self, ControlEvent, Model, OriginalDecoder, SerialMessage};
use hidapi::{HidApi, HidDevice};
use std::{ffi::CString, io, time::Duration};
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Address {
    Hid(CString),
    Serial(String),
}

#[derive(Clone, Debug)]
pub struct DeviceDescriptor {
    pub address: Address,
    pub model: Model,
    pub vid: Option<u16>,
    pub pid: Option<u16>,
    pub serial: Option<String>,
    pub product: Option<String>,
    /// USB bcdDevice, not necessarily a firmware version.
    pub release_number: Option<u16>,
    pub interface: Option<i32>,
    pub usage_page: Option<u16>,
    pub usage: Option<u16>,
}

impl DeviceDescriptor {
    pub fn selected_original(port: String) -> Self {
        Self {
            address: Address::Serial(port),
            model: Model::Original,
            vid: None,
            pid: None,
            serial: None,
            product: None,
            release_number: None,
            interface: None,
            usage_page: None,
            usage: None,
        }
    }

    /// Connection key, not a persistent user mapping ID. Paths can change on reconnect.
    pub fn key(&self) -> String {
        match &self.address {
            Address::Hid(path) => format!("hid:{}", path.to_string_lossy()),
            Address::Serial(port) => format!("serial:{port}"),
        }
    }
}

#[derive(Debug, Error)]
pub enum HardwareError {
    #[error("HID access failed: {0}. Check device permissions and close other PCPanel software.")]
    Hid(#[from] hidapi::HidError),
    #[error("serial access failed: {0}. Check the selected port, permissions and other PCPanel software.")]
    Serial(#[from] serialport::Error),
    #[error("device I/O failed: {0}")]
    Io(#[from] io::Error),
    #[error("incomplete HID initialization write: {0}/65 bytes")]
    ShortWrite(usize),
    #[error("transport returned an invalid read length: {0}")]
    InvalidRead(usize),
}

pub fn discover_hid(api: &mut HidApi) -> Result<Vec<DeviceDescriptor>, HardwareError> {
    api.refresh_devices()?;
    Ok(api
        .device_list()
        .filter_map(|info| {
            let model = Model::from_usb(info.vendor_id(), info.product_id())?;
            Some(DeviceDescriptor {
                address: Address::Hid(info.path().to_owned()),
                model,
                vid: Some(info.vendor_id()),
                pid: Some(info.product_id()),
                serial: info.serial_number().map(str::to_owned),
                product: info.product_string().map(str::to_owned),
                release_number: Some(info.release_number()),
                interface: Some(info.interface_number()),
                usage_page: Some(info.usage_page()),
                usage: Some(info.usage()),
            })
        })
        .collect())
}

/// A serial-port listing is evidence of a port, never evidence of a PCPanel.
pub fn serial_candidates() -> Result<Vec<serialport::SerialPortInfo>, HardwareError> {
    Ok(serialport::available_ports()?)
}

/// Mock seam used by session tests. Reads return 0 on timeout, errors on disconnect.
pub trait Transport: Send {
    fn read(&mut self, data: &mut [u8]) -> Result<usize, HardwareError>;
    fn write(&mut self, data: &[u8]) -> Result<usize, HardwareError>;
}

struct HidTransport(HidDevice);
impl Transport for HidTransport {
    fn read(&mut self, data: &mut [u8]) -> Result<usize, HardwareError> {
        Ok(self.0.read_timeout(data, 100)?)
    }
    fn write(&mut self, data: &[u8]) -> Result<usize, HardwareError> {
        Ok(self.0.write(data)?)
    }
}

struct SerialTransport(Box<dyn serialport::SerialPort>);
impl Transport for SerialTransport {
    fn read(&mut self, data: &mut [u8]) -> Result<usize, HardwareError> {
        match self.0.read(data) {
            Ok(0) => Err(io::Error::new(io::ErrorKind::UnexpectedEof, "serial port closed").into()),
            Ok(n) => Ok(n),
            Err(e)
                if matches!(
                    e.kind(),
                    io::ErrorKind::TimedOut
                        | io::ErrorKind::WouldBlock
                        | io::ErrorKind::Interrupted
                ) =>
            {
                Ok(0)
            }
            Err(e) => Err(e.into()),
        }
    }
    fn write(&mut self, data: &[u8]) -> Result<usize, HardwareError> {
        Ok(self.0.write(data)?)
    }
}

pub fn open(descriptor: &DeviceDescriptor) -> Result<Box<dyn Transport>, HardwareError> {
    match &descriptor.address {
        Address::Hid(path) => Ok(Box::new(HidTransport(HidApi::new()?.open_path(path)?))),
        Address::Serial(port) => {
            // No probing writes or DTR toggles. Opening a serial port can still reset some boards.
            let port = serialport::new(port, 9600)
                .data_bits(serialport::DataBits::Eight)
                .parity(serialport::Parity::None)
                .stop_bits(serialport::StopBits::One)
                .flow_control(serialport::FlowControl::None)
                .timeout(Duration::from_millis(100))
                .open()?;
            Ok(Box::new(SerialTransport(port)))
        }
    }
}

#[derive(Debug)]
pub struct ReadBatch {
    pub raw: Vec<u8>,
    pub events: Vec<Result<ControlEvent, protocol::ParseError>>,
    pub heartbeats: usize,
}

pub struct Session {
    model: Model,
    transport: Box<dyn Transport>,
    serial_decoder: OriginalDecoder,
}

impl Session {
    pub fn new(
        model: Model,
        mut transport: Box<dyn Transport>,
        initialize: bool,
    ) -> Result<Self, HardwareError> {
        if initialize && model != Model::Original {
            let report = protocol::initialization_report();
            let written = transport.write(&report)?;
            if written != report.len() {
                return Err(HardwareError::ShortWrite(written));
            }
        }
        Ok(Self {
            model,
            transport,
            serial_decoder: OriginalDecoder::default(),
        })
    }

    pub fn read(&mut self) -> Result<ReadBatch, HardwareError> {
        let mut buffer = [0; 256];
        let n = self.transport.read(&mut buffer)?;
        if n > buffer.len() {
            return Err(HardwareError::InvalidRead(n));
        }
        let mut batch = ReadBatch {
            raw: buffer[..n].to_vec(),
            events: Vec::new(),
            heartbeats: 0,
        };
        if n == 0 {
            return Ok(batch);
        }
        if self.model == Model::Original {
            for message in self.serial_decoder.feed(&buffer[..n]) {
                match message {
                    Ok(SerialMessage::Heartbeat) => batch.heartbeats += 1,
                    Ok(SerialMessage::Control(event)) => batch.events.push(Ok(event)),
                    Err(e) => batch.events.push(Err(e)),
                }
            }
        } else {
            batch
                .events
                .push(protocol::parse_hid(self.model, &buffer[..n]));
        }
        Ok(batch)
    }
}
