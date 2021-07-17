#[actix_web::main]
async fn main() -> std::io::Result<()> {
    assassin_server::run_server().await
}
