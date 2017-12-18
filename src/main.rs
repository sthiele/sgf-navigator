#![feature(io)]
extern crate sgf;
extern crate termion;

use std::env;
use std::path::Path;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::error::Error;
use sgf::sgf_node::SgfCollection;
use sgf::sgf_node::SgfNode;
use sgf::sgf_node::SgfError;

use termion::clear;
use termion::color;

fn main() {

    // iterator to the command line options
    let mut options = env::args();

    if let Some(x) = options.nth(1) {

        // Create a path to the file
        let path = Path::new(&x);
        let display = path.display();

        // Open the path in read-only mode, returns `io::Result<File>`
        let mut file = match File::open(&path) {
            // The `description` method of `io::Error` returns a string that
            // describes the error
            Err(why) => panic!("couldn't open {}: {}", display, why.description()),
            Ok(file) => file,
        };

        // Read the file contents into a string, returns `io::Result<usize>`
        let mut s = String::new();
        match file.read_to_string(&mut s) {
            Err(why) => panic!("couldn't read {}: {}", display, why.description()),
            Ok(_) => println!("{} read", display),
        }

        let c = SgfCollection::from_sgf(&s).expect("Error parsing SGF");

        println!("collection of {} games: {}", c.len(), c);

        if c.len() > 0 {
            let sgf_game = c.first().unwrap();
            let mut game_count = 1;
            match get_board(&sgf_game) {
                Ok(mut game) => {
                    let mut ioin = io::stdin();
                    for char in ioin.lock().chars() {
                        show_board(&game);
                        let c = char.unwrap();
                        match c {
                            'w' => {
                                println!("You pressed char {:?}", c);
                                previous_board(&mut game);
                            }
                            'a' => {
                                println!("You pressed char {:?}", c);
                            }
                            's' => {
                                println!("You pressed char {:?}", c);
                                next_board(&mut game);
                            }
                            'd' => {
                                println!("You pressed char {:?}", c);
                            }
                            'q' => {
                                break;
                            }
                            _ => {}
                        }
                    }
                }
                Err(m) => println!("some error: "),
            }

        } else {
            println!("Empty SGF");
        }
    } else {
        println!("Usage: sgf-reader filename");
    }
}

enum RuleSet {
    AGA, // rules of the American Go Association
    GOE, // the Ing rules of Goe
    Japanese, // the Nihon-Kiin rule set
    NZ, // New Zealand rules
}

struct Game<'a> {
    node: &'a SgfNode,
    path: std::vec::Vec<usize>,

    app: Option<String>,
    annotation: Option<String>,
    copyright: Option<String>,
    date: Option<String>,
    event: Option<String>,
    round: Option<String>,
    game_name: Option<String>,
    game_info: Option<String>,
    opening: Option<String>,
    rules: Option<RuleSet>,
    over_time: Option<String>,
    result: Option<String>,
    source: Option<String>,
    time_limits: Option<f32>,
    user: Option<String>,

    width: usize,
    height: usize,
    white_name: Option<String>,
    white_team: Option<String>,
    white_rank: Option<i32>,
    black_name: Option<String>,
    black_team: Option<String>,
    black_rank: Option<i32>,
}

fn get_board<'a>(node: &'a SgfNode) -> Result<Game<'a>, SgfError> {
    let char_set = node.get_text("CA");
    let file_format = node.get_number("FF");
    let game_type = node.get_number("GM").unwrap();
    if game_type != 1 {
        panic!("This is not a Go game");
    }

    //    let style = node.get_number("ST").unwrap();
    let (width, height) = match node.get_number("SZ") {
        Err(m) => {
            println!("no quadratic field");
            let (w, h) = node.get_number_number("SZ").expect(
                "Error no field size defined!",
            );
            (w as usize, h as usize)
        }
        Ok(w) => (w as usize, w as usize),
    };

    Ok(Game {
        node: node,
        path: vec![],

        app: node.get_text("AP").ok(),
        annotation: node.get_text("AN").ok(),
        copyright: node.get_text("CP").ok(),
        date: node.get_text("DT").ok(),
        event: node.get_text("EV").ok(),
        round: node.get_text("RO").ok(),
        game_name: node.get_text("GN").ok(),
        game_info: node.get_text("GC").ok(),
        opening: node.get_text("ON").ok(),
        rules: match node.get_text("RU") {
            Ok(s) => {
                if s == "AGA" {
                    Some(RuleSet::AGA)
                } else if s == "GOE" {
                    Some(RuleSet::GOE)
                } else if s == "Japanese" {
                    Some(RuleSet::Japanese)
                } else if s == "NZ" {
                    Some(RuleSet::NZ)
                } else {
                    None
                }
            }
            _ => None,
        },
        over_time: node.get_text("OT").ok(),
        result: node.get_text("RE").ok(),
        source: node.get_text("SO").ok(),
        time_limits: node.get_real("TM").ok(),
        user: node.get_text("US").ok(),

        width: width,
        height: height,
        white_name: node.get_text("PW").ok(),
        white_team: node.get_text("WT").ok(),
        white_rank: node.get_number("WR").ok(),
        black_name: node.get_text("PB").ok(),
        black_team: node.get_text("BT").ok(),
        black_rank: node.get_number("BR").ok(),
    })
}

