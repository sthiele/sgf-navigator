#![feature(io)]
extern crate sgf;

use std::env;
use std::path::Path;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::error::Error;
use sgf::sgf_node::SgfCollection;
use sgf::sgf_node::SgfNode;


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

        if let Some(board) = get_board(&c){

          let mut game_count = 1;
          let mut ioin = io::stdin();
          for char in ioin.lock().chars() {
             show_board(&board);
                    let c = char.unwrap();
                    match c {
                        'w' => { println!("You pressed char {:?}", c);  },
                        'a' => { println!("You pressed char {:?}", c);  },
                        's' => { println!("You pressed char {:?}", c);  },
                        'd' => { println!("You pressed char {:?}", c);  },
                        'q' => { break;  },
                        _   => {           },
                    }
           }
        }
        else {
           println!("Empty SGF");
        }
    }
    else {
        println!("Usage: sgf-reader filename");
    }
}

struct Board<'a> {
  node: &'a SgfNode,
  field : std::vec::Vec<u8>,
}

fn get_board<'a>(c: &'a SgfCollection) -> Option<Board<'a>> {
    println!("collection: {}",c);
    if c.len() == 0 { () }
    let node = c.first().unwrap();
    Some(Board {
    node : node,
    field : vec![],
    })
}

fn show_board(b: &Board) {
  let player = b.node.get_text("PB").unwrap();
  println!("Player : {}",player);
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
