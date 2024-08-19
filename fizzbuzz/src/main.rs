use std::io::stdin;

fn main() {
    let mut buffer = String::new();
    stdin().read_line(&mut buffer).expect("error occurred.");

    let max = buffer.trim().parse::<u32>().unwrap();

    for num in 1..=max {
        match num {
            n if n % 15 == 0 => println!("Fizz Buzz!!"),
            n if n % 3 == 0 => println!("Fizz"),
            n if n % 5 == 0 => println!("Buzz"),
            n => println!("{}", n),
        }
    }
}
