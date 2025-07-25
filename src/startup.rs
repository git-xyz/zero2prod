
use actix_web::{HttpResponse, Responder};
use actix_web::{dev::Server, web, App,  HttpServer};
use actix_web::web::Data;
use serde::{Deserialize, Serialize};
use sqlx::{ postgres::PgPoolOptions, PgPool};
use tracing_actix_web::TracingLogger;
use std::{ net::TcpListener};
use crate::configuration::{DatabaseSettings};

use crate::{configuration::Settings, email_client::{ EmailClient}, routes};

pub struct Application {
    port: u16,
    server: Server
}

impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, std::io::Error> {
        let connection_pool = get_connection_pool(&configuration.database);
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
        let listener = TcpListener::bind(address)?;
        let port = listener.local_addr().unwrap().port();
        
        let server = run(listener, connection_pool, email_client)?;
        
        Ok(Self {
            port,
            server
        })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub fn run(
    listener: TcpListener,
    db_pool: PgPool,
    email_client: EmailClient
) -> Result<Server, std::io::Error> {
    let db_pool = web::Data::new(db_pool);
    let email_client = Data::new(email_client);
    let server = HttpServer::new(move|| {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(routes::health_check))
            .route("/subscriptions", web::post().to(routes::subscribe))
            .route("/test", web::post().to(test_handler))
            .app_data(db_pool.clone())
            .app_data(email_client.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}

#[derive(Serialize, Deserialize)]
pub struct MyParams {
    name: String,
}
pub async fn test_handler(params: web::Form<MyParams>) -> HttpResponse{
    // "Test handler"
    HttpResponse::Ok()
        .content_type("text/plain")
        .body(format!("Your name is {}", params.name))
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

pub fn get_connection_pool(
    configuration: &DatabaseSettings
) -> PgPool {
    PgPoolOptions::new()
        .connect_lazy_with(configuration.with_db())
}