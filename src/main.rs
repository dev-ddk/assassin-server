#[actix_web::main]
async fn main() {
    assassin_server::run_server().await;
}
