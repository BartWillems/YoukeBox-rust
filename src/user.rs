extern crate diesel;
extern crate bcrypt;

use diesel::pg::PgConnection;
use lower;
use rocket::http::Status;
use rocket::response::Failure;
use self::bcrypt::{DEFAULT_COST, hash, verify};
use std::time::SystemTime;
use super::schema::users;

#[derive(Queryable, Identifiable)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password_hash: String,
    pub created_at: SystemTime,
    pub updated_at: Option<SystemTime>,
}

#[derive(Insertable)]
#[derive(Deserialize)]
#[table_name="users"]
pub struct NewUser {
    pub username: String,
    pub password: String,
}

impl User {
    pub fn verify_name(name: &str) -> Result<String, String> {
        let name = name.trim();

        if name.is_empty() {
            return Err("Name cannot be empty.".to_string());
        }

        if name.len() > 20 {
            return Err("Name must not exceed 20 characters.".to_string());
        }

        for c in name.chars() {
            if !c.is_alphanumeric() && c != ' ' && c !=  '_' {
                return Err(format!("Illegal character in name: {}", c).to_string());
            }
        }

        return Ok(name.to_string());
    }

    pub fn create(conn: &PgConnection, mut new_user: NewUser) -> Result<User, Failure> {
        use diesel::prelude::*;
        // use schema::users;

        match hash(&new_user.password[..], DEFAULT_COST) {
            Ok(hashed) => {
                new_user.password = hashed;
            },
            Err(e) => {
                println!("Errow while hasing a password: {}", e);
                return Err(Failure(Status::InternalServerError));
            }
        };

        let result = diesel::insert(&new_user)
                        .into(users::table)
                        .get_result(conn);

        match result {
            Ok(result) => {
                return Ok(result);
            },
            Err(_) => {
                return Err(Failure(Status::InternalServerError));
            }
        }
    }

    pub fn authenticate(conn: &PgConnection, uname: String, pw: String) -> Result<bool, Failure> {
        use diesel::prelude::*;
        use schema::users::dsl::*;

        let uname = uname.to_lowercase();

        let result = users.filter(lower(username).eq(uname))
                    .first::<User>(conn);

        if let Err(_) = result {
            return Err(Failure(Status::InternalServerError));
        }

        match verify(&result.unwrap().password_hash[..], &pw[..]) {
            Ok(_) => return Ok(true),
            Err(_)  => return Ok(false)
        }
    }

    pub fn find(conn: &PgConnection, user_id: i32) -> Result<User, Failure> {
        use diesel::prelude::*;
        use schema::users::dsl::*;

        let result = users.filter(id.eq(user_id)).first::<User>(conn);

        match result {
            Ok(result) => {
                return Ok(result);
            },
            Err(e) => {
                println!("Could not find user with id: {}", e);
                return Err(Failure(Status::NotFound));
            }
        }
    }

    pub fn all(conn: &PgConnection) -> Result<Vec<User>, Failure> {
        use diesel::prelude::*;
        use schema::users::dsl::*;

        let result = users.load::<User>(conn);

        match result {
            Ok(result) => {
                return Ok(result);
            },
            Err(e) => {
                println!("Error while fetching the users: {}", e);
                return Err(Failure(Status::InternalServerError));
            }
        }
    }
}