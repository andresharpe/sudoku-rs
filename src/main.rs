use std::io::Write;
use std::fs::OpenOptions;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::str;
use rand::seq::SliceRandom;
use rand::thread_rng;
use clap::{Arg, App};
use std::time::{Instant};

static mut LIMIT_SOLUTIONS: u32 = 1;
static mut SOLUTIONS_FOUND: u32 = 0;
static mut PUZZLE: [u8;81] = [0; 81];

fn main() {
    let now = Instant::now();

    let app = App::new("Sudoko CLI")
        .version("0.1.0")
        .author("Andre Sharpe <andre.sharpe@gmail.com>")
        .about("Solves and generates Sudoku puzzles")
        .arg(Arg::with_name("solve")
                 .short("s")
                 .long("solve")
                 .takes_value(false)
                 .conflicts_with("generate")
                 .help("Solves puzzles in a text file"))
        .arg(Arg::with_name("generate")
                 .short("g")
                 .long("generate")
                 .takes_value(false)
                 .conflicts_with("solve")
                 .help("Generates puzzles and appends them to a text file"))
        .arg(Arg::with_name("file")
                 .short("f")
                 .long("file")
                 .takes_value(true)
                 .help("A file containing puzzles, one per line. Defaults to .\\puzzles.txt"))
        .arg(Arg::with_name("number")
                 .short("n")
                 .conflicts_with("solve")
                 .long("number")
                 .takes_value(true)
                 .help("The number of puzzles to generate and append to file"));

    let matches = app.get_matches();
    let filename = matches.value_of("file").unwrap_or(".\\puzzle.txt");
    
    if matches.is_present("solve") {
        solve_from_file( &filename );
    }
    else if matches.is_present("generate"){
        let number = matches.value_of("number").unwrap_or("10").parse::<u32>().unwrap_or(10);
        generate_to_file( &filename, number );
    }

    println!("Elapsed time: {} seconds.", now.elapsed().as_secs());

}

fn solve_from_file( filename: &str ){
    let mut l = 0;
    if let Ok(lines) = read_lines(filename) {
        // Consumes the iterator, returns an (Optional) String
        for line in lines {
            if let Ok(s_puzzle) = line {
                unsafe {  // because it updates static variables
                    l += 1;
                    println!("Solving puzzle: {}", l);
                    string_to_puzzle(s_puzzle);
                    print_puzzle();
                    solve_puzzle();
                    print_puzzle();
                }
            }
        }
    }
}

fn generate_to_file( filename: &str, number: u32){
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(filename)
        .unwrap();
    unsafe {  // because it updates static variables
        for i in 0..number {
            println!("Generating puzzle {} of {}:", i+1, number );
            generate_puzzle();
            print_puzzle();
            let s_puzzle = puzzle_to_string();
            file.write_all("\n".as_bytes()).expect("Write failed.");
            file.write_all(s_puzzle.as_bytes()).expect("Write failed.");
        }
    }
}

unsafe fn string_to_puzzle( s_puzzle: String ) {
    let x_puzzle = s_puzzle.as_bytes();
    for i in 0..81{
        PUZZLE[i] = x_puzzle[i] - 48;
    }
}

unsafe fn puzzle_to_string() -> String {
    let mut x_puzzle: [u8;81] = [0; 81];
    for i in 0..81{
        x_puzzle[i] = PUZZLE[i] + 48;
    }
    let s_puzzle = str::from_utf8(&x_puzzle).unwrap();
    return String::from(s_puzzle);
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

unsafe fn print_puzzle() {
    println!( " +---------+---------+---------+ " ); 
    for i in 0..81{
        if i % 9 == 0  { print!(" |"); }        
        if PUZZLE[i] == 0 { print!(" . ") } else { print!(" {} ", PUZZLE[i] ) };
        if (i+1) % 3 == 0 { print!("|"); }      
        if i != 80 {                            
            if (i+1) % 9 == 0  { println!(); }  
            if (i+1) % 27 == 0 { 
                println!(" |---------+---------+---------| "); 
            } 
        }   
    }
    println!();
    println!( " +---------+---------+---------+ " );
    println!();
}

unsafe fn solve_puzzle() {
    SOLUTIONS_FOUND = 0;
    solve();
}

unsafe fn solve() {
    let mut numbers = [1,2,3,4,5,6,7,8,9];
    let mut random_number_gen = thread_rng();
    for i in 0..81{
        if PUZZLE[i] == 0 {
            numbers.shuffle(&mut random_number_gen);
            for value in 0..9{
                if is_valid_value( i, numbers[value] ){
                    PUZZLE[i] = numbers[value];
                    solve(); // recurse!
                    if SOLUTIONS_FOUND >= LIMIT_SOLUTIONS { return; }
                    PUZZLE[i] = 0;
                }
            }
            return;
        }
    }
    SOLUTIONS_FOUND += 1;  // only reaches this point recursively when all cells are solved
}

unsafe fn is_valid_value( pos: usize, value: u8 ) -> bool {
    let y = pos / 9;
    let x = pos % 9;
    // Check for value in columns and rows
    for i in 0..9 {
        if PUZZLE[ y*9 + i ] == value { return false; } // check row
        if PUZZLE[ i*9 + x ] == value { return false; } // check column
    }
    // check for value in same block
    let block_row = y/3 * 3; // find top row
    let block_col = x/3 * 3; // find left column
    for block_y in block_row..block_row+2 {
        for block_x in block_col..block_col+2 {
            if PUZZLE[ (block_y*9) + block_x ] == value { return false; } 
        }
    }
    return true;
}

unsafe fn generate_puzzle() {

    // generate a random solution
    clear_puzzle();
    LIMIT_SOLUTIONS = 1;
    solve_puzzle();

    let mut new_puzzle: [u8;81] = [0; 81];
    for i in 0..81 { new_puzzle[i] = PUZZLE[i]; }

    // remove numbers from solved board
    let mut removelist: [usize;81] = [0; 81];
    for i in 0..81 { removelist[i] = i; }
    let mut random_number_gen = thread_rng();
    removelist.shuffle(&mut random_number_gen);

    // systematically remove a number and confirm there is only one solution all the way or reverse it
    LIMIT_SOLUTIONS = 2;
    for i in 0..81 { 
        let save_item = new_puzzle[ removelist[i] ];
        new_puzzle[ removelist[i] ] = 0;
        set_puzzle( new_puzzle ); 
        solve_puzzle();
        if SOLUTIONS_FOUND != 1 {
            new_puzzle[ removelist[i] ] = save_item;
        }
    }

    // done
    set_puzzle( new_puzzle );
    LIMIT_SOLUTIONS = 1;
}

unsafe fn clear_puzzle() {
    for i in 0..81 { PUZZLE[i] = 0; }
}

unsafe fn set_puzzle( puzzle: [u8;81] ) {
    for i in 0..81 { PUZZLE[i] = puzzle[i]; }
}