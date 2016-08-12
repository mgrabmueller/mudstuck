// Copyright 2016 Martin Grabmueller. See the LICENSE file at the
// top-level directory of this distribution for license information.

//! Mudstuck commands.  Commands are simple sentences of the form VERB
//! [OBJECT] [CONNECTOR OBJECT], where the first object as well as the
//! connector word and the second object are optional.  Verbs indicate
//! the intended action.  The first object is the object that is to be
//! manipulated in some way, and the connector and second object give
//! details on how to do that.  Objects are named by one or more
//! words, each of which may not be a valid connector (so that parsing
//! works).
//!
//! For example, the following commands are syntactically valid:
//!
//! eat
//! get lamp
//! put coin into purse


use super::types;
use super::error;

#[derive(Debug)]
pub struct Command {
    pub verb: Verb,
    pub direct_object: Option<types::Name>,
    pub indirect_object: Option<(Connector, types::Name)>,
}

#[derive(Debug, Clone, Copy)]
pub enum Verb {
    Get,
    Put,
    Use,
    Move,
    Buy,
    Drink,
    Eat,
    Sleep,
}

#[derive(Debug, Clone, Copy)]
pub enum Connector {
    Into,
    Onto,
    Under,
    Beside,
    To,
    From,
    With,
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

pub const VERBS: &'static[(&'static str, Verb)] =
    &[
        ("get", Verb::Get),
        ("take", Verb::Get),
        ("acquire", Verb::Get),
        ("put", Verb::Put),
        ("give", Verb::Put),
        ("toss", Verb::Put),
        ("drop", Verb::Put),
        ("use", Verb::Use),
        ("move", Verb::Move),
        ("go", Verb::Move),
        ("buy", Verb::Buy),
        ("drink", Verb::Drink),
        ("eat", Verb::Eat),
        ("sleep", Verb::Sleep),
    ];

pub const CONNECTORS: &'static[(&'static str, Connector)] =
    &[
        ("in", Connector::Into),
        ("into", Connector::Into),
        ("on", Connector::Onto),
        ("onto", Connector::Onto),
        ("under", Connector::Under),
        ("to", Connector::To),
        ("from", Connector::From),
        ("with", Connector::With),
    ];


pub const DIRECTIONS: &'static[(&'static str, Direction)] =
    &[
        ("north", Direction::North),
        ("east", Direction::East),
        ("south", Direction::South),
        ("west", Direction::West),
    ];

pub const IGNORED: &'static [&'static str] =
    &[
        "a",
        "an",
        "the",
    ];


/// Return true if the given word should be ignored in commands, false
/// otherwise.
fn is_ignored(s: &str) -> bool {
    IGNORED.iter().any(|t| s == *t)
}

/// Find the connector matching string s, or None if there is no
/// match.
fn find_connector(s: &str) -> Option<Connector> {
    CONNECTORS.iter().find(|&&(t, _)| s == t).map(|&(_, conn)| conn)
}

/// Find the verb matching string s, or None if there is no match.
fn find_verb(s: &str) -> Option<Verb> {
    VERBS.iter().find(|&&(t, _)| s == t).map(|&(_, vrb)| vrb)
}

/// Find the verb matching string s, or None if there is no match.
fn find_direction(s: &str) -> Option<Direction> {
    DIRECTIONS.iter().find(|&&(t, _)| s == t).map(|&(_, dir)| dir)
}

/// Parse a string as a MUD-like command.  Return either a command
/// structure or an error when the string cannot be parsed.
pub fn parse(s: &str) -> Result<Command, error::Error> {
    // Convert string slice to iterator over non-empty lowercase words.
    let mut words = s.split(' ').filter(|s| s.len() > 0).map(|s| s.to_lowercase());

    let mut direct_object: Vec<String> = vec![];

    // Parse the first word as a verb and return an error if something
    // is wrong.
    let verb_str = try!(words.next().
                        ok_or(error::Error::CommandParse("command expected")));
    let verb = if let Some(_dir) = find_direction(&verb_str) {
        direct_object.push(verb_str.clone());
        Verb::Move
    } else {
        try!(find_verb(&verb_str).
             ok_or(error::Error::CommandParse("not a valid verb")))
    };

    // Parse a sequence of words as the description of an object, up
    // to a connector word or the end of the command string.

    let mut word = words.next();
    loop {
        match word {
            None =>
                break,
            Some(ref w) =>
                if let Some(_conn) = find_connector(&w) {
                    break;
                } else if is_ignored(&w) {
                    // Simply ignore this word.
                } else {
                    direct_object.push(w.clone());
                }
        }
        word = words.next();
    }

    // Parse optional connector and following indirect object.

    let mut connector = None;
    let mut indirect_object: Vec<String> = vec![];

    // Now check the word immediately following the direct object.
    match word {
        None =>
        {},
        Some(w) =>
            if let Some(conn) = find_connector(&w) {
                connector = Some(conn);
            } else if is_ignored(&w) {
                // Simply ignore this word.
            } else {
                indirect_object.push(w.clone());
            }
    }
    indirect_object.extend(words.filter(|w| !is_ignored(w)));

    if connector.is_some() && indirect_object.len() == 0 {
        return Err(error::Error::CommandParse("indirect object required after connector"));
    }

    Ok(Command{verb: verb,
               direct_object: if direct_object.len() > 0 {
                   Some(direct_object)
               } else {
                   None
               },
               indirect_object: if indirect_object.len() > 0 {
                   Some((connector.unwrap(), indirect_object))
               } else {
                   None
               },
    })
}
