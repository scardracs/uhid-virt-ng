//! Codec definitions for encoding and decoding Linux UHID events.

use std::mem::size_of;

use crate::sys;
use crate::uhid_device::CreateParams;

/// Errors that can occur when reading or decoding events from `/dev/uhid`.
#[derive(Debug)]
pub enum StreamError {
    /// Standard I/O error when reading from or writing to the device handle.
    Io(std::io::Error),
    /// Unrecognized or unsupported event type received from the kernel.
    UnknownEventType(u32),
}

impl std::fmt::Display for StreamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "UHID I/O error: {err}"),
            Self::UnknownEventType(t) => write!(f, "Unknown UHID event type: {t}"),
        }
    }
}

impl std::error::Error for StreamError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            Self::UnknownEventType(_) => None,
        }
    }
}

/// Each of these flags defines whether a given report-type uses numbered reports.
///
/// If numbered reports are used for a type, all messages from the kernel already have the report-number as prefix.
/// Otherwise, no prefix is added by the kernel.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u64)]
pub enum DevFlags {
    /// Feature reports use numbered report IDs.
    FeatureReportsNumbered = sys::UHID_DEV_NUMBERED_FEATURE_REPORTS,
    /// Output reports use numbered report IDs.
    OutputReportsNumbered = sys::UHID_DEV_NUMBERED_OUTPUT_REPORTS,
    /// Input reports use numbered report IDs.
    InputReportsNumbered = sys::UHID_DEV_NUMBERED_INPUT_REPORTS,
}

impl DevFlags {
    /// Returns the bitmask value for this flag.
    #[must_use]
    pub const fn bits(self) -> u64 {
        self as u64
    }

    /// Extracts all recognized `DevFlags` from the given bitmask into a `Vec<DevFlags>`.
    #[must_use]
    pub fn from_bits_truncate(bits: u64) -> Vec<Self> {
        let mut flags = Vec::new();
        if bits & sys::UHID_DEV_NUMBERED_FEATURE_REPORTS != 0 {
            flags.push(Self::FeatureReportsNumbered);
        }
        if bits & sys::UHID_DEV_NUMBERED_OUTPUT_REPORTS != 0 {
            flags.push(Self::OutputReportsNumbered);
        }
        if bits & sys::UHID_DEV_NUMBERED_INPUT_REPORTS != 0 {
            flags.push(Self::InputReportsNumbered);
        }
        flags
    }
}

/// HID report types used in `GetReport` and `SetReport` requests.
///
/// See <https://www.kernel.org/doc/html/latest/hid/uhid.html#read>.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ReportType {
    /// Feature report.
    Feature = sys::UHID_REPORT_TYPE_UHID_FEATURE_REPORT,
    /// Output report.
    Output = sys::UHID_REPORT_TYPE_UHID_OUTPUT_REPORT,
    /// Input report.
    Input = sys::UHID_REPORT_TYPE_UHID_INPUT_REPORT,
}

impl TryFrom<u8> for ReportType {
    type Error = StreamError;

    /// Attempts to convert a raw numeric report type into a [`ReportType`].
    ///
    /// # Errors
    ///
    /// Returns [`StreamError::UnknownEventType`] if the value does not correspond to a valid report type.
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            sys::UHID_REPORT_TYPE_UHID_FEATURE_REPORT => Ok(Self::Feature),
            sys::UHID_REPORT_TYPE_UHID_OUTPUT_REPORT => Ok(Self::Output),
            sys::UHID_REPORT_TYPE_UHID_INPUT_REPORT => Ok(Self::Input),
            other => Err(StreamError::UnknownEventType(u32::from(other))),
        }
    }
}

/// Hardware bus type identifier for the virtual HID device.
///
/// See <https://elixir.bootlin.com/linux/latest/ident/BUS_INTEL_ISHTP>.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
pub enum Bus {
    /// PCI bus.
    PCI = 1,
    /// ISA Plug-and-Play bus.
    ISAPNP = 2,
    /// Universal Serial Bus.
    USB = 3,
    /// Hewlett-Packard Interface Loop.
    HIL = 4,
    /// Bluetooth transport.
    BLUETOOTH = 5,
    /// Virtual bus.
    VIRTUAL = 6,
    /// Industry Standard Architecture bus.
    ISA = 16,
    /// Intel 8042 keyboard/mouse controller.
    I8042 = 17,
    /// IBM PC/XT keyboard controller.
    XTKBD = 18,
    /// RS-232 serial port.
    RS232 = 19,
    /// Gameport connector.
    GAMEPORT = 20,
    /// Parallel printer port.
    PARPORT = 21,
    /// Commodore Amiga bus.
    AMIGA = 22,
    /// Apple Desktop Bus.
    ADB = 23,
    /// Inter-Integrated Circuit bus.
    I2C = 24,
    /// Host bus adapter.
    HOST = 25,
    /// GSC bus.
    GSC = 26,
    /// Atari keyboard bus.
    ATARI = 27,
    /// Serial Peripheral Interface bus.
    SPI = 28,
    /// Synaptics RMI bus.
    RMI = 29,
    /// Consumer Electronics Control bus.
    CEC = 30,
    /// Intel Integrated Sensor Hub transport.
    INTEL_ISHTP = 31,
}

