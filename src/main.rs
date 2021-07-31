use actix_web::{web, App, HttpResponse, HttpServer};
use actix_web_httpauth::middleware::HttpAuthentication;
use listenfd::ListenFd;
use tracing_actix_web::TracingLogger;

use assassin_server::db;
use assassin_server::routes;
use assassin_server::utils::auth;
use assassin_server::utils::config::CFG;
use assassin_server::utils::logging;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut listenfd = ListenFd::from_env();

    lazy_static::initialize(&CFG);
    lazy_static::initialize(&auth::VALIDATOR);

    color_eyre::install().unwrap();

    match CFG.enable_bunyan {
        false => logging::get_subscriber("info".into()),
        true => logging::get_subscriber_bunyan("assassin-server".into(), "info".into()),
    };

    db::init();

    let mut server = HttpServer::new(move || {
        let auth = HttpAuthentication::bearer(auth::bearer_auth_validator);
        App::new()
            .wrap(TracingLogger)
            .configure(routes::health::config)
            .default_service(web::route().to(|| HttpResponse::NotFound()))
            .service(
                //Protected routes in the official API
                web::scope("/v1")
                    .wrap(auth)
                    .configure(routes::debug::config)
                    .configure(routes::auth::config)
                    .configure(routes::game::config),
            )
    });

    server = match listenfd.take_tcp_listener(0)? {
        Some(listener) => server.listen(listener)?,
        None => server.bind(format!("{}:{}", CFG.host, CFG.port))?,
    };

    server.run().await
}
