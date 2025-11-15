use text_io::read;

fn main() {
    println!("Welcome to my physics simulation playground");
    println!("Please type: test or exit");
    // read until a whitespace (but not including it)
    let word: String = read!(); // same as read!("{}")
    if word == "test" {
        rhysics::run().unwrap();
    } else {
        println!("exiting...")
    }
}
