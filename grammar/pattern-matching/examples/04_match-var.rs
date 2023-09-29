fn main() {
    let x = Some(5);
    let y = 10;

    match x {
        Some(50) => println!("Got 50"),
        Some(y) => println!("Matched, y is {:?}", y),
        _ => println!("Default case, x is {:?}", x),
    }
    println!("at the end: x = {:?}, y = {:?}", x, y);
}
