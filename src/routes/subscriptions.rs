
use actix_web::{web, HttpResponse};
use sqlx::{ PgPool};
use tracing::Instrument;


#[derive(serde::Deserialize)]
pub struct FormData{
    pub email: String,
    pub name: String
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, pool),
    fields(
        // request_id = %uuid::Uuid::new_v4(),
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
) -> HttpResponse {

    // let _reqwest_span_guard = reqwest_span.enter();
    // let query_span = tracing::info_span!(
    //     "Saving new subscriber details in the database"
    // );
    
    match insert_subscriber(&pool, &form)
    .await
    {
        Ok(_) => {
            // tracing::info!("request_id {} Saved new subscriber details", reqwest_id);
            HttpResponse::Ok().finish()
        },
        Err(e) => {
            tracing::error!("Failed to execute query: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}


#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(form, pool)
)]
pub async fn insert_subscriber(
    pool: &PgPool,
    form: &FormData
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at) 
        VALUES ($1, $2, $3, $4)
        "#,
        uuid::Uuid::new_v4(),
        form.email,
        form.name,
        chrono::Utc::now(),
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}