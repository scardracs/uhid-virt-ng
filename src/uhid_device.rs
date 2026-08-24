//! Higher-level safe wrapper around `/dev/uhid`.

use std::fs::{File, OpenOptions};
use std::io::{self, prelude::*};
use std::os::fd::{AsFd, AsRawFd, BorrowedFd, RawFd};
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

use crate::codec::{Bus, InputEvent, OutputEvent, StreamError, UHID_EVENT_SIZE};

/// Character misc-device handle for interacting with a virtual UHID device.
pub struct UHIDDevice<T: Read + Write> {
    handle: T,
}

/// Contains configuration and report descriptor information used when creating a `UHIDDevice`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateParams {
    /// Device name reported to the kernel and user space.
    pub name: String,
    /// Physical device path (can be empty).
    pub phys: String,
    /// Unique identifier for the device (can be empty).
    pub uniq: String,
    /// Bus type (e.g. `Bus::USB`, `Bus::BLUETOOTH`, `Bus::VIRTUAL`).
    pub bus: Bus,
    /// Vendor ID.
    pub vendor: u32,
    /// Product ID.
    pub product: u32,
    /// Version number.
    pub version: u32,
    /// Country code (usually 0).
    pub country: u32,
    /// Raw HID report descriptor bytes.
    pub rd_data: Vec<u8>,
}

impl<T: Read + Write> UHIDDevice<T> {
    /// Creates a new `UHIDDevice` wrapping the given I/O handle.
    #[must_use]
    pub const fn new(handle: T) -> Self {
        Self { handle }
    }

    /// Consumes the `UHIDDevice`, returning the underlying I/O handle.
    #[must_use]
    pub fn into_inner(self) -> T {
        self.handle
    }

    /// Gets a reference to the underlying I/O handle.
    #[must_use]
    pub const fn get_ref(&self) -> &T {
        &self.handle
    }

    /// Gets a mutable reference to the underlying I/O handle.
    pub const fn get_mut(&mut self) -> &mut T {
        &mut self.handle
    }

    /// Writes a raw input event data payload to the device.
    ///
    /// # Errors
    ///
    /// Returns an [`io::Error`] if writing to the underlying handle fails.
    pub fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        let event: [u8; UHID_EVENT_SIZE] = InputEvent::Input { data }.into();
        self.handle.write_all(&event)?;
        Ok(UHID_EVENT_SIZE)
    }

    /// Writes a `SetReportReply` event in response to a `SetReport` output event from the kernel.
    ///
    /// # Errors
    ///
    /// Returns an [`io::Error`] if writing to the underlying handle fails.
    pub fn write_set_report_reply(&mut self, id: u32, err: u16) -> io::Result<usize> {
        let event: [u8; UHID_EVENT_SIZE] = InputEvent::SetReportReply { id, err }.into();
        self.handle.write_all(&event)?;
        Ok(UHID_EVENT_SIZE)
    }

    /// Writes a `GetReportReply` event with data in response to a `GetReport` output event from the kernel.
    ///
    /// # Errors
    ///
    /// Returns an [`io::Error`] if writing to the underlying handle fails.
    pub fn write_get_report_reply(
        &mut self,
        id: u32,
        err: u16,
        data: Vec<u8>,
    ) -> io::Result<usize> {
        let event: [u8; UHID_EVENT_SIZE] = InputEvent::GetReportReply { id, err, data }.into();
        self.handle.write_all(&event)?;
        Ok(UHID_EVENT_SIZE)
    }

    /// Reads a queued output event from the kernel.
    ///
    /// # Errors
    ///
    /// Returns a [`StreamError`] if reading fails or if the kernel delivers an invalid/unrecognized event.
    pub fn read(&mut self) -> Result<OutputEvent, StreamError> {
        let mut event = [0u8; UHID_EVENT_SIZE];
        self.handle
            .read_exact(&mut event)
            .map_err(StreamError::Io)?;
        OutputEvent::try_from(event)
    }

    /// Destroys the virtual HID device in the kernel.
    ///
    /// # Errors
    ///
    /// Returns an [`io::Error`] if writing the destroy command fails.
    pub fn destroy(&mut self) -> io::Result<usize> {
        let event: [u8; UHID_EVENT_SIZE] = InputEvent::Destroy.into();
        self.handle.write_all(&event)?;
        Ok(UHID_EVENT_SIZE)
    }
}

impl UHIDDevice<File> {
    /// Opens the character misc-device at `/dev/uhid` and creates the virtual device.
    ///
    /// # Errors
    ///
    /// Returns an [`io::Error`] if opening `/dev/uhid` or writing the `Create` event fails.
    pub fn create(params: CreateParams) -> io::Result<Self> {
        Self::create_with_path(params, Path::new("/dev/uhid"))
    }

