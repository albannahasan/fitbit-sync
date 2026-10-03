use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::meals::{CreateMealInput, Meal, MealsPage, MealsQuery};

const LOCAL_DEV_SUBJECT: &str = "local-dev-user";

#[derive(Debug, Clone, Serialize)]
pub struct LoggedMeal {
    pub meal_id: Uuid,
    pub status: &'static str,
    pub message: String,
}

pub async fn get_meals(pool: &PgPool, query: MealsQuery) -> Result<MealsPage, sqlx::Error> {
    let MealsQuery { cursor, limit } = query;
    let limit = limit.unwrap_or(20);
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

    // Decode the cursor, if provided.
    // Query this user's meals.

    let meals: Vec<Meal> = if let Some(cursor) = cursor {
        let mut parts = cursor.split('|');
        let eaten_at = parts.next();
        let meal_id = parts.next();

        sqlx::query_as::<_, Meal>(
            "SELECT
            id, name, energy_kcal, eaten_at, meal_type,
            protein_g::DOUBLE PRECISION AS protein_g,
            carbohydrate_g::DOUBLE PRECISION AS carbohydrate_g,
            fat_g::DOUBLE PRECISION AS fat_g,
            fiber_g::DOUBLE PRECISION AS fiber_g,
            confidence::DOUBLE PRECISION AS confidence
            FROM meals
            WHERE user_id = $1 AND (eaten_at, id) < ($2, $3)
            ORDER BY eaten_at DESC, id DESC
            LIMIT $4;
            ",
        )
        .bind(user_id)
        .bind(eaten_at)
        .bind(meal_id)
        .bind(limit)
        .fetch_all(&mut *transaction)
        .await?
    } else {
        sqlx::query_as::<_, Meal>(
            "SELECT id, name, energy_kcal, eaten_at, meal_type,
            protein_g::DOUBLE PRECISION AS protein_g,
            carbohydrate_g::DOUBLE PRECISION AS carbohydrate_g,
            fat_g::DOUBLE PRECISION AS fat_g,
            fiber_g::DOUBLE PRECISION AS fiber_g,
            confidence::DOUBLE PRECISION AS confidence
            FROM meals
            WHERE user_id = $1
            ORDER BY eaten_at DESC, id DESC
            LIMIT $2;
            ",
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(&mut *transaction)
        .await?
    };

    Ok(MealsPage {
        meals,
        next_cursor: None,
    })
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
    let new_meal_id: Option<Uuid> = sqlx::query_scalar::<_, Uuid>(
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
