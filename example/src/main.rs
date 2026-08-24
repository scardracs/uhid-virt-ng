use std::error::Error;
use std::io::{self, Write};
use uhid_virt_ng::{Bus, CreateParams, UHIDDevice};

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

fn main() -> Result<(), Box<dyn Error>> {
    let rd_data = RDESC.to_vec();
    let create_params = CreateParams {
        name: String::from("test-uhid-device"),
        phys: String::new(),
        uniq: String::new(),
        bus: Bus::USB,
        vendor: 0x15d9,
        product: 0x0a37,
        version: 0,
        country: 0,
        rd_data,
    };

    let mut uhid_device = UHIDDevice::create(create_params)?;
    println!(">>> Virtual UHID device registered successfully with Linux kernel!");
    println!(">>> Press [ENTER] to move mouse right by +50 pixels (or type 'q' + ENTER to exit)");

    let button_flags = 0;
    let mouse_rel_x: u8 = 50;
    let mouse_rel_y: u8 = 0;
    let wheel: u8 = 0;
    // Format: [Report ID (1), Buttons (0), Rel X (+50), Rel Y (0), Wheel (0)]
    let report: [u8; 5] = [1, button_flags, mouse_rel_x, mouse_rel_y, wheel];

    let mut input = String::new();
    loop {
        print!("> Press ENTER to move mouse (+50px) ... ");
        io::stdout().flush()?;
        input.clear();
        io::stdin().read_line(&mut input)?;

        if input.trim() == "q" {
            println!("Destroying UHID device and exiting...");
            uhid_device.destroy()?;
            break;
        }

        let bytes_written = uhid_device.write(&report)?;
        println!("Sent {bytes_written} bytes to /dev/uhid (X=+50). Check cursor on screen!");
    }

    Ok(())
}
