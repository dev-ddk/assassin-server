use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel_migrations::embed_migrations;
use lazy_static::lazy_static;
use tracing::info;

pub use crate::utils::config::CFG;

type ConnPool = Pool<ConnectionManager<PgConnection>>;
pub type DbConnection = PooledConnection<ConnectionManager<PgConnection>>;

embed_migrations!();

lazy_static! {
    static ref POOL: ConnPool = {
        let manager = ConnectionManager::<PgConnection>::new(&CFG.postgres_url);
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

pub fn connection() -> Result<DbConnection, r2d2::Error> {
    POOL.get()
}
