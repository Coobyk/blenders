//! Port of `intern/unit.cc` — the unit system: `bUnitDef`
//! tables for every system (natural/metric/imperial) and type
//! (length/area/volume/mass/rotation/time/velocity/
//! acceleration/camera/power/temperature/wavelength/
//! color-temperature/frequency), best-fit selection,
//! dual-unit ("2ft 3in") split, string conversion, and the
//! `BKE_unit_*` scalar/name/validity API.
//!
//! `PORT_PENDING`: `BKE_unit_replace_string` (regex-driven
//! in-place rewrite), the expression-evaluator path of
//! `unit_split`, user-pref plumbing beyond `UnitSettings`
//! fields (`scale_length`, per-type preferred units as ints).

/// `UN_SC_*` — base-unit scalars (verbatim `#define`s).
pub const UN_SC_KM: f64 = 1000.0;
/// `UN_SC_HM`.
pub const UN_SC_HM: f64 = 1000.0 / 10.0;
/// `UN_SC_DAM`.
pub const UN_SC_DAM: f64 = 10.0;
/// `UN_SC_M`.
pub const UN_SC_M: f64 = 1.0;
/// `UN_SC_DM`.
pub const UN_SC_DM: f64 = 0.1;
/// `UN_SC_CM`.
pub const UN_SC_CM: f64 = 0.01;
/// `UN_SC_MM`.
pub const UN_SC_MM: f64 = 0.001;
/// `UN_SC_UM`.
pub const UN_SC_UM: f64 = 0.000001;
/// `UN_SC_MI`.
pub const UN_SC_MI: f64 = 1609.344;
/// `UN_SC_FUR`.
pub const UN_SC_FUR: f64 = 201.168;
/// `UN_SC_CH`.
pub const UN_SC_CH: f64 = 20.1168;
/// `UN_SC_YD`.
pub const UN_SC_YD: f64 = 0.9144;
/// `UN_SC_FT`.
pub const UN_SC_FT: f64 = 0.3048;
/// `UN_SC_IN`.
pub const UN_SC_IN: f64 = 0.0254;
/// `UN_SC_MIL`.
pub const UN_SC_MIL: f64 = 0.0000254;
/// `UN_SC_MTON`.
pub const UN_SC_MTON: f64 = 1000.0;
/// `UN_SC_QL`.
pub const UN_SC_QL: f64 = 100.0;
/// `UN_SC_KG`.
pub const UN_SC_KG: f64 = 1.0;
/// `UN_SC_HG`.
pub const UN_SC_HG: f64 = 0.1;
/// `UN_SC_DAG`.
pub const UN_SC_DAG: f64 = 0.01;
/// `UN_SC_G`.
pub const UN_SC_G: f64 = 0.001;
/// `UN_SC_MG`.
pub const UN_SC_MG: f64 = 0.000001;
/// `UN_SC_ITON`.
pub const UN_SC_ITON: f64 = 907.18474;
/// `UN_SC_CWT`.
pub const UN_SC_CWT: f64 = 45.359237;
/// `UN_SC_ST`.
pub const UN_SC_ST: f64 = 6.35029318;
/// `UN_SC_LB`.
pub const UN_SC_LB: f64 = 0.45359237;
/// `UN_SC_OZ`.
pub const UN_SC_OZ: f64 = 0.028349523125;
/// `UN_SC_FAH`.
pub const UN_SC_FAH: f64 = 0.555555555555;

/// `B_UNIT_DEF_NONE`.
pub const B_UNIT_DEF_NONE: i32 = 0;
/// `B_UNIT_DEF_SUPPRESS`.
pub const B_UNIT_DEF_SUPPRESS: i32 = 1;
/// `B_UNIT_DEF_TENTH`.
pub const B_UNIT_DEF_TENTH: i32 = 2;
/// `B_UNIT_DEF_CASE_SENSITIVE`.
pub const B_UNIT_DEF_CASE_SENSITIVE: i32 = 4;
/// `B_UNIT_DEF_NO_SPACE`.
pub const B_UNIT_DEF_NO_SPACE: i32 = 8;

/// `B_UNIT_*` type ids (`B_UNIT_TYPE_TOT` = 16).
pub const B_UNIT_NONE: i32 = 0;
/// `B_UNIT_LENGTH`.
pub const B_UNIT_LENGTH: i32 = 1;
/// `B_UNIT_AREA`.
pub const B_UNIT_AREA: i32 = 2;
/// `B_UNIT_VOLUME`.
pub const B_UNIT_VOLUME: i32 = 3;
/// `B_UNIT_MASS`.
pub const B_UNIT_MASS: i32 = 4;
/// `B_UNIT_ROTATION`.
pub const B_UNIT_ROTATION: i32 = 5;
/// `B_UNIT_TIME`.
pub const B_UNIT_TIME: i32 = 6;
/// `B_UNIT_TIME_ABSOLUTE`.
pub const B_UNIT_TIME_ABSOLUTE: i32 = 7;
/// `B_UNIT_VELOCITY`.
pub const B_UNIT_VELOCITY: i32 = 8;
/// `B_UNIT_ACCELERATION`.
pub const B_UNIT_ACCELERATION: i32 = 9;
/// `B_UNIT_CAMERA`.
pub const B_UNIT_CAMERA: i32 = 10;
/// `B_UNIT_POWER`.
pub const B_UNIT_POWER: i32 = 11;
/// `B_UNIT_TEMPERATURE`.
pub const B_UNIT_TEMPERATURE: i32 = 12;
/// `B_UNIT_WAVELENGTH`.
pub const B_UNIT_WAVELENGTH: i32 = 13;
/// `B_UNIT_COLOR_TEMPERATURE`.
pub const B_UNIT_COLOR_TEMPERATURE: i32 = 14;
/// `B_UNIT_FREQUENCY`.
pub const B_UNIT_FREQUENCY: i32 = 15;
/// `B_UNIT_TYPE_TOT`.
pub const B_UNIT_TYPE_TOT: i32 = 16;

/// `UNIT_SYSTEM_NONE`/`METRIC`/`IMPERIAL`/`TOT`.
pub const UNIT_SYSTEM_NONE: i32 = 0;
/// `UNIT_SYSTEM_METRIC`.
pub const UNIT_SYSTEM_METRIC: i32 = 1;
/// `UNIT_SYSTEM_IMPERIAL`.
pub const UNIT_SYSTEM_IMPERIAL: i32 = 2;
/// `UNIT_SYSTEM_TOT`.
pub const UNIT_SYSTEM_TOT: i32 = 3;

/// `USER_UNIT_ADAPTIVE`/`USER_UNIT_ROT_RADIANS`.
pub const USER_UNIT_ADAPTIVE: i32 = 0;
/// `USER_UNIT_ROT_RADIANS`.
pub const USER_UNIT_ROT_RADIANS: i32 = 4;
/// `USER_UNIT_NONE`.
pub const USER_UNIT_NONE: i32 = UNIT_SYSTEM_NONE;

/// `EPS` guard on scalar comparisons.
const EPS: f64 = 0.00001;
/// `std::f64::consts::PI` as used by table entries.
const M_PI: f64 = std::f64::consts::PI;

/// `bUnitDef`.
pub struct BUnitDef {
    /// `name` — canonical (also parse key).
    pub name: &'static str,
    /// `name_plural` — display plural.
    pub name_plural: Option<&'static str>,
    /// `name_short` — short display suffix.
    pub name_short: Option<&'static str>,
    /// `name_alt` — ASCII-friendly `name_short`.
    pub name_alt: Option<&'static str>,
    /// `name_display` — menu label.
    pub name_display: Option<&'static str>,
    /// `identifier` — RNA identifier.
    pub identifier: Option<&'static str>,
    /// `scalar` — base-units per this unit.
    pub scalar: f64,
    /// `bias` — additive offset (temperature).
    pub bias: f64,
    /// `flag` — `B_UNIT_DEF_*`.
    pub flag: i32,
}