fn show_board(game: &Game) {
    println!("{}", clear::All);

    println!("White : {:?} {:?}", game.white_name, game.white_rank);
    println!("Black : {:?} {:?}", game.black_name, game.black_rank);

    let mut board = vec![0; game.width * game.height];
    // collect_moves
    let moves = collect_moves(game.node, &game.path);
    for (x, y, i) in moves {
        board[(x - 1) * game.width + y] = i;
    }

    for y in 1..game.height {
        for x in 1..game.width {
            match board[x * y - 1] {
                0 => print!("+"),
                1 => {
                    print!(
                        "{red}●{reset}",
                        red = color::Fg(color::Red),
                        reset = color::Fg(color::Reset)
                    )
                }
                2 => {
                    print!(
                        "{blue}●{reset}",
                        blue = color::Fg(color::Blue),
                        reset = color::Fg(color::Reset)
                    )
                }
                _ => {
                    println!(
                        "{red}Unknown player{reset}",
                        red = color::Fg(color::Red),
                        reset = color::Fg(color::Reset)
                    )
                }

            }
        }
        println!("");
    }
}

fn next_board(game: &mut Game) {
    game.path.push(0);
    if let Some(cur_node) = traverse(game.node, &game.path) {

    } else {
        println!("Last node");
        game.path.pop();
    }
}

fn previous_board(game: &mut Game) {
    game.path.pop();
}

fn traverse<'a>(node: &'a SgfNode, path: &[usize]) -> Option<&'a SgfNode> {
    if let Some((first, elements)) = path.split_first() {
        if node.children.len() > *first {
            traverse(&node.children[*first], elements)
        } else {
            None
        }
    } else {
        Some(node)
    }
}

fn collect_moves<'a>(node: &'a SgfNode, path: &[usize]) -> Vec<(usize, usize, u8)> {
    let mut moves = vec![];
    if let Some((first, elements)) = path.split_first() {
        if node.children.len() > *first {
            moves = collect_moves(&node.children[*first], elements)
        }
    }
    if let Ok(s) = node.get_point("W") {
        let (x, y) = coordinates_to_position(&s);
        moves.push((x, y, 1));
    }
    if let Ok(s) = node.get_point("B") {
        let (x, y) = coordinates_to_position(&s);
        moves.push((x, y, 2));
    }
    moves
}

fn coordinates_to_position(s: &str) -> (usize, usize) {
    (
        char2int(s.chars().nth(0).unwrap()),
        char2int(s.chars().nth(1).unwrap()),
    )
}

fn char2int(c: char) -> usize {
    match c {
        'a' => 1,
        'b' => 2,
        'c' => 3,
        'd' => 4,
        'e' => 5,
        'f' => 6,
        'g' => 7,
        'h' => 8,
        'i' => 9,
        'j' => 10,
        'k' => 11,
        'l' => 12,
        'm' => 13,
        'n' => 14,
        'o' => 15,
        'p' => 16,
        'q' => 17,
        'r' => 18,
        's' => 19,
        't' => 20,
        'u' => 21,
        'v' => 22,
        'w' => 23,
        'x' => 24,
        'y' => 25,
        'z' => 26,
        'A' => 27,
        'B' => 28,
        'C' => 29,
        'D' => 30,
        'E' => 31,
        'F' => 32,
        'G' => 33,
        'H' => 34,
        'I' => 35,
        'J' => 36,
        'K' => 37,
        'L' => 38,
        'M' => 39,
        'N' => 40,
        'O' => 41,
        'P' => 42,
        'Q' => 43,
        'R' => 44,
        'S' => 45,
        'T' => 46,
        'U' => 47,
        'V' => 48,
        'W' => 49,
        'X' => 50,
        'Y' => 51,
        'Z' => 52,
        _ => panic!("cannot handle coordinate {}", c),
    }
}



#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
