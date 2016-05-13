use std::env;
use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Waited lexems file");
        return;
    }

    let lexems_file = File::open(&args[1]).unwrap();
    let file = BufReader::new(&lexems_file);

    let mut lines = vec![];

    for line in file.lines() {
        lines.push(line.unwrap());
    }

    for keyword in &lines {
        println!("T_{},", keyword);
    }

    println!("------------------------------");

    for keyword in &lines {
        println!("r#\"{}\"# => (Token::T_{}, text),", keyword, keyword);
    }

    println!("------------------------------");

    for keyword in &lines {
        print!(" &T_{} |", keyword);
    }
}