    /// Opens the character device at the given path and creates the virtual device.
    ///
    /// # Errors
    ///
    /// Returns an [`io::Error`] if opening the path or writing the `Create` event fails.
    pub fn create_with_path(params: CreateParams, path: &Path) -> io::Result<Self> {
        let mut options = OpenOptions::new();
        options.read(true);
        options.write(true);
        if cfg!(unix) {
            options
                .custom_flags(crate::sys::O_RDWR | crate::sys::O_CLOEXEC | crate::sys::O_NONBLOCK);
        }
        let mut handle = options.open(path)?;
        let event: [u8; UHID_EVENT_SIZE] = InputEvent::Create(params).into();
        handle.write_all(&event)?;
        Ok(Self { handle })
    }
}

impl<T: Read + Write + AsFd> AsFd for UHIDDevice<T> {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.handle.as_fd()
    }
}

impl<T: Read + Write + AsRawFd> AsRawFd for UHIDDevice<T> {
    fn as_raw_fd(&self) -> RawFd {
        self.handle.as_raw_fd()
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::wildcard_imports,
    clippy::indexing_slicing
)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[derive(Default)]
    struct MockStream {
        read_buf: Cursor<Vec<u8>>,
        write_buf: Vec<u8>,
    }

    impl MockStream {
        fn with_read_data(data: Vec<u8>) -> Self {
            Self {
                read_buf: Cursor::new(data),
                write_buf: Vec::new(),
            }
        }
    }

    impl Read for MockStream {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            self.read_buf.read(buf)
        }
    }

    impl Write for MockStream {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.write_buf.extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn device_new_and_accessors() {
        let stream = MockStream::default();
        let mut device = UHIDDevice::new(stream);

        assert_eq!(device.get_ref().write_buf.len(), 0);
        device.get_mut().write_buf.push(42);
        assert_eq!(device.get_ref().write_buf, vec![42]);

        let inner = device.into_inner();
        assert_eq!(inner.write_buf, vec![42]);
    }

    #[test]
    fn device_write_input_event() {
        let stream = MockStream::default();
        let mut device = UHIDDevice::new(stream);

        let data = [0x01, 0x02, 0x03];
        let bytes_written = device.write(&data).expect("write failed");
        assert_eq!(bytes_written, UHID_EVENT_SIZE);

        let written = &device.get_ref().write_buf;
        assert_eq!(written.len(), UHID_EVENT_SIZE);

        let expected: [u8; UHID_EVENT_SIZE] = InputEvent::Input { data: &data }.into();
        assert_eq!(written.as_slice(), &expected[..]);
    }

    #[test]
    fn device_write_set_report_reply() {
        let stream = MockStream::default();
        let mut device = UHIDDevice::new(stream);

        let bytes_written = device
            .write_set_report_reply(42, 0)
            .expect("write_set_report_reply failed");
        assert_eq!(bytes_written, UHID_EVENT_SIZE);

        let written = &device.get_ref().write_buf;
        let expected: [u8; UHID_EVENT_SIZE] = InputEvent::SetReportReply { id: 42, err: 0 }.into();
        assert_eq!(written.as_slice(), &expected[..]);
    }

    #[test]
    fn device_write_get_report_reply() {
        let stream = MockStream::default();
        let mut device = UHIDDevice::new(stream);

        let reply_data = vec![0xaa, 0xbb, 0xcc];
        let bytes_written = device
            .write_get_report_reply(10, 0, reply_data.clone())
            .expect("write_get_report_reply failed");
        assert_eq!(bytes_written, UHID_EVENT_SIZE);

        let written = &device.get_ref().write_buf;
        let expected: [u8; UHID_EVENT_SIZE] = InputEvent::GetReportReply {
            id: 10,
            err: 0,
            data: reply_data,
        }
        .into();
        assert_eq!(written.as_slice(), &expected[..]);
    }

    #[test]
    fn device_destroy() {
        let stream = MockStream::default();
        let mut device = UHIDDevice::new(stream);

        let bytes_written = device.destroy().expect("destroy failed");
        assert_eq!(bytes_written, UHID_EVENT_SIZE);

        let written = &device.get_ref().write_buf;
        let expected: [u8; UHID_EVENT_SIZE] = InputEvent::Destroy.into();
        assert_eq!(written.as_slice(), &expected[..]);
    }

    #[test]
    fn device_read_output_event() {
        let mut raw_event = [0u8; UHID_EVENT_SIZE];
        // UHID_START = 2
        raw_event[0] = 2;

        let stream = MockStream::with_read_data(raw_event.to_vec());
        let mut device = UHIDDevice::new(stream);

        let event = device.read().expect("read failed");
        assert_eq!(event, OutputEvent::Start { dev_flags: vec![] });
    }

    #[test]
    fn device_read_eof_error() {
        let stream = MockStream::with_read_data(vec![0u8; 10]); // Truncated data
        let mut device = UHIDDevice::new(stream);

        let err = device.read().unwrap_err();
        match err {
            StreamError::Io(io_err) => {
                assert_eq!(io_err.kind(), io::ErrorKind::UnexpectedEof);
            }
            StreamError::UnknownEventType(_) => panic!("Expected IO UnexpectedEof error"),
        }
    }
}
