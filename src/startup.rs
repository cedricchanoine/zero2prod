use actix_web::{web, App, HttpResponse, HttpRequest, HttpServer, Responder,};
use actix_web::dev::Server;
use tracing_actix_web::TracingLogger;

use crate::routes;

pub fn run(address :std::net::TcpListener, db_connection: sqlx::PgPool) -> Result<Server, std::io::Error> {

    let connection = web::Data::new(db_connection);

    let server = HttpServer::new(move || {
        App::new()
        .wrap(TracingLogger::default())
        .route("/health_check", web::get().to(routes::health_check))
        .route("/subscriptions", web::post().to(routes::subscription))
        .app_data(connection.clone())
        }
        ).listen(address)?
        .run();

        Ok(server)
}
