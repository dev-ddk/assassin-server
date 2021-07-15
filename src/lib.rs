#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate diesel;

use actix_web::{get, http::StatusCode, post, web, HttpResponse, Responder};
use actix_web::{middleware::Logger, App, HttpServer};
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

#[post("/echo")]
pub async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(format!("Ok good, echoing: {}", req_body))
}

#[post("/login")]
pub async fn login(token: web::Json<auth::IdToken>) -> impl Responder {
    let claims = auth::VALIDATOR.validate_token(&token.id_token);

    match claims {
        Some(claims) => {
            HttpResponse::Ok().body(format!("Hi {}, you're now authenticated!", claims.email))
        }
        None => {
            info!("Invalid token submitted");
            HttpResponse::build(StatusCode::FORBIDDEN).body("Bad token!")
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RegisterInfo {
    id_token: String,
    nickname: String,
    email: String,
}

#[post("/register")]
pub async fn register(info: web::Json<RegisterInfo>) -> impl Responder {
    let claims = auth::VALIDATOR.validate_token(&info.id_token);

    match claims {
        Some(claims) => {
            info!("Looking for a player with uid {}", &claims.user_id);
            if player::Account::find_by_uid(&claims.user_id).is_err() {
                info!("No such player found: registering new player");
                let account = player::Account::register(
                    info.nickname.clone(),
                    info.email.clone(),
                    claims.user_id,
                )
                .unwrap();
                HttpResponse::Ok().body(format!(
                    "[Authenticated as {}] Registered player info: {}",
                    claims.email,
                    serde_json::to_string(&account).expect("Could not serialize user")
                ))
            } else {
                HttpResponse::build(StatusCode::BAD_REQUEST).body("Already registered")
            }
        }
        None => {
            info!("Invalid token submitted");
            HttpResponse::build(StatusCode::FORBIDDEN).body("Bad token!")
        }
    }
}

#[get("/player/{id}")]
pub async fn search(
    token: web::Json<auth::IdToken>,
    web::Path(id): web::Path<i32>,
) -> impl Responder {
    let claims = auth::VALIDATOR.validate_token(&token.id_token);

    match claims {
        Some(claims) => {
            let account = player::Account::find(id);
            match account {
                Ok(account) => HttpResponse::Ok().body(format!(
                    "[Authenticated as {}] Player info: {}",
                    claims.email,
                    serde_json::to_string(&account).expect("Could not serialize user")
                )),
                Err(_) => HttpResponse::build(StatusCode::NOT_FOUND).body("User not found"),
            }

            //HttpResponse::Ok().body(format!("Hi {}, you're now authenticated!", claims.email))
        }
        None => {
            info!("Invalid token submitted");
            HttpResponse::build(StatusCode::FORBIDDEN).body("Bad token!")
        }
    }
}

pub async fn run_server() -> std::io::Result<()> {
    let mut listenfd = ListenFd::from_env();

    db::init();

    let mut server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .service(hello_world)
            .service(echo)
            .service(login)
            .service(search)
            .service(register)
    });

    server = match listenfd.take_tcp_listener(0)? {
        Some(listener) => server.listen(listener)?,
        None => server.bind(format!("{}:{}", CFG.host, CFG.port))?,
    };

    server.run().await
}