/// `bUnitCollection`.
pub struct BUnitCollection {
    /// `units` — def table (base first-by-index).
    pub units: &'static [BUnitDef],
    /// `base_unit` — index used when unspecified.
    pub base_unit: usize,
    /// `flag` — collection options.
    pub flag: i32,
}

macro_rules! coll {
    ($units:expr, $base:expr) => {
        BUnitCollection {
            units: $units,
            base_unit: $base,
            flag: 0,
        }
    };
}
use coll;
/// `static bUnitDef buDummyDef[]`.
pub const BUDUMMYDEF: &[BUnitDef] = &[
    BUnitDef { name: "", name_plural: None, name_short: Some(""), name_alt: None, name_display: None, identifier: None, scalar: 1.0, bias: 0.0, flag: 0 },
];
/// `static bUnitDef buMetricLenDef[]`.
pub const BUMETRICLENDEF: &[BUnitDef] = &[
    BUnitDef { name: "kilometer", name_plural: Some("kilometers"), name_short: Some("km"), name_alt: None, name_display: Some("Kilometers"), identifier: Some("KILOMETERS"), scalar: UN_SC_KM, bias: 0.0, flag: B_UNIT_DEF_NONE },
    BUnitDef { name: "hectometer", name_plural: Some("hectometers"), name_short: Some("hm"), name_alt: None, name_display: Some("100 Meters"), identifier: Some("HECTOMETERS"), scalar: UN_SC_HM, bias: 0.0, flag: B_UNIT_DEF_SUPPRESS },
    BUnitDef { name: "dekameter", name_plural: Some("dekameters"), name_short: Some("dam"), name_alt: None, name_display: Some("10 Meters"), identifier: Some("DEKAMETERS"), scalar: UN_SC_DAM, bias: 0.0, flag: B_UNIT_DEF_SUPPRESS },
    BUnitDef { name: "meter", name_plural: Some("meters"), name_short: Some("m"), name_alt: None, name_display: Some("Meters"), identifier: Some("METERS"), scalar: UN_SC_M, bias: 0.0, flag: B_UNIT_DEF_NONE },
    BUnitDef { name: "decimeter", name_plural: Some("decimeters"), name_short: Some("dm"), name_alt: None, name_display: Some("10 Centimeters"), identifier: Some("DECIMETERS"), scalar: UN_SC_DM, bias: 0.0, flag: B_UNIT_DEF_SUPPRESS },
    BUnitDef { name: "centimeter", name_plural: Some("centimeters"), name_short: Some("cm"), name_alt: None, name_display: Some("Centimeters"), identifier: Some("CENTIMETERS"), scalar: UN_SC_CM, bias: 0.0, flag: B_UNIT_DEF_NONE },
    BUnitDef { name: "millimeter", name_plural: Some("millimeters"), name_short: Some("mm"), name_alt: None, name_display: Some("Millimeters"), identifier: Some("MILLIMETERS"), scalar: UN_SC_MM, bias: 0.0, flag: B_UNIT_DEF_NONE | B_UNIT_DEF_TENTH },
    BUnitDef { name: "micrometer", name_plural: Some("micrometers"), name_short: Some("µm"), name_alt: Some("um"), name_display: Some("Micrometers"), identifier: Some("MICROMETERS"), scalar: UN_SC_UM, bias: 0.0, flag: B_UNIT_DEF_NONE },
    BUnitDef { name: "nanometer", name_plural: Some("nanometers"), name_short: Some("nm"), name_alt: None, name_display: Some("Nanometers"), identifier: Some("NANOMETERS"), scalar: 0.000000001, bias: 0.0, flag: B_UNIT_DEF_NONE },
    BUnitDef { name: "picometer", name_plural: Some("picometers"), name_short: Some("pm"), name_alt: None, name_display: Some("Picometers"), identifier: Some("PICOMETERS"), scalar: 0.000000000001, bias: 0.0, flag: B_UNIT_DEF_NONE },
];
/// `static bUnitDef buImperialLenDef[]`.
pub const BUIMPERIALLENDEF: &[BUnitDef] = &[
    BUnitDef { name: "mile", name_plural: Some("miles"), name_short: Some("mi"), name_alt: None, name_display: Some("Miles"), identifier: Some("MILES"), scalar: UN_SC_MI, bias: 0.0, flag: B_UNIT_DEF_NONE },
    BUnitDef { name: "furlong", name_plural: Some("furlongs"), name_short: Some("fur"), name_alt: None, name_display: Some("Furlongs"), identifier: Some("FURLONGS"), scalar: UN_SC_FUR, bias: 0.0, flag: B_UNIT_DEF_SUPPRESS },
    BUnitDef { name: "chain", name_plural: Some("chains"), name_short: Some("ch"), name_alt: None, name_display: Some("Chains"), identifier: Some("CHAINS"), scalar: UN_SC_CH, bias: 0.0, flag: B_UNIT_DEF_SUPPRESS },
    BUnitDef { name: "yard", name_plural: Some("yards"), name_short: Some("yd"), name_alt: None, name_display: Some("Yards"), identifier: Some("YARDS"), scalar: UN_SC_YD, bias: 0.0, flag: B_UNIT_DEF_SUPPRESS },
    BUnitDef { name: "foot", name_plural: Some("feet"), name_short: Some("'"), name_alt: Some("ft"), name_display: Some("Feet"), identifier: Some("FEET"), scalar: UN_SC_FT, bias: 0.0, flag: B_UNIT_DEF_NONE | B_UNIT_DEF_NO_SPACE },
    BUnitDef { name: "inch", name_plural: Some("inches"), name_short: Some("\""), name_alt: Some("in"), name_display: Some("Inches"), identifier: Some("INCHES"), scalar: UN_SC_IN, bias: 0.0, flag: B_UNIT_DEF_NONE | B_UNIT_DEF_NO_SPACE },
    BUnitDef { name: "thou", name_plural: Some("thou"), name_short: Some("thou"), name_alt: Some("mil"), name_display: Some("Thou"), identifier: Some("THOU"), scalar: UN_SC_MIL, bias: 0.0, flag: B_UNIT_DEF_NONE },
];
/// `static bUnitDef buWavelengthLenDef[]`.
pub const BUWAVELENGTHLENDEF: &[BUnitDef] = &[
    BUnitDef { name: "millimeter", name_plural: Some("millimeters"), name_short: Some("mm"), name_alt: None, name_display: Some("Millimeters"), identifier: None, scalar: 1e6, bias: 0.0, flag: B_UNIT_DEF_NONE },
    BUnitDef { name: "micrometer", name_plural: Some("micrometers"), name_short: Some("µm"), name_alt: Some("um"), name_display: Some("Micrometers"), identifier: None, scalar: 1e3, bias: 0.0, flag: B_UNIT_DEF_NONE },
    BUnitDef { name: "nanometer", name_plural: Some("nanometers"), name_short: Some("nm"), name_alt: None, name_display: Some("Nanometers"), identifier: None, scalar: 1.0, bias: 0.0, flag: B_UNIT_DEF_NONE },
    BUnitDef { name: "picometer", name_plural: Some("picometers"), name_short: Some("pm"), name_alt: None, name_display: Some("Picometers"), identifier: None, scalar: 1e-3, bias: 0.0, flag: B_UNIT_DEF_NONE },
];
/// `static bUnitDef buMetricAreaDef[]`.
pub const BUMETRICAREADEF: &[BUnitDef] = &[
    BUnitDef { name: "square kilometer", name_plural: Some("square kilometers"), name_short: Some("km\u{b2}"), name_alt: Some("km2"), name_display: Some("Square Kilometers"), identifier: None, scalar: UN_SC_KM * UN_SC_KM, bias: 0.0, flag: B_UNIT_DEF_NONE },
    BUnitDef { name: "square hectometer", name_plural: Some("square hectometers"), name_short: Some("hm\u{b2}"), name_alt: Some("hm2"), name_display: Some("Square Hectometers"), identifier: None, scalar: UN_SC_HM * UN_SC_HM, bias: 0.0, flag: B_UNIT_DEF_SUPPRESS },
    BUnitDef { name: "square dekameter", name_plural: Some("square dekameters"), name_short: Some("dam\u{b2}"), name_alt: Some("dam2"), name_display: Some("Square Dekameters"), identifier: None, scalar: UN_SC_DAM * UN_SC_DAM, bias: 0.0, flag: B_UNIT_DEF_SUPPRESS },
    BUnitDef { name: "square meter", name_plural: Some("square meters"), name_short: Some("m\u{b2}"), name_alt: Some("m2"), name_display: Some("Square Meters"), identifier: None, scalar: UN_SC_M * UN_SC_M, bias: 0.0, flag: B_UNIT_DEF_NONE },
    BUnitDef { name: "square decimeter", name_plural: Some("square decimeters"), name_short: Some("dm\u{b2}"), name_alt: Some("dm2"), name_display: Some("Square Decimeters"), identifier: None, scalar: UN_SC_DM * UN_SC_DM, bias: 0.0, flag: B_UNIT_DEF_SUPPRESS },
    BUnitDef { name: "square centimeter", name_plural: Some("square centimeters"), name_short: Some("cm\u{b2}"), name_alt: Some("cm2"), name_display: Some("Square Centimeters"), identifier: None, scalar: UN_SC_CM * UN_SC_CM, bias: 0.0, flag: B_UNIT_DEF_NONE },
    BUnitDef { name: "square millimeter", name_plural: Some("square millimeters"), name_short: Some("mm\u{b2}"), name_alt: Some("mm2"), name_display: Some("Square Millimeters"), identifier: None, scalar: UN_SC_MM * UN_SC_MM, bias: 0.0, flag: B_UNIT_DEF_NONE | B_UNIT_DEF_TENTH },
    BUnitDef { name: "square micrometer", name_plural: Some("square micrometers"), name_short: Some("µm\u{b2}"), name_alt: Some("um2"), name_display: Some("Square Micrometers"), identifier: None, scalar: UN_SC_UM * UN_SC_UM, bias: 0.0, flag: B_UNIT_DEF_NONE },
];
/// `static bUnitDef buImperialAreaDef[]`.
pub const BUIMPERIALAREADEF: &[BUnitDef] = &[
    BUnitDef { name: "square mile", name_plural: Some("square miles"), name_short: Some("sq mi"), name_alt: None, name_display: Some("Square Miles"), identifier: None, scalar: UN_SC_MI * UN_SC_MI, bias: 0.0, flag: B_UNIT_DEF_NONE },
    BUnitDef { name: "square furlong", name_plural: Some("square furlongs"), name_short: Some("sq fur"), name_alt: None, name_display: Some("Square Furlongs"), identifier: None, scalar: UN_SC_FUR * UN_SC_FUR, bias: 0.0, flag: B_UNIT_DEF_SUPPRESS },
    BUnitDef { name: "square chain", name_plural: Some("square chains"), name_short: Some("sq ch"), name_alt: None, name_display: Some("Square Chains"), identifier: None, scalar: UN_SC_CH * UN_SC_CH, bias: 0.0, flag: B_UNIT_DEF_SUPPRESS },
    BUnitDef { name: "square yard", name_plural: Some("square yards"), name_short: Some("sq yd"), name_alt: None, name_display: Some("Square Yards"), identifier: None, scalar: UN_SC_YD * UN_SC_YD, bias: 0.0, flag: B_UNIT_DEF_NONE },
    BUnitDef { name: "square foot", name_plural: Some("square feet"), name_short: Some("sq ft"), name_alt: None, name_display: Some("Square Feet"), identifier: None, scalar: UN_SC_FT * UN_SC_FT, bias: 0.0, flag: B_UNIT_DEF_NONE },
    BUnitDef { name: "square inch", name_plural: Some("square inches"), name_short: Some("sq in"), name_alt: None, name_display: Some("Square Inches"), identifier: None, scalar: UN_SC_IN * UN_SC_IN, bias: 0.0, flag: B_UNIT_DEF_NONE },
    BUnitDef { name: "square thou", name_plural: Some("square thou"), name_short: Some("sq mil"), name_alt: None, name_display: Some("Square Thou"), identifier: None, scalar: UN_SC_MIL * UN_SC_MIL, bias: 0.0, flag: B_UNIT_DEF_NONE },
];
/// `static bUnitDef buMetricVolDef[]`.
pub const BUMETRICVOLDEF: &[BUnitDef] = &[
    BUnitDef { name: "cubic kilometer", name_plural: Some("cubic kilometers"), name_short: Some("km\u{b3}"), name_alt: Some("km3"), name_display: Some("Cubic Kilometers"), identifier: None, scalar: UN_SC_KM * UN_SC_KM * UN_SC_KM, bias: 0.0, flag: B_UNIT_DEF_NONE },
    BUnitDef { name: "cubic hectometer", name_plural: Some("cubic hectometers"), name_short: Some("hm\u{b3}"), name_alt: Some("hm3"), name_display: Some("Cubic Hectometers"), identifier: None, scalar: UN_SC_HM * UN_SC_HM * UN_SC_HM, bias: 0.0, flag: B_UNIT_DEF_SUPPRESS },
    BUnitDef { name: "cubic dekameter", name_plural: Some("cubic dekameters"), name_short: Some("dam\u{b3}"), name_alt: Some("dam3"), name_display: Some("Cubic Dekameters"), identifier: None, scalar: UN_SC_DAM * UN_SC_DAM * UN_SC_DAM, bias: 0.0, flag: B_UNIT_DEF_SUPPRESS },
    BUnitDef { name: "cubic meter", name_plural: Some("cubic meters"), name_short: Some("m\u{b3}"), name_alt: Some("m3"), name_display: Some("Cubic Meters"), identifier: None, scalar: UN_SC_M * UN_SC_M * UN_SC_M, bias: 0.0, flag: B_UNIT_DEF_NONE },
    BUnitDef { name: "cubic decimeter", name_plural: Some("cubic decimeters"), name_short: Some("dm\u{b3}"), name_alt: Some("dm3"), name_display: Some("Cubic Decimeters"), identifier: None, scalar: UN_SC_DM * UN_SC_DM * UN_SC_DM, bias: 0.0, flag: B_UNIT_DEF_SUPPRESS },
    BUnitDef { name: "cubic centimeter", name_plural: Some("cubic centimeters"), name_short: Some("cm\u{b3}"), name_alt: Some("cm3"), name_display: Some("Cubic Centimeters"), identifier: None, scalar: UN_SC_CM * UN_SC_CM * UN_SC_CM, bias: 0.0, flag: B_UNIT_DEF_NONE },
    BUnitDef { name: "cubic millimeter", name_plural: Some("cubic millimeters"), name_short: Some("mm\u{b3}"), name_alt: Some("mm3"), name_display: Some("Cubic Millimeters"), identifier: None, scalar: UN_SC_MM * UN_SC_MM * UN_SC_MM, bias: 0.0, flag: B_UNIT_DEF_NONE | B_UNIT_DEF_TENTH },
    BUnitDef { name: "cubic micrometer", name_plural: Some("cubic micrometers"), name_short: Some("µm\u{b3}"), name_alt: Some("um3"), name_display: Some("Cubic Micrometers"), identifier: None, scalar: UN_SC_UM * UN_SC_UM * UN_SC_UM, bias: 0.0, flag: B_UNIT_DEF_NONE },
];
/// `static bUnitDef buImperialVolDef[]`.
pub const BUIMPERIALVOLDEF: &[BUnitDef] = &[
    BUnitDef { name: "cubic mile", name_plural: Some("cubic miles"), name_short: Some("cu mi"), name_alt: None, name_display: Some("Cubic Miles"), identifier: None, scalar: UN_SC_MI * UN_SC_MI * UN_SC_MI, bias: 0.0, flag: B_UNIT_DEF_NONE },
    BUnitDef { name: "cubic furlong", name_plural: Some("cubic furlongs"), name_short: Some("cu fur"), name_alt: None, name_display: Some("Cubic Furlongs"), identifier: None, scalar: UN_SC_FUR * UN_SC_FUR * UN_SC_FUR, bias: 0.0, flag: B_UNIT_DEF_SUPPRESS },
    BUnitDef { name: "cubic chain", name_plural: Some("cubic chains"), name_short: Some("cu ch"), name_alt: None, name_display: Some("Cubic Chains"), identifier: None, scalar: UN_SC_CH * UN_SC_CH * UN_SC_CH, bias: 0.0, flag: B_UNIT_DEF_SUPPRESS },
    BUnitDef { name: "cubic yard", name_plural: Some("cubic yards"), name_short: Some("cu yd"), name_alt: None, name_display: Some("Cubic Yards"), identifier: None, scalar: UN_SC_YD * UN_SC_YD * UN_SC_YD, bias: 0.0, flag: B_UNIT_DEF_NONE },
    BUnitDef { name: "cubic foot", name_plural: Some("cubic feet"), name_short: Some("cu ft"), name_alt: None, name_display: Some("Cubic Feet"), identifier: None, scalar: UN_SC_FT * UN_SC_FT * UN_SC_FT, bias: 0.0, flag: B_UNIT_DEF_NONE },
    BUnitDef { name: "cubic inch", name_plural: Some("cubic inches"), name_short: Some("cu in"), name_alt: None, name_display: Some("Cubic Inches"), identifier: None, scalar: UN_SC_IN * UN_SC_IN * UN_SC_IN, bias: 0.0, flag: B_UNIT_DEF_NONE },
    BUnitDef { name: "cubic thou", name_plural: Some("cubic thou"), name_short: Some("cu mil"), name_alt: None, name_display: Some("Cubic Thou"), identifier: None, scalar: UN_SC_MIL * UN_SC_MIL * UN_SC_MIL, bias: 0.0, flag: B_UNIT_DEF_NONE },
];
/// `static bUnitDef buMetricMassDef[]`.
pub const BUMETRICMASSDEF: &[BUnitDef] = &[
    BUnitDef { name: "tonne", name_plural: Some("tonnes"), name_short: Some("t"), name_alt: Some("ton"), name_display: Some("Tonnes"), identifier: Some("TONNES"), scalar: UN_SC_MTON, bias: 0.0, flag: B_UNIT_DEF_NONE },
    BUnitDef { name: "quintal", name_plural: Some("quintals"), name_short: Some("ql"), name_alt: Some("q"), name_display: Some("100 Kilograms"), identifier: Some("QUINTALS"), scalar: UN_SC_QL, bias: 0.0, flag: B_UNIT_DEF_SUPPRESS },
    BUnitDef { name: "kilogram", name_plural: Some("kilograms"), name_short: Some("kg"), name_alt: None, name_display: Some("Kilograms"), identifier: Some("KILOGRAMS"), scalar: UN_SC_KG, bias: 0.0, flag: B_UNIT_DEF_NONE },
    BUnitDef { name: "hectogram", name_plural: Some("hectograms"), name_short: Some("hg"), name_alt: None, name_display: Some("Hectograms"), identifier: Some("HECTOGRAMS"), scalar: UN_SC_HG, bias: 0.0, flag: B_UNIT_DEF_SUPPRESS },
    BUnitDef { name: "dekagram", name_plural: Some("dekagrams"), name_short: Some("dag"), name_alt: None, name_display: Some("10 Grams"), identifier: Some("DEKAGRAMS"), scalar: UN_SC_DAG, bias: 0.0, flag: B_UNIT_DEF_SUPPRESS },
    BUnitDef { name: "gram", name_plural: Some("grams"), name_short: Some("g"), name_alt: None, name_display: Some("Grams"), identifier: Some("GRAMS"), scalar: UN_SC_G, bias: 0.0, flag: B_UNIT_DEF_NONE },
    BUnitDef { name: "milligram", name_plural: Some("milligrams"), name_short: Some("mg"), name_alt: None, name_display: Some("Milligrams"), identifier: Some("MILLIGRAMS"), scalar: UN_SC_MG, bias: 0.0, flag: B_UNIT_DEF_NONE },
];
/// `static bUnitDef buImperialMassDef[]`.
pub const BUIMPERIALMASSDEF: &[BUnitDef] = &[
    BUnitDef { name: "ton", name_plural: Some("tons"), name_short: Some("tn"), name_alt: None, name_display: Some("Tons"), identifier: Some("TONNES"), scalar: UN_SC_ITON, bias: 0.0, flag: B_UNIT_DEF_NONE },
    BUnitDef { name: "centum weight", name_plural: Some("centum weights"), name_short: Some("cwt"), name_alt: None, name_display: Some("Centum weights"), identifier: Some("CENTUM_WEIGHTS"), scalar: UN_SC_CWT, bias: 0.0, flag: B_UNIT_DEF_NONE },
    BUnitDef { name: "stone", name_plural: Some("stones"), name_short: Some("st"), name_alt: None, name_display: Some("Stones"), identifier: Some("STONES"), scalar: UN_SC_ST, bias: 0.0, flag: B_UNIT_DEF_NONE },
    BUnitDef { name: "pound", name_plural: Some("pounds"), name_short: Some("lb"), name_alt: None, name_display: Some("Pounds"), identifier: Some("POUNDS"), scalar: UN_SC_LB, bias: 0.0, flag: B_UNIT_DEF_NONE },
    BUnitDef { name: "ounce", name_plural: Some("ounces"), name_short: Some("oz"), name_alt: None, name_display: Some("Ounces"), identifier: Some("OUNCES"), scalar: UN_SC_OZ, bias: 0.0, flag: B_UNIT_DEF_NONE },
];
/// `static bUnitDef buMetricVelDef[]`.
pub const BUMETRICVELDEF: &[BUnitDef] = &[
    BUnitDef { name: "meter per second", name_plural: Some("meters per second"), name_short: Some("m/s"), name_alt: None, name_display: Some("Meters per second"), identifier: None, scalar: UN_SC_M, bias: 0.0, flag: B_UNIT_DEF_NONE },
    BUnitDef { name: "kilometer per hour", name_plural: Some("kilometers per hour"), name_short: Some("km/h"), name_alt: Some("kph"), name_display: Some("Kilometers per hour"), identifier: None, scalar: UN_SC_KM / 3600.0, bias: 0.0, flag: B_UNIT_DEF_SUPPRESS },
];
/// `static bUnitDef buImperialVelDef[]`.
pub const BUIMPERIALVELDEF: &[BUnitDef] = &[
    BUnitDef { name: "foot per second", name_plural: Some("feet per second"), name_short: Some("ft/s"), name_alt: Some("fps"), name_display: Some("Feet per second"), identifier: None, scalar: UN_SC_FT, bias: 0.0, flag: B_UNIT_DEF_NONE },
    BUnitDef { name: "mile per hour", name_plural: Some("miles per hour"), name_short: Some("mph"), name_alt: None, name_display: Some("Miles per hour"), identifier: None, scalar: UN_SC_MI / 3600.0, bias: 0.0, flag: B_UNIT_DEF_SUPPRESS },
];
/// `static bUnitDef buMetricAclDef[]`.
pub const BUMETRICACLDEF: &[BUnitDef] = &[
    BUnitDef { name: "meter per second squared", name_plural: Some("meters per second squared"), name_short: Some("m/s\u{b2}"), name_alt: Some("m/s2"), name_display: Some("Meters per second squared"), identifier: None, scalar: UN_SC_M, bias: 0.0, flag: B_UNIT_DEF_NONE },
];
/// `static bUnitDef buImperialAclDef[]`.
pub const BUIMPERIALACLDEF: &[BUnitDef] = &[
    BUnitDef { name: "foot per second squared", name_plural: Some("feet per second squared"), name_short: Some("ft/s\u{b2}"), name_alt: Some("ft/s2"), name_display: Some("Feet per second squared"), identifier: None, scalar: UN_SC_FT, bias: 0.0, flag: B_UNIT_DEF_NONE },
];
/// `static bUnitDef buNaturalTimeDef[]`.
pub const BUNATURALTIMEDEF: &[BUnitDef] = &[
    BUnitDef { name: "day", name_plural: Some("days"), name_short: Some("d"), name_alt: None, name_display: Some("Days"), identifier: Some("DAYS"), scalar: 86400.0, bias: 0.0, flag: B_UNIT_DEF_NONE },
    BUnitDef { name: "hour", name_plural: Some("hours"), name_short: Some("h"), name_alt: Some("hr"), name_display: Some("Hours"), identifier: Some("HOURS"), scalar: 3600.0, bias: 0.0, flag: B_UNIT_DEF_NONE },
    BUnitDef { name: "minute", name_plural: Some("minutes"), name_short: Some("min"), name_alt: Some("m"), name_display: Some("Minutes"), identifier: Some("MINUTES"), scalar: 60.0, bias: 0.0, flag: B_UNIT_DEF_NONE },
    BUnitDef { name: "second", name_plural: Some("seconds"), name_short: Some("s"), name_alt: Some("sec"), name_display: Some("Seconds"), identifier: Some("SECONDS"), scalar: 1.0, bias: 0.0, flag: B_UNIT_DEF_NONE },
    BUnitDef { name: "millisecond", name_plural: Some("milliseconds"), name_short: Some("ms"), name_alt: None, name_display: Some("Milliseconds"), identifier: Some("MILLISECONDS"), scalar: 0.001, bias: 0.0, flag: B_UNIT_DEF_NONE },
    BUnitDef { name: "microsecond", name_plural: Some("microseconds"), name_short: Some("µs"), name_alt: Some("us"), name_display: Some("Microseconds"), identifier: Some("MICROSECONDS"), scalar: 0.000001, bias: 0.0, flag: B_UNIT_DEF_NONE },
];
/// `static bUnitDef buNaturalRotDef[]`.
pub const BUNATURALROTDEF: &[BUnitDef] = &[
    BUnitDef { name: "degree", name_plural: Some("degrees"), name_short: Some("\u{b0}"), name_alt: Some("d"), name_display: Some("Degrees"), identifier: Some("DEGREES"), scalar: std::f64::consts::PI / 180.0, bias: 0.0, flag: B_UNIT_DEF_NONE | B_UNIT_DEF_NO_SPACE },
    BUnitDef { name: "arcminute", name_plural: Some("arcminutes"), name_short: Some("'"), name_alt: Some("amin"), name_display: Some("Arcminutes"), identifier: Some("ARCMINUTES"), scalar: (M_PI / 180.0) / 60.0, bias: 0.0, flag: B_UNIT_DEF_SUPPRESS | B_UNIT_DEF_NO_SPACE },
    BUnitDef { name: "arcsecond", name_plural: Some("arcseconds"), name_short: Some("\""), name_alt: Some("asec"), name_display: Some("Arcseconds"), identifier: Some("ARCSECONDS"), scalar: (M_PI / 180.0) / 3600.0, bias: 0.0, flag: B_UNIT_DEF_SUPPRESS | B_UNIT_DEF_NO_SPACE },
    BUnitDef { name: "radian", name_plural: Some("radians"), name_short: Some("rad"), name_alt: Some("r"), name_display: Some("Radians"), identifier: Some("RADIANS"), scalar: 1.0, bias: 0.0, flag: B_UNIT_DEF_NONE },
    BUnitDef { name: "turn", name_plural: Some("turns"), name_short: Some("t"), name_alt: None, name_display: Some("Turns"), identifier: None, scalar: 1.0 / (std::f64::consts::PI * 2.0), bias: 0.0, flag: B_UNIT_DEF_NONE },
];
/// `static bUnitDef buCameraLenDef[]`.
pub const BUCAMERALENDEF: &[BUnitDef] = &[
    BUnitDef { name: "meter", name_plural: Some("meters"), name_short: Some("m"), name_alt: None, name_display: Some("Meters"), identifier: None, scalar: UN_SC_KM, bias: 0.0, flag: B_UNIT_DEF_NONE },
    BUnitDef { name: "decimeter", name_plural: Some("decimeters"), name_short: Some("dm"), name_alt: None, name_display: Some("10 Centimeters"), identifier: None, scalar: UN_SC_HM, bias: 0.0, flag: B_UNIT_DEF_SUPPRESS },
    BUnitDef { name: "centimeter", name_plural: Some("centimeters"), name_short: Some("cm"), name_alt: None, name_display: Some("Centimeters"), identifier: None, scalar: UN_SC_DAM, bias: 0.0, flag: B_UNIT_DEF_SUPPRESS },
    BUnitDef { name: "millimeter", name_plural: Some("millimeters"), name_short: Some("mm"), name_alt: None, name_display: Some("Millimeters"), identifier: None, scalar: UN_SC_M, bias: 0.0, flag: B_UNIT_DEF_NONE },
    BUnitDef { name: "micrometer", name_plural: Some("micrometers"), name_short: Some("µm"), name_alt: Some("um"), name_display: Some("Micrometers"), identifier: None, scalar: UN_SC_MM, bias: 0.0, flag: B_UNIT_DEF_SUPPRESS },
];
/// `static bUnitDef buPowerDef[]`.
pub const BUPOWERDEF: &[BUnitDef] = &[
    BUnitDef { name: "gigawatt", name_plural: Some("gigawatts"), name_short: Some("GW"), name_alt: None, name_display: Some("Gigawatts"), identifier: None, scalar: 1e9, bias: 0.0, flag: B_UNIT_DEF_NONE },
    BUnitDef { name: "megawatt", name_plural: Some("megawatts"), name_short: Some("MW"), name_alt: None, name_display: Some("Megawatts"), identifier: None, scalar: 1e6, bias: 0.0, flag: B_UNIT_DEF_CASE_SENSITIVE },
    BUnitDef { name: "kilowatt", name_plural: Some("kilowatts"), name_short: Some("kW"), name_alt: None, name_display: Some("Kilowatts"), identifier: None, scalar: 1e3, bias: 0.0, flag: B_UNIT_DEF_SUPPRESS },
    BUnitDef { name: "watt", name_plural: Some("watts"), name_short: Some("W"), name_alt: None, name_display: Some("Watts"), identifier: None, scalar: 1.0, bias: 0.0, flag: B_UNIT_DEF_NONE },
    BUnitDef { name: "milliwatt", name_plural: Some("milliwatts"), name_short: Some("mW"), name_alt: None, name_display: Some("Milliwatts"), identifier: None, scalar: 1e-3, bias: 0.0, flag: B_UNIT_DEF_CASE_SENSITIVE },
    BUnitDef { name: "microwatt", name_plural: Some("microwatts"), name_short: Some("µW"), name_alt: Some("uW"), name_display: Some("Microwatts"), identifier: None, scalar: 1e-6, bias: 0.0, flag: B_UNIT_DEF_NONE },
    BUnitDef { name: "nanowatt", name_plural: Some("nanowatts"), name_short: Some("nW"), name_alt: None, name_display: Some("Nanowatts"), identifier: None, scalar: 1e-9, bias: 0.0, flag: B_UNIT_DEF_NONE },
];
/// `static bUnitDef buMetricTempDef[]`.
pub const BUMETRICTEMPDEF: &[BUnitDef] = &[
    BUnitDef { name: "kelvin", name_plural: Some("kelvin"), name_short: Some("K"), name_alt: None, name_display: Some("Kelvin"), identifier: Some("KELVIN"), scalar: 1.0, bias: 0.0, flag: B_UNIT_DEF_NONE },
    BUnitDef { name: "celsius", name_plural: Some("celsius"), name_short: Some("\u{b0}C"), name_alt: Some("C"), name_display: Some("Celsius"), identifier: Some("CELSIUS"), scalar: 1.0, bias: 273.15, flag: B_UNIT_DEF_NONE },
];
/// `static bUnitDef buImperialTempDef[]`.
pub const BUIMPERIALTEMPDEF: &[BUnitDef] = &[
    BUnitDef { name: "kelvin", name_plural: Some("kelvin"), name_short: Some("K"), name_alt: None, name_display: Some("Kelvin"), identifier: Some("KELVIN"), scalar: 1.0, bias: 0.0, flag: B_UNIT_DEF_NONE },
    BUnitDef { name: "fahrenheit", name_plural: Some("fahrenheit"), name_short: Some("\u{b0}F"), name_alt: Some("F"), name_display: Some("Fahrenheit"), identifier: Some("FAHRENHEIT"), scalar: UN_SC_FAH, bias: 459.67, flag: B_UNIT_DEF_NONE },
];
/// `static bUnitDef buColorTempDef[]`.
pub const BUCOLORTEMPDEF: &[BUnitDef] = &[
    BUnitDef { name: "kelvin", name_plural: Some("kelvin"), name_short: Some("K"), name_alt: None, name_display: Some("Kelvin"), identifier: Some("KELVIN"), scalar: 1.0, bias: 0.0, flag: B_UNIT_DEF_NONE },
];
/// `static bUnitDef buFrequencyDef[]`.
pub const BUFREQUENCYDEF: &[BUnitDef] = &[
    BUnitDef { name: "hertz", name_plural: Some("hertz"), name_short: Some("Hz"), name_alt: None, name_display: Some("Hertz"), identifier: Some("HERTZ"), scalar: 1.0, bias: 0.0, flag: B_UNIT_DEF_NONE },
    BUnitDef { name: "kilohertz", name_plural: Some("kilohertz"), name_short: Some("kHz"), name_alt: None, name_display: Some("Kilohertz"), identifier: Some("KILOHERTZ"), scalar: 1e3, bias: 0.0, flag: B_UNIT_DEF_NONE },
];