/// Fixed size in bytes of a raw `sys::uhid_event` struct.
pub const UHID_EVENT_SIZE: usize = size_of::<sys::uhid_event>();

/// Events sent from user space to `/dev/uhid`.
///
/// See <https://www.kernel.org/doc/html/latest/hid/uhid.html#write>.
pub enum InputEvent<'a> {
    /// Create and register a virtual HID device with the kernel.
    Create(CreateParams),
    /// Destroy the virtual HID device.
    Destroy,
    /// Send an input report data payload from user space to the kernel.
    Input {
        /// Raw report bytes.
        data: &'a [u8],
    },
    /// Send an output report.
    Output {
        /// Raw output report bytes.
        data: Vec<u8>,
    },
    /// Send a legacy input event.
    OutputEv {
        /// Linux input event type.
        type_: u16,
        /// Linux input event code.
        code: u16,
        /// Linux input event value.
        value: i32,
    },
    /// Reply to a `GetReport` request received from the kernel.
    GetReportReply {
        /// Request ID matching the original `GetReport` request.
        id: u32,
        /// Error status (0 for success).
        err: u16,
        /// Report data payload.
        data: Vec<u8>,
    },
    /// Reply to a `SetReport` request received from the kernel.
    SetReportReply {
        /// Request ID matching the original `SetReport` request.
        id: u32,
        /// Error status (0 for success).
        err: u16,
    },
    /// Legacy feature request.
    Feature {
        /// Request ID.
        id: u32,
        /// Report number.
        report_num: u8,
    },
    /// Legacy feature answer.
    FeatureAnswer {
        /// Request ID matching the original request.
        id: u32,
        /// Error status (0 for success).
        err: u16,
        /// Feature report data payload.
        data: Vec<u8>,
    },
    /// Set a report on the virtual device.
    SetReport {
        /// Request ID.
        id: u32,
        /// Report number.
        report_num: u8,
        /// Report data payload.
        data: Vec<u8>,
    },
}

