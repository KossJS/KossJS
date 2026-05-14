use std::sync::mpsc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

pub type WorkerId = usize;

#[derive(Debug)]
pub enum WorkerCommand {
    Execute {
        code: String,
        id: u64,
    },
    PostMessage {
        data: String,
    },
    Terminate,
}

#[derive(Debug)]
pub enum WorkerEvent {
    Result {
        worker_id: WorkerId,
        id: u64,
        success: bool,
        value: String,
    },
    Message {
        worker_id: WorkerId,
        data: String,
    },
    Error {
        worker_id: WorkerId,
        message: String,
    },
}

pub struct WorkerPool {
    workers: Vec<WorkerHandle>,
    worker_count: usize,
    main_rx: mpsc::Receiver<WorkerEvent>,
    #[allow(dead_code)]
    main_tx: mpsc::Sender<WorkerEvent>,
    running: Arc<AtomicBool>,
}

struct WorkerHandle {
    thread: Option<thread::JoinHandle<()>>,
    tx: mpsc::Sender<WorkerCommand>,
}

impl WorkerPool {
    pub fn new(size: usize) -> Self {
        let (main_tx, main_rx) = mpsc::channel();
        let running = Arc::new(AtomicBool::new(true));
        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            let (tx, rx) = mpsc::channel::<WorkerCommand>();
            let main_tx_clone = main_tx.clone();
            let running_clone = running.clone();

            let handle = thread::Builder::new()
                .name(format!("koss-worker-{id}"))
                .spawn(move || {
                    let rt = match tokio::runtime::Runtime::new() {
                        Ok(r) => r,
                        Err(e) => {
                            let _ = main_tx_clone.send(WorkerEvent::Error {
                                worker_id: id,
                                message: format!("failed to create runtime: {e}"),
                            });
                            return;
                        }
                    };

                    let mut ctx = match boa_engine::context::ContextBuilder::default()
                        .build()
                    {
                        Ok(c) => c,
                        Err(_) => {
                            let _ = main_tx_clone.send(WorkerEvent::Error {
                                worker_id: id,
                                message: "failed to create Boa context".into(),
                            });
                            return;
                        }
                    };

                    let _ = main_tx_clone.send(WorkerEvent::Message {
                        worker_id: id,
                        data: format!("{{\"type\":\"ready\",\"workerId\":{id}}}"),
                    });

                    let _ = rt.block_on(async {
                        loop {
                            match rx.recv() {
                                Ok(cmd) => match cmd {
                                    WorkerCommand::Execute { code, id: cmd_id } => {
                                        let source = boa_engine::Source::from_bytes(code.as_bytes());
                                        match ctx.eval(source) {
                                            Ok(val) => {
                                                let s = match val.to_string(&mut ctx) {
                                                    Ok(s) => s.to_std_string_escaped(),
                                                    Err(_) => format!("{val:?}"),
                                                };
                                                let _ = main_tx_clone.send(WorkerEvent::Result {
                                                    worker_id: id,
                                                    id: cmd_id,
                                                    success: true,
                                                    value: s,
                                                });
                                            }
                                            Err(err) => {
                                                let s = match err.try_native(&mut ctx) {
                                                    Ok(native) => native.message().to_string(),
                                                    Err(_) => format!("{err:?}"),
                                                };
                                                let _ = main_tx_clone.send(WorkerEvent::Result {
                                                    worker_id: id,
                                                    id: cmd_id,
                                                    success: false,
                                                    value: s,
                                                });
                                            }
                                        }
                                    }
                                    WorkerCommand::PostMessage { data } => {
                                        let code = format!(
                                            "if (typeof self !== 'undefined' && self.__koss_onmessage) {{ self.__koss_onmessage({data}); }}"
                                        );
                                        let source = boa_engine::Source::from_bytes(code.as_bytes());
                                        let _ = ctx.eval(source);
                                        let _ = main_tx_clone.send(WorkerEvent::Message {
                                            worker_id: id,
                                            data,
                                        });
                                    }
                                    WorkerCommand::Terminate => {
                                        break;
                                    }
                                },
                                Err(_) => break,
                            }

                            if !running_clone.load(Ordering::SeqCst) {
                                break;
                            }
                        }
                    });
                });

            match handle {
                Ok(h) => {
                    workers.push(WorkerHandle {
                        thread: Some(h),
                        tx,
                    });
                }
                Err(e) => {
                    eprintln!("Failed to spawn worker {id}: {e}");
                }
            }
        }

        WorkerPool {
            worker_count: workers.len(),
            workers,
            main_rx,
            main_tx,
            running,
        }
    }

    pub fn len(&self) -> usize {
        self.worker_count
    }

    pub fn is_empty(&self) -> bool {
        self.worker_count == 0
    }

    pub fn execute(&self, worker_id: WorkerId, code: &str) -> Result<u64, String> {
        if worker_id >= self.workers.len() {
            return Err(format!("worker {worker_id} does not exist"));
        }
        let id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        self.workers[worker_id]
            .tx
            .send(WorkerCommand::Execute {
                code: code.to_string(),
                id,
            })
            .map_err(|e| e.to_string())?;
        Ok(id)
    }

    pub fn post_message(&self, worker_id: WorkerId, data: &str) -> Result<(), String> {
        if worker_id >= self.workers.len() {
            return Err(format!("worker {worker_id} does not exist"));
        }
        self.workers[worker_id]
            .tx
            .send(WorkerCommand::PostMessage {
                data: data.to_string(),
            })
            .map_err(|e| e.to_string())
    }

    pub fn try_recv(&self) -> Option<WorkerEvent> {
        self.main_rx.try_recv().ok()
    }

    pub fn recv_timeout(&self, timeout: std::time::Duration) -> Result<WorkerEvent, mpsc::RecvTimeoutError> {
        self.main_rx.recv_timeout(timeout)
    }

    pub fn terminate(&self, worker_id: WorkerId) -> Result<(), String> {
        if worker_id >= self.workers.len() {
            return Err(format!("worker {worker_id} does not exist"));
        }
        self.workers[worker_id]
            .tx
            .send(WorkerCommand::Terminate)
            .map_err(|e| e.to_string())
    }

    pub fn shutdown(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        for (_i, worker) in self.workers.iter_mut().enumerate() {
            let _ = worker.tx.send(WorkerCommand::Terminate);
            if let Some(handle) = worker.thread.take() {
                let _ = handle.join();
            }
        }
        self.workers.clear();
    }
}

impl Drop for WorkerPool {
    fn drop(&mut self) {
        self.shutdown();
    }
}
