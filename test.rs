extern fn lua_func() {}

mod namespace {
    fn test() {

    }
}

fn main() {
    let x = 5;
    let y = "Test\u{242}";
    let b = true;

    namespace::test();

    x += 5;

    let y = |a, b, c| {
        print(a + b * c);
    };

    y(1, 2, 3);

    let array = [1, 2, 3];

    fn test(a: i32) {
        print("Hello", a)
    }

    /*if true {
        print("so true");
    } else if false {
        print("so false");
    } else {
        print("?")
    }*/

    loop {
        print("Loop");
        break;
    }

    while true {
        break;
    }

    test(5)
}