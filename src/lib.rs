// Copyright 2016 Martin Grabmueller. See the LICENSE file at the
// top-level directory of this distribution for license information.

//! Mudstuck errors and machinery to make them work with `try!'.

extern crate uuid;

use uuid::Uuid;
use std::collections::BTreeMap;

pub mod types;
mod error;
mod scanner;
mod template;
pub mod command;

use template::Ast;
use types::*;

pub fn make_example_world() -> World {
    let mut entities = vec![];

    let r1_name = Uuid::new_v4();
    let r2_name = Uuid::new_v4();
    let d1_name = Uuid::new_v4();
    let d1 = Entity {
        id: d1_name,
        name: vec!["rusty".to_string(), "metal".to_string(), "door".to_string()],
        alias: Some("metal_door_1".to_string()),
        short_description: "Metalltür".to_string(),
        long_description: "Eine verbeulte, rostige Tür aus Metall.#(if (closed rusty.metal.door) \" Die Tür ist geschlossen.\" \"\")".to_string(),
        attributes: vec![
            Attribute::Doorlike(Connection { endpoints: (r1_name, r2_name) }),
            Attribute::Closable(true),
            Attribute::Lockable(false),
        ],
    };
    let r1 = Entity {
        id: r1_name,
        name: vec!["small".to_string(), "rock".to_string(), "room".to_string()],
        alias: None,
        short_description: "Ein kleiner Raum mit Wänden aus rohem Fels".to_string(),
        long_description:  "Der Raum hat eine Größe von etwa sechs Quadratmetern. Der Boden, die Decke und die Wände bestehen aus roh behauenem Fels. Der Boden ist mit Schutt bedeckt.  In einer der Wände befindet sich eine zugemauerte Türöffnung, gegenüber ist eine #(if (closed rusty.metal.door) \"geschlossene\" \"geöffnete\")#(if (locked rusty.metal.door) \" verriegelte\" \"\") Metalltür eingelassen.".to_string(),
        attributes: vec![
            Attribute::Roomlike(Room {
                entities: vec![d1_name]
            }),
        ],
    };
    let r2 = Entity {
        id: r2_name,
        name: vec!["cramped".to_string(), "rock".to_string(), "tunnel".to_string()],
        alias: None,
        short_description: "Ein niedriger Felstunnel".to_string(),
        long_description: "Ein schmaler, niedriger Tunnel, etwa 1,70 Meter hoch und einen Meter breit. Der Tunnel führt leicht bergab und hat an beiden Enden Metalltüren".to_string(),
        attributes: vec![
            Attribute::Roomlike(Room {
                entities: vec![d1_name]
            }),
        ],
    };
    entities.push(d1);
    entities.push(r1);
    entities.push(r2);

    let mut map = BTreeMap::new();
    for (i, e) in entities.iter().enumerate() {
        map.insert(e.id, i);
    }

    let world = World {
        name: "Example World".to_string(),
        entities: entities,
        start_location: r1_name,
        entity_map: map,
    };
    world
}

impl World {
    /// Return a reference to the entity with the given name, if
    /// possible.
    fn entity(&self, name: &InternalName) -> Option<&Entity> {
        self.entity_map.get(name).and_then(|idx| self.entities.get(*idx))
    }

    /// Evaluate a string in the context of the world.  The string can
    /// contain expressions (marked with #) which will be evaluated in
    /// the state that the world itself is currently.  Returns the
    /// (possibly interpolated) string or an error message.  Note that
    /// an error message indicates a syntax or logic error in the
    /// input string.  Correct strings will never return errors.
    fn eval_str(&self, txt: &str) -> Result<String, String> {
        match template::parse(txt) {
            Ok(ast) => {
                match self.eval(ast) {
                    Err(e) => Err(e),
                    Ok(Value::Str(s)) => Ok(s),
                    Ok(val) => Err(format!("invalid value: {:?}", val))
                }
            },
            Err(e) => Err(e)
        }
    }

    fn get_by_name(&self, name: &Name) -> Option<InternalName> {
        let mut res = None;
        for e in self.entities.iter() {
            if e.name == *name {
                res = Some(e.id);
            }
        }
        res
    }
    
    /// Evaluate a list of expressions into a list of values, or an
    /// error message.
    fn eval_list(&self, args: Vec<Ast>) -> Result<Vec<Value>, String> {
        let mut res = Vec::new();
        for a in args {
            let ar = try!(self.eval(a));
            res.push(ar);
        }
        Ok(res)
    }

    fn from_script_name(&self, s: &str) -> Name {
        let mut res = vec![];
        for n in s.split('.') {
            res.push(n.to_string())
        }
        res
    }
    
