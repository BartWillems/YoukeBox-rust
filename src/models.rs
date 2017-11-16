#[derive(Serialize)]
pub struct HttpStatus {
    pub status: u16,
    pub message: String,
}