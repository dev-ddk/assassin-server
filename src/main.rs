use actix_web::{middleware::Logger, web, App, HttpServer};

mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = utils::config::Config::from_env().expect("Server configuration");

    HttpServer::new(|| {
        let validator = web::Data::new(utils::auth::Validator::new());
        App::new()
            .wrap(Logger::default())
            .data(validator.clone())
            .service(assassin_server::hello_world)
            .service(assassin_server::echo)
            .service(assassin_server::login)
    })
    .bind(format!("{}:{}", config.host, config.port))?
    .run()
    .await?;

    Ok(())
}
