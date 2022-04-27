use zero2prod::{startup, configuration::get_configuration};
use zero2prod::telemetry;
use sqlx::postgres::PgPoolOptions;
use secrecy::ExposeSecret;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let configuration = get_configuration().expect("failed to read configuration");

    let subscriber = telemetry::get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    telemetry::init_subscriber(subscriber);

    let address = format!("{}:{}", configuration.application.host, configuration.application.port);
    let socket = std::net::TcpListener::bind(address).expect("can't bind...");
    let pool = PgPoolOptions::new().connect_timeout(std::time::Duration::from_secs(2))
       .connect_lazy(configuration.database.connection_string().expose_secret()).expect("can't connect to DB");

    startup::run(socket, pool)?.await?;
    Ok(())
}
