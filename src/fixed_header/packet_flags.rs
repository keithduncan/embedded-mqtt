use core::{
    convert::{From, TryFrom, TryInto},
    result::Result,
    fmt::Debug,
};

use crate::qos;

use bitfield::BitRange;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct PacketFlags(pub u8);

#[allow(dead_code)]
impl PacketFlags {
    pub const CONNECT: PacketFlags = PacketFlags(0b0000);
    pub const CONNACK: PacketFlags = PacketFlags(0b0000);
    // publish is special
    pub const PUBACK: PacketFlags = PacketFlags(0b0000);
    pub const PUBREC: PacketFlags = PacketFlags(0b0000);
    pub const PUBREL: PacketFlags = PacketFlags(0b0010);
    pub const PUBCOMP: PacketFlags = PacketFlags(0b0000);
    pub const SUBSCRIBE: PacketFlags = PacketFlags(0b0010);
    pub const SUBACK: PacketFlags = PacketFlags(0b0000);
    pub const UNSUBSCRIBE: PacketFlags = PacketFlags(0b0010);
    pub const UNSUBACK: PacketFlags = PacketFlags(0b0000);
    pub const PINGREQ: PacketFlags = PacketFlags(0b0000);
    pub const PINGRESP: PacketFlags = PacketFlags(0b0000);
    pub const DISCONNECT: PacketFlags = PacketFlags(0b0000);
}

impl From<PublishFlags> for PacketFlags {
    fn from(flags: PublishFlags) -> Self {
        PacketFlags(flags.0)
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Default)]
pub struct PublishFlags(u8);

bitfield_bitrange! {
    struct PublishFlags(u8)
}

impl PublishFlags {
    bitfield_fields! {
        bool;
        pub dup,    set_dup    : 3;
        pub retain, set_retain : 0;
    }

    pub fn qos(&self) -> Result<qos::QoS, qos::Error> {
        let qos_bits: u8 = self.bit_range(2, 1);
        qos_bits.try_into()
    }

    #[allow(dead_code)]
    pub fn set_qos(&mut self, qos: qos::QoS) {
        self.set_bit_range(2, 1, u8::from(qos))
    }
}

impl Debug for PublishFlags {
    bitfield_debug! {
        struct PublishFlags;
        pub dup, _                : 3;
        pub into qos::QoS, qos, _ : 2, 1;
        pub retain, _             : 0;
    }
}

impl TryFrom<PacketFlags> for PublishFlags {
    type Error = qos::Error;
    fn try_from(flags: PacketFlags) -> Result<Self, Self::Error> {
        let flags = PublishFlags(flags.0);
        flags.qos()?;
        Ok(flags)
    }
}
