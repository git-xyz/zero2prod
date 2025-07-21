





use actix_web::{dev::Server, web, App,  HttpServer};
use sqlx::{ postgres::PgPoolOptions, PgPool};
use tracing_actix_web::TracingLogger;
use std::{f32::consts::E, net::TcpListener};

use crate::{configuration::Settings, email_client::{self, EmailClient}, routes};

pub fn run(
    listener: TcpListener,
    db_pool: PgPool,
    email_client: EmailClient
) -> Result<Server, std::io::Error> {
    let db_pool = web::Data::new(db_pool);
    let server = HttpServer::new(move|| {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(routes::health_check))
            .route("/subscriptions", web::post().to(routes::subscribe))
            .app_data(db_pool.clone())
            .app_data(email_client.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}

pub async fn build(configuration: Settings) -> Result<Server, std::io::Error> {
    let connection_pool = PgPoolOptions::new()
        .connect_lazy_with(configuration.database.with_db());

    let sender_email = configuration.email_client.sender()
        .expect("Invalid email configuration");
    let timeout = configuration.email_client.timeout();
    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        sender_email,
        configuration.email_client.authorization_token,
        timeout
    );

    let address = format!(
        "{}:{}", 
        configuration.application.host, 
        configuration.application.port
    );
    let listener = TcpListener::bind(address);
    run(listener?, connection_pool, email_client)
}