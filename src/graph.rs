use std::borrow::Borrow;
use std::cell::UnsafeCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::hash::Hash;
use std::mem;
use chrono::{NaiveDateTime, Duration};
use {Result, Error, Stop, Connection};

fn departure_ord(e1: &Edge, e2: &Edge) -> Ordering { e1.departs.cmp(&e2.departs) }

pub struct Graph<'a> {
    nodes: HashMap<String, UnsafeCell<StopNode<'a>>>,
}

pub struct StopNode<'g> {
    stop: &'g Stop,
    connections: Vec<Edge<'g>>
}

pub struct Edge<'g> {
    to: &'g StopNode<'g>,
    departs: NaiveDateTime,
    duration: Duration
}

impl<'a> Graph<'a> {
    pub fn new() -> Graph<'a> {
        Graph::with_capacity(0)
    }

    pub fn with_capacity(capacity: usize) -> Graph<'a> {
        Graph { nodes: HashMap::with_capacity(capacity) }
    }

    pub fn construct(&'a mut self, stops: &'a Vec<Stop>, cnx: &Vec<Connection>)
        -> Result<()>
    {
        // construct the nodes
        for i in 0..stops.len() {
            let node = StopNode {
                stop: &stops[i],
                connections: Vec::new()
            };
            self.nodes.insert(stops[i].id.to_owned(), UnsafeCell::new(node));
        }

        // construct the edges and add them to the nodes
        for c in cnx.iter() {
            let to = self.nodes.get(&c.to)
                .ok_or(Error::Data(format!("Destination stop not found: {}", c.to)));
            let from = self.nodes.get(&c.from)
                .ok_or(Error::Data(format!("Origin stop not found: {}", c.from)));

            let edge = Edge {
                to: unsafe { &*try!(to).get() },
                departs: c.departs,
                duration: c.duration
            };

            unsafe { (*try!(from).get()).connections.push(edge); }
        }

        // sort the edges by departure time for quick retrieval
        for (_, node) in self.nodes.iter_mut() {
           unsafe { (*node.get()).connections.sort_by(departure_ord); }
        }

        Ok(())
    }

    pub fn get_stop<Q: ?Sized>(&'a self, id: &Q) -> Option<&'a StopNode>
            where String: Borrow<Q>, Q: Hash + Eq {
        unsafe { self.nodes.get(id).map(|n| &*n.get()) }
    }
}
