//! Low-level FFI definitions matching the Linux Kernel UHID interface (`<linux/uhid.h>`).
#![allow(non_camel_case_types, non_upper_case_globals)]

/// Maximum size of HID report descriptor supported by the Linux kernel HID subsystem (4096 bytes).
pub const HID_MAX_DESCRIPTOR_SIZE: usize = 4096;

/// Maximum data payload size for UHID input/output reports (4096 bytes).
pub const UHID_DATA_MAX: usize = 4096;

/// Raw UHID event type identifier.
pub type UhidEventType = u32;

/// Raw UHID report type identifier.
pub type UhidReportType = u8;

/// Destroy virtual HID device event type.
pub const UHID_EVENT_TYPE_UHID_DESTROY: u32 = 1;
/// Start event received when the kernel opens the device.
pub const UHID_EVENT_TYPE_UHID_START: u32 = 2;
/// Stop event received when the kernel stops the device.
pub const UHID_EVENT_TYPE_UHID_STOP: u32 = 3;
/// Open event received when first user space client opens hidraw.
pub const UHID_EVENT_TYPE_UHID_OPEN: u32 = 4;
/// Close event received when last user space client closes hidraw.
pub const UHID_EVENT_TYPE_UHID_CLOSE: u32 = 5;
/// Output event carrying an output report sent from the kernel to the device.
pub const UHID_EVENT_TYPE_UHID_OUTPUT: u32 = 6;
/// Legacy output event carrying input events.
pub const UHID_EVENT_TYPE_UHID_OUTPUT_EV: u32 = 7;
/// Request sent from the kernel to get a report.
pub const UHID_EVENT_TYPE_UHID_GET_REPORT: u32 = 9;
/// Response sent from user space for a `GET_REPORT` request.
pub const UHID_EVENT_TYPE_UHID_GET_REPORT_REPLY: u32 = 10;
/// Event sent from user space to create a virtual device (v2).
pub const UHID_EVENT_TYPE_UHID_CREATE2: u32 = 11;
/// Input event carrying an input report from user space to the kernel (v2).
pub const UHID_EVENT_TYPE_UHID_INPUT2: u32 = 12;
/// Request sent from the kernel to set a report on the device.
pub const UHID_EVENT_TYPE_UHID_SET_REPORT: u32 = 13;
/// Response sent from user space for a `SET_REPORT` request.
pub const UHID_EVENT_TYPE_UHID_SET_REPORT_REPLY: u32 = 14;

/// Legacy feature report request event type.
pub const UHID_LEGACY_EVENT_TYPE_UHID_FEATURE: u32 = 9;
/// Legacy feature answer event type.
pub const UHID_LEGACY_EVENT_TYPE_UHID_FEATURE_ANSWER: u32 = 10;
/// Legacy output event type.
pub const UHID_LEGACY_EVENT_TYPE_UHID_OUTPUT_EV: u32 = 7;

/// Feature report type.
pub const UHID_REPORT_TYPE_UHID_FEATURE_REPORT: u8 = 0;
/// Output report type.
pub const UHID_REPORT_TYPE_UHID_OUTPUT_REPORT: u8 = 1;
/// Input report type.
pub const UHID_REPORT_TYPE_UHID_INPUT_REPORT: u8 = 2;

/// Device flag indicating feature reports use report IDs.
pub const UHID_DEV_NUMBERED_FEATURE_REPORTS: u64 = 1 << 0;
/// Device flag indicating output reports use report IDs.
pub const UHID_DEV_NUMBERED_OUTPUT_REPORTS: u64 = 1 << 1;
/// Device flag indicating input reports use report IDs.
pub const UHID_DEV_NUMBERED_INPUT_REPORTS: u64 = 1 << 2;

/// Linux open flag for read/write access (`O_RDWR`).
pub const O_RDWR: i32 = 0o2;

/// Linux open flag for non-blocking I/O (`O_NONBLOCK`).
#[cfg(any(target_arch = "sparc", target_arch = "sparc64"))]
pub const O_NONBLOCK: i32 = 0x4000;
/// Linux open flag for non-blocking I/O (`O_NONBLOCK`).
#[cfg(any(
    target_arch = "mips",
    target_arch = "mips64",
    target_arch = "mips32r6",
    target_arch = "mips64r6"
))]
pub const O_NONBLOCK: i32 = 0x80;
/// Linux open flag for non-blocking I/O (`O_NONBLOCK`).
#[cfg(not(any(
    target_arch = "sparc",
    target_arch = "sparc64",
    target_arch = "mips",
    target_arch = "mips64",
    target_arch = "mips32r6",
    target_arch = "mips64r6"
)))]
pub const O_NONBLOCK: i32 = 0o4000;

/// Linux open flag to set close-on-exec (`O_CLOEXEC`).
#[cfg(any(target_arch = "sparc", target_arch = "sparc64"))]
pub const O_CLOEXEC: i32 = 0x400000;
/// Linux open flag to set close-on-exec (`O_CLOEXEC`).
#[cfg(not(any(target_arch = "sparc", target_arch = "sparc64")))]
pub const O_CLOEXEC: i32 = 0o2_000_000;

