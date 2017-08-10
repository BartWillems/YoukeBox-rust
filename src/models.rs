use super::schema::posts;

#[derive(Serialize, Deserialize)]
#[derive(Queryable)]
pub struct Post {
	pub id: i32,
	pub title: String,
	pub body: String,
	pub published: bool,
}

#[derive(Insertable)]
#[derive(Deserialize)]
#[table_name="posts"]
pub struct NewPost<'a> {
	pub title: &'a str,
	pub body: &'a str,
}

#[derive(Deserialize)]
pub struct AddPost {
	pub title: String,
	pub body: String,
}