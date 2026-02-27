use color_eyre::eyre::{Context, Result, eyre};
use diesel::{Connection as _, SqliteConnection};
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::sync_connection_wrapper::SyncConnectionWrapper;
use diesel_async::{AsyncConnectionCore, RunQueryDsl, SimpleAsyncConnection};
use diesel_migrations::MigrationHarness;
use diesel_migrations::{EmbeddedMigrations, embed_migrations};
use rocket::Request;
use rocket::State;
use rocket::http::Status;
use rocket::outcome::try_outcome;
use rocket::request::FromRequest;
use rocket::request::Outcome;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub type Pool = diesel_async::pooled_connection::bb8::Pool<SyncConnectionWrapper<SqliteConnection>>;

const DATABASE_URL: &str = "./web_host.db";

pub async fn pool() -> Result<Pool> {
    let connection_manager =
        AsyncDieselConnectionManager::<SyncConnectionWrapper<SqliteConnection>>::new(DATABASE_URL);
    let pool = bb8::Pool::builder().build(connection_manager).await?;

    Ok(pool)
}

pub fn run_pending_migrations() -> Result<()> {
    let mut connection = SqliteConnection::establish(DATABASE_URL)
        .with_context(|| format!("Error connecting to {}", DATABASE_URL))?;

    connection
        .run_pending_migrations(MIGRATIONS)
        .map_err(|err| eyre!(err))?;

    Ok(())
}

type InnerConnection<'r> = ::bb8::PooledConnection<
    'r,
    AsyncDieselConnectionManager<SyncConnectionWrapper<SqliteConnection>>,
>;

pub struct Db<'r>(pub InnerConnection<'r>);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Db<'r> {
    type Error = color_eyre::Report;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let pool = try_outcome!(
            request
                .guard::<&State<Pool>>()
                .await
                .map_error(|(status, ())| (status, eyre!("failed to get db pool")))
        );
        let conn = match pool.get().await {
            Ok(conn) => conn,
            Err(err) => {
                return Outcome::Error((
                    Status::InternalServerError,
                    eyre!("failed to get db connection: {err}"),
                ));
            }
        };
        let db = Db(conn);

        Outcome::Success(db)
    }
}

impl<'r> SimpleAsyncConnection for Db<'r> {
    fn batch_execute(
        &mut self,
        query: &str,
    ) -> impl Future<Output = diesel::QueryResult<()>> + Send {
        SimpleAsyncConnection::batch_execute(&mut self.0, query)
    }
}

impl<'r> AsyncConnectionCore for Db<'r> {
    type ExecuteFuture<'conn, 'query> =
        <InnerConnection<'r> as AsyncConnectionCore>::ExecuteFuture<'conn, 'query>;
    type LoadFuture<'conn, 'query> =
        <InnerConnection<'r> as AsyncConnectionCore>::LoadFuture<'conn, 'query>;
    type Stream<'conn, 'query> =
        <InnerConnection<'r> as AsyncConnectionCore>::Stream<'conn, 'query>;
    type Row<'conn, 'query> = <InnerConnection<'r> as AsyncConnectionCore>::Row<'conn, 'query>;
    type Backend = <InnerConnection<'r> as AsyncConnectionCore>::Backend;

    fn load<'conn, 'query, T>(&'conn mut self, source: T) -> Self::LoadFuture<'conn, 'query>
    where
        T: diesel::query_builder::AsQuery + 'query,
        T::Query: diesel::query_builder::QueryFragment<Self::Backend>
            + diesel::query_builder::QueryId
            + 'query,
    {
        AsyncConnectionCore::load(&mut self.0, source)
    }

    fn execute_returning_count<'conn, 'query, T>(
        &'conn mut self,
        source: T,
    ) -> Self::ExecuteFuture<'conn, 'query>
    where
        T: diesel::query_builder::QueryFragment<Self::Backend>
            + diesel::query_builder::QueryId
            + 'query,
    {
        AsyncConnectionCore::execute_returning_count(&mut self.0, source)
    }
}
