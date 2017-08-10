#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate diesel_demo;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate rocket;
extern crate rocket_contrib;
extern crate serde;
extern crate serde_json;

use rocket_contrib::{Json};
use self::diesel_demo::*;
use self::diesel_demo::models::*;

#[get("/")]
fn index() -> &'static str {
	"It works!"
}

#[get("/posts")]
fn display_posts(conn: DbConn) -> Json<Vec<Post>> {
	Json(get_posts(&conn))
}

#[get("/posts/<id>")]
fn display_post(conn: DbConn, id: i32) -> Json<Post> {
	Json(get_post(&conn, id))
}

#[post("/posts/add", format = "application/json", data = "<post>")]
fn add_post(conn: DbConn, post: Json<AddPost>) -> Json<Post> {
	Json(create_post(&conn, &post.title[..], &post.body[..]))
}


fn main() {
	rocket::ignite()
		.manage(init_pool())
		.mount("/", routes![index, add_post, display_posts, display_post])
		.launch();
}