impl<'a> From<InputEvent<'a>> for sys::uhid_event {
    #[allow(clippy::too_many_lines)]
    fn from(input: InputEvent<'a>) -> Self {
        let mut event = Self::default();

        match input {
            InputEvent::Create(CreateParams {
                name,
                phys,
                uniq,
                bus,
                vendor,
                product,
                version,
                country,
                rd_data,
            }) => {
                event.type_ = sys::UHID_EVENT_TYPE_UHID_CREATE2;
                // SAFETY: event is initialized with zeroed memory and union variant create2 is active.
                let payload = unsafe { &mut event.u.create2 };

                let name_bytes = name.as_bytes();
                let name_len = name_bytes.len().min(payload.name.len());
                payload.name[..name_len].copy_from_slice(&name_bytes[..name_len]);

                let phys_bytes = phys.as_bytes();
                let phys_len = phys_bytes.len().min(payload.phys.len());
                payload.phys[..phys_len].copy_from_slice(&phys_bytes[..phys_len]);

                let uniq_bytes = uniq.as_bytes();
                let uniq_len = uniq_bytes.len().min(payload.uniq.len());
                payload.uniq[..uniq_len].copy_from_slice(&uniq_bytes[..uniq_len]);

                let rd_len = rd_data.len().min(sys::HID_MAX_DESCRIPTOR_SIZE);
                payload.rd_data[..rd_len].copy_from_slice(&rd_data[..rd_len]);

                #[allow(clippy::cast_possible_truncation)]
                {
                    payload.rd_size = rd_len as u16;
                }
                payload.bus = bus as u16;
                payload.vendor = vendor;
                payload.product = product;
                payload.version = version;
                payload.country = country;
            }
            InputEvent::Destroy => {
                event.type_ = sys::UHID_EVENT_TYPE_UHID_DESTROY;
            }
            InputEvent::Input { data } => {
                event.type_ = sys::UHID_EVENT_TYPE_UHID_INPUT2;
                // SAFETY: event is initialized with zeroed memory and union variant input2 is active.
                let payload = unsafe { &mut event.u.input2 };
                let len = data.len().min(sys::UHID_DATA_MAX);
                payload.data[..len].copy_from_slice(&data[..len]);
                #[allow(clippy::cast_possible_truncation)]
                {
                    payload.size = len as u16;
                }
            }
            InputEvent::Output { data } => {
                event.type_ = sys::UHID_EVENT_TYPE_UHID_OUTPUT;
                // SAFETY: event is initialized with zeroed memory and union variant output is active.
                let payload = unsafe { &mut event.u.output };
                let len = data.len().min(sys::UHID_DATA_MAX);
                payload.data[..len].copy_from_slice(&data[..len]);
                #[allow(clippy::cast_possible_truncation)]
                {
                    payload.size = len as u16;
                }
                payload.rtype = sys::UHID_REPORT_TYPE_UHID_OUTPUT_REPORT;
            }
            InputEvent::OutputEv { type_, code, value } => {
                event.type_ = sys::UHID_LEGACY_EVENT_TYPE_UHID_OUTPUT_EV;
                // SAFETY: event is initialized with zeroed memory and union variant output_ev is active.
                let payload = unsafe { &mut event.u.output_ev };
                payload.type_ = type_;
                payload.code = code;
                payload.value = value;
            }
            InputEvent::GetReportReply { err, id, data, .. } => {
                event.type_ = sys::UHID_EVENT_TYPE_UHID_GET_REPORT_REPLY;
                // SAFETY: event is initialized with zeroed memory and union variant get_report_reply is active.
                let payload = unsafe { &mut event.u.get_report_reply };
                payload.err = err;
                let len = data.len().min(sys::UHID_DATA_MAX);
                payload.data[..len].copy_from_slice(&data[..len]);
                #[allow(clippy::cast_possible_truncation)]
                {
                    payload.size = len as u16;
                }
                payload.id = id;
            }
            InputEvent::SetReportReply { err, id, .. } => {
                event.type_ = sys::UHID_EVENT_TYPE_UHID_SET_REPORT_REPLY;
                // SAFETY: event is initialized with zeroed memory and union variant set_report_reply is active.
                let payload = unsafe { &mut event.u.set_report_reply };
                payload.err = err;
                payload.id = id;
            }
            InputEvent::Feature { id, report_num } => {
                event.type_ = sys::UHID_LEGACY_EVENT_TYPE_UHID_FEATURE;
                // SAFETY: event is initialized with zeroed memory and union variant feature is active.
                let payload = unsafe { &mut event.u.feature };
                payload.id = id;
                payload.rnum = report_num;
                payload.rtype = sys::UHID_REPORT_TYPE_UHID_INPUT_REPORT;
            }
            InputEvent::FeatureAnswer { err, id, data, .. } => {
                event.type_ = sys::UHID_LEGACY_EVENT_TYPE_UHID_FEATURE_ANSWER;
                // SAFETY: event is initialized with zeroed memory and union variant feature_answer is active.
                let payload = unsafe { &mut event.u.feature_answer };
                payload.err = err;
                let len = data.len().min(sys::UHID_DATA_MAX);
                payload.data[..len].copy_from_slice(&data[..len]);
                #[allow(clippy::cast_possible_truncation)]
                {
                    payload.size = len as u16;
                }
                payload.id = id;
            }
            InputEvent::SetReport {
                id,
                report_num,
                data,
            } => {
                event.type_ = sys::UHID_EVENT_TYPE_UHID_SET_REPORT;
                // SAFETY: event is initialized with zeroed memory and union variant set_report is active.
                let payload = unsafe { &mut event.u.set_report };
                let len = data.len().min(sys::UHID_DATA_MAX);
                payload.data[..len].copy_from_slice(&data[..len]);
                #[allow(clippy::cast_possible_truncation)]
                {
                    payload.size = len as u16;
                }
                payload.id = id;
                payload.rnum = report_num;
                payload.rtype = sys::UHID_REPORT_TYPE_UHID_INPUT_REPORT;
            }
        }

        event
    }
}

/// Events read from `/dev/uhid` emitted by the Linux kernel.
///
/// See <https://www.kernel.org/doc/html/latest/hid/uhid.html#read>.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputEvent {
    /// Start event indicating device flags.
    Start {
        /// List of active device flags.
        dev_flags: Vec<DevFlags>,
    },
    /// Stop event indicating the kernel has stopped the device.
    Stop,
    /// Open event indicating a user space hidraw client opened the device.
    Open,
    /// Close event indicating the last user space hidraw client closed the device.
    Close,
    /// Output report received from the kernel.
    Output {
        /// Raw output report bytes.
        data: Vec<u8>,
    },
    /// `GetReport` request received from the kernel.
    GetReport {
        /// Request identifier to include in the reply.
        id: u32,
        /// Requested report number.
        report_number: u8,
        /// Requested report type.
        report_type: ReportType,
    },
    /// `SetReport` request received from the kernel.
    SetReport {
        /// Request identifier to include in the reply.
        id: u32,
        /// Report number.
        report_number: u8,
        /// Report type.
        report_type: ReportType,
        /// Report data to set.
        data: Vec<u8>,
    },
}

