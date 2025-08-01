use once_cell::sync::Lazy;
use sqlx::{PgPool, Connection, PgConnection, Executor};
use wiremock::MockServer;
use uuid::Uuid;
use zero2prod::configuration::{get_configuration, DatabaseSettings};
use zero2prod::startup::{get_connection_pool, Application};
use zero2prod::telemetry::{get_subscribe, init_subscriber};



static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscribe(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscribe(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});


pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
    pub email_server: MockServer,
}

impl TestApp {
    pub async fn post_subscriptions(
        &self,
        body:String
    ) -> reqwest::Response {
        reqwest::Client::new()
            .post(format!("{}/subscriptions", self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request")
    }
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);
    
    let email_server = MockServer::start().await;
    let configuration = {
        let mut c= get_configuration().expect("Failed to read config");
        c.database.database_name = Uuid::new_v4().to_string();
        c.email_client.base_url = email_server.uri();
        c
    };
    // configure_database(&configuration.database).await;
    let db_pool = configure_database(&configuration.database).await;
    let application = Application::build(configuration.clone())
        .await
        .expect("Failed to build application");
    let address = format!("http://127.0.0.1:{}", application.port());
    let _ = tokio::spawn(application.run_until_stopped());

    // let server = build(configuration).await
    //     .expect("Failed to start server");
    // let _ = tokio::spawn(server);
    
    TestApp {
        address,
        db_pool,
        email_server,
    }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // let mut connection = PgConnection::connect(
    //     &config.connection_string_without_db().expose_secret()
    // )
    // .await
    // .expect("Failed to connect to Postgres");
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres.");
    println!("==== Connected to Postgres {} successfully. ======", config.database_name);
    connection.execute(
            format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str()
        )
        .await
        .expect("Failed to create database.");

    let connection_pool = PgPool::connect_with(
        config.with_db()
    )
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");
    connection_pool
}
