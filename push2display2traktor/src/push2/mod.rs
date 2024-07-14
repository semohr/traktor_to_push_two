use padding::padding;
use rusb::{Context, Device, DeviceDescriptor, DeviceHandle, UsbContext};
use thiserror::Error;
use xor::xor;

mod padding;
mod xor;

pub struct Push2Display {
    handle: DeviceHandle<Context>,
}

#[derive(Error, Debug)]
pub enum Push2DisplayError {
    #[error("Ableton Push2 Not found")]
    Push2NotFound,

    /// Represents all other cases of `std::io::Error`.
    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[error(transparent)]
    USBError(#[from] rusb::Error),
}

// Push2 usb constants
const PUSH_2_VENDOR_ID: u16 = 0x2982;
const PUSH_2_PRODUCT_ID: u16 = 0x1967;
const PUSH2_BULK_EP_OUT: u8 = 0x01;
#[rustfmt::skip]
const HEADER: [u8; 16] = [
    0xff, 0xcc, 0xaa, 0x88,
    0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00
];

// Display specs
pub const DISPLAY_WIDTH: usize = 960;
pub const DISPLAY_HEIGHT: usize = 160;
pub const LINE_SIZE: usize = DISPLAY_WIDTH + 64; // padding of 64 pixels i.e. 128 bytes
pub const FRAME_SIZE: usize = DISPLAY_HEIGHT * LINE_SIZE * 2;

impl Push2Display {
    pub fn new() -> Result<Push2Display, Push2DisplayError> {
        let mut context = Context::new()?;
        let (_, _, handle) = open_device(&mut context, PUSH_2_VENDOR_ID, PUSH_2_PRODUCT_ID)
            .ok_or(Push2DisplayError::Push2NotFound)?;

        handle.claim_interface(0)?;

        Ok(Push2Display { handle })
    }

    pub fn send_buffer(&self, buffer: &[u8]) -> Result<(), Push2DisplayError> {
        let timeout = std::time::Duration::from_millis(50);
        let chunk_size = 2 * 1024; // 2KB
                                   // Write header to indicate frame buffer is next
        self.handle
            .write_bulk(PUSH2_BULK_EP_OUT, &HEADER, timeout)?;

        // Now push expects a frame (we will send 2kb at a time todo)
        // Send buffer in 2KB chunks
        self.handle.write_bulk(PUSH2_BULK_EP_OUT, buffer, timeout)?;
        //for chunk in buffer.chunks(chunk_size) {
        //}
        //self.handle.write_bulk(PUSH2_BULK_EP_OUT, buffer, timeout)?;

        Ok(())
    }
}

fn open_device<T: UsbContext>(
    context: &mut T,
    vid: u16,
    pid: u16,
) -> Option<(Device<T>, DeviceDescriptor, DeviceHandle<T>)> {
    let devices = match context.devices() {
        Ok(d) => d,
        Err(_) => return None,
    };

    for device in devices.iter() {
        let device_desc = match device.device_descriptor() {
            Ok(d) => d,
            Err(_) => continue,
        };

        if device_desc.vendor_id() == vid && device_desc.product_id() == pid {
            match device.open() {
                Ok(handle) => {
                    return Some((device, device_desc, handle));
                }
                Err(e) => {
                    println!("{}", e);
                    continue;
                }
            }
        }
    }

    None
}

pub fn encode_buffer(buffer: &[u16]) -> Vec<u8> {
    //Apply padding
    let mut p = padding(buffer);
    //Xor
    xor(&mut p);
    return p;
}

pub fn rgba8_to_bgr565(rgba_data: &Vec<u8>) -> Vec<u16> {
    let mut bgr565_data = vec![0u16; (DISPLAY_WIDTH * DISPLAY_HEIGHT) as usize];
    for (i, chunk) in rgba_data.chunks_exact(4).enumerate() {
        let r = chunk[0] >> 3;
        let g = chunk[1] >> 2;
        let b = chunk[2] >> 3;
        bgr565_data[i] = ((b as u16) << 11) | ((g as u16) << 5) | (r as u16);
    }
    return bgr565_data;
}