impl TryFrom<sys::uhid_event> for OutputEvent {
    type Error = StreamError;

    /// Attempts to decode a raw `sys::uhid_event` into an [`OutputEvent`].
    ///
    /// # Errors
    ///
    /// Returns [`StreamError::UnknownEventType`] if the event type or report type is unrecognized.
    fn try_from(event: sys::uhid_event) -> Result<Self, Self::Error> {
        match event.type_ {
            sys::UHID_EVENT_TYPE_UHID_START => {
                // SAFETY: event type is UHID_START, so accessing payload.start is valid.
                let start = unsafe { event.u.start };
                Ok(Self::Start {
                    dev_flags: DevFlags::from_bits_truncate(start.dev_flags),
                })
            }
            sys::UHID_EVENT_TYPE_UHID_STOP => Ok(Self::Stop),
            sys::UHID_EVENT_TYPE_UHID_OPEN => Ok(Self::Open),
            sys::UHID_EVENT_TYPE_UHID_CLOSE => Ok(Self::Close),
            sys::UHID_EVENT_TYPE_UHID_OUTPUT => {
                // SAFETY: event type is UHID_OUTPUT, so accessing payload.output is valid.
                let payload = unsafe { &event.u.output };
                let size = (payload.size as usize).min(sys::UHID_DATA_MAX);
                Ok(Self::Output {
                    data: payload.data[..size].to_vec(),
                })
            }
            sys::UHID_EVENT_TYPE_UHID_GET_REPORT => {
                // SAFETY: event type is UHID_GET_REPORT, so accessing payload.get_report is valid.
                let payload = unsafe { &event.u.get_report };
                Ok(Self::GetReport {
                    id: payload.id,
                    report_number: payload.rnum,
                    report_type: ReportType::try_from(payload.rtype)?,
                })
            }
            sys::UHID_EVENT_TYPE_UHID_SET_REPORT => {
                // SAFETY: event type is UHID_SET_REPORT, so accessing payload.set_report is valid.
                let payload = unsafe { &event.u.set_report };
                let size = (payload.size as usize).min(sys::UHID_DATA_MAX);
                Ok(Self::SetReport {
                    id: payload.id,
                    report_number: payload.rnum,
                    report_type: ReportType::try_from(payload.rtype)?,
                    data: payload.data[..size].to_vec(),
                })
            }
            other => Err(StreamError::UnknownEventType(other)),
        }
    }
}

impl TryFrom<[u8; UHID_EVENT_SIZE]> for OutputEvent {
    type Error = StreamError;

    /// Attempts to decode a raw byte buffer into an [`OutputEvent`].
    ///
    /// # Errors
    ///
    /// Returns [`StreamError::UnknownEventType`] if the event type or report type is unrecognized.
    fn try_from(src: [u8; UHID_EVENT_SIZE]) -> Result<Self, Self::Error> {
        // SAFETY: src is an array of UHID_EVENT_SIZE bytes. uhid_event is a packed C struct
        // composed of primitive types and byte arrays, valid for any byte sequence.
        let event: sys::uhid_event =
            unsafe { std::ptr::read_unaligned(src.as_ptr().cast::<sys::uhid_event>()) };
        Self::try_from(event)
    }
}

impl<'a> From<InputEvent<'a>> for [u8; UHID_EVENT_SIZE] {
    fn from(input: InputEvent<'a>) -> Self {
        let event: sys::uhid_event = input.into();
        // SAFETY: uhid_event has size UHID_EVENT_SIZE and is packed plain data with no padding invariants.
        unsafe { std::mem::transmute_copy(&event) }
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::cast_possible_truncation,
    clippy::wildcard_imports,
    clippy::indexing_slicing,
    clippy::too_many_lines
)]
mod tests {
    use super::*;

