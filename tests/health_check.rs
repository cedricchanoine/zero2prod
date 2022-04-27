use secrecy::ExposeSecret;
use sqlx::{PgConnection, Connection, query, PgPool, Executor};
use uuid::Uuid;
use zero2prod::configuration::{get_configuration, DatabaseSettings};
use zero2prod::telemetry;
use once_cell::sync::Lazy;

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();

    if std::env::var("TEST_LOG").is_ok(){

    let subscriber = telemetry::get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
    telemetry::init_subscriber(subscriber)
    } else {
    let subscriber = telemetry::get_subscriber(subscriber_name, default_filter_level, std::io::sink);
    telemetry::init_subscriber(subscriber)
    }
});


#[tokio::test]
async fn health_check_works(){
    let testapp = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client.get(testapp.address+"/health_check")
        .send()
        .await
        .expect("failed to execute request");
    assert!(response.status().is_success());
}


#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data(){
let testapp = spawn_app().await;
let mut connection = testapp.db_pool.acquire().await.unwrap();

let client = reqwest::Client::new();

let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
let response = client
        .post(testapp.address+"/subscriptions")
        .body(body)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .send()
        .await
        .expect("failed to execute request");
assert_eq!(response.status().as_u16(), 200);

let saved = query!("SELECT email, name FROM subscriptions",).fetch_one(&mut connection).await.expect("failed to fetch saved subscription");
assert_eq!(saved.name, "le guin");
assert_eq!(saved.email, "ursula_le_guin@gmail.com");


}

#[tokio::test]
async fn subscribe_returns_a_400_on_missing_data(){
let testapp = spawn_app().await;
let client = reqwest::Client::new();

let bodies = vec![  
    ("name=le%20guin", "missing the email"),
    ("email=ursula_le_guin%40gmail.com", "missing the name"),
    ("", "missing name and email"),
];

for (body, case) in bodies {
let response = client
        .post(format!("{}/subscriptions", testapp.address))
        .body(body)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .send()
        .await
        .expect("failed to execute request");
assert_eq!(400, response.status().as_u16(), "not 400 on case : {}" ,case)
}
}

async fn spawn_app() -> TestApp {
    let mut configuration = get_configuration().expect("failed to read configuration");
    configuration.database.database_name = Uuid::new_v4().to_string();
    Lazy::force(&TRACING);



    //let pool = sqlx::pool::Pool::connect(&configuration.database.connection_string_without_db()).await.expect("coult not connect to db");
    let pool = configure_database(&configuration.database).await;
    let socket = std::net::TcpListener::bind("localhost:0").expect("could not bind to random port");
    let port = socket.local_addr().unwrap().port();
    let server = zero2prod::startup::run(socket, pool.clone()).expect("failed to bind address");

    let _ = tokio::spawn(server);



    TestApp{
    address: format!("http://localhost:{}", port),
    db_pool: pool,
    }
}

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}


pub async fn configure_database(config: &DatabaseSettings) -> PgPool{
    //create database

    let mut connection = PgConnection::connect(config.connection_string_without_db().expose_secret())
        .await
        .expect("can't connect to postgres");

    connection.execute(format!(r#"CREATE DATABASE "{}";"#,config.database_name).as_str())
        .await
        .expect("failed to create database");
    
    let connection_pool = PgPool::connect(config.connection_string().expose_secret())
        .await
        .expect("failed to connect to postgres");
    //migrate database
   sqlx::migrate!("./migrations")
       .run(&connection_pool)
       .await
       .expect("can't migrate");
   connection_pool
 }



