#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]
#![plugin(dotenv_macros)]

#![recursion_limit="128"]

#[macro_use] extern crate diesel;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate lazy_static;

extern crate rocket_contrib;
extern crate rocket_cors;

extern crate bcrypt;
extern crate bytes;
extern crate dotenv;
extern crate image;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate rocket;
extern crate serde_json;
extern crate regex;
extern crate reqwest;

use diesel::prelude::*;
use diesel::pg::PgConnection;
use dotenv::dotenv;
use std::env;
use r2d2_diesel::ConnectionManager;
use std::ops::Deref;
use rocket::http::Status;
use rocket::request::{self, FromRequest};
use rocket::{Request, State, Outcome};
use diesel::sql_types;

pub mod schema;
pub mod routes;
pub mod http;
pub mod player;
pub mod user;
pub mod room;
pub mod playlist;
pub mod youtube;
pub mod video;

pub struct DbConn(pub r2d2::PooledConnection<ConnectionManager<PgConnection>>);

sql_function!(lower, lower_t, (a: sql_types::VarChar) -> sql_types::VarChar);

lazy_static! {
    static ref API_URL: &'static str = "https://www.googleapis.com/youtube/v3";
    static ref PICTURES_DIR: &'static str = "content/rooms/pictures";
}

// Return a single connection from the db pool
impl<'a, 'r> FromRequest<'a, 'r> for DbConn {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<DbConn, ()> {
        let pool = request.guard::<State<Pool>>()?;
        match pool.get() {
            Ok(conn) => Outcome::Success(DbConn(conn)),
            Err(_) => Outcome::Failure((Status::ServiceUnavailable, ()))
        }
    }
}

impl Deref for DbConn {
    type Target = PgConnection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub fn init_pool() -> Pool {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set, please ensure the '.env' file exists.");

    let manager = ConnectionManager::<PgConnection>::new(database_url);
    r2d2::Pool::builder().build(manager).expect("Failed to creat db pool")
}