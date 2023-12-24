use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use uuid::Uuid;
use zero2prod::{
    config::{get_config, DatabaseSettings},
    startup,
};

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

#[tokio::test]
async fn health_check_works() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/health-check", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_200_for_valid_form_data() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let body = "name=le%20guin&email=ursula%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("select email, name from subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!(saved.email, "ursula@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[tokio::test]
async fn subscribe_returns_400_when_data_is_missing() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (body, error) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 when the payload was {}",
            error
        );
    }
}

async fn spawn_app() -> TestApp {
    let mut config = get_config().expect("Failed to read config");

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    config.database.name = Uuid::new_v4().to_string();
    let db_pool = create_new_test_db_pool(&config.database).await;

    let server = startup::run(listener, db_pool.clone()).expect("Failed to bind address");
    let _ = tokio::spawn(server);

    TestApp { address, db_pool }
}

pub async fn create_new_test_db_pool(config: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect(&config.connect_string_without_db())
        .await
        .expect("Failed to connect to Postgres");

    connection
        .execute(format!(r#"create database "{}";"#, config.name).as_str())
        .await
        .expect("Failed to create database");

    let db_pool = PgPool::connect(&config.connect_string())
        .await
        .expect("Failed to connect to Postgres");

    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("Failed to migrate database");

    db_pool
}
