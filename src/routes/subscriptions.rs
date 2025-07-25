
use actix_web::{web, HttpResponse};
use sqlx::{ PgPool};
// use tracing::Instrument;
// use unicode_segmentation::UnicodeSegmentation;

use crate::{domain::{NewSubscriber, SubscriberEmail, SubscriberName}, email_client::{ EmailClient}};


#[derive(serde::Deserialize, serde::Serialize)]
pub struct FormData{
    pub email: String,
    pub name: String
}

impl TryFrom<FormData> for NewSubscriber{
    type Error = String;

    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        println!("Received form data: {:?}", value.email);
        let name = SubscriberName::parse(value.name)?;
        let email = SubscriberEmail::parse(value.email)?;
        Ok(NewSubscriber { email, name })
    }
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, pool, email_client),
    fields(
        // request_id = %uuid::Uuid::new_v4(),
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
) -> HttpResponse {
    let new_subscriber = match form.0.try_into() {
        Ok(form) => form,
        Err(_e) => return HttpResponse::BadRequest().finish()
    };
    if insert_subscriber(&pool, &new_subscriber).await.is_err() {
        return HttpResponse::InternalServerError().finish();
    }
    if send_confirmation_email(&email_client, new_subscriber)
        .await
        .is_err() {
            return HttpResponse::InternalServerError().finish();
        }

    HttpResponse::Ok().finish()
}


#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(pool, new_subscriber)
)]
pub async fn insert_subscriber(
    pool: &PgPool,
    // form: &FormData,
    new_subscriber: &NewSubscriber,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at, status) 
        VALUES ($1, $2, $3, $4, 'confirmed')
        "#,
        uuid::Uuid::new_v4(),
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
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

#[tracing::instrument(
    name = "send a confirmation email",
    skip(email_client, new_subscriber)
)]
pub async fn send_confirmation_email(
    email_client: &EmailClient,
    new_subscriber: NewSubscriber
) -> Result<(), reqwest::Error> {
    let confirmation_link = "https://example.com/confirm?subscription_token=some_token";
    
    let plain_body = format!(
        "Welcome to our newsletter! Click here to confirm your subscription: {}",
        confirmation_link
    );
    let html_body = format!(
        "Welcome to our newsletter! <a href=\"{}\">Click here to confirm your subscription</a>",
        confirmation_link
    );
    email_client
        .send_email(
            new_subscriber.email,
             "Welcome!", 
            &html_body,
            &plain_body
        )
        .await
}