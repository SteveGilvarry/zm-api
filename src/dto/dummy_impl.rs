// This file contains implementation of the fake::Dummy trait for various types used in our DTOs
use crate::entity::sea_orm_active_enums::*;
use fake::{Dummy, Faker};
use rand::prelude::*;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// Create a newtype wrapper for Decimal
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DecimalWrapper(pub Decimal);

// Implement conversion from DecimalWrapper to Decimal
impl From<DecimalWrapper> for Decimal {
    fn from(wrapper: DecimalWrapper) -> Self {
        wrapper.0
    }
}

// Implement conversion from Decimal to DecimalWrapper
impl From<Decimal> for DecimalWrapper {
    fn from(decimal: Decimal) -> Self {
        DecimalWrapper(decimal)
    }
}

// Implement Dummy for DecimalWrapper (our own type)
impl Dummy<Faker> for DecimalWrapper {
    fn dummy_with_rng<R: Rng + ?Sized>(_: &Faker, rng: &mut R) -> Self {
        let integer_part = rng.gen_range(0..1000);
        let decimal_part = rng.gen_range(0..1000000);
        DecimalWrapper(Decimal::new(integer_part * 1000000 + decimal_part, 6))
    }
}

// Implement Dummy for EventCloseMode
impl Dummy<Faker> for EventCloseMode {
    fn dummy_with_rng<R: Rng + ?Sized>(_: &Faker, rng: &mut R) -> Self {
        let variants = [
            EventCloseMode::System,
            EventCloseMode::Time,
            EventCloseMode::Duration,
            EventCloseMode::Idle,
            EventCloseMode::Alarm,
        ];
        variants[rng.gen_range(0..variants.len())].clone()
    }
}

// Implement Dummy for DefaultCodec
impl Dummy<Faker> for DefaultCodec {
    fn dummy_with_rng<R: Rng + ?Sized>(_: &Faker, rng: &mut R) -> Self {
        let variants = [
            DefaultCodec::Auto,
            DefaultCodec::Mp4,
            DefaultCodec::Mjpeg,
        ];
        variants[rng.gen_range(0..variants.len())].clone()
    }
}

// Implement Dummy for Importance
impl Dummy<Faker> for Importance {
    fn dummy_with_rng<R: Rng + ?Sized>(_: &Faker, rng: &mut R) -> Self {
        let variants = [
            Importance::Normal,
            Importance::Less,
            Importance::Not,
        ];
        variants[rng.gen_range(0..variants.len())].clone()
    }
}

// Implement Dummy for Analysing
impl Dummy<Faker> for Analysing {
    fn dummy_with_rng<R: Rng + ?Sized>(_: &Faker, rng: &mut R) -> Self {
        let variants = [
            Analysing::None,
            Analysing::Always,
        ];
        variants[rng.gen_range(0..variants.len())].clone()
    }
}

// Implement Dummy for AnalysisSource
impl Dummy<Faker> for AnalysisSource {
    fn dummy_with_rng<R: Rng + ?Sized>(_: &Faker, rng: &mut R) -> Self {
        let variants = [
            AnalysisSource::Primary,
            AnalysisSource::Secondary,
        ];
        variants[rng.gen_range(0..variants.len())].clone()
    }
}

// Implement Dummy for AnalysisImage
impl Dummy<Faker> for AnalysisImage {
    fn dummy_with_rng<R: Rng + ?Sized>(_: &Faker, rng: &mut R) -> Self {
        let variants = [
            AnalysisImage::FullColour,
            AnalysisImage::YChannel,
        ];
        variants[rng.gen_range(0..variants.len())].clone()
    }
}

// Implement Dummy for Recording
impl Dummy<Faker> for Recording {
    fn dummy_with_rng<R: Rng + ?Sized>(_: &Faker, rng: &mut R) -> Self {
        let variants = [
            Recording::None,
            Recording::OnMotion,
            Recording::Always,
        ];
        variants[rng.gen_range(0..variants.len())].clone()
    }
}