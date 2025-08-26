use std::sync::{mpsc, Arc, Mutex};
use std::thread;

pub struct ThreadPool {
    sender: Option<mpsc::Sender<Message>>,
    workers: Vec<Worker>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

enum Message {
    NewJob(Job),
    Terminate,
}

impl ThreadPool {
    /// Creates a new ThreadPool with the given size.
    ///
    /// # Panics
    ///
    /// Panics if the size is zero.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);

        for _ in 0..size {
            workers.push(Worker::new(receiver.clone()));
        }

        ThreadPool {
            sender: Some(sender),
            workers,
        }
    }

    /// Executes a function in the thread pool.
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.sender
            .as_ref()
            .expect("ThreadPool has been shut down")
            .send(Message::NewJob(Box::new(f)))
            .expect("Failed to send job to workers");
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        if let Some(sender) = self.sender.take() {
            // Send the Terminate message to all workers to shut them down
            for _ in &self.workers {
                sender.send(Message::Terminate).unwrap();
            }
        }

        // Wait for all worker threads to finish
        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv().unwrap();

            match message {
                Message::NewJob(job) => job(),
                Message::Terminate => break,
            }
        });

        Worker { thread: Some(thread) }
    }
}

