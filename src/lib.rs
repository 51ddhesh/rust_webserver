// Import synchronization primitives and threading utilities from the standard library
use std::sync::{Arc, Mutex, mpsc}; // Arc and Mutex for shared state, mpsc for message passing
use std::thread;                   // For spawning threads

/// A thread pool for executing jobs concurrently.
/// 
/// The ThreadPool manages a set of worker threads and a channel for sending jobs to them.
#[allow(unused)]
pub struct ThreadPool {
    workers: Vec<Worker>,           // Vector holding all worker threads
    sender: mpsc::Sender<Job>,      // Channel sender to dispatch jobs to workers
}

/// Type alias for a job that can be executed by the thread pool.
/// A job is any closure or function that takes no arguments and returns nothing,
/// but must be Send (can be transferred across threads) and 'static (no borrowed refs).
type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    /// Create a new ThreadPool.
    /// 
    /// # Arguments
    /// * `size` - The number of worker threads to spawn in the pool.
    /// 
    /// # Panics
    /// Panics if `size` is zero.
    pub fn new(size: usize) -> ThreadPool {
        // Ensure the pool has at least one thread.
        assert!(size > 0);

        // Create a channel for sending jobs to workers.
        let (sender, receiver) = mpsc::channel();
        // Wrap the receiver in Arc<Mutex<...>> so it can be shared and safely accessed by multiple threads.
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);

        // Spawn the specified number of worker threads.
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }

    /// Execute a job (closure) on the thread pool.
    /// 
    /// # Arguments
    /// * `f` - The closure or function to execute. Must be Send and 'static.
    pub fn execute<F>(&self, f: F)
    where F: FnOnce() + Send + 'static, {
        // Box the closure to fit the Job type.
        let job = Box::new(f);
        // Send the job to the worker threads via the channel.
        self.sender.send(job).unwrap();
    }
}


/// Represents a single worker in the thread pool.
/// Each worker has a unique id and owns a thread handle.
#[allow(unused)]
struct Worker {
    id: usize,                      // Worker id (for logging/debugging)
    thread: thread::JoinHandle<()>, // Handle to the spawned thread
}

impl Worker {
    /// Create a new worker thread.
    /// 
    /// # Arguments
    /// * `id` - The worker's unique identifier.
    /// * `receiver` - Shared receiver for jobs, protected by Arc<Mutex<...>>.
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        // Spawn a new thread that waits for jobs and executes them as they arrive.
        let thread = thread::spawn(move || {
            // Loop, waiting for jobs to be received from the channel.
            while let Ok(job) = receiver.lock().unwrap().recv() {
                println!("Worker {id} got a job, executing...");
                job(); // Execute the job (closure)
            }
        });
        Worker { id, thread }
    }
}


