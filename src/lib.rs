use std::collections::HashMap;

type DSpan<'a> = Vec<&'a DocElement<'a>>;

enum DocElement<'a> {
	DString(String),
	DGroup(HashMap<String, String>, DSpan<'a>),
}

#[test]
fn try_this() {
	fn it_works(val:DocElement) {
		match val {
			DocElement::DString(..) => {},
			DocElement::DGroup(..) => {},
		}
	}

	it_works(DocElement::DString("Hello world!".to_string()));
	it_works(DocElement::DGroup(HashMap::new(), vec![]));
}


fn main() {
	let val = DocElement::DString("Hello world!".to_string());
	match val {
		_ => {},
	}
}