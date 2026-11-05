//! Bounded observation transport with one owned consumer.
use crate::Observation;
use std::{
    collections::VecDeque,
    io,
    marker::PhantomData,
    panic::{AssertUnwindSafe, catch_unwind},
    rc::Rc,
    sync::{Arc, Condvar, Mutex},
    thread::{self, JoinHandle},
};

/// Failure to start a delivery worker.
#[derive(Debug)]
pub enum StartError {
    /// Capacity must be between 1 and 1024.
    InvalidCapacity(usize),
    /// The operating system refused to spawn the worker.
    Spawn(io::Error),
}

/// Result of offering one observation without waiting for consumer progress.
#[derive(Debug)]
pub enum Offer {
    /// Queued for delivery.
    Accepted,
    /// Waiting queue is full.
    Full(Observation),
    /// Admission has closed.
    Closed(Observation),
}

/// Reason the worker stopped.
#[derive(Debug, PartialEq)]
pub enum Stop<E> {
    /// All accepted observations were delivered.
    Drained,
    /// Cancellation stopped delivery.
    Cancelled,
    /// The callback returned an error.
    ConsumerError(E),
    /// The callback unwound.
    ConsumerPanicked,
}

/// FIFO disposition counts for the caller's accepted input record.
#[derive(Debug, PartialEq)]
pub struct DeliveryReport<E> {
    /// Number of offers accepted.
    pub accepted: usize,
    /// Number of callbacks that succeeded.
    pub completed: usize,
    /// Number of observations discarded without starting a callback.
    pub discarded: usize,
    /// Terminal outcome. A consumer failure attempted one additional item.
    pub stop: Stop<E>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Open,
    Drain,
    Cancel,
}

struct Queue {
    pending: VecDeque<Observation>,
    accepted: usize,
    mode: Mode,
}

struct Shared {
    queue: Mutex<Queue>,
    ready: Condvar,
}

/// Owns one delivery worker. Keep the owner on its creating thread.
/// Callbacks must eventually return so shutdown can join the worker.
/// The captured consumer must have a non-panicking destructor.
pub struct AsyncDelivery<E> {
    shared: Arc<Shared>,
    capacity: usize,
    worker: Option<JoinHandle<DeliveryReport<E>>>,
    owner_thread: PhantomData<Rc<()>>,
}

impl<E: Send + 'static> AsyncDelivery<E> {
    /// Start a worker with room for the given number of waiting observations.
    pub fn start(
        capacity: usize,
        consume: impl FnMut(Observation) -> Result<(), E> + Send + 'static,
    ) -> Result<Self, StartError> {
        Self::start_inner(
            capacity,
            consume,
            #[cfg(test)]
            || {},
        )
    }

    fn start_inner(
        capacity: usize,
        mut consume: impl FnMut(Observation) -> Result<(), E> + Send + 'static,
        #[cfg(test)] mut after_claim: impl FnMut() + Send + 'static,
    ) -> Result<Self, StartError> {
        if !(1..=1024).contains(&capacity) {
            return Err(StartError::InvalidCapacity(capacity));
        }
        let shared = Arc::new(Shared {
            queue: Mutex::new(Queue {
                pending: VecDeque::with_capacity(capacity),
                accepted: 0,
                mode: Mode::Open,
            }),
            ready: Condvar::new(),
        });
        let input = Arc::clone(&shared);
        let worker = thread::Builder::new()
            .name("observation-delivery".into())
            .spawn(move || {
                let mut completed = 0;
                loop {
                    let observation = {
                        let queue = input.queue.lock().unwrap();
                        let mut queue = input
                            .ready
                            .wait_while(queue, |q| q.mode == Mode::Open && q.pending.is_empty())
                            .unwrap();
                        match queue.pending.pop_front() {
                            Some(observation) => observation,
                            None => {
                                let stop = if queue.mode == Mode::Cancel {
                                    Stop::Cancelled
                                } else {
                                    Stop::Drained
                                };
                                return report(&queue, completed, stop);
                            }
                        }
                    };
                    #[cfg(test)]
                    after_claim();
                    let mut panic_payload = None;
                    let stop = match catch_unwind(AssertUnwindSafe(|| consume(observation))) {
                        Ok(Ok(())) => {
                            completed += 1;
                            continue;
                        }
                        Ok(Err(error)) => Stop::ConsumerError(error),
                        Err(payload) => {
                            panic_payload = Some(payload);
                            Stop::ConsumerPanicked
                        }
                    };
                    let mut queue = input.queue.lock().unwrap();
                    queue.mode = Mode::Cancel;
                    queue.pending.clear();
                    let report = report(&queue, completed, stop);
                    drop(queue);
                    if let Some(payload) = panic_payload
                        && let Err(nested) = catch_unwind(AssertUnwindSafe(|| drop(payload)))
                    {
                        // A second hostile payload has no safe destructor contract.
                        std::mem::forget(nested);
                    }
                    return report;
                }
            })
            .map_err(StartError::Spawn)?;
        Ok(Self {
            shared,
            capacity,
            worker: Some(worker),
            owner_thread: PhantomData,
        })
    }

    /// Offer an observation. Closed admission takes precedence over capacity.
    pub fn offer(&mut self, observation: Observation) -> Offer {
        let mut queue = self.shared.queue.lock().unwrap();
        if queue.mode != Mode::Open {
            return Offer::Closed(observation);
        }
        if queue.pending.len() == self.capacity {
            return Offer::Full(observation);
        }
        queue.pending.push_back(observation);
        queue.accepted += 1;
        self.shared.ready.notify_one();
        Offer::Accepted
    }

    /// Close admission, drain the queue and join.
    pub fn finish(mut self) -> DeliveryReport<E> {
        self.shutdown(Mode::Drain).unwrap()
    }

    /// Close admission, discard waiting observations and join.
    pub fn cancel(mut self) -> DeliveryReport<E> {
        self.shutdown(Mode::Cancel).unwrap()
    }
}

