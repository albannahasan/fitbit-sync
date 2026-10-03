use chrono::{DateTime, FixedOffset};
use serde::Deserialize;

pub mod service;

#[derive(Debug, Clone, Deserialize)]
pub struct CreateMealInput {
    // Required
    pub name: String,
    pub energy_kcal: i32,
    pub eaten_at: DateTime<FixedOffset>,
    pub idempotency_key: String,

    // Optional
    pub meal_type: Option<String>,
    pub protein_g: Option<f64>,
    pub carbohydrate_g: Option<f64>,
    pub fat_g: Option<f64>,
    pub fiber_g: Option<f64>,
    pub confidence: Option<f64>,
}

impl CreateMealInput {
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("Name cannot be blank.".to_owned());
        }

        if !(1..=20_000).contains(&self.energy_kcal) {
            return Err("Energy must be between 1 and 20,000 kcal.".to_owned());
        }

        if self.idempotency_key.trim().is_empty() {
            return Err("Idempotency key cannot be blank.".to_owned());
        }

        if let Some(meal_type) = &self.meal_type {
            if !matches!(
                meal_type.as_str(),
                "breakfast" | "lunch" | "dinner" | "snack" | "unknown"
            ) {
                return Err(
                    "Meal type must be breakfast, lunch, dinner, snack, or unknown.".to_owned(),
                );
            }
        }

        for value in [
            self.protein_g,
            self.carbohydrate_g,
            self.fat_g,
            self.fiber_g,
        ] {
            if let Some(value) = value {
                if !value.is_finite() || value < 0.0 {
                    return Err("Macro nutrients must be non-negative numbers.".to_owned());
                }
            }
        }

        if let Some(confidence) = self.confidence {
            if !confidence.is_finite() || !(0.0..=1.0).contains(&confidence) {
                return Err("Confidence must be between 0 and 1.".to_owned());
            }
        }

        Ok(())
    }
}
