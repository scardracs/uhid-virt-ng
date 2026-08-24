# uhid-virt-ng

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![MSRV: 1.93](https://img.shields.io/badge/MSRV-1.93-orange.svg)](https://blog.rust-lang.org)
[![Rust Edition: 2024](https://img.shields.io/badge/Rust-Edition%202024-purple.svg)](https://doc.rust-lang.org/edition-guide/rust-2024/)

Safe, zero-dependency Rust interface to Linux UHID (user-space HID transport drivers) through `/dev/uhid`.

---

## Key Features

- **Zero External Dependencies**: Pure Rust implementation with self-contained Linux kernel ABI definitions (`<linux/uhid.h>`).
- **Rust Edition 2024 & MSRV 1.93**: Idiomatic modern Rust, safe Unix file descriptor integration (`std::os::fd::{AsFd, AsRawFd}`).
- **Memory-Safe & Sound**: Eliminates unaligned references, bounds-checks all raw kernel buffers, and enforces strict linter compliance.
- **Fast & Minimalist**: Zero proc-macro compile overhead and optimized memory operations.
- **Full Protocol Coverage**: Implements all UHID commands (`UHID_CREATE2`, `UHID_INPUT2`, `UHID_OUTPUT`, `UHID_GET_REPORT`, `UHID_SET_REPORT`, etc.).

---

## What is UHID?

Linux UHID allows user-space processes to implement HID transport drivers. Without writing a kernel module, you can register virtual HID devices directly with the kernel HID subsystem.

Typical use cases:
- Emulating mice, keyboards, gamepads, or custom touchpads in user space.
- Forwarding input events across software daemons (e.g. system control daemons).
- Reverse engineering and implementing drivers for hardware with non-standard protocols.
- Automated testing and hardware simulation in CI/CD environments.

For kernel details, see the [Linux Kernel UHID documentation](https://www.kernel.org/doc/html/latest/hid/uhid.html).

---

## Quick Start

Add `uhid-virt-ng` to your `Cargo.toml`:

```toml
[dependencies]
uhid-virt-ng = "0.0.1"
```

### Example: Virtual Mouse

```rust
use std::error::Error;
use std::io;
use uhid_virt_ng::{Bus, CreateParams, UHIDDevice};

// Minimal HID Mouse Report Descriptor (Report ID 1: Mouse buttons, X, Y, Wheel)
const RDESC: [u8; 85] = [
    0x05, 0x01, 0x09, 0x02, 0xa1, 0x01, 0x09, 0x01, 0xa1, 0x00,
    0x85, 0x01, 0x05, 0x09, 0x19, 0x01, 0x29, 0x03, 0x15, 0x00,
    0x25, 0x01, 0x95, 0x03, 0x75, 0x01, 0x81, 0x02, 0x95, 0x01,
    0x75, 0x05, 0x81, 0x01, 0x05, 0x01, 0x09, 0x30, 0x09, 0x31,
    0x09, 0x38, 0x15, 0x81, 0x25, 0x7f, 0x75, 0x08, 0x95, 0x03,
    0x81, 0x06, 0xc0, 0xc0, 0x05, 0x01, 0x09, 0x06, 0xa1, 0x01,
    0x85, 0x02, 0x05, 0x08, 0x19, 0x01, 0x29, 0x03, 0x15, 0x00,
    0x25, 0x01, 0x95, 0x03, 0x75, 0x01, 0x91, 0x02, 0x95, 0x01,
    0x75, 0x05, 0x91, 0x01, 0xc0,
];

fn main() -> Result<(), Box<dyn Error>> {
    let params = CreateParams {
        name: "virtual-uhid-mouse".to_string(),
        phys: String::new(),
        uniq: String::new(),
        bus: Bus::USB,
        vendor: 0x15d9,
        product: 0x0a37,
        version: 0,
        country: 0,
        rd_data: RDESC.to_vec(),
    };

    // Open /dev/uhid and register the virtual device
    let mut dev = UHIDDevice::create(params)?;

    // Report: [Report ID (1), Buttons, X-delta (+20), Y-delta (0), Wheel (0)]
    let report: [u8; 5] = [1, 0, 20, 0, 0];

    // Write input event to kernel
    dev.write(&report)?;

    Ok(())
}
```

> **Note:** Accessing `/dev/uhid` typically requires root privileges or appropriate udev permissions (e.g. membership in the `input` group or custom udev rule).

---

## Running Tests

```bash
# Run unit test suite
cargo test --all

# Run strict clippy verification
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Check code formatting
cargo fmt --all -- --check
```

---

## License

Distributed under the [MIT License](LICENSE).
