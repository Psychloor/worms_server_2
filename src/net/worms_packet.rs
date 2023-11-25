use crate::net::packet_code::PacketCode;
use crate::net::session_info::SessionInfo;
use anyhow::anyhow;
use encoding_rs::WINDOWS_1251;
use log::error;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use tokio_util::bytes::{BufMut, Bytes, BytesMut};

pub(crate) const MAX_NAME_LENGTH: usize = 20;

#[derive(Default)]
pub struct WormsPacket {
    pub header_code: PacketCode,
    pub(super) flags: u32,
    pub value_0: Option<u32>,
    pub value_1: Option<u32>,
    pub value_2: Option<u32>,
    pub value_3: Option<u32>,
    pub value_4: Option<u32>,
    pub value_10: Option<u32>,
    pub data: Option<String>,
    pub error_code: Option<u32>,
    pub name: Option<String>,
    pub session: Option<SessionInfo>,
}

pub enum PacketField {
    Value0,
    Value1,
    Value2,
    Value3,
    Value4,
    Value10,
    DataLength,
    Data,
    ErrorCode,
    Name,
    Session,
}

impl PacketField {
    #[inline]
    pub fn into_bit(self) -> u32 {
        match self {
            PacketField::Value0 => 1 << 0,
            PacketField::Value1 => 1 << 1,
            PacketField::Value2 => 1 << 2,
            PacketField::Value3 => 1 << 3,
            PacketField::Value4 => 1 << 4,
            PacketField::Value10 => 1 << 10,
            PacketField::DataLength => 1 << 5,
            PacketField::Data => 1 << 6,
            PacketField::ErrorCode => 1 << 7,
            PacketField::Name => 1 << 8,
            PacketField::Session => 1 << 9,
        }
    }

    #[inline]
    pub fn is_flag_set(flag: u32, field: PacketField) -> bool {
        flag & field.into_bit() != 0
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
        self.flags |= PacketField::Value0.into_bit();
        self
    }

    pub fn with_value_1(mut self, value: u32) -> Self {
        self.value_1 = Some(value);
        self.flags |= PacketField::Value1.into_bit();
        self
    }

    pub fn with_value_2(mut self, value: u32) -> Self {
        self.value_2 = Some(value);
        self.flags |= PacketField::Value2.into_bit();
        self
    }
    pub fn with_value_3(mut self, value: u32) -> Self {
        self.value_3 = Some(value);
        self.flags |= PacketField::Value3.into_bit();
        self
    }
    pub fn with_value_4(mut self, value: u32) -> Self {
        self.value_4 = Some(value);
        self.flags |= PacketField::Value4.into_bit();
        self
    }
    pub fn with_value_10(mut self, value: u32) -> Self {
        self.value_10 = Some(value);
        self.flags |= PacketField::Value10.into_bit();
        self
    }
    pub fn with_data(mut self, value: &str) -> Self {
        if !value.is_empty() {
            self.data = Some(value.to_string());
            // Length then Data
            self.flags |= PacketField::DataLength.into_bit() | PacketField::Data.into_bit();
        }

        self
    }

    pub fn with_error_code(mut self, value: u32) -> Self {
        self.error_code = Some(value);
        self.flags |= PacketField::ErrorCode.into_bit();
        self
    }

    pub fn with_name(mut self, value: &str) -> Self {
        if value.len() <= MAX_NAME_LENGTH {
            self.name = Some(value.to_string());
        } else {
            // Truncate the string to MAX_NAME_LENGTH
            self.name = Some(value.chars().take(MAX_NAME_LENGTH).collect());
        }
        self.flags |= PacketField::Name.into_bit();
        self
    }

    pub fn with_session(mut self, value: SessionInfo) -> Self {
        self.session = Some(value);
        self.flags |= PacketField::Session.into_bit();
        self
    }

    pub fn build(self) -> anyhow::Result<Arc<Bytes>> {
        let mut dst = BytesMut::new();
        dst.put_u32_le(self.header_code.into());
        dst.put_u32_le(self.flags);

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
            let (encoded, _, had_error) = WINDOWS_1251.encode(value);
            if had_error {
                error!("Packet Data: Windows-1251 encode error");
                return Err(anyhow!("Windows-1251 encode error"));
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
            let (encoded, _, had_error) = WINDOWS_1251.encode(value);

            if had_error {
                error!("Packet Name: Windows-1251 encode error");
                return Err(anyhow!("Windows-1251 encode error"));
            }

            let length = encoded.len().min(MAX_NAME_LENGTH);
            buffer[..length].copy_from_slice(&encoded[..length]);

            dst.extend_from_slice(&buffer);
        }

        if let Some(session) = &self.session {
            dst.put_u32_le(crate::net::worms_codec::CRC_FIRST);
            dst.put_u32_le(crate::net::worms_codec::CRC_SECOND);
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