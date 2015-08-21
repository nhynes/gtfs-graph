use std::borrow::Borrow;
use std::cell::UnsafeCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::hash::Hash;
use std::iter::Skip;
use std::slice::Iter;
use chrono::{NaiveDateTime, Duration};
use {Result, Error, Stop, Connection};

fn departure_ord<T>(d1: &Departure<T>, d2: &Departure<T>) -> Ordering { d1.time.cmp(&d2.time) }

pub struct Graph<'a, T: 'a> {
    nodes: HashMap<String, UnsafeCell<StopNode<'a, T>>>,
}

pub struct StopNode<'g, T: 'g> {
    pub stop: &'g Stop,
    connections: Vec<Departure<'g, T>>,
    data: Option<T>
}

pub struct Departure<'g, T: 'g> {
    pub destination: &'g StopNode<'g, T>,
    pub time: NaiveDateTime,
    pub duration: Duration
}

impl<'g, T> StopNode<'g, T> {
    pub fn departures_after(&'g self, time: &NaiveDateTime) -> Skip<Iter<'g, Departure<T>>> {
        let i = match self.connections.binary_search_by(|d| d.time.cmp(time)) {
            Ok(i) => i,
            Err(i) => i
        };
        self.connections.iter().skip(i)
    }

    pub fn augment(&'g mut self, data: T) {
        self.data = Some(data)
    }

    pub fn get_augment(&'g self) -> &Option<T> {
        &self.data
    }
}

impl<'a, T> Graph<'a, T> {
    pub fn new() -> Graph<'a, T> {
        Graph::with_capacity(0)
    }

    pub fn with_capacity(capacity: usize) -> Graph<'a, T> {
        Graph { nodes: HashMap::with_capacity(capacity) }
    }

    pub fn construct(&'a mut self, stops: &'a Vec<Stop>, cnx: &Vec<Connection>)
        -> Result<()>
    {
        // construct the nodes
        for i in 0..stops.len() {
            let node = StopNode {
                stop: &stops[i],
                connections: Vec::new(),
                data: None
            };
            self.nodes.insert(stops[i].id.to_owned(), UnsafeCell::new(node));
        }

        // construct the edges and add them to the nodes
        for c in cnx.iter() {
            let to = self.nodes.get(&c.to)
                .ok_or(Error::Data(format!("Destination stop not found: {}", c.to)));
            let from = self.nodes.get(&c.from)
                .ok_or(Error::Data(format!("Origin stop not found: {}", c.from)));

            let edge = Departure {
                destination: unsafe { &*try!(to).get() },
                time: c.departs,
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
