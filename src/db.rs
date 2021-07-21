use color_eyre::{Report, Result};
use diesel::pg::PgConnection;
use diesel::r2d2;
use diesel_migrations::embed_migrations;
use lazy_static::lazy_static;
use tracing::info;

pub use crate::utils::config::CFG;

type Pool = r2d2::Pool<r2d2::ConnectionManager<PgConnection>>;
pub type DbConnection = r2d2::PooledConnection<r2d2::ConnectionManager<PgConnection>>;

embed_migrations!();

lazy_static! {
    static ref POOL: Pool = {
        let manager = r2d2::ConnectionManager::<PgConnection>::new(&CFG.postgres_url);
        Pool::new(manager).expect("Failed to create DB pool")
    };
}

pub fn init() {
    info!("Initializing Database");
    lazy_static::initialize(&POOL);
    let conn = connection().expect("Failed to connect to DB");
    info!("Running migrations");
    embedded_migrations::run(&conn).unwrap();
}

pub fn connection() -> Result<DbConnection> {
    POOL.get()
        .map_err(|e| Report::new(e).wrap_err("Could not obtain connection from DB pool"))
}
