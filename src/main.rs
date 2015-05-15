use std::env;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() > 1 {
        let ref url = args[1];
        println!("url: {}", url);
    } else {
        // TODO: exit non-zero.
        println!("Please provide an URL.")
    }
}
