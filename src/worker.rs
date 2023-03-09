use image::{ImageBuffer, Rgba};

use crate::dither::DitherJob;
use std::{fmt::Debug, thread::JoinHandle};

#[derive(Debug)]
pub struct Manager<Job> {
    pub worker_count: u32,
    workers: Vec<Worker<Job>>,
}

#[derive(Debug)]
pub struct Worker<Job> {
    pub handler: JoinHandle<Job>,
    pub id: u32,
}

impl<Job: Debug> Manager<Job> {
    pub fn new(worker_count: u32) -> Self {
        Manager {
            worker_count,
            workers: vec![],
        }
    }
    pub fn set_workers(&mut self, worker_job: &dyn Fn(u32) -> JoinHandle<Job>) {
        for id in 0..self.worker_count {
            let handler = worker_job(id);
            self.workers.push(Worker { handler, id });
        }
    }
}

impl WorkerCollection<ImageBuffer<Rgba<u8>, Vec<u8>>> for Manager<DitherJob> {
    /// Not sure if this is good practice, but it works.
    /// After `collect` is ran, manager cannot reference self anymore.
    /// It _could_ be to force workers to drop.
    /// I'm going to treat it as a feature lol.
    fn collect(self, aggregator: &mut ImageBuffer<Rgba<u8>, Vec<u8>>) {
        for worker in self.workers.into_iter() {
            let Worker { handler, id } = worker;

            match handler.join() {
                Ok(DitherJob { buffer, from, to }) => {
                    for i in from..to + 1 {
                        let x = i % aggregator.width();
                        let y = i / aggregator.height();
                        aggregator.put_pixel(x, y, *buffer.get_pixel(x, y));
                    }
                    println!("Worker #{} job appended.", id)
                }
                Err(error) => panic!("{:?}", error),
            }
        }
        println!("All workers collected.");
    }
}

pub trait WorkerCollection<Aggregator> {
    fn collect(self, aggregator: &mut Aggregator);
}