const BU_DUMMY: BUnitCollection = coll!(BUDUMMYDEF, 0);
const BU_METRIC_LEN: BUnitCollection = coll!(BUMETRICLENDEF, 3);
const BU_IMPERIAL_LEN: BUnitCollection = coll!(BUIMPERIALLENDEF, 4);
const BU_WAVELENGTH_LEN: BUnitCollection = coll!(BUWAVELENGTHLENDEF, 0);
const BU_METRIC_AREA: BUnitCollection = coll!(BUMETRICAREADEF, 0);
const BU_IMPERIAL_AREA: BUnitCollection = coll!(BUIMPERIALAREADEF, 0);
const BU_METRIC_VOL: BUnitCollection = coll!(BUMETRICVOLDEF, 0);
const BU_IMPERIAL_VOL: BUnitCollection = coll!(BUIMPERIALVOLDEF, 0);
const BU_METRIC_MASS: BUnitCollection = coll!(BUMETRICMASSDEF, 0);
const BU_IMPERIAL_MASS: BUnitCollection = coll!(BUIMPERIALMASSDEF, 0);
const BU_METRIC_VEL: BUnitCollection = coll!(BUMETRICVELDEF, 0);
const BU_IMPERIAL_VEL: BUnitCollection = coll!(BUIMPERIALVELDEF, 0);
const BU_METRIC_ACL: BUnitCollection = coll!(BUMETRICACLDEF, 0);
const BU_IMPERIAL_ACL: BUnitCollection = coll!(BUIMPERIALACLDEF, 0);
const BU_NATURAL_TIME: BUnitCollection = coll!(BUNATURALTIMEDEF, 0);
const BU_NATURAL_ROT: BUnitCollection = coll!(BUNATURALROTDEF, 0);
const BU_CAMERA_LEN: BUnitCollection = coll!(BUCAMERALENDEF, 0);
const BU_POWER: BUnitCollection = coll!(BUPOWERDEF, 0);
const BU_METRIC_TEMP: BUnitCollection = coll!(BUMETRICTEMPDEF, 0);
const BU_IMPERIAL_TEMP: BUnitCollection = coll!(BUIMPERIALTEMPDEF, 0);
const BU_COLOR_TEMP: BUnitCollection = coll!(BUCOLORTEMPDEF, 0);
const BU_FREQUENCY: BUnitCollection = coll!(BUFREQUENCYDEF, 0);

