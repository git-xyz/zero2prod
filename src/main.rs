use sqlx::{PgPool};
use zero2prod::{configuration::get_configuration};
use std::net::TcpListener;
use zero2prod::startup::run;


#[tokio::main]
async fn main() -> Result<(), std::io::Error>{
    let configuration = get_configuration().expect("msg");

    let address = format!("127.0.0.1:{}", configuration.application_port);
    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");
    let listener = TcpListener::bind(address)
        .expect("msg");
    // let port = listener.local_addr().unwrap().port();
    run(listener, connection_pool)?.await
}

//export https_proxy=http://127.0.0.1:7890 http_proxy=http://127.0.0.1:7890 all_proxy=socks5://127.0.0.1:7890
