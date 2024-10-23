use crate::net::packet_code::PacketCode;
use crate::net::session_info::SessionInfo;
use anyhow::anyhow;
use encoding_rs::WINDOWS_1252;
use log::error;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use tokio_util::bytes::{BufMut, Bytes, BytesMut};

pub(crate) const MAX_NAME_LENGTH: usize = 20;

pub struct WormsPacket {
    pub header_code: PacketCode,
    pub(super) flags: PacketFlags,
    pub value_0: Option<u32>,
    pub value_1: Option<u32>,
    pub value_2: Option<u32>,
    pub value_3: Option<u32>,
    pub value_4: Option<u32>,
    pub value_10: Option<u32>,
    pub data: Option<String>,
    pub error_code: Option<u32>,
    pub name: Option<String>,
    pub session: Option<Arc<SessionInfo>>,
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct PacketFlags: u32 {
        const VALUE0     = 1 << 0;
        const VALUE1     = 1 << 1;
        const VALUE2     = 1 << 2;
        const VALUE3     = 1 << 3;
        const VALUE4     = 1 << 4;
        const VALUE10    = 1 << 10;
        const DATALENGTH = 1 << 5;
        const DATA       = 1 << 6;
        const ERRORCODE  = 1 << 7;
        const NAME       = 1 << 8;
        const SESSION    = 1 << 9;
    }
}

impl Default for WormsPacket {
    fn default() -> Self {
        Self {
            header_code: PacketCode::default(),
            flags: PacketFlags::empty(),
            value_0: None,
            value_1: None,
            value_2: None,
            value_3: None,
            value_4: None,
            value_10: None,
            data: None,
            error_code: None,
            name: None,
            session: None,
        }
    }
}

impl WormsPacket {
    pub fn create(header: PacketCode) -> Self {
        WormsPacket {
            header_code: header,
            ..Default::default()
        }
    }

    pub fn with_value_0(mut self, value: u32) -> Self {
        self.value_0 = Some(value);
        self.flags.set(PacketFlags::VALUE0, true);
        self
    }

    pub fn with_value_1(mut self, value: u32) -> Self {
        self.value_1 = Some(value);
        self.flags.set(PacketFlags::VALUE1, true);
        self
    }

    pub fn with_value_2(mut self, value: u32) -> Self {
        self.value_2 = Some(value);
        self.flags.set(PacketFlags::VALUE2, true);
        self
    }
    pub fn with_value_3(mut self, value: u32) -> Self {
        self.value_3 = Some(value);
        self.flags.set(PacketFlags::VALUE3, true);
        self
    }
    pub fn with_value_4(mut self, value: u32) -> Self {
        self.value_4 = Some(value);
        self.flags.set(PacketFlags::VALUE4, true);
        self
    }
    pub fn with_value_10(mut self, value: u32) -> Self {
        self.value_10 = Some(value);
        self.flags.set(PacketFlags::VALUE10, true);
        self
    }
    pub fn with_data(mut self, value: &str) -> Self {
        if !value.is_empty() {
            self.data = Some(value.to_string());
            // Length then Data
            self.flags.set(PacketFlags::DATALENGTH, true);
            self.flags.set(PacketFlags::DATA, true);
        }

        self
    }

    pub fn with_error_code(mut self, value: u32) -> Self {
        self.error_code = Some(value);
        self.flags.set(PacketFlags::ERRORCODE, true);
        self
    }

    pub fn with_name(mut self, value: &str) -> Self {
        if value.len() <= MAX_NAME_LENGTH {
            self.name = Some(value.to_string());
        } else {
            // Truncate the string to MAX_NAME_LENGTH
            self.name = Some(value.chars().take(MAX_NAME_LENGTH).collect());
        }
        self.flags.set(PacketFlags::NAME, true);
        self
    }

    pub fn with_session(mut self, value: &Arc<SessionInfo>) -> Self {
        self.session = Some(Arc::clone(value));
        self.flags.set(PacketFlags::SESSION, true);
        self
    }

    pub fn build(self) -> anyhow::Result<Arc<Bytes>> {
        let mut dst = BytesMut::new();
        dst.put_u32_le(self.header_code.into());
        dst.put_u32_le(self.flags.bits());

        if let Some(value) = self.value_0 {
            dst.put_u32_le(value);
        }
        if let Some(value) = self.value_1 {
            dst.put_u32_le(value);
        }
        if let Some(value) = self.value_2 {
            dst.put_u32_le(value);
        }
        if let Some(value) = self.value_3 {
            dst.put_u32_le(value);
        }
        if let Some(value) = self.value_4 {
            dst.put_u32_le(value);
        }
        if let Some(value) = self.value_10 {
            dst.put_u32_le(value);
        }
        if let Some(value) = &self.data {
            let (encoded, _, had_error) = WINDOWS_1252.encode(value);
            if had_error {
                error!("Packet Data: Windows-1252 encode error");
                return Err(anyhow!("Windows-1252 encode error"));
            }

            dst.put_u32_le(u32::try_from(encoded.len() + 1)?);
            dst.extend_from_slice(&encoded);
            dst.put_u8(b'\0');
        }
        if let Some(value) = self.error_code {
            dst.put_u32_le(value);
        }
        if let Some(value) = &self.name {
            let mut buffer = [b'\0'; MAX_NAME_LENGTH];
            let (encoded, _, had_error) = WINDOWS_1252.encode(value);

            if had_error {
                error!("Packet Name: Windows-1252 encode error");
                return Err(anyhow!("Windows-1252 encode error"));
            }

            let length = encoded.len().min(MAX_NAME_LENGTH);
            buffer[..length].copy_from_slice(&encoded[..length]);

            dst.extend_from_slice(&buffer);
        }

        if let Some(session) = &self.session {
            dst.put_u64_le(crate::net::worms_codec::CRC);
            dst.put_u8(session.nation.into());
            dst.put_u8(49);

            //dst.put_u8(session.game_release);
            dst.put_u8(49);

            dst.put_u8(session.session_type.into());
            dst.put_u8(session.access.into());
            dst.put_u8(1);
            dst.put_u8(0);
            dst.extend_from_slice(&crate::net::worms_codec::EMPTY_BUFFER);
        }

        Ok(Arc::new(dst.freeze()))
    }
}

impl Debug for WormsPacket {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Header: {:?} ", self.header_code)?;

        if let Some(value) = self.value_0 {
            write!(f, "Value 0: {} ", value)?;
        }
        if let Some(value) = self.value_1 {
            write!(f, "Value 1: {} ", value)?;
        }
        if let Some(value) = self.value_2 {
            write!(f, "Value 2: {} ", value)?;
        }
        if let Some(value) = self.value_3 {
            write!(f, "Value 3: {} ", value)?;
        }
        if let Some(value) = self.value_4 {
            write!(f, "Value 4: {} ", value)?;
        }
        if let Some(value) = self.value_10 {
            write!(f, "Value 10: {} ", value)?;
        }
        if let Some(value) = &self.data {
            write!(f, "Data: {} ", value)?;
        }
        if let Some(value) = self.error_code {
            write!(f, "Error Code: {} ", value)?;
        }
        if let Some(value) = &self.name {
            write!(f, "Name: {} ", value)?;
        }
        if let Some(value) = &self.session {
            write!(f, "Session: {:?} ", value)?;
        }

        Ok(())
    }
}