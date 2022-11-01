fn main() {
    let x = 5;
    let y = "Test\u{242}";
    let b = true;

    fn test(a: i32) {
        print("Hello", a)
    }

    test(5)
}