/// `bUnitSystems` — `[system][type]` table (verbatim cell
/// list: Natural/Metric/Imperial rows).
fn unit_get_system(system: i32, type_: i32) -> Option<&'static BUnitCollection> {
    let row: [Option<&BUnitCollection>; B_UNIT_TYPE_TOT as usize] = match system {
        UNIT_SYSTEM_NONE => [
            None, None, None, None, None,
            Some(&BU_NATURAL_ROT), Some(&BU_NATURAL_TIME), Some(&BU_NATURAL_TIME),
            None, None, None, None, None, None, None, None,
        ],
        UNIT_SYSTEM_METRIC => [
            None,
            Some(&BU_METRIC_LEN), Some(&BU_METRIC_AREA), Some(&BU_METRIC_VOL),
            Some(&BU_METRIC_MASS), Some(&BU_NATURAL_ROT), Some(&BU_NATURAL_TIME),
            Some(&BU_NATURAL_TIME), Some(&BU_METRIC_VEL), Some(&BU_METRIC_ACL),
            Some(&BU_CAMERA_LEN), Some(&BU_POWER), Some(&BU_METRIC_TEMP),
            Some(&BU_WAVELENGTH_LEN), Some(&BU_COLOR_TEMP), Some(&BU_FREQUENCY),
        ],
        UNIT_SYSTEM_IMPERIAL => [
            None,
            Some(&BU_IMPERIAL_LEN), Some(&BU_IMPERIAL_AREA), Some(&BU_IMPERIAL_VOL),
            Some(&BU_IMPERIAL_MASS), Some(&BU_NATURAL_ROT), Some(&BU_NATURAL_TIME),
            Some(&BU_NATURAL_TIME), Some(&BU_IMPERIAL_VEL), Some(&BU_IMPERIAL_ACL),
            Some(&BU_CAMERA_LEN), Some(&BU_POWER), Some(&BU_IMPERIAL_TEMP),
            Some(&BU_WAVELENGTH_LEN), Some(&BU_COLOR_TEMP), Some(&BU_FREQUENCY),
        ],
        _ => return None,
    };
    if type_ < 0 || type_ >= B_UNIT_TYPE_TOT {
        return None;
    }
    row[type_ as usize]
}

