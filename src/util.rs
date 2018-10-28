//! Utility

use ring::digest::{digest, SHA256};

#[derive(Deserialize, Debug, Clone)]
pub struct NewContent {
	pub token: String,
	pub content: String,
}


/// Generuje hash ze zdroje
pub fn make_hash(src: &str) -> String {
	digest(&SHA256, src.as_bytes())
		.as_ref()
		.iter()
		.fold(String::new(), |mut a, x| {a.push_str(&format!("{:x}", x)); a})
}
