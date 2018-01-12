#![feature(io)]
extern crate petgraph;
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

use petgraph::Graph;
use petgraph::algo::tarjan_scc;

use Instruction::*;

#[derive(Debug, PartialEq)]
enum GoColor {
    White,
    Black,
}
#[derive(Debug)]
enum PointSt {
    White,
    Black,
    Free,
}
#[derive(Debug)]
enum Instruction {
    Move((usize, usize), GoColor),
    Setup((usize, usize), PointSt),
    NextPlayer(GoColor),
}

enum RuleSet {
    AGA,      // rules of the American Go Association
    GOE,      // the Ing rules of Goe
    Japanese, // the Nihon-Kiin rule set
    NZ,       // New Zealand rules
}

struct Model<'a> {
    node: &'a SgfNode,
    path: std::vec::Vec<usize>,

    annotation: Option<String>,
    copyright: Option<String>,
    date: Option<String>,
    place: Option<String>,
    event: Option<String>,
    round: Option<String>,
    game_name: Option<String>,
    game_info: Option<String>,
    handicap: Option<i32>,
    komi: Option<f32>,
    opening: Option<String>,
    rules: Option<RuleSet>,
    over_time: Option<String>,
    result: Option<String>,
    source: Option<String>,
    time_limits: Option<f32>,
    user: Option<String>,

    width: usize,
    height: usize,
}

fn get_board<'a>(node: &'a SgfNode) -> Result<Model<'a>, SgfError> {
    let char_set = node.get_text("CA");
    let file_format = node.get_number("FF");
    let game_type = node.get_number("GM").unwrap();
    if game_type != 1 {
        panic!("Error: this is not a Go game!");
    }

    //    let style = node.get_number("ST").unwrap();
    let (width, height) = match node.get_number("SZ") {
        Err(m) => {
            println!("no quadratic field");
            let (w, h) = node.get_number_number("SZ")
                .expect("Error no field size defined!");
            (w as usize, h as usize)
        }
        Ok(w) => (w as usize, w as usize),
    };

    // Root properties
    Ok(Model {
        node: node,
        path: vec![],

        annotation: node.get_text("AN").ok(),
        copyright: node.get_text("CP").ok(),
        date: node.get_text("DT").ok(),
        place: node.get_text("PC").ok(),
        event: node.get_text("EV").ok(),
        round: node.get_text("RO").ok(),
        game_name: node.get_text("GN").ok(),
        game_info: node.get_text("GC").ok(),
        handicap: node.get_number("HA").ok(),
        komi: node.get_real("KM").ok(),
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
    })
}

fn show_details(game: &Model) {
    if let Ok(ref name) = game.node.get_text("AP") {
        println!("Application: {}", name);
    } else {
        println!("Application:");
    }
}