/// `unit_default`.
fn unit_default(usys: &BUnitCollection) -> &'static BUnitDef {
    &usys.units[usys.base_unit]
}

/// `unit_best_fit` — first unit whose scalar (scaled by the
/// TENTH guard) fits `value`, else the default.
fn unit_best_fit<'a>(
    value: f64,
    usys: &'a BUnitCollection,
    unit_start: Option<&'a BUnitDef>,
    suppress: bool,
) -> &'a BUnitDef {
    let value_abs = value.abs();
    let start = match unit_start {
        Some(u) => {
            /* find u in the table to continue from it */
            usys.units
                .iter()
                .position(|x| std::ptr::eq(x, u))
                .unwrap_or(0)
        }
        None => 0,
    };
    for unit in &usys.units[start..] {
        if suppress && (unit.flag & B_UNIT_DEF_SUPPRESS) != 0 {
            continue;
        }
        if (unit.flag & B_UNIT_DEF_TENTH) != 0 {
            if value_abs >= unit.scalar * (0.1 - EPS) {
                return unit;
            }
        } else if value_abs >= unit.scalar * (1.0 - EPS) {
            return unit;
        }
    }
    unit_default(usys)
}

/// `unit_dual_convert` — split into a large+small unit pair.
fn unit_dual_convert<'a>(
    value: f64,
    usys: &'a BUnitCollection,
    main_unit: Option<&'a BUnitDef>,
) -> (&'a BUnitDef, &'a BUnitDef, f64, f64) {
    let unit = main_unit.unwrap_or_else(|| unit_best_fit(value, usys, None, true));
    let scaled_value = value / unit.scalar;
    let value_a = scaled_value.trunc() * unit.scalar;
    let value_b = value - value_a;
    let unit_b = unit_best_fit(value_b, usys, Some(unit), true);
    (unit, unit_b, value_a, value_b)
}

