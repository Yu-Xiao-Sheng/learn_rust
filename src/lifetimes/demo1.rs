fn as_str<'a>(data: &'a u32) -> &'a str {
    'b: {
        let s = format!("{}", data);
        return &s
    }
}

#[test]
fn main() {
    'c: {
        let x: u32 = 0;
        'd: {
            // 引入了一个匿名作用域，因为借用不需要持续整个 x 的有效期。as_str 的返回值必须在此函数调用之前找到一个 str。显然不可能。
            println!("{}", as_str(&x));
        }
    }
}