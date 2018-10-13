#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;

use std::path::{PathBuf, Path};
use rocket::response::NamedFile;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/static/<file..>", rank = 10)]
pub fn static_server(file: PathBuf) -> Option<NamedFile> {
	NamedFile::open(Path::new("static/").join(file)).ok()
}

fn main() {
    rocket::ignite().mount("/", routes![
    	static_server
    	])
    .launch();
}