    /// Evaluate an expression into a value, or an error message.
    fn eval(&self, ast: Ast) -> Result<Value, String> {
        match ast {
            Ast::Empty =>
                Ok(Value::Str("".to_string())),
            Ast::Chr(c) =>
                Ok(Value::Str(format!("{}", c))),
            Ast::Str(s) =>
                Ok(Value::Str(s.clone())),
            Ast::Id(s) => {
                match s.as_str() {
                    "if" => Ok(Value::Fun(Function::If, "if", true, 3, 3)),
                    "closed" => Ok(Value::Fun(Function::Closed, "closed", false, 1, 1)),
                    "locked" => Ok(Value::Fun(Function::Locked, "locked", false, 1, 1)),
                    _ => {
                        let sv = self.from_script_name(&s);
                        match self.get_by_name(&sv) {
                            None => Err(format!("undefined identifier: {}", s)),
                            Some(name) => Ok(Value::Reference(name))
                        }
                    }
                }
            },
            Ast::Seq(l, r) => {
                let lhs = try!(self.eval(*l));
                let rhs = try!(self.eval(*r));
                match (lhs, rhs) {
                    (Value::Str(l), Value::Str(r)) =>
                        Ok(Value::Str(format!("{}{}", l, r))),
                    _ => Err("invalid operand for concatenation".to_string())
                }
            },
            Ast::Call(f, args) => {
                let fun = try!(self.eval(*f));
                match fun {
                    Value::Fun(_, name, special, min_args, max_args) => {
                        let acnt = args.len();
                        if acnt < min_args {
                            return Err(format!("function {} requires at least {} arguments, got {}", name, min_args, acnt));
                        }
                        if acnt > max_args {
                            return Err(format!("function {} requires at most {} arguments, got {}", name.clone(), max_args, acnt));
                        }
                        let arguments = if special {
                            args.into_iter().map(|a| Value::Expr(a.clone())).collect()
                        } else {
                            try!(self.eval_list(args))
                        };
                        self.apply(fun.clone(), arguments)
                    },
                    _ =>
                        Err("non-function in function position".to_string())
                }
            }
        }
    }

    /// Apply a functional value to a list of argument values.
    fn apply(&self, f: Value, args: Vec<Value>) -> Result<Value, String> {
        match f {
            Value::Fun(fun_id, _,  _, _, _) =>
                match fun_id {
                    Function::If => {
                        if let &Value::Expr(ref cond) = args.get(0).unwrap() {
                            let cval = try!(self.eval(cond.clone()));
                            match cval {
                                Value::Bool(b) => {
                                    let e = if b { args.get(1).unwrap() } else { args.get(2).unwrap() };
                                    if let &Value::Expr(ref ee) = e {
                                        self.eval(ee.clone())
                                    } else {
                                        Err("internal error, if expression already evaluated".to_string())
                                    }
                                },
                                _ => {
                                    Err("if expects boolean expression as first argument".to_string())
                                }
                            }
                        } else {
                            Err("internal error, if condition already evaluated".to_string())
                        }
                    },
                    Function::Closed => {
                        if let Some(&Value::Reference(ref name)) = args.get(0) {
                            let ent = self.entity(name).unwrap();
                            match ent.attributes.iter().find(|&a| match a { &Attribute::Closable(_) => true, _ => false }) {
                                Some(&Attribute::Closable(closed)) =>
                                    Ok(Value::Bool(closed)),
                                _ =>
                                    Ok(Value::Bool(false)),
                            }
                        } else {
                            Err("function closed requires a name of an entity".to_string())
                        }
                    },
                    Function::Locked => {
                        if let Some(&Value::Reference(ref name)) = args.get(0) {
                            let ent = self.entity(name).unwrap();
                            match ent.attributes.iter().find(|&a| match a { &Attribute::Lockable(_) => true, _ => false }) {
                                Some(&Attribute::Lockable(closed)) =>
                                    Ok(Value::Bool(closed)),
                                _ =>
                                    Ok(Value::Bool(false)),
                            }
                        } else {
                            Err("function locked requires a name of an entity".to_string())
                        }
                    },
                },
            _ =>
                Err("non-function in function position".to_string()),
        }
    }
}




#[derive(Debug, Clone)]
pub enum Function {
    If,
    Closed,
    Locked,
}

#[derive(Debug, Clone)]
pub enum Value {
    Fun(Function, &'static str, bool, usize, usize),
    Reference(InternalName),
    Str(String),
    Bool(bool),
    Expr(Ast),
}

fn print_wrap(txt: &str, width: usize) {
    let mut pos = 0;
    for w in txt.split(' ') {
        let w_len = w.chars().collect::<Vec<_>>().len();
//        print!("{} {}", pos, w_len);
        if pos + w_len > width {
            println!("");
            pos = 0;
        }
        if pos > 0 {
            print!(" ");
            pos += 1;
        }
        print!("{}", w);
        pos += w_len;
    }
    if pos > 0 {
        println!("");
    }
}

impl<'a> PlayerState<'a> {
    pub fn look(&self) {
        let w = self.world;
        let loc = w.entity(&self.location).unwrap();
        let shrt = w.eval_str(&loc.short_description);
        let lng = w.eval_str(&loc.long_description);
        match shrt {
            Ok(s) =>
                print_wrap(&s, 72),
            Err(e) =>
                println!("an error has occurred: {}", e)
        }
        match lng {
            Ok(s) =>
                print_wrap(&s, 72),
            Err(e) =>
                println!("an error has occurred: {}", e)
        }
    }
    pub fn describe(&self, name: &str) {
        let w = self.world;
        match w.get_by_name(&w.from_script_name(name)) {
            None => {
                println!("Es gibt nichts, was {} heißt.", name);
            },
            Some(n) => {
                let ent = w.entity(&n).unwrap();
                let shrt = w.eval_str(&ent.short_description);
                let lng = w.eval_str(&ent.long_description);
                match shrt {
                    Ok(s) =>
                        print_wrap(&s, 72),
                    Err(e) =>
                        println!("an error has occurred: {}", e)
                }
                match lng {
                    Ok(s) =>
                        print_wrap(&s, 72),
                    Err(e) =>
                        println!("an error has occurred: {}", e)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
