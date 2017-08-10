#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket_contrib;

extern crate serde;
extern crate serde_json;
extern crate rocket;
extern crate diesel_demo;
extern crate r2d2_diesel;
extern crate r2d2;

use self::diesel_demo::*;
use self::diesel_demo::models::*;
use rocket_contrib::{Json};

#[get("/")]
fn index() -> &'static str {
	"It works!"
}

#[post("/posts/add", data = "<post>")]
fn add_post(conn: DbConn, post: Json<AddPost>) -> Json<Post> {
	Json(create_post(&conn, &post.title[..], &post.body[..]))
}

#[get("/posts")]
fn display_posts(conn: DbConn) -> Json<Vec<Post>> {
	Json(get_posts(&conn))
}


fn main() {
	rocket::ignite()
		.manage(init_pool())
		.mount("/", routes![index, add_post, display_posts])
		.launch();
}