    const RDESC: [u8; 85] = [
        0x05, 0x01, /* USAGE_PAGE (Generic Desktop) */
        0x09, 0x02, /* USAGE (Mouse) */
        0xa1, 0x01, /* COLLECTION (Application) */
        0x09, 0x01, /* USAGE (Pointer) */
        0xa1, 0x00, /* COLLECTION (Physical) */
        0x85, 0x01, /* REPORT_ID (1) */
        0x05, 0x09, /* USAGE_PAGE (Button) */
        0x19, 0x01, /* USAGE_MINIMUM (Button 1) */
        0x29, 0x03, /* USAGE_MAXIMUM (Button 3) */
        0x15, 0x00, /* LOGICAL_MINIMUM (0) */
        0x25, 0x01, /* LOGICAL_MAXIMUM (1) */
        0x95, 0x03, /* REPORT_COUNT (3) */
        0x75, 0x01, /* REPORT_SIZE (1) */
        0x81, 0x02, /* INPUT (Data,Var,Abs) */
        0x95, 0x01, /* REPORT_COUNT (1) */
        0x75, 0x05, /* REPORT_SIZE (5) */
        0x81, 0x01, /* INPUT (Cnst,Var,Abs) */
        0x05, 0x01, /* USAGE_PAGE (Generic Desktop) */
        0x09, 0x30, /* USAGE (X) */
        0x09, 0x31, /* USAGE (Y) */
        0x09, 0x38, /* USAGE (WHEEL) */
        0x15, 0x81, /* LOGICAL_MINIMUM (-127) */
        0x25, 0x7f, /* LOGICAL_MAXIMUM (127) */
        0x75, 0x08, /* REPORT_SIZE (8) */
        0x95, 0x03, /* REPORT_COUNT (3) */
        0x81, 0x06, /* INPUT (Data,Var,Rel) */
        0xc0, /* END_COLLECTION */
        0xc0, /* END_COLLECTION */
        0x05, 0x01, /* USAGE_PAGE (Generic Desktop) */
        0x09, 0x06, /* USAGE (Keyboard) */
        0xa1, 0x01, /* COLLECTION (Application) */
        0x85, 0x02, /* REPORT_ID (2) */
        0x05, 0x08, /* USAGE_PAGE (Led) */
        0x19, 0x01, /* USAGE_MINIMUM (1) */
        0x29, 0x03, /* USAGE_MAXIMUM (3) */
        0x15, 0x00, /* LOGICAL_MINIMUM (0) */
        0x25, 0x01, /* LOGICAL_MAXIMUM (1) */
        0x95, 0x03, /* REPORT_COUNT (3) */
        0x75, 0x01, /* REPORT_SIZE (1) */
        0x91, 0x02, /* Output (Data,Var,Abs) */
        0x95, 0x01, /* REPORT_COUNT (1) */
        0x75, 0x05, /* REPORT_SIZE (5) */
        0x91, 0x01, /* Output (Cnst,Var,Abs) */
        0xc0, /* END_COLLECTION */
    ];

    fn assert_bytes_eq(actual: &[u8], expected: &[u8]) {
        assert_eq!(actual.len(), expected.len(), "Size of slices differs");
        for index in 0..actual.len() {
            assert_eq!(
                actual[index], expected[index],
                "Bytes differ at index {index}"
            );
        }
    }

