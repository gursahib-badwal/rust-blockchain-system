use std::sync::mpsc;
use std::thread;

pub trait Task {
    type Output: Send;
    fn run(&self) -> Option<Self::Output>;
}

pub struct WorkQueue<TaskType: 'static + Task + Send> {
    send_tasks: Option<spmc::Sender<TaskType>>, // Option because it will be set to None to close the queue
    recv_tasks: spmc::Receiver<TaskType>,
    //send_output: mpsc::Sender<TaskType::Output>, // not need in the struct: each worker will have its own clone.
    recv_output: mpsc::Receiver<TaskType::Output>,
    workers: Vec<thread::JoinHandle<()>>,
}

impl<TaskType: 'static + Task + Send> WorkQueue<TaskType> {
    pub fn new(n_workers: usize) -> WorkQueue<TaskType> {
        // TODO: create the channels; start the worker threads; record their JoinHandles
        // unimplemented!()
        let mut i=0;
        let (send_output, recv_output) = mpsc::channel();
        let (send_tasks, recv_tasks) = spmc::channel();

        let mut handles = Vec::new();
        while i < n_workers {
            let rt_copy = recv_tasks.clone();
            let so_copy = send_output.clone();
            let handle = thread::spawn(move || {
                WorkQueue::<TaskType>::run(rt_copy, so_copy);
            });

            handles.push(handle);
            i+=1;
        }

        return WorkQueue {
            send_tasks: Some(send_tasks),
            recv_tasks: recv_tasks,
            recv_output: recv_output,
            workers: handles,
        };
    }

    fn run(recv_tasks: spmc::Receiver<TaskType>, send_output: mpsc::Sender<TaskType::Output>) {
        // TODO: the main logic for a worker thread
        loop {
            let task_result = recv_tasks.recv();
            // NOTE: task_result will be Err() if the spmc::Sender has been destroyed and no more messages can be received here
            // unimplemented!()
            match task_result {
                Ok(current_task) => {
                    if let Some(result) = current_task.run(){
                        let _x =send_output.send(result);
                    }
                }
                Err(_err) => {
                    return
                }
            }
        }
    }

    pub fn enqueue(&mut self, t: TaskType) -> Result<(), spmc::SendError<TaskType>> {
        // TODO: send this task to a worker
        // unimplemented!()
        let sender = self.send_tasks.as_mut().unwrap(); //as_mut allows to avoid patter matching and its an inbuilt function
        let result = sender.send(t);
        return result;
    } 

    // Helper methods that let you receive results in various ways
    pub fn iter(&mut self) -> mpsc::Iter<TaskType::Output> {
        self.recv_output.iter()
    }
    pub fn recv(&mut self) -> TaskType::Output {
        self.recv_output
            .recv()
            .expect("I have been shutdown incorrectly")
    }
    pub fn try_recv(&mut self) -> Result<TaskType::Output, mpsc::TryRecvError> {
        self.recv_output.try_recv()
    }
    pub fn recv_timeout(
        &self,
        timeout: std::time::Duration,
    ) -> Result<TaskType::Output, mpsc::RecvTimeoutError> {
        self.recv_output.recv_timeout(timeout)
    }

    pub fn shutdown(&mut self) {
        // TODO: destroy the spmc::Sender so everybody knows no more tasks are incoming;
        // drain any pending tasks in the queue; wait for each worker thread to finish.
        // HINT: Vec.drain(..)
        // unimplemented!()
        self.send_tasks = None;
        loop {
            if let Err(_err) = self.recv_tasks.recv() {
            break;
            }
        }
        for worker in self.workers.drain(..) {
            worker.join().unwrap(); 
        }
    }
}

impl<TaskType: 'static + Task + Send> Drop for WorkQueue<TaskType> {
    fn drop(&mut self) {
        // "Finalisation in destructors" pattern: https://rust-unofficial.github.io/patterns/idioms/dtor-finally.html
        match self.send_tasks {
            None => {} // already shut down
            Some(_) => self.shutdown(),
        }
    }
}
