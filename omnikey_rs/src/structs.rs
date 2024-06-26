//! Structures representing the physical
//! reader as well as the data read.

use std::time::Duration;

use rusb::{
    Device,
    DeviceList,
    DeviceDescriptor,
    DeviceHandle,
    GlobalContext
};

/// The USB Vendor ID for the OMNIKEY 5025CL
const OMNIKEY_VENDOR_ID: u16 = 0x076B;
/// The USB Product ID for the OMNIKEY 5025CL
const OMNIKEY_PRODUCT_ID: u16 = 0x502A;

/// A struct representing the physical OMNIKEY
/// reader.
///
/// The Reader struct is the physical device
/// plugged into the host device. Its primary
/// operations are to initialize ([Reader::new()]),
/// set legacy mode (which is required for CCID operations),
/// as well as checking for and returning RFID data.
#[allow(dead_code)]
pub struct Reader {
    descriptor: DeviceDescriptor,
    device: Device<GlobalContext>,
    handle: DeviceHandle<GlobalContext>
}

impl Reader {

    /// Initializes the physical OMNIKEY device
    /// and returns it to the user.
    /// 
    /// # Return
    /// A result where:
    /// - On `Ok()`, returns an object that maps to
    /// the physical OMNIKEY Reader
    /// - On `Err()`, returns a String detailing the error
    pub fn new() -> Result<Reader, String> {
        
        let mut target: Option<Device<GlobalContext>> = None;
        let mut target_desc: Option<DeviceDescriptor> = None;

        for device in DeviceList::new().unwrap().iter() {
            let device_desc = match device.device_descriptor() {
                Ok(d) => d,
                Err(_) => continue
            };

            if  device_desc.vendor_id() == OMNIKEY_VENDOR_ID &&
                device_desc.product_id() == OMNIKEY_PRODUCT_ID {
                    target = Some(device);
                    target_desc = Some(device_desc);
                    break;
            }
        }

        if target.is_none() {
            return Err("Could not find Omnikey Reader.".to_string());
        }

        let target = target.unwrap();
        let target_desc = target_desc.unwrap();

        let handle: DeviceHandle<GlobalContext> = match target.open() {
            Ok(h) => h,
            Err(e) => {
                return Err(format!("Error opening handle: {}", e));
            }
        };

        Ok(Reader {
            descriptor: target_desc,
            device: target,
            handle
        })
    }

    /// Tells the reader to do operations on
    /// the legacy CCID encryption mode, rather
    /// than its usual encryption mode. Required
    /// for RPI ID scans.
    /// 
    /// # Returns
    /// A result where:
    /// - On `Ok()`, returns nothing.
    /// - On `Err()`, returns a String detailing the error
    pub fn set_legacy_ccid_mode(&self) -> Result<(), String> {

        let cmd: [u8;13] = [
            0xFF, 0x70, 0x07, 0x6B, 0x07,
            0xA2, 0x05, 0xA1, 0x03, 0x8B,
            0x01, 0x00, 0x00];
        let mut buf: [u8;17] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,];
        
        let timeout = Duration::from_millis(100);
        
        let bytes_sent = self.handle.write_bulk(0x01, &cmd, timeout);
        let bytes_sent = match bytes_sent {
            Ok(b) => b,
            Err(e) => {
                return Err(format!("Error sending legacy code: {}", e));
            }
        };

        if bytes_sent != cmd.len() {
            return Err(format!("Expected to send {} bytes, actually sent {}",
                cmd.len(), bytes_sent));
        }

        let bytes_sent = self.handle.read_bulk(0x82, &mut buf, timeout);
        match bytes_sent {
            Ok(_) => {},
            Err(e) => {
                return Err(format!("Error reading return value of Legacy command: {}", e));
            }
        };

