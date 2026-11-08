//! This module implements per-component parallelism.
//! It should be possible to implement per-row parallelism as well,
//! which should also boost performance of grayscale images
//! and allow scaling to more cores.
//! However, that would be more complex, so we use this as a starting point.

use super::immediate::ImmediateWorker;
use super::{RowData, Worker};
use crate::decoder::MAX_COMPONENTS;
use crate::error::Result;
use std::{
    mem,
    sync::mpsc::{self, Receiver, Sender},
};

enum WorkerMsg {
    Start(RowData),
    AppendRow(Vec<i16>),
    GetResult(Sender<Vec<u8>>),
}

#[derive(Default)]
pub struct MpscWorker {
    senders: [Option<Sender<WorkerMsg>>; MAX_COMPONENTS],
}

impl MpscWorker {
    fn start_with(
        &mut self,
        row_data: RowData,
        spawn_worker: impl FnOnce(usize) -> Result<Sender<WorkerMsg>>,
    ) -> Result<()> {
        // if there is no worker thread for this component yet, start one
        let component = row_data.index;
        if self.senders[component].is_none() {
            let sender = spawn_worker(component)?;
            self.senders[component] = Some(sender);
        }

        // we do the "take out value and put it back in once we're done" dance here
        // and in all other message-passing methods because there's not that many rows
        // and this should be cheaper than spawning MAX_COMPONENTS many threads up front
        let sender = self.senders[component].as_mut().unwrap();
        sender
            .send(WorkerMsg::Start(row_data))
            .expect("jpeg-decoder worker thread error");
        Ok(())
    }

    fn append_row(&mut self, row: (usize, Vec<i16>)) -> Result<()> {
        let component = row.0;
        let sender = self.senders[component].as_mut().unwrap();
        sender
            .send(WorkerMsg::AppendRow(row.1))
            .expect("jpeg-decoder worker thread error");
        Ok(())
    }

    fn get_result_with(
        &mut self,
        index: usize,
        collect: impl FnOnce(Receiver<Vec<u8>>) -> Vec<u8>,
    ) -> Result<Vec<u8>> {
        let (tx, rx) = mpsc::channel();
        let sender = mem::take(&mut self.senders[index]).unwrap();
        sender
            .send(WorkerMsg::GetResult(tx))
            .expect("jpeg-decoder worker thread error");
        Ok(collect(rx))
    }
}

impl Worker for MpscWorker {
    fn start(&mut self, row_data: RowData) -> Result<()> {
        self.start_with(row_data, spawn_worker_thread)
    }
    fn append_row(&mut self, row: (usize, Vec<i16>)) -> Result<()> {
        MpscWorker::append_row(self, row)
    }
    fn get_result(&mut self, index: usize) -> Result<Vec<u8>> {
        self.get_result_with(index, collect_worker_thread)
    }
}

fn create_worker() -> (Sender<WorkerMsg>, impl FnOnce() + 'static) {
    let (tx, rx) = mpsc::channel();
    let closure = move || {
        let mut worker = ImmediateWorker::default();

        while let Ok(message) = rx.recv() {
            match message {
                WorkerMsg::Start(mut data) => {
                    // we always set component index to 0 for worker threads
                    // because they only ever handle one per thread and we don't want them
                    // to attempt to access nonexistent components
                    data.index = 0;
                    worker.start_immediate(data);
                }
                WorkerMsg::AppendRow(row) => {
                    worker.append_row_immediate((0, row));
                }
                WorkerMsg::GetResult(chan) => {
                    let _ = chan.send(worker.get_result_immediate(0));
                    break;
                }
            }
        }
    };

    (tx, closure)
}

fn spawn_worker_thread(component: usize) -> Result<Sender<WorkerMsg>> {
    let (tx, worker) = create_worker();
    let thread_builder =
        std::thread::Builder::new().name(format!("worker thread for component {}", component));
    thread_builder.spawn(worker)?;
    Ok(tx)
}

fn collect_worker_thread(rx: Receiver<Vec<u8>>) -> Vec<u8> {
    rx.recv().expect("jpeg-decoder worker thread error")
}