    #[test]
    fn encode_create_request() {
        let mut expected = [0; UHID_EVENT_SIZE];
        expected[0] = 0x0b;
        expected[4] = 0x74;
        expected[5] = 0x65;
        expected[6] = 0x73;
        expected[7] = 0x74;
        expected[8] = 0x2d;
        expected[9] = 0x75;
        expected[10] = 0x68;
        expected[11] = 0x69;
        expected[12] = 0x64;
        expected[13] = 0x2d;
        expected[14] = 0x64;
        expected[15] = 0x65;
        expected[16] = 0x76;
        expected[17] = 0x69;
        expected[18] = 0x63;
        expected[19] = 0x65;
        expected[260] = 0x55;
        expected[262] = 0x03;
        expected[264] = 0xd9;
        expected[265] = 0x15;
        expected[268] = 0x37;
        expected[269] = 0x0a;
        expected[280] = 0x05;
        expected[281] = 0x01;
        expected[282] = 0x09;
        expected[283] = 0x02;
        expected[284] = 0xa1;
        expected[285] = 0x01;
        expected[286] = 0x09;
        expected[287] = 0x01;
        expected[288] = 0xa1;
        expected[290] = 0x85;
        expected[291] = 0x01;
        expected[292] = 0x05;
        expected[293] = 0x09;
        expected[294] = 0x19;
        expected[295] = 0x01;
        expected[296] = 0x29;
        expected[297] = 0x03;
        expected[298] = 0x15;
        expected[300] = 0x25;
        expected[301] = 0x01;
        expected[302] = 0x95;
        expected[303] = 0x03;
        expected[304] = 0x75;
        expected[305] = 0x01;
        expected[306] = 0x81;
        expected[307] = 0x02;
        expected[308] = 0x95;
        expected[309] = 0x01;
        expected[310] = 0x75;
        expected[311] = 0x05;
        expected[312] = 0x81;
        expected[313] = 0x01;
        expected[314] = 0x05;
        expected[315] = 0x01;
        expected[316] = 0x09;
        expected[317] = 0x30;
        expected[318] = 0x09;
        expected[319] = 0x31;
        expected[320] = 0x09;
        expected[321] = 0x38;
        expected[322] = 0x15;
        expected[323] = 0x81;
        expected[324] = 0x25;
        expected[325] = 0x7f;
        expected[326] = 0x75;
        expected[327] = 0x08;
        expected[328] = 0x95;
        expected[329] = 0x03;
        expected[330] = 0x81;
        expected[331] = 0x06;
        expected[332] = 0xc0;
        expected[333] = 0xc0;
        expected[334] = 0x05;
        expected[335] = 0x01;
        expected[336] = 0x09;
        expected[337] = 0x06;
        expected[338] = 0xa1;
        expected[339] = 0x01;
        expected[340] = 0x85;
        expected[341] = 0x02;
        expected[342] = 0x05;
        expected[343] = 0x08;
        expected[344] = 0x19;
        expected[345] = 0x01;
        expected[346] = 0x29;
        expected[347] = 0x03;
        expected[348] = 0x15;
        expected[350] = 0x25;
        expected[351] = 0x01;
        expected[352] = 0x95;
        expected[353] = 0x03;
        expected[354] = 0x75;
        expected[355] = 0x01;
        expected[356] = 0x91;
        expected[357] = 0x02;
        expected[358] = 0x95;
        expected[359] = 0x01;
        expected[360] = 0x75;
        expected[361] = 0x05;
        expected[362] = 0x91;
        expected[363] = 0x01;
        expected[364] = 0xc0;

        let result: [u8; UHID_EVENT_SIZE] = InputEvent::Create(CreateParams {
            name: String::from("test-uhid-device"),
            phys: String::new(),
            uniq: String::new(),
            bus: Bus::USB,
            vendor: 0x15d9,
            product: 0x0a37,
            version: 0,
            country: 0,
            rd_data: RDESC.to_vec(),
        })
        .into();

        assert_bytes_eq(&result[..], &expected);
    }

    #[test]
    fn encode_destroy_request() {
        let mut expected = vec![0; size_of::<sys::uhid_event>()];
        expected[0] = 0x01;

        let result: [u8; UHID_EVENT_SIZE] = InputEvent::Destroy.into();
        assert_bytes_eq(&result[..], &expected);
    }

    #[test]
    fn dev_flags_parsing() {
        let flags = DevFlags::from_bits_truncate(
            sys::UHID_DEV_NUMBERED_FEATURE_REPORTS | sys::UHID_DEV_NUMBERED_INPUT_REPORTS,
        );
        assert_eq!(
            flags,
            vec![
                DevFlags::FeatureReportsNumbered,
                DevFlags::InputReportsNumbered
            ]
        );
        assert_eq!(DevFlags::from_bits_truncate(0), Vec::<DevFlags>::new());
        assert_eq!(
            DevFlags::from_bits_truncate(
                sys::UHID_DEV_NUMBERED_FEATURE_REPORTS
                    | sys::UHID_DEV_NUMBERED_OUTPUT_REPORTS
                    | sys::UHID_DEV_NUMBERED_INPUT_REPORTS
                    | 0xff00
            ),
            vec![
                DevFlags::FeatureReportsNumbered,
                DevFlags::OutputReportsNumbered,
                DevFlags::InputReportsNumbered
            ]
        );
    }

    #[test]
    fn dev_flags_bits() {
        assert_eq!(
            DevFlags::FeatureReportsNumbered.bits(),
            sys::UHID_DEV_NUMBERED_FEATURE_REPORTS
        );
        assert_eq!(
            DevFlags::OutputReportsNumbered.bits(),
            sys::UHID_DEV_NUMBERED_OUTPUT_REPORTS
        );
        assert_eq!(
            DevFlags::InputReportsNumbered.bits(),
            sys::UHID_DEV_NUMBERED_INPUT_REPORTS
        );
    }

    #[test]
    fn report_type_conversions() {
        assert_eq!(ReportType::try_from(0).unwrap(), ReportType::Feature);
        assert_eq!(ReportType::try_from(1).unwrap(), ReportType::Output);
        assert_eq!(ReportType::try_from(2).unwrap(), ReportType::Input);
        assert!(ReportType::try_from(99).is_err());
    }

    #[test]
    fn encode_input_event() {
        let data = [0x01, 0x02, 0x03, 0x04];
        let event: sys::uhid_event = InputEvent::Input { data: &data }.into();
        assert_eq!({ event.type_ }, sys::UHID_EVENT_TYPE_UHID_INPUT2);
        // SAFETY: union variant input2 is initialized for UHID_INPUT2
        let payload = unsafe { event.u.input2 };
        assert_eq!({ payload.size }, 4);
        let payload_data = payload.data;
        assert_eq!(&payload_data[..4], &data);
    }

