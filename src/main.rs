use zero2prod::configuration::get_configuration;
use zero2prod::startup::{build};
use zero2prod::telemetry::{get_subscribe, init_subscriber};


#[tokio::main]
async fn main() -> Result<(), std::io::Error>{

    let subscriber = get_subscribe(
        "zero2prod".into(), 
        "info".into(),
        std::io::stdout
    );
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("msg");
    let server = build(configuration)
        .await
        .expect("Failed to build server");
    server.await?;

    Ok(())
}

//export https_proxy=http://127.0.0.1:7890 http_proxy=http://127.0.0.1:7890 all_proxy=socks5://127.0.0.1:7890
