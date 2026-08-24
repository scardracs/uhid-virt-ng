//! Interface to Linux UHID (user-space HID transport drivers).
//!
//! This crate provides low-level Linux UHID bindings and high-level safe abstractions
//! for creating and interacting with virtual HID devices through `/dev/uhid`.

pub mod codec;
pub mod sys;
pub mod uhid_device;

pub use codec::{Bus, DevFlags, InputEvent, OutputEvent, ReportType, StreamError, UHID_EVENT_SIZE};
pub use uhid_device::{CreateParams, UHIDDevice};
