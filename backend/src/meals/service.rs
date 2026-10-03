use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::meals::CreateMealInput;

const LOCAL_DEV_SUBJECT: &str = "local-dev-user";

#[derive(Debug, Clone, Serialize)]
pub struct LoggedMeal {
    pub meal_id: Uuid,
    pub status: &'static str,
    pub message: String,
}

pub async fn save_meal(pool: &PgPool, input: CreateMealInput) -> Result<LoggedMeal, sqlx::Error> {
    let mut transaction = pool.begin().await?;

    let user_id = sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO users (auth_subject)
         VALUES ($1)
         ON CONFLICT (auth_subject) DO UPDATE
         SET auth_subject = EXCLUDED.auth_subject
         RETURNING id",
    )
    .bind(LOCAL_DEV_SUBJECT)
    .fetch_one(&mut *transaction)
    .await?;

    let meal_type = input.meal_type.as_deref().unwrap_or("unknown");
    let new_meal_id = sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO meals (
            user_id,
            name,
            energy_kcal,
            protein_g,
            carbohydrate_g,
            fat_g,
            fiber_g,
            confidence,
            meal_type,
            eaten_at,
            idempotency_key
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        ON CONFLICT (user_id, idempotency_key) DO NOTHING
        RETURNING id",
    )
    .bind(user_id)
    .bind(&input.name)
    .bind(input.energy_kcal)
    .bind(input.protein_g)
    .bind(input.carbohydrate_g)
    .bind(input.fat_g)
    .bind(input.fiber_g)
    .bind(input.confidence)
    .bind(meal_type)
    .bind(input.eaten_at)
    .bind(&input.idempotency_key)
    .fetch_optional(&mut *transaction)
    .await?;

    let (meal_id, created) = match new_meal_id {
        Some(meal_id) => (meal_id, true),
        None => {
            let meal_id = sqlx::query_scalar::<_, Uuid>(
                "SELECT id
                 FROM meals
                 WHERE user_id = $1 AND idempotency_key = $2",
            )
            .bind(user_id)
            .bind(&input.idempotency_key)
            .fetch_one(&mut *transaction)
            .await?;

            (meal_id, false)
        }
    };

    if created {
        sqlx::query(
            "INSERT INTO meal_events (meal_id, event_type, payload)
             VALUES ($1, 'meal_logged', jsonb_build_object('meal_id', $1::text))",
        )
        .bind(meal_id)
        .execute(&mut *transaction)
        .await?;
    }

    transaction.commit().await?;

    Ok(LoggedMeal {
        meal_id,
        status: "logged",
        message: format!("{} kcal logged", input.energy_kcal),
    })
}
