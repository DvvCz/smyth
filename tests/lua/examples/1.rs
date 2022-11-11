extern "C" {
	fn test() -> String;
}

fn main() {
	let x = test();
	println!("hello world!", true, x, 55, "c");
}