fn show_board(game: &Model) {
    println!("{}", clear::All);

    // show some root properties
    if let Ok(ref name) = game.node.get_text("PW") {
        print!("White: {}, ", name);
    } else {
        print!("White: Unknown, ");
    }
    if let Ok(ref name) = game.node.get_text("WT") {
        print!("Team: {}, ", name);
    }
    if let Ok(ref rank) = game.node.get_text("WR") {
        println!("Rank: {}", rank);
    } else {
        println!("Rank: ? ");
    }
    if let Ok(ref name) = game.node.get_text("PB") {
        print!("Black: {}, ", name);
    } else {
        print!("Black: Unknown, ");
    }
    if let Ok(ref name) = game.node.get_text("BT") {
        print!("Team: {}, ", name);
    }
    if let Ok(ref rank) = game.node.get_text("BR") {
        println!("Rank: {}", rank);
    } else {
        println!("Rank: ? ");
    }

    let width = game.width;
    let height = game.height;

    println!("path: {:?}", game.path);
    let mut previous = game.path.clone();
    if previous.len() > 0 {
        previous.pop();
        if let Some(prev_node) = traverse(game.node, &previous) {
            println!("alternatives: {:?}", prev_node.children);
        } else {
            panic!("Error invalid path.");
        }
    }

    if let Some(cur_node) = traverse(game.node, &game.path) {
        let mut board = vec![0; width * height];

        let mut white_connections = Graph::<usize, usize>::new();
        let mut black_connections = Graph::<usize, usize>::new();

        // collect instructions
        let instructions = collect_moves(game.node, &game.path);

        // TODO: some state for the display
        let mut next_player = GoColor::White;
        for instr in instructions {
            match instr {
                Move((x, y), GoColor::White) => {
                    board[y * width + x] = 1;
                    let mut neighbors = vec![];
                    if x > 0 {
                        neighbors.push(y * width + x.checked_sub(1).unwrap());
                        if y > 0 {
                            neighbors.push(
                                y.checked_sub(1).unwrap() * width + x.checked_sub(1).unwrap(),
                            );
                        }
                    }
                    if y > 0 {
                        neighbors.push(y.checked_sub(1).unwrap() * width + x);
                    }
                    if x + 1 < width {
                        neighbors.push(y * width + x + 1);
                        if y + 1 < height {
                            neighbors.push(y + 1 * width + x + 1);
                        }
                    }
                    if y + 1 < height {
                        neighbors.push(y + 1 * width + x);
                    }

                    for pos in neighbors {
                        if board[pos] == 1 {
                            let a = white_connections.add_node(y * width + x);
                            let b = white_connections.add_node(pos);
                            let e = white_connections.add_edge(a, b, 1);
                        }
                    }

                    next_player = GoColor::Black;
                }
                Move((x, y), GoColor::Black) => {
                    board[y * width + x] = 2;
                    let mut neighbors = vec![];
                    if x > 0 {
                        neighbors.push(y * width + x.checked_sub(1).unwrap());
                        if y > 0 {
                            neighbors.push(
                                y.checked_sub(1).unwrap() * width + x.checked_sub(1).unwrap(),
                            );
                        }
                    }
                    if y > 0 {
                        neighbors.push(y.checked_sub(1).unwrap() * width + x);
                    }
                    if x + 1 < width {
                        neighbors.push(y * width + x + 1);
                        if y + 1 < height {
                            neighbors.push(y + 1 * width + x + 1);
                        }
                    }
                    if y + 1 < height {
                        neighbors.push(y + 1 * width + x);
                    }

                    for pos in neighbors {
                        if board[pos] == 2 {
                            let a = black_connections.add_node(y * width + x);
                            let b = black_connections.add_node(pos);
                            let e = black_connections.add_edge(a, b, 1);
                        }
                    }

                    next_player = GoColor::White;
                }
                Setup((x, y), PointSt::White) => {
                    board[y * width + x] = 1;

                    let mut neighbors = vec![];
                    if x > 0 {
                        neighbors.push(y * width + x.checked_sub(1).unwrap());
                        if y > 0 {
                            neighbors.push(
                                y.checked_sub(1).unwrap() * width + x.checked_sub(1).unwrap(),
                            );
                        }
                    }
                    if y > 0 {
                        neighbors.push(y.checked_sub(1).unwrap() * width + x);
                    }
                    if x + 1 < width {
                        neighbors.push(y * width + x + 1);
                        if y + 1 < height {
                            neighbors.push(y + 1 * width + x + 1);
                        }
                    }
                    if y + 1 < height {
                        neighbors.push(y + 1 * width + x);
                    }

                    for pos in neighbors {
                        if board[pos] == 1 {
                            let a = white_connections.add_node(y * width + x);
                            let b = white_connections.add_node(pos);
                            let e = white_connections.add_edge(a, b, 1);
                        }
                    }
                }
                Setup((x, y), PointSt::Black) => {
                    board[y * width + x] = 2;
                    let mut neighbors = vec![];
                    if x > 0 {
                        neighbors.push(y * width + x.checked_sub(1).unwrap());
                        if y > 0 {
                            neighbors.push(
                                y.checked_sub(1).unwrap() * width + x.checked_sub(1).unwrap(),
                            );
                        }
                    }
                    if y > 0 {
                        neighbors.push(y.checked_sub(1).unwrap() * width + x);
                    }
                    if x + 1 < width {
                        neighbors.push(y * width + x + 1);
                        if y + 1 < height {
                            neighbors.push(y + 1 * width + x + 1);
                        }
                    }
                    if y + 1 < height {
                        neighbors.push(y + 1 * width + x);
                    }

                    for pos in neighbors {
                        if board[pos] == 2 {
                            let a = black_connections.add_node(y * width + x);
                            let b = black_connections.add_node(pos);
                            let e = black_connections.add_edge(a, b, 1);
                        }
                    }
                }
                Setup((x, y), PointSt::Free) => board[y * width + x] = 0,
                NextPlayer(color) => next_player = color,
                i => panic!("Error: unexpected Instruction {:?}", i),
            }
        }

        //compute scc's
        if next_player == GoColor::White {
            let mut hr = white_connections.into_edge_type::<petgraph::Undirected>();
            let components = tarjan_scc(&hr);
            for c in components {
                println!("component:{:?}", c);
            }
        }

        if next_player == GoColor::Black {
            let mut hr = black_connections.into_edge_type::<petgraph::Undirected>();
            let components = tarjan_scc(&hr);
            for c in components {
                println!("component:{:?}", c);
            }
        }

        for y in 0..(height) {
            for x in 0..width {
                match board[y * width + x] {
                    0 => print!("+"),
                    1 => print!(
                        "{red}●{reset}",
                        red = color::Fg(color::Red),
                        reset = color::Fg(color::Reset)
                    ),
                    2 => print!(
                        "{blue}●{reset}",
                        blue = color::Fg(color::Blue),
                        reset = color::Fg(color::Reset)
                    ),
                    _ => println!(
                        "{red}Unknown player{reset}",
                        red = color::Fg(color::Red),
                        reset = color::Fg(color::Reset)
                    ),
                }
            }
            println!();
        }

        // Show current node annotations
        if let Ok(node_name) = cur_node.get_simple_text("N") {
            println!("Node name: {}", node_name);
        }
        if let Ok(comment) = cur_node.get_text("C") {
            println!("Comment: {}", comment);
        }
        if let Ok(n) = cur_node.get_double("DM") {
            println!("Even position! {}", n);
        }
        if let Ok(n) = cur_node.get_double("GB") {
            println!("Position is good for black! {}", n);
        }
        if let Ok(n) = cur_node.get_double("GW") {
            println!("Position is good for white! {}", n);
        }
        if let Ok(n) = cur_node.get_double("HO") {
            println!("Hotspot! {}", n);
        }
        if let Ok(n) = cur_node.get_double("UC") {
            println!("Unclear position! {}", n);
        }
        if let Ok(n) = cur_node.get_double("V") {
            println!("Value! {}", n);
        }

        // Move annotations properties
        if let Ok(_) = cur_node.get_double("BM") {
            println!("Bad move!");
        }
        if let Ok(_) = cur_node.get_text("DO") {
            println!("Doubtful move!");
        }
        if let Ok(_) = cur_node.get_text("IT") {
            println!("Interesting move!");
        }
        if let Ok(_) = cur_node.get_text("TE") {
            println!("Tesuji!");
        }

        // Markup properties
        if let Ok(labels) = cur_node.get_points("LB") {
            println!("labels: {:?}", labels);
        }
        if let Ok(points) = cur_node.get_points("MA") {
            println!("mark x: {:?}", points);
        }
        if let Ok(points) = cur_node.get_points("CR") {
            println!("circles: {:?}", points);
        }
        if let Ok(points) = cur_node.get_points("SQ") {
            println!("squares: {:?}", points);
        }
        if let Ok(points) = cur_node.get_points("TR") {
            println!("triangles: {:?}", points);
        }
        if let Ok(points) = cur_node.get_points("SL") {
            println!("selected: {:?}", points);
        }
        if let Ok(points) = cur_node.get_points("DD") {
            println!("DD dim: {:?}", points);
        }
        if let Ok(points) = cur_node.get_points("AR") {
            println!("arrows: {:?}", points);
        }
        if let Ok(points) = cur_node.get_points("LN") {
            println!("lines: {:?}", points);
        }
    } else {
        panic!("Error invalid path");
    }
}

