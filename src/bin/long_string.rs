fn main() {
    let s = "👩‍👩‍👧‍👦";
    for char in s.chars() {
        println!("{} {}", char, char.len_utf8());
    }
    let len = s.len();
    println!("{}", len);
}
