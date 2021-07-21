#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate diesel;

use actix_web::{
    get, http::StatusCode, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use actix_web_httpauth::middleware::HttpAuthentication;
use color_eyre::eyre::Report;
use listenfd::ListenFd;
use serde::Deserialize;
use tracing::{info, instrument, Subscriber};
use tracing_actix_web::TracingLogger;

mod db;
mod models;
mod schema;
mod utils;

pub use crate::models::errors::ApiError;
pub use crate::models::player;
pub use crate::utils::auth;
pub use crate::utils::config::CFG;
pub use crate::utils::logging;

#[get("/")]
#[instrument]
pub async fn hello_world() -> impl Responder {
    HttpResponse::Ok().body("If you see this, it means that the web server is working correctly!!")
}

#[post("/login")]
pub async fn login(req: HttpRequest) -> Result<HttpResponse, ApiError> {
    let ext = req.extensions();

    let claims = ext.get::<auth::UserClaims>().unwrap();

    Err(ApiError::InternalServerError(Report::msg(
        "Some kind of error!",
    )))

    //HttpResponse::Ok().body(format!("Hi {}, you're now authenticated!", claims.email))
}

#[derive(Debug, Deserialize)]
pub struct RegisterInfo {
    nickname: String,
}

#[post("/register")]
#[instrument]
pub async fn register(req: HttpRequest, info: web::Json<RegisterInfo>) -> impl Responder {
    let ext = req.extensions();

    let claims = ext.get::<auth::UserClaims>().unwrap();

    info!("Looking for a player with uid {}", &claims.user_id);

    if player::Account::find_by_uid(&claims.user_id).is_err() {
        info!("No such player found: registering new player");

        let account = player::Account::register(
            info.nickname.clone(),
            claims.email.clone(),
            claims.user_id.clone(),
        )
        .unwrap();

        let response = format!(
            "[Authenticated as {}] Registered player info: {}",
            claims.email,
            serde_json::to_string(&account).expect("Could not serialize user")
        );

        HttpResponse::Ok().body(response)
    } else {
        HttpResponse::build(StatusCode::BAD_REQUEST).body("Already registered")
    }
}

#[get("/player/{id}")]
#[instrument]
pub async fn search(req: HttpRequest, web::Path(id): web::Path<i32>) -> impl Responder {
    let ext = req.extensions();

    let claims = ext.get::<auth::UserClaims>().unwrap();

    let account = player::Account::find(id);

    match account {
        Ok(account) => {
            let response = format!(
                "[Authenticated as {}] Player info: {}",
                claims.email,
                serde_json::to_string(&account).expect("Could not serialize user")
            );

            HttpResponse::Ok().body(response)
        }
        Err(_) => HttpResponse::build(StatusCode::NOT_FOUND).body("User not found"),
    }
}

pub async fn run_server() -> std::io::Result<()> {
    let mut listenfd = ListenFd::from_env();

    lazy_static::initialize(&CFG);
    lazy_static::initialize(&auth::VALIDATOR);

    color_eyre::install().unwrap();
    db::init();

    match CFG.enable_bunyan {
        false => logging::get_subscriber("info".into()),
        true => logging::get_subscriber_bunyan("assassin-server".into(), "info".into()),
    };

    let mut server = HttpServer::new(move || {
        let auth = HttpAuthentication::bearer(auth::bearer_auth_validator);
        App::new()
            .wrap(TracingLogger)
            .service(hello_world)
            .default_service(web::route().to(|| HttpResponse::NotFound()))
            .service(
                //Protected routes in the official API
                web::scope("/v1")
                    .wrap(auth)
                    .service(login)
                    .service(search)
                    .service(register),
            )
    });

    server = match listenfd.take_tcp_listener(0)? {
        Some(listener) => server.listen(listener)?,
        None => server.bind(format!("{}:{}", CFG.host, CFG.port))?,
    };

    server.run().await
}