fn compute_black_groups() {}

fn compute_white_groups() {}

fn next_board(game: &mut Model) {
    game.path.push(0);
    if let Some(cur_node) = traverse(game.node, &game.path) {

    } else {
        println!("Last node");
        game.path.pop();
    }
}

fn alt_right(game: &mut Model) {
    if let Some(mut last) = game.path.pop() {
        if let Some(node) = traverse(game.node, &game.path) {
            if last + 1 < node.children.len() {
                game.path.push(last + 1);
            } else {
                game.path.push(last);
            }
        } else {
            panic!("Invalid path");
        }
    }
}

fn alt_left(game: &mut Model) {
    if let Some(mut last) = game.path.pop() {
        if last > 0 {
            game.path.push(last - 1);
        } else {
            game.path.push(0);
        }
    }
}

fn previous_board(game: &mut Model) {
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

fn collect_moves<'a>(node: &'a SgfNode, path: &[usize]) -> Vec<Instruction> {
    let mut moves = vec![];
    if let Some((first, elements)) = path.split_first() {
        if node.children.len() > *first {
            moves = collect_moves(&node.children[*first], elements)
        }
    }

    // setup properties
    if let Ok(list) = node.get_points("AW") {
        for s in list {
            let (x, y) = str_to_position(&s);
            moves.push(Setup((x, y), PointSt::White))
        }
    }
    if let Ok(list) = node.get_points("AB") {
        for s in list {
            let (x, y) = str_to_position(&s);
            moves.push(Setup((x, y), PointSt::Black))
        }
    }
    if let Ok(list) = node.get_points("AE") {
        for s in list {
            let (x, y) = str_to_position(&s);
            moves.push(Setup((x, y), PointSt::Free))
        }
    }
    if let Ok(c) = node.get_color("PL") {
        //TODO detect color
        println!("set next player {}", c);
        moves.push(NextPlayer(GoColor::White))
    }

    // move properties
    if let Ok(s) = node.get_point("W") {
        let (x, y) = str_to_position(&s);
        moves.push(Move((x, y), GoColor::White))
    }
    if let Ok(s) = node.get_point("B") {
        let (x, y) = str_to_position(&s);
        moves.push(Move((x, y), GoColor::Black))
    }
    if let Ok(_) = node.get_text("KO") {
        //TODO set move status to illegal
    }
    if let Ok(n) = node.get_number("MN") {
        //TODO set move number
    }
    //TODO: Move annotations properties
    //         if let Ok(_) = cur_node.get_double("BM"){
    //           println!("Bad move!");
    //         }
    //         if let Ok(_) = cur_node.get_text("DO"){
    //           println!("Doubtful move!");
    //         }
    //         if let Ok(_) = cur_node.get_text("IT"){
    //           println!("Interesting move!");
    //         }
    //         if let Ok(_) = cur_node.get_text("TE"){
    //           println!("Tesuji!");
    //         }
    moves
}

