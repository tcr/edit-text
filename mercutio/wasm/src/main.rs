extern crate serde_json;
extern crate mercutio_web;

use mercutio_web::{command_safe, NativeRequest, NativeResponse};
// use mercutio_web::command_safe;

fn main() {
    let input = r#"
{
      "RenameGroup":
        [{"CurSkip":1},{"CurWithGroup":[{"CurWithGroup":["CurGroup"]}]}]
    }
    "#;

    let req: NativeRequest = serde_json::from_str(&input).unwrap();
    println!("{:?}", command_safe(req));
}