use actix_web::{middleware::Logger, App, HttpServer};
use listenfd::ListenFd;

mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = utils::config::Config::from_env().expect("Server configuration");
    let mut listenfd = ListenFd::from_env();

    let mut server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .service(assassin_server::hello_world)
            .service(assassin_server::echo)
            .service(assassin_server::login)
    });

    server = match listenfd.take_tcp_listener(0)? {
        Some(listener) => server.listen(listener)?,
        None => server.bind(format!("{}:{}", config.host, config.port))?,
    };

    server.run().await
}
