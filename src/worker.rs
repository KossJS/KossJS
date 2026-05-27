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
        let size = size.min(64); // MAX_WORKER_POOL_SIZE (CWE-400)
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
                        // Use JSON.parse for safe deserialization instead of eval
                        // (CWE-94: avoid code injection via user-controlled data)
                        let json_str = serde_json::to_string(&data)
                            .unwrap_or_else(|_| "null".to_string());
                        let escaped = crate::runtime::escape_js_string(&json_str);
                        let code = format!(
                            "if (typeof self !== 'undefined' && typeof self.__koss_onmessage === 'function') {{ self.__koss_onmessage(JSON.parse('{}')); }}",
                            escaped
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
        self.worker_count = 0;
    }
}

impl Drop for WorkerPool {
    fn drop(&mut self) {
        self.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worker_pool_new_zero() {
        let pool = WorkerPool::new(0);
        assert_eq!(pool.len(), 0);
        assert!(pool.is_empty());
    }

    #[test]
    fn test_worker_pool_new_one() {
        let mut pool = WorkerPool::new(1);
        assert_eq!(pool.len(), 1);
        assert!(!pool.is_empty());

        // Worker sends ready message on startup
        match pool.recv_timeout(std::time::Duration::from_secs(5)) {
            Ok(WorkerEvent::Message { worker_id, data }) => {
                assert_eq!(worker_id, 0);
                assert!(data.contains("ready"));
            }
            other => panic!("expected ready message, got {other:?}"),
        }

        pool.shutdown();
    }

    #[test]
    fn test_worker_execute_and_result() {
        let mut pool = WorkerPool::new(1);

        // Drain ready message
        let _ = pool.recv_timeout(std::time::Duration::from_secs(5));

        // Execute simple JS code
        let cmd_id = pool.execute(0, "1 + 2").unwrap();
        assert!(cmd_id > 0);

        // Receive result
        match pool.recv_timeout(std::time::Duration::from_secs(5)) {
            Ok(WorkerEvent::Result {
                worker_id,
                id,
                success,
                value,
            }) => {
                assert_eq!(worker_id, 0);
                assert_eq!(id, cmd_id);
                assert!(success);
                assert!(value.contains('3'));
            }
            other => panic!("expected result, got {other:?}"),
        }

        pool.shutdown();
    }

    #[test]
    fn test_worker_execute_error() {
        let mut pool = WorkerPool::new(1);
        let _ = pool.recv_timeout(std::time::Duration::from_secs(5));

        let _cmd_id = pool.execute(0, "throw new Error('test error')").unwrap();

        match pool.recv_timeout(std::time::Duration::from_secs(5)) {
            Ok(WorkerEvent::Result {
                success, value, ..
            }) => {
                assert!(!success);
                assert!(value.contains("test error"));
            }
            other => panic!("expected error result, got {other:?}"),
        }

        pool.shutdown();
    }

    #[test]
    fn test_worker_post_message() {
        let mut pool = WorkerPool::new(1);
        let _ = pool.recv_timeout(std::time::Duration::from_secs(5));

        // Post a message to the worker
        pool.post_message(0, r#"{"greeting":"hello"}"#).unwrap();

        // Worker should echo the message back
        match pool.recv_timeout(std::time::Duration::from_secs(5)) {
            Ok(WorkerEvent::Message {
                worker_id, data, ..
            }) => {
                assert_eq!(worker_id, 0);
                assert!(data.contains("hello"));
            }
            other => panic!("expected message echo, got {other:?}"),
        }

        pool.shutdown();
    }

    #[test]
    fn test_worker_terminate() {
        let mut pool = WorkerPool::new(1);
        let _ = pool.recv_timeout(std::time::Duration::from_secs(5));

        // Terminate the worker
        pool.terminate(0).unwrap();

        // After terminate, execute should fail (worker loop exited)
        // Give it a moment to stop
        std::thread::sleep(std::time::Duration::from_millis(100));

        // send should still succeed (channel buffer) but no result will come
        // Just verify we can still call methods without panic
        let _ = pool.execute(0, "1+1"); // may succeed or fail depending on timing

        pool.shutdown();
    }

    #[test]
    fn test_worker_invalid_id() {
        let mut pool = WorkerPool::new(1);
        let _ = pool.recv_timeout(std::time::Duration::from_secs(5));

        assert!(pool.execute(99, "1+1").is_err());
        assert!(pool.post_message(99, "data").is_err());
        assert!(pool.terminate(99).is_err());

        pool.shutdown();
    }

    #[test]
    fn test_worker_try_recv_empty() {
        let mut pool = WorkerPool::new(1);
        let _ = pool.recv_timeout(std::time::Duration::from_secs(5));

        // No messages pending, try_recv should return None quickly
        let start = std::time::Instant::now();
        let result = pool.try_recv();
        let elapsed = start.elapsed();
        assert!(result.is_none());
        assert!(elapsed < std::time::Duration::from_secs(1));

        pool.shutdown();
    }

    #[test]
    fn test_worker_shutdown() {
        let mut pool = WorkerPool::new(2);
        assert_eq!(pool.len(), 2);

        pool.shutdown();
        assert_eq!(pool.len(), 0);
        assert!(pool.is_empty());
    }

    #[test]
    fn test_worker_multiple_executions() {
        let mut pool = WorkerPool::new(1);
        let _ = pool.recv_timeout(std::time::Duration::from_secs(5));

        // Execute three times sequentially
        for i in 0..3u64 {
            let cmd_id = pool.execute(0, &format!("{} + 1", i)).unwrap();
            match pool.recv_timeout(std::time::Duration::from_secs(5)) {
                Ok(WorkerEvent::Result {
                    id, success, value, ..
                }) => {
                    assert_eq!(id, cmd_id);
                    assert!(success);
                    assert!(value.contains(&(i + 1).to_string()));
                }
                other => panic!("expected result #{i}, got {other:?}"),
            }
        }

        pool.shutdown();
    }

    #[test]
    fn test_worker_pool_two_workers() {
        let mut pool = WorkerPool::new(2);
        assert_eq!(pool.len(), 2);

        // Drain ready messages from both workers
        for _ in 0..2 {
            match pool.recv_timeout(std::time::Duration::from_secs(5)) {
                Ok(WorkerEvent::Message { .. }) => {}
                other => panic!("expected ready message, got {other:?}"),
            }
        }

        // Execute on worker 0
        let id0 = pool.execute(0, "'worker0'").unwrap();
        match pool.recv_timeout(std::time::Duration::from_secs(5)) {
            Ok(WorkerEvent::Result { id, value, .. }) => {
                assert_eq!(id, id0);
                assert!(value.contains("worker0"));
            }
            other => panic!("expected worker0 result, got {other:?}"),
        }

        // Execute on worker 1
        let id1 = pool.execute(1, "'worker1'").unwrap();
        match pool.recv_timeout(std::time::Duration::from_secs(5)) {
            Ok(WorkerEvent::Result { id, value, .. }) => {
                assert_eq!(id, id1);
                assert!(value.contains("worker1"));
            }
            other => panic!("expected worker1 result, got {other:?}"),
        }

        pool.shutdown();
    }
}
