#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;

use std::path::{PathBuf, Path};
use rocket::response::NamedFile;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/static/<file..>")]
pub fn static_server(file: PathBuf) -> Option<NamedFile> {
	NamedFile::open(Path::new("web/static/").join(file)).ok()
}

#[get("/")]
pub fn index() -> Option<NamedFile> {
	NamedFile::open(Path::new("web/index.html").join(file)).ok()
}

fn main() {
    rocket::ignite().mount("/", routes![
    	static_server,
    	index])
    	.launch();
}

