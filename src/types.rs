// Copyright 2016 Martin Grabmueller. See the LICENSE file at the
// top-level directory of this distribution for license information.

//! Shared data types.

use uuid::Uuid;
use std::collections::BTreeMap;

pub type InternalName = Uuid;
pub type Name = Vec<String>;

pub struct PlayerState<'a> {
    pub world: &'a World,
    pub location: InternalName,
}

pub struct World {
    pub name: String,
    pub entities: Vec<Entity>,
    pub entity_map: BTreeMap<InternalName, usize>,
    pub start_location: InternalName,
}

pub struct Entity {
    pub id: InternalName,
    pub name: Name,
    pub alias: Option<String>,
    pub short_description: String,
    pub long_description: String,
    pub attributes: Vec<Attribute>,
}

pub enum Attribute {
    Lockable(bool),
    Closable(bool),
    Doorlike(Connection),
    Roomlike(Room),
    Characterlike(Character),
}

pub struct Connection {
    pub endpoints: (InternalName, InternalName),
}

pub struct Room {
    pub entities: Vec<InternalName>,
}

pub struct Character {
    pub inventory: Vec<InternalName>,
}

/// String to be used as a verb.
pub struct Verb(String);

/// String to be used as a connector word.
pub struct Connector(String);
