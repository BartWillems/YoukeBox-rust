#![feature(plugin)]
#![plugin(rocket_codegen)]
#![plugin(dotenv_macros)]

extern crate diesel_demo;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate rocket;
extern crate rocket_contrib;
extern crate serde;
extern crate serde_json;

use rocket::response::status;
use rocket_contrib::{Json};
use self::diesel_demo::*;
use self::diesel_demo::models::*;

static APPLICATION_URL: &'static str = dotenv!("APPLICATION_URL");

#[get("/")]
fn index() -> &'static str {
	"It works!"
}

#[get("/posts")]
fn display_posts(conn: DbConn) -> Json<Vec<Post>> {
	Json(get_posts(&conn))
}

#[get("/posts/<id>")]
fn display_post(conn: DbConn, id: i32) -> Option<Json<Post>> {
    let post = get_post(&conn, id);

    match post {
        Some(post)  => return Some(Json(post)),
        None        => return None,
    }
}

#[post("/posts", format = "application/json", data = "<post>")]
fn add_post(conn: DbConn, post: Json<AddPost>) -> status::Created<Json<Post>> {
    let post = create_post(&conn, &post.title[..], &post.body[..]);

    return status::Created(format!("{}/posts/{}", APPLICATION_URL, post.id), Some(Json(post)))
}

fn main() {
	rocket::ignite()
		.manage(init_pool())
		.mount("/", routes![index, add_post, display_posts, display_post])
		.launch();
}