/// Payload for `UHID_CREATE2` event to register a new virtual device.
#[repr(C, packed)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct uhid_create2_req {
    /// Device name (null-terminated or zero-padded UTF-8 string).
    pub name: [u8; 128],
    /// Physical device path (null-terminated or zero-padded string).
    pub phys: [u8; 64],
    /// Unique identifier for the device (null-terminated or zero-padded string).
    pub uniq: [u8; 64],
    /// Size of the report descriptor in bytes.
    pub rd_size: u16,
    /// Bus type identifier (e.g. USB = 3, Bluetooth = 5).
    pub bus: u16,
    /// Vendor identifier.
    pub vendor: u32,
    /// Product identifier.
    pub product: u32,
    /// Version number.
    pub version: u32,
    /// Country code.
    pub country: u32,
    /// Report descriptor raw bytes.
    pub rd_data: [u8; HID_MAX_DESCRIPTOR_SIZE],
}

/// Payload for `UHID_START` event containing device flags.
#[repr(C, packed)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct uhid_start_req {
    /// Bitmask of device flags (`UHID_DEV_NUMBERED_*`).
    pub dev_flags: u64,
}

/// Payload for `UHID_INPUT2` event to deliver input reports to the kernel.
#[repr(C, packed)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct uhid_input2_req {
    /// Size of the data payload in bytes.
    pub size: u16,
    /// Raw input report data.
    pub data: [u8; UHID_DATA_MAX],
}

/// Payload for `UHID_OUTPUT` event carrying output reports from the kernel.
#[repr(C, packed)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct uhid_output_req {
    /// Raw output report data buffer.
    pub data: [u8; UHID_DATA_MAX],
    /// Size of the valid output data in bytes.
    pub size: u16,
    /// Report type (`UHID_OUTPUT_REPORT`).
    pub rtype: u8,
}

/// Payload for legacy `UHID_OUTPUT_EV` event.
#[repr(C, packed)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct uhid_output_ev_req {
    /// Event type (Linux input subsystem event type).
    pub type_: u16,
    /// Event code.
    pub code: u16,
    /// Event value.
    pub value: i32,
}

/// Payload for `UHID_GET_REPORT` event requesting a report from user space.
#[repr(C, packed)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct uhid_get_report_req {
    /// Unique request ID.
    pub id: u32,
    /// Report number requested.
    pub rnum: u8,
    /// Report type requested.
    pub rtype: u8,
}

/// Payload for `UHID_GET_REPORT_REPLY` response sent back to the kernel.
#[repr(C, packed)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct uhid_get_report_reply_req {
    /// Request ID matching the `GET_REPORT` request.
    pub id: u32,
    /// Error code (0 on success, or standard errno on failure).
    pub err: u16,
    /// Size of the returned data.
    pub size: u16,
    /// Returned report data.
    pub data: [u8; UHID_DATA_MAX],
}

/// Payload for `UHID_SET_REPORT` event instructing user space to set a report on the device.
#[repr(C, packed)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct uhid_set_report_req {
    /// Unique request ID.
    pub id: u32,
    /// Report number.
    pub rnum: u8,
    /// Report type.
    pub rtype: u8,
    /// Size of the data payload.
    pub size: u16,
    /// Data payload to set.
    pub data: [u8; UHID_DATA_MAX],
}

/// Payload for `UHID_SET_REPORT_REPLY` response sent back to the kernel.
#[repr(C, packed)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct uhid_set_report_reply_req {
    /// Request ID matching the `SET_REPORT` request.
    pub id: u32,
    /// Error code (0 on success, or standard errno on failure).
    pub err: u16,
}

/// Payload for legacy `UHID_FEATURE` request event.
#[repr(C, packed)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct uhid_feature_req {
    /// Unique request ID.
    pub id: u32,
    /// Report number.
    pub rnum: u8,
    /// Report type.
    pub rtype: u8,
}

/// Payload for legacy `UHID_FEATURE_ANSWER` reply event.
#[repr(C, packed)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct uhid_feature_answer_req {
    /// Request ID matching the `FEATURE` request.
    pub id: u32,
    /// Error code (0 on success, or standard errno on failure).
    pub err: u16,
    /// Size of the data payload.
    pub size: u16,
    /// Feature report data payload.
    pub data: [u8; UHID_DATA_MAX],
}

/// Union of all possible UHID event payloads.
#[repr(C)]
#[derive(Copy, Clone)]
pub union uhid_event_payload {
    /// Output report payload.
    pub output: uhid_output_req,
    /// Legacy output event payload.
    pub output_ev: uhid_output_ev_req,
    /// Legacy feature request payload.
    pub feature: uhid_feature_req,
    /// Get report request payload.
    pub get_report: uhid_get_report_req,
    /// Legacy feature reply payload.
    pub feature_answer: uhid_feature_answer_req,
    /// Get report reply payload.
    pub get_report_reply: uhid_get_report_reply_req,
    /// Create virtual device payload (v2).
    pub create2: uhid_create2_req,
    /// Input report payload (v2).
    pub input2: uhid_input2_req,
    /// Set report request payload.
    pub set_report: uhid_set_report_req,
    /// Set report reply payload.
    pub set_report_reply: uhid_set_report_reply_req,
    /// Start device payload.
    pub start: uhid_start_req,
}

/// Main C ABI event structure transferred to and from `/dev/uhid`.
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct uhid_event {
    /// Event type identifier (one of `UHID_EVENT_TYPE_*`).
    pub type_: u32,
    /// Variant payload depending on `type_`.
    pub u: uhid_event_payload,
}

impl Default for uhid_event {
    fn default() -> Self {
        // SAFETY: uhid_event consists of primitive integers and byte buffers where all-zeros is a valid representation.
        unsafe { std::mem::zeroed() }
    }
}
