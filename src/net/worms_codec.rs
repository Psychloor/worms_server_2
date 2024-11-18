use crate::net::session_info::SessionInfo;
use crate::net::worms_packet::{PacketFlags, WormsPacket};
use encoding_rs::WINDOWS_1252;
use eyre::{bail, eyre, Result};
use log::error;
use std::sync::Arc;
use tokio_util::bytes::{Buf, Bytes, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

pub struct WormCodec;

pub const EMPTY_BUFFER: [u8; 35] = [0u8; 35];
pub const CRC_FIRST: u32 = 0x17171717;
pub const CRC_SECOND: u32 = u32::from_be_bytes([0x02, 0x01, 0x01, 0x01]);

const MAX_DATA_LENGTH: usize = 0x200;
const ZEROES_EXPECTED: usize = 35;

impl Encoder<Arc<Bytes>> for WormCodec {
    type Error = eyre::Error;
    fn encode(&mut self, item: Arc<Bytes>, dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.extend_from_slice(&item);
        Ok(())
    }
}

impl Decoder for WormCodec {
    type Item = Arc<WormsPacket>;
    type Error = eyre::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 8 {
            // tell the frame we need more
            return Ok(None);
        }

        let mut packet = WormsPacket::default();

        match src.get_u32_le().try_into() {
            Ok(code) => {
                packet.header_code = code;
            }
            Err(e) => {
                bail!("Invalid Packet Header! {}", e);
            }
        }
        packet.flags = PacketFlags::from_bits_truncate(src.get_u32_le());

        if packet.flags.contains(PacketFlags::VALUE0) {
            if src.remaining() < 4 {
                return Ok(None);
            }
            packet.value_0 = Option::from(src.get_u32_le());
        }

        if packet.flags.contains(PacketFlags::VALUE1) {
            if src.remaining() < 4 {
                return Ok(None);
            }
            packet.value_1 = Option::from(src.get_u32_le());
        }

        if packet.flags.contains(PacketFlags::VALUE2) {
            if src.remaining() < 4 {
                return Ok(None);
            }
            packet.value_2 = Option::from(src.get_u32_le());
        }

        if packet.flags.contains(PacketFlags::VALUE3) {
            if src.remaining() < 4 {
                return Ok(None);
            }
            packet.value_3 = Option::from(src.get_u32_le());
        }

        if packet.flags.contains(PacketFlags::VALUE4) {
            if src.remaining() < 4 {
                return Ok(None);
            }
            packet.value_4 = Option::from(src.get_u32_le());
        }

        if packet.flags.contains(PacketFlags::VALUE10) {
            if src.remaining() < 4 {
                return Ok(None);
            }
            packet.value_10 = Option::from(src.get_u32_le());
        }

        if packet.flags.contains(PacketFlags::DATALENGTH) {
            if src.remaining() < 4 {
                return Ok(None);
            }
            let length = src.get_u32_le() as usize;
            if length > MAX_DATA_LENGTH {
                bail!("Data Length too long! {}", length);
            }

            if packet.flags.contains(PacketFlags::DATA) {
                if src.remaining() < length {
                    return Ok(None);
                }

                let data_bytes = src.split_to(length);
                let filtered_bytes: Vec<u8> = data_bytes
                    .iter()
                    .cloned()
                    .filter(|&byte| byte != b'\0')
                    .collect();
                let (decoded, _, had_error) = WINDOWS_1252.decode(&filtered_bytes);
                if had_error {
                    error!("Packet Data: Windows-1252 decode error");
                    bail!("Windows-1252 decode error");
                }

                packet.data = Some(decoded.to_string());
            }
        }

        if packet.flags.contains(PacketFlags::ERRORCODE) {
            if src.remaining() < 4 {
                return Ok(None);
            }
            packet.error_code = Some(src.get_u32_le());
        }

        if packet.flags.contains(PacketFlags::NAME) {
            if src.remaining() < super::worms_packet::MAX_NAME_LENGTH {
                return Ok(None);
            }

            let data_bytes = src.split_to(super::worms_packet::MAX_NAME_LENGTH);
            let filtered_bytes: Vec<u8> = data_bytes
                .iter()
                .cloned()
                .filter(|&byte| byte != b'\0')
                .collect();

            let (decoded, _, had_error) = WINDOWS_1252.decode(&filtered_bytes);
            if had_error {
                error!("Packet Name: Windows-1252 decode error");
                bail!("Windows-1252 decode error");
            }

            packet.name = Some(decoded.to_string());
        }

        if packet.flags.contains(PacketFlags::SESSION) {
            if src.remaining() < 50 {
                return Ok(None);
            }
            let mut session_info = SessionInfo::default();

            // endianness doesn't matter on first. same no matter which order
            if src.get_u32() != CRC_FIRST {
                bail!("Invalid first CRC");
            }

            if src.get_u32_le() != CRC_SECOND {
                bail!("Invalid second CRC");
            }

            session_info.nation = src.get_u8().into();

            // should be 49
            let _game_version = src.get_u8();

            session_info.game_release = src.get_u8();

            session_info.session_type = src
                .get_u8()
                .try_into()
                .map_err(|e| eyre!("Session type invalid! {:?}", e))?;
            session_info.access = src
                .get_u8()
                .try_into()
                .map_err(|e| eyre!("Session access invalid! {:?}", e))?;

            if src.get_u8() != 1 {
                bail!("Invalid Data! Expected 1");
            }
            if src.get_u8() != 0 {
                bail!("Invalid Data! Expected 0");
            }

            if src.remaining() < ZEROES_EXPECTED
                || src[..ZEROES_EXPECTED].iter().any(|&byte| byte != 0)
            {
                bail!("Invalid Data Buffer! Non-zero byte found or insufficient data");
            }

            src.advance(ZEROES_EXPECTED);

            packet.session = Some(Arc::new(session_info));
        }

        Ok(Some(Arc::new(packet)))
    }
}
