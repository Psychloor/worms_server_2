/// <summary>
/// Represents the flag sent with in a <see cref="SessionInfo"/>.
/// </summary>
/// <remarks>Names are in ISO 3166 Alpha-2 notation.</remarks>
#[derive(Debug, Default, PartialOrd, PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum Nation {
    /// <summary>No flag.</summary>
    #[default]
    None,
    /// <summary>United Kingdom</summary>
    UK,
    /// <summary>Argentina</summary>
    AR,
    /// <summary>Australia</summary>
    AU,
    /// <summary>Austria</summary>
    AT,
    /// <summary>Belgium</summary>
    BE,
    /// <summary>Brazil</summary>
    BR,
    /// <summary>Canada</summary>
    CA,
    /// <summary>Croatia</summary>
    HR,
    /// <summary>Bosnia and Herzegovina (old flag)</summary>
    BA,
    /// <summary>Cyprus</summary>
    CY,
    /// <summary>Czech Republic</summary>
    CZ,
    /// <summary>Denmark</summary>
    DK,
    /// <summary>Finland</summary>
    FI,
    /// <summary>France</summary>
    FR,
    /// <summary>Georgia</summary>
    GE,
    /// <summary>Germany</summary>
    DE,
    /// <summary>Greece</summary>
    GR,
    /// <summary>Hong Kong SAR</summary>
    HK,
    /// <summary>Hungary</summary>
    HU,
    /// <summary>Iceland</summary>
    IS,
    /// <summary>India</summary>
    IN,
    /// <summary>Indonesia</summary>
    ID,
    /// <summary>Iran</summary>
    IR,
    /// <summary>Iraq</summary>
    IQ,
    /// <summary>Ireland</summary>
    IE,
    /// <summary>Israel</summary>
    IL,
    /// <summary>Italy</summary>
    IT,
    /// <summary>Japan</summary>
    JP,
    /// <summary>Liechtenstein</summary>
    LI,
    /// <summary>Luxembourg</summary>
    LU,
    /// <summary>Malaysia</summary>
    MY,
    /// <summary>Malta</summary>
    MT,
    /// <summary>Mexico</summary>
    MX,
    /// <summary>Morocco</summary>
    MA,
    /// <summary>Netherlands</summary>
    NL,
    /// <summary>New Zealand</summary>
    NZ,
    /// <summary>Norway</summary>
    NO,
    /// <summary>Poland</summary>
    PL,
    /// <summary>Portugal</summary>
    PT,
    /// <summary>Puerto Rico</summary>
    PR,
    /// <summary>Romania</summary>
    RO,
    /// <summary>Russian Federation</summary>
    RU,
    /// <summary>Singapore</summary>
    SG,
    /// <summary>South Africa</summary>
    ZA,
    /// <summary>Spain</summary>
    ES,
    /// <summary>Sweden</summary>
    SE,
    /// <summary>Switzerland</summary>
    CH,
    /// <summary>Turkey</summary>
    TR,
    /// <summary>United States</summary>
    US,
    /// <summary>Custom skull flag</summary>
    Skull,
    /// <summary>Custom Team17 flag</summary>
    Team17,
}

impl From<Nation> for u8 {
    fn from(value: Nation) -> Self {
        match value {
            Nation::None => 0,
            Nation::UK => 1,
            Nation::AR => 2,
            Nation::AU => 3,
            Nation::AT => 4,
            Nation::BE => 5,
            Nation::BR => 6,
            Nation::CA => 7,
            Nation::HR => 8,
            Nation::BA => 9,
            Nation::CY => 10,
            Nation::CZ => 11,
            Nation::DK => 12,
            Nation::FI => 13,
            Nation::FR => 14,
            Nation::GE => 15,
            Nation::DE => 16,
            Nation::GR => 17,
            Nation::HK => 18,
            Nation::HU => 19,
            Nation::IS => 20,
            Nation::IN => 21,
            Nation::ID => 22,
            Nation::IR => 23,
            Nation::IQ => 24,
            Nation::IE => 25,
            Nation::IL => 26,
            Nation::IT => 27,
            Nation::JP => 28,
            Nation::LI => 29,
            Nation::LU => 30,
            Nation::MY => 31,
            Nation::MT => 32,
            Nation::MX => 33,
            Nation::MA => 34,
            Nation::NL => 35,
            Nation::NZ => 36,
            Nation::NO => 37,
            Nation::PL => 38,
            Nation::PT => 39,
            Nation::PR => 40,
            Nation::RO => 41,
            Nation::RU => 42,
            Nation::SG => 43,
            Nation::ZA => 44,
            Nation::ES => 45,
            Nation::SE => 46,
            Nation::CH => 47,
            Nation::TR => 48,
            Nation::US => 49,
            Nation::Skull => 50,
            Nation::Team17 => 51,
        }
    }
}

impl From<u8> for Nation {
    fn from(value: u8) -> Self {
        match value {
            0 => Nation::None,
            1 => Nation::UK,
            2 => Nation::AR,
            3 => Nation::AU,
            4 => Nation::AT,
            5 => Nation::BE,
            6 => Nation::BR,
            7 => Nation::CA,
            8 => Nation::HR,
            9 => Nation::BA,
            10 => Nation::CY,
            11 => Nation::CZ,
            12 => Nation::DK,
            13 => Nation::FI,
            14 => Nation::FR,
            15 => Nation::GE,
            16 => Nation::DE,
            17 => Nation::GR,
            18 => Nation::HK,
            19 => Nation::HU,
            20 => Nation::IS,
            21 => Nation::IN,
            22 => Nation::ID,
            23 => Nation::IR,
            24 => Nation::IQ,
            25 => Nation::IE,
            26 => Nation::IL,
            27 => Nation::IT,
            28 => Nation::JP,
            29 => Nation::LI,
            30 => Nation::LU,
            31 => Nation::MY,
            32 => Nation::MT,
            33 => Nation::MX,
            34 => Nation::MA,
            35 => Nation::NL,
            36 => Nation::NZ,
            37 => Nation::NO,
            38 => Nation::PL,
            39 => Nation::PT,
            40 => Nation::PR,
            41 => Nation::RO,
            42 => Nation::RU,
            43 => Nation::SG,
            44 => Nation::ZA,
            45 => Nation::ES,
            46 => Nation::SE,
            47 => Nation::CH,
            48 => Nation::TR,
            49 => Nation::US,
            50 => Nation::Skull,
            51 => Nation::Team17,
            _ => Nation::None, // Default case if the u8 value doesn't match any variant
        }
    }
}