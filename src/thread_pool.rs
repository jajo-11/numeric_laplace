//! # Thread Pool
//!
//! this is bare bones implementation for thread pools closely  modeled after the one explained in
//! the second edition of the rust book
extern crate crossbeam_utils;

use std::sync::mpsc;
use std::sync::Mutex;
use std::sync::Arc;
use std::thread;
use std::slice;
use std::io;
use std::io::Write;

struct Worker {
    handle: Option<thread::JoinHandle<()>>,
}

impl Worker {
    pub fn new(receiver: Arc<Mutex<mpsc::Receiver<Message>>>,
               sender: mpsc::Sender<(usize, f64)>, nodes: &mut [f64], nodes_width: usize,
               dynamic_nodes: &[usize], over_relaxation: f64) -> Worker {
        Worker {
            handle: Some(unsafe { crossbeam_utils::scoped::spawn_unsafe(move || loop {
                let message = receiver.lock().unwrap().recv().unwrap();
                match message {
                    Message::Do((sector, start, end)) => {
                        let mut max_delta = 0.0;
                        for &i in dynamic_nodes[start..end].iter() {
                            let top = nodes[i - nodes_width];
                            let right = nodes[i - 1];
                            let left = nodes[i + 1];
                            let bottom = nodes[i + nodes_width];

                            let new_value = (top + right + left + bottom) / 4.0;

                            let mut delta = nodes[i] - new_value;
                            nodes[i] -= over_relaxation * delta;
                            if delta.abs() > max_delta { max_delta = delta.abs() };
                        }
                        sender.send((sector, max_delta)).unwrap();
                    }
                    Message::Terminate => {
                        break;
                    }
                }
            })}),
        }
    }
}

pub enum Message<> {
    Do((usize, usize, usize)),
    Terminate,
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
    receiver: mpsc::Receiver<(usize, f64)>,
}

#[derive(Debug, Clone, PartialEq)]
enum Lock {
    None,
    One,
    Two,
}

impl Lock {

    fn up(&mut self) {
        match *self {
            Lock::None => *self = Lock::One,
            Lock::One => *self = Lock::Two,
            Lock::Two => panic!("Added a third lock, not good!"),
        }
    }
}

impl ThreadPool {
    pub fn new(size: usize, grid: &mut super::Grid, over_relaxation: f64) -> Option<ThreadPool> {
        assert!(size > 0);

        let (sender_pool, receiver_threads) = mpsc::channel();
        let receiver_threads = Arc::new(Mutex::new(receiver_threads));

        let (sender_threads, receiver_pool) = mpsc::channel();

        let max_sections = grid.dynamic_nodes_indices.len() / (2 * grid.width);
        if max_sections == 1 { return None; } //multi-threading makes no sense
        let size = if size * 2 > max_sections { max_sections } else { size * 2 };

        let ptr = grid.nodes.as_mut_ptr();

        let mut workers = Vec::with_capacity(size / 2);
        for _id in 0..workers.capacity() {
            workers.push(Worker::new(Arc::clone(&receiver_threads),
                                     mpsc::Sender::clone(&sender_threads),
                                     unsafe { slice::from_raw_parts_mut(ptr, grid.nodes.len())},
                                     grid.width,
                                     grid.dynamic_nodes_indices.as_slice(),
                                     over_relaxation));
        }

        Some(ThreadPool { workers, sender: sender_pool, receiver: receiver_pool })
    }

    pub fn evaluate(&self, accepted_delta: f64, dynamic_indices_len: usize) {
        let sector_width = dynamic_indices_len / (self.workers.len() * 2);
        let mut dynamic_sectors = Vec::with_capacity(self.workers.len() * 2 + 1);
        for i in 0..self.workers.len() * 2 {
            dynamic_sectors.push(sector_width * i)
        }
        dynamic_sectors.push(dynamic_indices_len - 1);

        let mut sector_locks = vec![Lock::Two; self.workers.len() * 2];
        let last_sector_index = self.workers.len() * 2 - 1;

        for i in (0..last_sector_index).step_by(2) {
            sector_locks[i] = Lock::None;
            self.sender.send(Message::Do((i, dynamic_sectors[i], dynamic_sectors[i+1])))
                .expect("Could not send message to Threads!");
        }
        sector_locks[last_sector_index] = Lock::One;

        let max_delta = accepted_delta + 1.0;
        let mut sector_deltas = vec![max_delta; self.workers.len() * 2];
        while sector_deltas.iter().any(|&d| d > accepted_delta) {
            let (sector, delta) = self.receiver.recv().unwrap();
            print!("\rdelta {}", delta);
            io::stdout().flush().expect("Could not flush stdout!");
            sector_deltas[sector] = delta;
            if sector == 0 {
                match sector_locks[sector + 1] {
                    Lock::None => {panic!{"Sector is unlocked and neighbouring sector was just returning!"}},
                    Lock::One => {
                        sector_locks[sector + 1] = Lock::None;
                        sector_locks[sector].up();
                        self.sender.send(Message::Do((sector + 1, dynamic_sectors[sector + 1], dynamic_sectors[sector + 2])))
                            .expect("Could not send message to Threads!");
                        if sector + 1 != last_sector_index {
                            sector_locks[sector + 2].up();
                        }
                    },
                    Lock::Two => {sector_locks[sector + 1] = Lock::One},
                }
            } else if  sector == last_sector_index {
                match sector_locks[sector - 1] {
                    Lock::None => {panic!{"Sector is unlocked and neighbouring sector was just returning!"}},
                    Lock::One => {
                        sector_locks[sector - 1] = Lock::None;
                        sector_locks[sector].up();
                        self.sender.send(Message::Do((sector - 1, dynamic_sectors[sector - 1], dynamic_sectors[sector])))
                            .expect("Could not send message to Threads!");
                        if sector - 1 != 0 {
                            sector_locks[sector - 2].up();
                        }
                    },
                    Lock::Two => {sector_locks[sector - 1] = Lock::One},
                }
            } else {
                match sector_locks[sector + 1] {
                    Lock::None => {panic!{"Sector is unlocked and neighbouring sector was just returning!"}},
                    Lock::One => {
                        sector_locks[sector + 1] = Lock::None;
                        sector_locks[sector].up();
                        self.sender.send(Message::Do((sector + 1, dynamic_sectors[sector + 1], dynamic_sectors[sector + 2])))
                            .expect("Could not send message to Threads!");
                        if sector + 1 != last_sector_index {
                            sector_locks[sector + 2].up();
                        }
                    },
                    Lock::Two => {sector_locks[sector + 1] = Lock::One},
                }

                match sector_locks[sector - 1] {
                    Lock::None => {panic!{"Sector is unlocked and neighbouring sector was just returning!"}},
                    Lock::One => {
                        sector_locks[sector - 1] = Lock::None;
                        sector_locks[sector].up();
                        self.sender.send(Message::Do((sector - 1, dynamic_sectors[sector - 1], dynamic_sectors[sector])))
                            .expect("Could not send message to Threads!");
                        if sector - 1 != 0 {
                            sector_locks[sector - 2].up();
                        }
                    },
                    Lock::Two => {sector_locks[sector - 1] = Lock::One},
                }
            }
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        print!("Shutting down threads...");
        for _ in &mut self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }
        for worker in &mut self.workers {
            if let Some(handle) = worker.handle.take() {
                handle.join().unwrap();
            }
        }
        print!("\r");
    }
}