fn str_to_position(s: &str) -> (usize, usize) {
    (
        char2int(s.chars().nth(0).unwrap()),
        char2int(s.chars().nth(1).unwrap()),
    )
}

fn char2int(c: char) -> usize {
    match c {
        'a' => 0,
        'b' => 1,
        'c' => 2,
        'd' => 3,
        'e' => 4,
        'f' => 5,
        'g' => 6,
        'h' => 7,
        'i' => 8,
        'j' => 9,
        'k' => 10,
        'l' => 11,
        'm' => 12,
        'n' => 13,
        'o' => 14,
        'p' => 15,
        'q' => 16,
        'r' => 17,
        's' => 18,
        't' => 19,
        'u' => 20,
        'v' => 21,
        'w' => 22,
        'x' => 23,
        'y' => 24,
        'z' => 25,
        'A' => 26,
        'B' => 27,
        'C' => 28,
        'D' => 29,
        'E' => 30,
        'F' => 31,
        'G' => 32,
        'H' => 33,
        'I' => 34,
        'J' => 35,
        'K' => 36,
        'L' => 37,
        'M' => 38,
        'N' => 39,
        'O' => 40,
        'P' => 41,
        'Q' => 42,
        'R' => 43,
        'S' => 44,
        'T' => 45,
        'U' => 46,
        'V' => 47,
        'W' => 48,
        'X' => 49,
        'Y' => 50,
        'Z' => 51,
        _ => panic!("cannot handle coordinate {}", c),
    }
}

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
                                alt_left(&mut game);
                            }
                            's' => {
                                println!("You pressed char {:?}", c);
                                next_board(&mut game);
                            }
                            'd' => {
                                println!("You pressed char {:?}", c);
                                alt_right(&mut game);
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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
