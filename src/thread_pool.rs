//! # Thread Pool
//!
//! this is bare bones implementation for thread pools closely  modeled after the one explained in
//! the second edition of the rust book

use std::sync::mpsc;
use std::sync::Mutex;
use std::sync::Arc;
use std::thread;

struct Worker {
    id: usize,
    handle: Option<thread::JoinHandle<()>>,
}

impl Worker {
    pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>,
               sender: mpsc::Sender<(usize, f64)>, node_section_length: usize,
               dynamic_nodes: &[usize], nodes: &mut [f64]) -> Worker {
        Worker {id, handle: Some(thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv().unwrap();
            match message {
                Message::Do(node_sector) => {
                    let delta = 0.0;
                    sender.send((id, delta)).unwrap();
                },
                Message::Terminate => {
                    break;
                }
            }
    }))}
    }
}

pub enum Message {
    Do(usize),
    Terminate,
}

pub struct ThreadPool{
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
    receiver: mpsc::Receiver<(usize, f64)>,
}

impl ThreadPool {
    pub fn new(size: usize, mut grid: super::Grid) -> Option<ThreadPool> {
        assert!(size > 0);

        let (sender_pool, receiver_threads) = mpsc::channel();
        let receiver_threads = Arc::new(Mutex::new(receiver_threads));

        let (sender_threads, receiver_pool) = mpsc::channel();

        let max_sections = grid.dynamic_nodes_indices.len() / (2 * grid.width);
        if max_sections == 1 {return None} //multi-threading makes no sense
        let size = if size * 2 > max_sections { max_sections} else { size * 2 };
        let node_section_length = grid.dynamic_nodes_indices.len() / size;

        let nodes_ptr = grid.nodes.as_mut_ptr();

        let mut workers = Vec::with_capacity(size / 2);
        for id in 0..size / 2 {
            workers.push(Worker::new(id,
                                     Arc::clone(&receiver_threads),
                                     mpsc::Sender::clone(&sender_threads),
                                     node_section_length,
                                     grid.dynamic_nodes_indices.as_slice(),
                                     unsafe {grid.nodes.as_mut_slice()}));
        }

        Some(ThreadPool{ workers, sender: sender_pool, receiver: receiver_pool })
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