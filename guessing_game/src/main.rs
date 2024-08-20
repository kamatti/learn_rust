use std::io;
use rand::Rng;
use std::cmp::Ordering;

fn main() {

    let secret_number = rand::thread_rng().gen_range(1..=100);

    loop {
        println!("Please input your guess (0-100): ");
        let mut guessed_number = String::new();
        io::stdin()
            .read_line(&mut guessed_number)
            .expect("error occurred.");
        let guessed_number = guessed_number.trim().parse::<u32>().expect("you can input only number");

        match guessed_number.cmp(&secret_number) {
            Ordering::Greater => println!("Too big"),
            Ordering::Less => println!("Too small"),
            Ordering::Equal => {
                println!("You win!");
                break;
            }
        }
    }
}