    #[test]
    fn encode_output_event() {
        let data = vec![0x10, 0x20];
        let event: sys::uhid_event = InputEvent::Output { data: data.clone() }.into();
        assert_eq!({ event.type_ }, sys::UHID_EVENT_TYPE_UHID_OUTPUT);
        // SAFETY: union variant output is initialized for UHID_OUTPUT
        let payload = unsafe { event.u.output };
        assert_eq!({ payload.size }, 2);
        assert_eq!({ payload.rtype }, sys::UHID_REPORT_TYPE_UHID_OUTPUT_REPORT);
        let payload_data = payload.data;
        assert_eq!(&payload_data[..2], &data[..]);
    }

    #[test]
    fn encode_output_ev_event() {
        let event: sys::uhid_event = InputEvent::OutputEv {
            type_: 1,
            code: 2,
            value: -3,
        }
        .into();
        assert_eq!({ event.type_ }, sys::UHID_LEGACY_EVENT_TYPE_UHID_OUTPUT_EV);
        // SAFETY: union variant output_ev is initialized for legacy UHID_OUTPUT_EV
        let payload = unsafe { event.u.output_ev };
        assert_eq!({ payload.type_ }, 1);
        assert_eq!({ payload.code }, 2);
        assert_eq!({ payload.value }, -3);
    }

    #[test]
    fn encode_get_report_reply_event() {
        let data = vec![0xaa, 0xbb];
        let event: sys::uhid_event = InputEvent::GetReportReply {
            id: 7,
            err: 0,
            data: data.clone(),
        }
        .into();
        assert_eq!({ event.type_ }, sys::UHID_EVENT_TYPE_UHID_GET_REPORT_REPLY);
        // SAFETY: union variant get_report_reply is initialized for UHID_GET_REPORT_REPLY
        let payload = unsafe { event.u.get_report_reply };
        assert_eq!({ payload.id }, 7);
        assert_eq!({ payload.err }, 0);
        assert_eq!({ payload.size }, 2);
        let payload_data = payload.data;
        assert_eq!(&payload_data[..2], &data[..]);
    }

    #[test]
    fn encode_set_report_reply_event() {
        let event: sys::uhid_event = InputEvent::SetReportReply { id: 8, err: 1 }.into();
        assert_eq!({ event.type_ }, sys::UHID_EVENT_TYPE_UHID_SET_REPORT_REPLY);
        // SAFETY: union variant set_report_reply is initialized for UHID_SET_REPORT_REPLY
        let payload = unsafe { event.u.set_report_reply };
        assert_eq!({ payload.id }, 8);
        assert_eq!({ payload.err }, 1);
    }

    #[test]
    fn encode_feature_and_feature_answer_event() {
        let event: sys::uhid_event = InputEvent::Feature {
            id: 9,
            report_num: 3,
        }
        .into();
        assert_eq!({ event.type_ }, sys::UHID_LEGACY_EVENT_TYPE_UHID_FEATURE);
        // SAFETY: union variant feature is initialized for legacy UHID_FEATURE
        let payload = unsafe { event.u.feature };
        assert_eq!({ payload.id }, 9);
        assert_eq!({ payload.rnum }, 3);
        assert_eq!({ payload.rtype }, sys::UHID_REPORT_TYPE_UHID_INPUT_REPORT);

        let data = vec![0x55];
        let answer: sys::uhid_event = InputEvent::FeatureAnswer {
            id: 10,
            err: 0,
            data: data.clone(),
        }
        .into();
        assert_eq!(
            { answer.type_ },
            sys::UHID_LEGACY_EVENT_TYPE_UHID_FEATURE_ANSWER
        );
        // SAFETY: union variant feature_answer is initialized for legacy UHID_FEATURE_ANSWER
        let payload = unsafe { answer.u.feature_answer };
        assert_eq!({ payload.id }, 10);
        assert_eq!({ payload.err }, 0);
        assert_eq!({ payload.size }, 1);
        let payload_data = payload.data;
        assert_eq!(&payload_data[..1], &data[..]);
    }

    #[test]
    fn encode_set_report_event() {
        let data = vec![0x11, 0x22, 0x33];
        let event: sys::uhid_event = InputEvent::SetReport {
            id: 12,
            report_num: 5,
            data: data.clone(),
        }
        .into();
        assert_eq!({ event.type_ }, sys::UHID_EVENT_TYPE_UHID_SET_REPORT);
        // SAFETY: union variant set_report is initialized for UHID_SET_REPORT
        let payload = unsafe { event.u.set_report };
        assert_eq!({ payload.id }, 12);
        assert_eq!({ payload.rnum }, 5);
        assert_eq!({ payload.rtype }, sys::UHID_REPORT_TYPE_UHID_INPUT_REPORT);
        assert_eq!({ payload.size }, 3);
        let payload_data = payload.data;
        assert_eq!(&payload_data[..3], &data[..]);
    }

