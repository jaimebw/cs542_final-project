use std::future::Future;
use std::hint::spin_loop;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering::SeqCst;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::Semaphore;
use tokio::task::yield_now;

/// The primary goal of the rate limiter is to limit the rate in which functions passed to
/// [RateLimit::perform_rate_limited] are called. For this purpose, it has been designed to
/// guarantee that:
///  - No more than `permits` calls are in progress at a given time
///  - A call will not begin until `cool_down` has passed since the previous call
///  - This structure can be used synchronously from multiple threads
pub struct RateLimit {
    cool_down: Duration,
    permits: Semaphore,
    total_permits: usize,
    last: AtomicU64,
}

impl RateLimit {
    pub fn new(permits: usize, cool_down: Duration) -> Self {
        RateLimit {
            cool_down,
            permits: Semaphore::new(permits),
            total_permits: permits,
            last: AtomicU64::new(0),
        }
    }

    pub fn max_sync_usages(&self) -> usize {
        self.total_permits
    }

    pub async fn perform_rate_limited<F, R, A>(&self, func: F) -> R
        where
            F: FnOnce() -> A,
            A: Future<Output = R>,
    {
        let permit = match self.permits.acquire().await {
            Ok(permit) => permit,
            Err(_) => unreachable!("Semaphore will never close unless thread panics"),
        };

        self.wait_until_period().await;

        let result = func().await;
        drop(permit);
        result
    }

    async fn wait_until_period(&self) {
        loop {
            let now = SystemTime::now();
            let last_occurrence = self.last.load(SeqCst);
            let last_time = SystemTime::UNIX_EPOCH + Duration::from_nanos(last_occurrence);

            // Wait for availability
            if let Ok(x) = (last_time + self.cool_down).duration_since(now) {
                spin_yield_until(Instant::now() + x).await;
                continue;
            }

            let next = match now.duration_since(SystemTime::UNIX_EPOCH) {
                Ok(v) => v,
                Err(_) => unreachable!("Unix epoch should always be before current system time"),
            };

            if self
                .last
                .compare_exchange(last_occurrence, next.as_nanos() as u64, SeqCst, SeqCst)
                .is_ok()
            {
                break;
            }
        }
    }
}

/// This isn't perfect, but it is good enough for now.
#[cold]
async fn spin_yield_until(end_time: Instant) {
    const SPIN_LIMIT: u32 = 8;

    let mut n = 0;
    while Instant::now() < end_time {
        if n == SPIN_LIMIT {
            yield_now().await;
            continue;
        }

        for _ in 0..1 << n {
            spin_loop();
        }
        n += 1;
    }
}
