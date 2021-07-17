#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate diesel;

use actix_web::{get, http::StatusCode, post, web, HttpRequest, HttpResponse, Responder};
use actix_web::{middleware::Logger, App, HttpServer};
use actix_web_httpauth::middleware::HttpAuthentication;
use listenfd::ListenFd;
use serde::Deserialize;
use tracing::info;

mod db;
mod models;
mod schema;
mod utils;

pub use crate::models::player;
pub use crate::utils::auth;
pub use crate::utils::config::CFG;

#[get("/")]
pub async fn hello_world() -> impl Responder {
    HttpResponse::Ok().body("If you see this, it means that the web server is working correctly!!")
}

#[post("/login")]
pub async fn login(req: HttpRequest) -> impl Responder {
    let ext = req.extensions();

    let claims = ext.get::<auth::UserClaims>().unwrap();

    HttpResponse::Ok().body(format!("Hi {}, you're now authenticated!", claims.email))
}

#[derive(Debug, Deserialize)]
pub struct RegisterInfo {
    nickname: String,
}

#[post("/register")]
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

    db::init();

    let mut server = HttpServer::new(move || {
        let auth = HttpAuthentication::bearer(auth::bearer_auth_validator);
        App::new()
            .wrap(Logger::default())
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