/// `is_valid_unit_collection`.
fn is_valid_unit_collection(usys: &BUnitCollection) -> bool {
    !usys.units.is_empty() && !usys.units[0].name.is_empty()
}

/// `integer_digits_d` helper — digits left of the point.
fn integer_digits_d(v: f64) -> i32 {
    let v = v.abs();
    if v < 1.0 {
        return 0;
    }
    v.log10().trunc() as i32 + 1
}

/// `unit_as_string` — format `value` in `unit` with suffix
/// (verbatim: prec adapt → `%.*f` → suffix).
pub fn unit_as_string(
    value: f64,
    mut prec: i32,
    variable_width: bool,
    usys: &BUnitCollection,
    unit: Option<&BUnitDef>,
    pad: char,
    do_unit_suffix: bool,
) -> String {
    let unit = unit.unwrap_or_else(|| {
        if value == 0.0 {
            unit_default(usys)
        } else {
            unit_best_fit(value, usys, None, true)
        }
    });
    let value_conv = (value / unit.scalar) - unit.bias;
    if variable_width {
        prec -= integer_digits_d(value_conv);
    }
    prec = prec.clamp(0, 6);
    let mut s = format!("{:.*}", prec as usize, value_conv);
    if do_unit_suffix {
        if let Some(short) = unit.name_short.or(unit.name_alt) {
            if (unit.flag & B_UNIT_DEF_NO_SPACE) == 0 {
                s.push(' ');
            }
            s.push_str(short);
        }
    }
    if pad != '\0' {
        while s.len() < 12 {
            s.insert(0, pad);
        }
    }
    s
}