    #[test]
    fn decode_lifecycle_output_events() {
        let stop_event = sys::uhid_event {
            type_: sys::UHID_EVENT_TYPE_UHID_STOP,
            ..Default::default()
        };
        assert_eq!(
            OutputEvent::try_from(stop_event).unwrap(),
            OutputEvent::Stop
        );

        let open_event = sys::uhid_event {
            type_: sys::UHID_EVENT_TYPE_UHID_OPEN,
            ..Default::default()
        };
        assert_eq!(
            OutputEvent::try_from(open_event).unwrap(),
            OutputEvent::Open
        );

        let close_event = sys::uhid_event {
            type_: sys::UHID_EVENT_TYPE_UHID_CLOSE,
            ..Default::default()
        };
        assert_eq!(
            OutputEvent::try_from(close_event).unwrap(),
            OutputEvent::Close
        );
    }

    #[test]
    fn decode_output_and_report_events() {
        let mut raw_output = sys::uhid_event {
            type_: sys::UHID_EVENT_TYPE_UHID_OUTPUT,
            ..Default::default()
        };
        let mut output_payload = sys::uhid_output_req {
            data: [0; sys::UHID_DATA_MAX],
            size: 3,
            rtype: sys::UHID_REPORT_TYPE_UHID_OUTPUT_REPORT,
        };
        output_payload.data[..3].copy_from_slice(&[0x01, 0x02, 0x03]);
        raw_output.u.output = output_payload;

        let decoded = OutputEvent::try_from(raw_output).unwrap();
        assert_eq!(
            decoded,
            OutputEvent::Output {
                data: vec![0x01, 0x02, 0x03]
            }
        );

        let mut get_report_event = sys::uhid_event {
            type_: sys::UHID_EVENT_TYPE_UHID_GET_REPORT,
            ..Default::default()
        };
        get_report_event.u.get_report = sys::uhid_get_report_req {
            id: 15,
            rnum: 2,
            rtype: sys::UHID_REPORT_TYPE_UHID_FEATURE_REPORT,
        };
        let decoded_get = OutputEvent::try_from(get_report_event).unwrap();
        assert_eq!(
            decoded_get,
            OutputEvent::GetReport {
                id: 15,
                report_number: 2,
                report_type: ReportType::Feature,
            }
        );

        let mut set_report_event = sys::uhid_event {
            type_: sys::UHID_EVENT_TYPE_UHID_SET_REPORT,
            ..Default::default()
        };
        let mut set_payload = sys::uhid_set_report_req {
            id: 20,
            rnum: 4,
            rtype: sys::UHID_REPORT_TYPE_UHID_OUTPUT_REPORT,
            size: 2,
            data: [0; sys::UHID_DATA_MAX],
        };
        set_payload.data[..2].copy_from_slice(&[0xaa, 0x55]);
        set_report_event.u.set_report = set_payload;

        let decoded_set = OutputEvent::try_from(set_report_event).unwrap();
        assert_eq!(
            decoded_set,
            OutputEvent::SetReport {
                id: 20,
                report_number: 4,
                report_type: ReportType::Output,
                data: vec![0xaa, 0x55],
            }
        );
    }

    #[test]
    fn decode_unknown_events_and_invalid_reports() {
        let unknown = sys::uhid_event {
            type_: 999,
            ..Default::default()
        };
        assert!(matches!(
            OutputEvent::try_from(unknown),
            Err(StreamError::UnknownEventType(999))
        ));

        let mut invalid_report = sys::uhid_event {
            type_: sys::UHID_EVENT_TYPE_UHID_GET_REPORT,
            ..Default::default()
        };
        invalid_report.u.get_report = sys::uhid_get_report_req {
            id: 1,
            rnum: 1,
            rtype: 99,
        };
        assert!(matches!(
            OutputEvent::try_from(invalid_report),
            Err(StreamError::UnknownEventType(99))
        ));
    }

    #[test]
    fn stream_error_display_and_trait() {
        let err_unknown = StreamError::UnknownEventType(42);
        assert_eq!(format!("{err_unknown}"), "Unknown UHID event type: 42");
        assert!(std::error::Error::source(&err_unknown).is_none());

        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "permission denied");
        let err_io = StreamError::Io(io_err);
        assert!(format!("{err_io}").contains("UHID I/O error"));
        assert!(std::error::Error::source(&err_io).is_some());
    }
}