        Ok(())
    }

    /// Queries the card reader for an RFID
    /// card.
    ///
    /// # Returns
    /// A result where:
    /// - On Ok(), returns a ReaderData object
    /// - On Err(), returns a String detailing the error
    pub fn check_for_rfid_card(&self) -> Result<ReaderData, String> {
        let cmd: [u8;15] = [
            0x6F, 0x05, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x01, 0x00, 0x00,
            0xFF, 0xCA, 0x00, 0x00, 0x00
        ];

        let mut buf: [u8;17] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,];

        let timeout = Duration::from_millis(100);

        let bytes_sent = self.handle.write_bulk(0x01, &cmd, timeout);
        let bytes_sent = match bytes_sent {
            Ok(b) => b,
            Err(e) => {
                return Err(format!("Error sending \"Get Data\" message: {}", e));
            }
        };
        if bytes_sent != cmd.len() {
            return Err(format!("Expected to send {} bytes, sent {}", cmd.len(), bytes_sent));
        }

        let bytes_sent = self.handle.read_bulk(0x82, &mut buf, timeout);
        match bytes_sent {
            Ok(_) => {},
            Err(e) => {
                return Err(format!("Error reading \"Get Data\" output: {}", e));
            }
        };

        let data = ReaderData::new(&buf);
        Ok(data)
    }
}

/// A data structure read from the OMNIKEY
/// based on the card's RFID data.
pub struct ReaderData {
    pub message_type: u8,
    pub length: u32,
    pub slot: u8,
    pub seq: u8,
    pub status: u8,
    pub error: u8,
    pub chain_parameter: u8,
    pub valid: bool,
    pub id: u64,
    pub adpu_status: u16
}

impl ReaderData {

    /// Creates a new ReaderData object from
    /// bytes read by the OMNIKEY device.
    /// 
    /// # Parameters
    /// - `data`: A slice of 17 bytes that
    /// make up the return message
    /// 
    /// # Return
    /// A new ReaderData object, whether
    /// some data be null or not.
    pub fn new(data: &[u8; 17]) -> ReaderData {
        let length: u32 =
            u32::from(data[4]) << 24 |
            u32::from(data[3]) << 16 |
            u32::from(data[2]) << 8 |
            u32::from(data[1]);
        
        let mut valid: bool = false;
        let mut id: u64 = 0;
        let mut adpu_status: u16 = 0;
    
        if length == 7 {
            valid = true;
            for i in 0..5 {
                id += u64::from(data[10 + i]) << ((4 - i) * 8);
            }
            adpu_status = u16::from(data[15]) << 8 | u16::from(data[16]);
        }
        else if length >= 2 {
            let ind1: usize = usize::try_from(length - 2).unwrap();
            let ind2: usize = usize::try_from(length - 1).unwrap();
            adpu_status = u16::from(data[ind1]) << 8 +
                u16::from(data[ind2]);
        }
    
        ReaderData {
            message_type: data[0],
            length,
            slot: data[5],
            seq: data[6],
            status: data[7],
            error: data[8],
            chain_parameter: data[9],
            valid,
            id,
            adpu_status
        }
    }

    /// Returns a string with lots of information
    /// about the card scanned.
    /// 
    /// # Return
    /// A String with information formatted similarly
    /// to the `lsusb` command on UNIX systems.
    pub fn to_string(&self) -> String {
        format!(
"Reader Data
  Message Type: 0x{:02x}
  Data Length: {}
  Slot Affected: {}
  Sequence Number: {}
  Status: 0x{:02x}
  Error: 0x{:02x}
  Chain Parameter: 0x{:02x}
  Data:
    isDataValid: {}
    ID (decimal): {}
    ID (hex): 0x{:010x}
    ADPU Status Code: 0x{:04x}",
            self.message_type,
            self.length,
            self.slot,
            self.seq,
            self.status,
            self.error,
            self.chain_parameter,
            self.valid,
            self.id,
            self.id,
            self.adpu_status
        )
    }
}

impl std::fmt::Display for ReaderData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}