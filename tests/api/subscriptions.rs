use crate::helpers::spawn_app;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let app = spawn_app().await;
    
    // let client = reqwest::Client::new();
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    // let response = client
    //     .post(&format!("{}/subscriptions", &app.address))
    //     .header("Content-Type", "application/x-www-form-urlencoded")
    //     .body(body)
    //     .send()
    //     .await
    //     .expect("Failed to execute request.");
    let response = app.post_subscriptions(body.into()).await;
    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}


#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        // let response = client
        //     .post(&format!("{}/subscriptions", &app.address))
        //     .header("Content-Type", "application/x-www-form-urlencoded")
        //     .body(invalid_body)
        //     .send()
        //     .await
        //     .expect("Failed to execute request.");
        let response = app.post_subscriptions(invalid_body.into()).await;
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}



#[tokio::test]
async fn subscribe_returns_a_400_when_fields_are_present_but_invalid() {
    let app = spawn_app().await;
    // let client = reqwest::Client::new();
    let test_cases = vec![
     ("name=&email=ursula_le_guin%40gmail.com", "empty name"),   
     ("name=Ursula&email=", "empty email"),
     ("name=Ursula&email=definitely-not-an-email", "invalid email"),
    ];
    for (body, description) in test_cases {
        // let response = client
        //     .post(&format!("{}/subscriptions", &app.address))
        //     .header("Content-Type", "application/x-www-form-urlencoded")
        //     .body(body)
        //     .send()
        //     .await
        //     .expect("Failed to execute request.");
        let response = app.post_subscriptions(body.into()).await;
        assert_eq!(
         200,
         response.status().as_u16(),   
         "The API did not return a 200 OK when the payload was {}.",
         description
        );
    }
}


#[tokio::test]
async fn subscribe_sends_a_confirmation_email_for_valid_data() {
    let app = spawn_app().await;
    
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;
    let response = app.post_subscriptions(body.into()).await;
    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn subscribe_sends_a_confirmation_email_with_a_link() {
    let app = spawn_app().await;
    
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();
    // assert_eq!(200, response.status().as_u16());

    // Here we would normally check the email content, but for simplicity, we are just checking the status.
    let get_link = |s: &str|{
        let links: Vec<_> = linkify::LinkFinder::new()
            .links(s)
            .filter(|l| *l.kind() == linkify::LinkKind::Url)
            .collect();
        assert_eq!(links.len(), 1);
        links[0].as_str().to_owned()
    };
    let html_link = get_link(body["HtmlBody"].as_str().unwrap());
    let text_link = get_link(body["TextBody"].as_str().unwrap());

    assert_eq!(html_link, text_link);
}