impl<E> AsyncDelivery<E> {
    fn close(&self, mode: Mode) {
        let mut queue = self.shared.queue.lock().unwrap();
        if queue.mode == Mode::Open {
            queue.mode = mode;
            if mode == Mode::Cancel {
                queue.pending.clear();
            }
        }
        self.shared.ready.notify_one();
    }

    fn shutdown(&mut self, mode: Mode) -> Option<DeliveryReport<E>> {
        let worker = self.worker.take()?;
        self.close(mode);
        Some(
            worker
                .join()
                .expect("delivery worker unwound outside callback"),
        )
    }
}

impl<E> Drop for AsyncDelivery<E> {
    fn drop(&mut self) {
        let _ = self.shutdown(Mode::Cancel);
    }
}

fn report<E>(queue: &Queue, completed: usize, stop: Stop<E>) -> DeliveryReport<E> {
    let failed = usize::from(matches!(
        &stop,
        Stop::ConsumerError(_) | Stop::ConsumerPanicked
    ));
    DeliveryReport {
        accepted: queue.accepted,
        completed,
        discarded: queue.accepted - completed - failed,
        stop,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Sensor;
    use std::{sync::mpsc, time::Duration};

    fn observation(sequence: u32) -> Observation {
        Observation::new(
            Sensor::Position,
            sequence,
            sequence * 20,
            sequence * 20 + 5,
            [1.0, -0.0],
        )
        .unwrap()
    }

    #[test]
    fn bounded_fifo_preserves_recorded_observations() {
        for capacity in [1, 2] {
            let (entered, entry) = mpsc::channel();
            let (release, gate) = mpsc::channel();
            let (record, results) = mpsc::channel();
            let mut owner = AsyncDelivery::start(capacity, move |item: Observation| {
                if item.id().sequence() == 3 {
                    entered.send(()).unwrap();
                    gate.recv_timeout(Duration::from_secs(5)).unwrap();
                }
                record.send(item).unwrap();
                Ok::<_, ()>(())
            })
            .unwrap();
            assert!(matches!(owner.offer(observation(3)), Offer::Accepted));
            entry.recv_timeout(Duration::from_secs(5)).unwrap();
            assert!(matches!(owner.offer(observation(1)), Offer::Accepted));
            if capacity == 2 {
                assert!(matches!(owner.offer(observation(1)), Offer::Accepted));
            }
            assert!(
                matches!(owner.offer(observation(2)), Offer::Full(item) if item.id().sequence() == 2)
            );
            release.send(()).unwrap();
            let report = owner.finish();
            assert_eq!(report.stop, Stop::Drained);
            assert_eq!(report.completed, capacity + 1);
            assert_eq!(report.discarded, 0);
            for expected in std::iter::once(3).chain(std::iter::repeat_n(1, capacity)) {
                let item = results.recv_timeout(Duration::from_secs(5)).unwrap();
                assert_eq!(item.id().sequence(), expected);
                assert_eq!(item.sample_tick(), expected * 20);
                assert_eq!(item.planned_arrival_tick(), expected * 20 + 5);
                assert_eq!(
                    item.value().map(f64::to_bits),
                    [1.0_f64, -0.0].map(f64::to_bits)
                );
            }
        }
    }

    #[test]
    fn cancellation_discards_waiting_and_preserves_inflight_failure() {
        for panics in [false, true] {
            let (entered, entry) = mpsc::channel();
            let (release, gate) = mpsc::channel();
            let mut owner = AsyncDelivery::start(1, move |_| {
                entered.send(()).unwrap();
                gate.recv_timeout(Duration::from_secs(5)).unwrap();
                assert!(!panics, "injected callback panic");
                Err(42)
            })
            .unwrap();
            assert!(matches!(owner.offer(observation(1)), Offer::Accepted));
            entry.recv_timeout(Duration::from_secs(5)).unwrap();
            assert!(matches!(owner.offer(observation(2)), Offer::Accepted));
            owner.close(Mode::Cancel);
            assert!(
                matches!(owner.offer(observation(3)), Offer::Closed(item) if item.id().sequence() == 3)
            );
            release.send(()).unwrap();
            let report = owner.cancel();
            assert_eq!(report.accepted, 2);
            assert_eq!(report.completed, 0);
            assert_eq!(report.discarded, 1);
            assert_eq!(
                report.stop,
                if panics {
                    Stop::ConsumerPanicked
                } else {
                    Stop::ConsumerError(42)
                }
            );
        }
    }

    #[test]
    fn idle_shutdown_and_invalid_capacity() {
        for invalid in [0, 1025, usize::MAX] {
            assert!(
                matches!(AsyncDelivery::start(invalid, |_| Ok::<_, ()>(())), Err(StartError::InvalidCapacity(n)) if n == invalid)
            );
        }
        let owner = AsyncDelivery::start(1, |_| Ok::<_, ()>(())).unwrap();
        assert_eq!(owner.cancel().stop, Stop::Cancelled);
    }

    #[test]
    fn claim_before_cancel_may_start_after_close_and_drop_joins() {
        for drop_owner in [false, true] {
            let (claimed, claim) = mpsc::channel();
            let (release, gate) = mpsc::channel();
            let (calls, called) = mpsc::channel();
            let mut owner = AsyncDelivery::start_inner(
                1,
                move |item: Observation| {
                    calls.send(item.id().sequence()).unwrap();
                    Ok::<_, ()>(())
                },
                move || {
                    claimed.send(()).unwrap();
                    gate.recv_timeout(Duration::from_secs(5)).unwrap();
                },
            )
            .unwrap();
            assert!(matches!(owner.offer(observation(3)), Offer::Accepted));
            claim.recv_timeout(Duration::from_secs(5)).unwrap();
            assert!(matches!(owner.offer(observation(1)), Offer::Accepted));
            let shared = Arc::clone(&owner.shared);
            let observer = thread::spawn(move || {
                let queue = shared.queue.lock().unwrap();
                let (queue, _) = shared
                    .ready
                    .wait_timeout_while(queue, Duration::from_secs(5), |q| q.mode == Mode::Open)
                    .unwrap();
                assert!(queue.mode == Mode::Cancel);
                assert!(queue.pending.is_empty());
                release.send(()).unwrap();
            });
            if drop_owner {
                drop(owner);
            } else {
                let report = owner.cancel();
                assert_eq!(
                    report,
                    DeliveryReport {
                        accepted: 2,
                        completed: 1,
                        discarded: 1,
                        stop: Stop::Cancelled
                    }
                );
            }
            observer.join().unwrap();
            assert_eq!(called.try_recv().unwrap(), 3);
            assert_eq!(called.try_recv(), Err(mpsc::TryRecvError::Disconnected));
        }
    }

    #[test]
    fn failure_closes_open_admission_even_when_panic_payload_drop_unwinds() {
        struct HostilePayload;
        impl Drop for HostilePayload {
            fn drop(&mut self) {
                panic!("payload destructor");
            }
        }
        for panics in [false, true] {
            let (entered, entry) = mpsc::channel();
            let (release, gate) = mpsc::channel();
            let mut owner = AsyncDelivery::start(1, move |_| {
                entered.send(()).unwrap();
                gate.recv_timeout(Duration::from_secs(5)).unwrap();
                if panics {
                    std::panic::panic_any(HostilePayload);
                }
                Err(7)
            })
            .unwrap();
            assert!(matches!(owner.offer(observation(3)), Offer::Accepted));
            entry.recv_timeout(Duration::from_secs(5)).unwrap();
            assert!(matches!(owner.offer(observation(1)), Offer::Accepted));
            release.send(()).unwrap();
            let report = owner.worker.take().unwrap().join().unwrap();
            assert_eq!(report.accepted, 2);
            assert_eq!(report.completed, 0);
            assert_eq!(report.discarded, 1);
            assert_eq!(
                report.stop,
                if panics {
                    Stop::ConsumerPanicked
                } else {
                    Stop::ConsumerError(7)
                }
            );
            assert!(matches!(owner.offer(observation(2)), Offer::Closed(_)));
        }
    }

    #[test]
    fn recorded_inputs_play_back_bitwise_under_two_pacings() {
        let play = |serial: bool| {
            let (send, receive) = mpsc::channel();
            let (release, gate) = mpsc::channel();
            let mut gate = Some(gate);
            let mut owner = AsyncDelivery::start(3, move |item: Observation| {
                if let Some(gate) = gate.take() {
                    gate.recv_timeout(Duration::from_secs(5)).unwrap();
                }
                send.send((
                    item.id(),
                    item.sample_tick(),
                    item.planned_arrival_tick(),
                    item.value().map(f64::to_bits),
                ))
                .unwrap();
                Ok::<_, ()>(())
            })
            .unwrap();
            let mut output = Vec::new();
            if serial {
                release.send(()).unwrap();
            }
            for sequence in [3, 1, 1] {
                assert!(matches!(
                    owner.offer(observation(sequence)),
                    Offer::Accepted
                ));
                if serial {
                    output.push(receive.recv_timeout(Duration::from_secs(5)).unwrap());
                }
            }
            if !serial {
                release.send(()).unwrap();
            }
            let report = owner.finish();
            output.extend(receive);
            (output, report)
        };
        assert_eq!(play(true), play(false));
    }
}