/// `unit_as_string_split_pair` — "2ft 3in" dual-unit string
/// (returns `None` when the second unit isn't smaller).
pub fn unit_as_string_split_pair(
    value: f64,
    mut prec: i32,
    variable_width: bool,
    usys: &BUnitCollection,
    main_unit: Option<&BUnitDef>,
) -> Option<String> {
    let (unit_a, unit_b, value_a, value_b) = unit_dual_convert(value, usys, main_unit);
    if unit_b.scalar < unit_a.scalar {
        let mut s = unit_as_string(value_a, prec, true, usys, Some(unit_a), '\0', true);
        if variable_width {
            prec -= integer_digits_d(value_a / unit_b.scalar)
                - integer_digits_d(value_b / unit_b.scalar);
            prec = prec.max(0);
        }
        s.push(' ');
        s.push_str(&unit_as_string(
            value_b,
            prec,
            variable_width,
            usys,
            Some(unit_b),
            '\0',
            true,
        ));
        return Some(s);
    }
    None
}

/// `unit_should_be_split`.
fn unit_should_be_split(type_: i32) -> bool {
    matches!(type_, B_UNIT_LENGTH | B_UNIT_MASS | B_UNIT_TIME | B_UNIT_CAMERA)
}

/// `PreferredUnits` — distilled `UnitSettings` ints.
#[derive(Clone, Copy)]
pub struct PreferredUnits {
    /// `system`.
    pub system: i32,
    /// `rotation` — `USER_UNIT_ROT_*`.
    pub rotation: i32,
    /// `length` — `USER_UNIT_ADAPTIVE` or collection index.
    pub length: i32,
    /// `mass`.
    pub mass: i32,
    /// `time`.
    pub time: i32,
    /// `temperature`.
    pub temperature: i32,
}

/// `is_unit_adaptive`.
fn is_unit_adaptive(type_: i32, units: &PreferredUnits) -> bool {
    match type_ {
        B_UNIT_LENGTH | B_UNIT_AREA | B_UNIT_VOLUME => units.length == USER_UNIT_ADAPTIVE,
        B_UNIT_MASS => units.mass == USER_UNIT_ADAPTIVE,
        B_UNIT_TIME => units.time == USER_UNIT_ADAPTIVE,
        B_UNIT_TEMPERATURE => units.temperature == USER_UNIT_ADAPTIVE,
        _ => false,
    }
}

/// `get_preferred_display_unit_index_if_used` (verbatim
/// switch incl. rotation `USER_UNIT_ROT_RADIANS` → 3).
fn get_preferred_display_unit_index_if_used(type_: i32, units: &PreferredUnits) -> i32 {
    let usys = match unit_get_system(units.system, type_) {
        Some(u) if is_valid_unit_collection(u) => u,
        _ => return -1,
    };
    if is_unit_adaptive(type_, units) {
        return -1;
    }
    let max_offset = (usys.units.len() - 1) as i32;
    match type_ {
        B_UNIT_LENGTH | B_UNIT_AREA | B_UNIT_VOLUME => units.length.min(max_offset),
        B_UNIT_MASS => units.mass.min(max_offset),
        B_UNIT_TIME => units.time.min(max_offset),
        B_UNIT_ROTATION => {
            if units.rotation == 0 {
                0
            } else if units.rotation == USER_UNIT_ROT_RADIANS {
                3
            } else {
                -1
            }
        }
        B_UNIT_TEMPERATURE => units.temperature.min(max_offset),
        _ => -1,
    }
}

/// `get_preferred_display_unit_if_used`.
fn get_preferred_display_unit_if_used(
    type_: i32,
    units: &PreferredUnits,
) -> Option<&'static BUnitDef> {
    let usys = unit_get_system(units.system, type_)?;
    let index = get_preferred_display_unit_index_if_used(type_, units);
    if index == -1 {
        return None;
    }
    usys.units.get(index as usize)
}

