use actix_web::{web, App, HttpResponse, HttpRequest, HttpServer, Responder,};
use actix_web::dev::Server;

pub mod routes;
pub mod configuration;
pub mod startup;
pub mod telemetry;
