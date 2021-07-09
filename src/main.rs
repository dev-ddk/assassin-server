use actix_web::{middleware::Logger, App, HttpServer};

mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = utils::config::Config::from_env().expect("Server configuration");

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .service(assassin_server::hello_world)
            .service(assassin_server::echo)
            .service(assassin_server::login)
    })
    .bind(format!("{}:{}", config.host, config.port))?
    .run()
    .await?;

    Ok(())
}
