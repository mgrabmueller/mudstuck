// Copyright 2016 Martin Grabmueller. See the LICENSE file at the
// top-level directory of this distribution for license information.

//! Mudstuck main binary.

extern crate mudstuck;
extern crate rustyline;

use mudstuck::*;
use mudstuck::types::*;

fn show_help() {
    println!("Commands:");
    println!("  help or h   show this help");
    println!("  quit or q   quit the game");
    println!("  look or l   describe your surroundings");
}

fn repl(ps: &PlayerState) {
    let mut rl = rustyline::Editor::new();
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(ref s) if s == "quit" || s == "q" => break,
            Ok(ref s) if s == "look" || s == "l" => ps.look(),
            Ok(ref s) if s == "help" || s == "h" => show_help(),
            Ok(ref s) if s == "desc" || s == "d" => ps.describe("rusty.metal.door"),
            Ok(ref s) =>
                match command::parse(s) {
                    Err(e) => {
                        println!("I don't know how to do that.");
                        println!("({})", e);
                    },
                    Ok(cmd) => {
                        println!("trying to {:?}", cmd);
                        println!("I don't know how to do that.");
                    },
                },
            Err(_)   => println!("No input"),
        }
    }
}

fn main() {
    println!("If you don't know what to do, type \"help\" (without the quotes).");
    println!("To leave the game, type \"quit\".");
    println!("");

    let w = make_example_world();
    let ps = PlayerState {
        world: &w,
        location: w.start_location,
    };

    repl(&ps);
}