/// `unit_as_string_main` — format `value` honoring preferred
/// units + optional split (verbatim control flow).
pub fn unit_as_string_main(
    value: f64,
    prec: i32,
    type_: i32,
    split: bool,
    pad: bool,
    do_unit_suffix: bool,
    units: &PreferredUnits,
    variable_width: bool,
) -> String {
    let mut usys = unit_get_system(units.system, type_);
    let mut main_unit = None;
    if !usys.map_or(false, is_valid_unit_collection) {
        usys = Some(&BU_DUMMY);
    } else {
        main_unit = get_preferred_display_unit_if_used(type_, units);
    }
    let usys = usys.unwrap();
    if split && unit_should_be_split(type_) {
        if let Some(s) =
            unit_as_string_split_pair(value, prec, variable_width, usys, main_unit)
        {
            return s;
        }
    }
    unit_as_string(
        value,
        prec,
        variable_width,
        usys,
        main_unit,
        if pad { ' ' } else { '\0' },
        do_unit_suffix,
    )
}

/// `BKE_unit_value_as_string_adaptive`.
pub fn unit_value_as_string_adaptive(
    value: f64,
    prec: i32,
    type_: i32,
    units: &PreferredUnits,
) -> String {
    unit_as_string_main(value, prec, type_, true, false, true, units, true)
}

/// `BKE_unit_value_as_string`.
pub fn unit_value_as_string(
    value: f64,
    prec: i32,
    type_: i32,
    units: &PreferredUnits,
    pad: char,
    do_unit_suffix: bool,
) -> String {
    unit_as_string_main(value, prec, type_, true, pad != '\0', do_unit_suffix, units, true)
}

/// `BKE_unit_value_scale` (verbatim switch incl. `pow`
/// exponents).
pub fn unit_value_scale(scale_length: f64, system: i32, unit_type: i32, value: f64) -> f64 {
    if system == USER_UNIT_NONE {
        return value;
    }
    match unit_type {
        B_UNIT_LENGTH | B_UNIT_VELOCITY | B_UNIT_ACCELERATION => value * scale_length,
        B_UNIT_AREA | B_UNIT_POWER => value * scale_length * scale_length,
        B_UNIT_VOLUME | B_UNIT_MASS => value * scale_length * scale_length * scale_length,
        _ => value,
    }
}

/// `BKE_unit_closest_scalar`.
pub fn unit_closest_scalar(value: f64, system: i32, type_: i32) -> f64 {
    match unit_get_system(system, type_) {
        None => -1.0,
        Some(usys) => unit_best_fit(value, usys, None, true).scalar,
    }
}

/// `BKE_unit_base_scalar`.
pub fn unit_base_scalar(system: i32, type_: i32) -> f64 {
    unit_get_system(system, type_).map_or(1.0, |u| unit_default(u).scalar)
}

/// `BKE_unit_is_valid`.
pub fn unit_is_valid(system: i32, type_: i32) -> bool {
    !(system < 0 || system > UNIT_SYSTEM_TOT || type_ < 0 || type_ > B_UNIT_TYPE_TOT)
}

/// `BKE_unit_system_get`.
pub fn unit_system_get(system: i32, type_: i32) -> Option<&'static BUnitCollection> {
    unit_get_system(system, type_)
}

/// `BKE_unit_base_get`.
pub fn unit_base_get(usys: &BUnitCollection) -> usize {
    usys.base_unit
}

/// `BKE_unit_base_of_type_get`.
pub fn unit_base_of_type_get(system: i32, type_: i32) -> i32 {
    match unit_get_system(system, type_) {
        Some(usys) if is_valid_unit_collection(usys) => usys.base_unit as i32,
        _ => -1,
    }
}

/// `BKE_unit_name_get`.
pub fn unit_name_get(usys: &BUnitCollection, index: usize) -> Option<&'static str> {
    usys.units.get(index).map(|u| u.name)
}

/// `BKE_unit_display_name_get`.
pub fn unit_display_name_get(usys: &BUnitCollection, index: usize) -> Option<&'static str> {
    usys.units
        .get(index)
        .and_then(|u| u.name_display.or(u.name_plural).or(Some(u.name)))
}

/// `BKE_unit_display_name_short_get`.
pub fn unit_display_name_short_get(
    usys: &BUnitCollection,
    index: usize,
) -> Option<&'static str> {
    usys.units
        .get(index)
        .and_then(|u| u.name_short.or(u.name_alt))
}

/// `BKE_unit_identifier_get`.
pub fn unit_identifier_get(usys: &BUnitCollection, index: usize) -> Option<&'static str> {
    usys.units.get(index).and_then(|u| u.identifier)
}

/// `BKE_unit_scalar_get`.
pub fn unit_scalar_get(usys: &BUnitCollection, index: usize) -> f64 {
    usys.units.get(index).map_or(1.0, |u| u.scalar)
}

/// `BKE_unit_is_suppressed`.
pub fn unit_is_suppressed(usys: &BUnitCollection, index: usize) -> bool {
    usys.units
        .get(index)
        .map_or(false, |u| (u.flag & B_UNIT_DEF_SUPPRESS) != 0)
}

/// `BKE_unit_is_adaptive`.
pub fn unit_is_adaptive(units: &PreferredUnits, type_: i32) -> bool {
    is_unit_adaptive(type_, units)
}

/// `BKE_unit_apply_preferred_unit` — `value * scalar + bias`.
pub fn unit_apply_preferred_unit(units: &PreferredUnits, type_: i32, value: f64) -> f64 {
    let unit = get_preferred_display_unit_if_used(type_, units);
    let scalar = unit.map_or_else(|| unit_base_scalar(units.system, type_), |u| u.scalar);
    let bias = unit.map_or(0.0, |u| u.bias);
    value * scalar + bias
}

/// `BKE_unit_name_to_alt` — substitute short names with alt
/// (ASCII) spellings in `orig` (verbatim scan).
pub fn unit_name_to_alt(orig: &str, system: i32, type_: i32) -> String {
    let usys = match unit_get_system(system, type_) {
        Some(u) => u,
        None => return orig.to_string(),
    };
    let mut out = String::new();
    let mut rest = orig;
    'outer: while !rest.is_empty() {
        for unit in usys.units {
            let (short, alt) = match (unit.name_short, unit.name_alt) {
                (Some(s), Some(a)) => (s, a),
                _ => continue,
            };
            let case_sensitive = (unit.flag & B_UNIT_DEF_CASE_SENSITIVE) != 0;
            let hay = if case_sensitive {
                rest.to_string()
            } else {
                rest.to_lowercase()
            };
            let needle = if case_sensitive {
                short.to_string()
            } else {
                short.to_lowercase()
            };
            if let Some(off) = hay.find(&needle) {
                out.push_str(&rest[..off]);
                out.push_str(alt);
                rest = &rest[off + short.len()..];
                continue 'outer;
            }
        }
        break;
    }
    out.push_str(rest);
    out
}

/// `BKE_unit_string_contains_unit` — any system's collection
/// mentions a unit name/short in `s`.
pub fn unit_string_contains_unit(s: &str, type_: i32) -> bool {
    for system in 0..UNIT_SYSTEM_TOT {
        let usys = match unit_get_system(system, type_) {
            Some(u) if is_valid_unit_collection(u) => u,
            _ => continue,
        };
        for unit in usys.units {
            let names = [unit.name_short, unit.name_alt, Some(unit.name)];
            for n in names.into_iter().flatten() {
                if !n.is_empty() && s.contains(n) {
                    return true;
                }
            }
        }
    }
    false
}

/// `BKE_unit_scale_length` — blender-unit → user scale
/// (`settings.scale_length` semantics: meters → chosen).
pub fn unit_scale_length(scale_length: f64, system: i32) -> f64 {
    if system == USER_UNIT_NONE {
        1.0
    } else {
        scale_length
    }
}
