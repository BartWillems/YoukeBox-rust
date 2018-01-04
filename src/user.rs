use diesel;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use lower;
use regex::Regex;
use rocket::http::Status;
use rocket::response::Failure;
use bcrypt::{hash, verify, DEFAULT_COST};
use std::time::SystemTime;

use schema::users;

#[derive(Queryable, Identifiable)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password_hash: String,
    pub created_at: SystemTime,
    pub updated_at: Option<SystemTime>,
}

#[derive(Insertable, Deserialize)]
#[table_name = "users"]
pub struct NewUser {
    pub username: String,
    pub password: String,
}

impl User {
    pub fn verify_name(name: &str) -> Result<String, String> {
        let name = name.trim();

        if name.is_empty() {
            return Err(String::from("Name cannot be empty."));
        }

        let regex = Regex::new(r"^[[:word:]]{3,20}$").unwrap();

        if !regex.is_match(name) {
            return Err(String::from("Invalid username."));
        }

        Ok(name.to_string())
    }

    pub fn create(conn: &PgConnection, mut new_user: NewUser) -> Result<User, Failure> {
        use diesel::prelude::*;

        match hash(&new_user.password[..], DEFAULT_COST) {
            Ok(hashed) => {
                new_user.password = hashed;
            }
            Err(e) => {
                println!("Errow while hasing a password: {}", e);
                return Err(Failure(Status::InternalServerError));
            }
        };

        let result = diesel::insert_into(users::table)
            .values(&new_user)
            .get_result(conn);

        match result {
            Ok(result) => Ok(result),
            Err(_) => Err(Failure(Status::InternalServerError)),
        }
    }

    // Verifies the user's password
    pub fn authenticate(conn: &PgConnection, user: &User) -> Result<bool, Failure> {
        use schema::users::dsl::*;

        let result = users
            .filter(lower(username).eq(user.username.to_lowercase()))
            .first::<User>(conn);

        if result.is_err() {
            return Err(Failure(Status::InternalServerError));
        }

        match verify(&result.unwrap().password_hash[..], &user.password_hash[..]) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    // Find & return a user by id
    pub fn find(conn: &PgConnection, user_id: i32) -> Result<User, Failure> {
        use diesel::prelude::*;
        use schema::users::dsl::*;

        let result = users.filter(id.eq(user_id)).first::<User>(conn);

        match result {
            Ok(result) => Ok(result),
            Err(e) => {
                println!("Could not find user with id: {}", e);
                Err(Failure(Status::NotFound))
            }
        }
    }

    // Return all users
    pub fn all(conn: &PgConnection) -> Result<Vec<User>, Failure> {
        use diesel::prelude::*;
        use schema::users::dsl::*;

        let result = users.load::<User>(conn);

        match result {
            Ok(result) => Ok(result),
            Err(e) => {
                println!("Error while fetching the users: {}", e);
                Err(Failure(Status::InternalServerError))
            }
        }
    }
}
