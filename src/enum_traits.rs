use crate::entity::sea_orm_active_enums::*;
use sea_orm::ActiveEnum;
use std::fmt;

macro_rules! impl_display_for_enum {
    ($enum_type:ty) => {
        impl fmt::Display for $enum_type {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", <$enum_type as ActiveEnum>::to_value(self))
            }
        }
    };
}

// Implement Display for all the enums
impl_display_for_enum!(MonitorType);
impl_display_for_enum!(Function);
impl_display_for_enum!(Capturing);
impl_display_for_enum!(Decoding);
impl_display_for_enum!(Rtsp2WebType);
impl_display_for_enum!(Orientation);
impl_display_for_enum!(OutputContainer);
impl_display_for_enum!(RecordingSource);
impl_display_for_enum!(DefaultCodec);
impl_display_for_enum!(EventCloseMode);
impl_display_for_enum!(Importance);
impl_display_for_enum!(Analysing);
impl_display_for_enum!(AnalysisSource);
impl_display_for_enum!(AnalysisImage);
impl_display_for_enum!(Recording);
impl_display_for_enum!(Status);
impl_display_for_enum!(Stream);
impl_display_for_enum!(System);
impl_display_for_enum!(ZoneType);
impl_display_for_enum!(DeviceType);
impl_display_for_enum!(Units);
impl_display_for_enum!(FrameType);
impl_display_for_enum!(StorageType);
