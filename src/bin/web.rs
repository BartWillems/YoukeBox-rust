#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate diesel_demo;
// #[macro_use] extern crate diesel;
// #[macro_use] extern crate diesel_codegen;
extern crate r2d2_diesel;
extern crate r2d2;

use self::diesel_demo::*;
// use self::diesel_demo::models::*;
// use self::diesel::prelude::*;
// use self::diesel::


#[get("/")]
fn index() -> &'static str {
	"It works!"
}

#[get("/posts/add")]
fn add_post(conn: DbConn) -> &'static str {
	// let connection = establish_connection();

	// println!("What would you like your title to be?");
	// let mut title = String::new();
	// stdin().read_line(&mut title).unwrap();

	// let title = &title[..(title.len() -1)]; // Drop the \n newline char
	// println!("\nOk! Let's write {} (Press {} when finished)\n", title, EOF);
	// let mut body = String::new();
	// stdin().read_to_string(&mut body).unwrap();

	// let post = create_post(&connection, title, &body);
	// println!("\nSaved draft {} with id {}", title, post.id);

	let title = "Bost title";
	let body = "SOme artikle body";

	create_post(&conn, title, body);

	"test"
}

fn main() {
	rocket::ignite()
		.manage(init_pool())
		.mount("/", routes![index, add_post])
		.launch();
}