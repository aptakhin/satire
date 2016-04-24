use std::env;
use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;

fn main() {
    let args: Vec<String> = env::args().collect();

    // The first argument is the path that was used to call the program.
    println!("My path is {}.", args[0]);

    if args.len() < 2 {
        println!("Waited lexems file");
        return;
    }

    let mut lexems_file = File::open(&args[1]).unwrap();
    let mut file = BufReader::new(&lexems_file);

    for line in file.lines() {

        let enum_token = line.unwrap();
        let keyword = enum_token.to_lowercase();
        //println!("r#\"{}\"# => (Token::{}, text),", keyword, enum_token);
        print!(" &{} |", enum_token);
    }

    //panic!("No");

    // The rest of the arguments are the passed command line parameters.
    // Call the program like this:
    //   $ ./args arg1 arg2
    //println!("I got {:?} arguments: {:?}.", args.len() - 1, &args[1..]);


    //let mut template = String::new();
    //template_file.read_to_string(&mut template);
}
