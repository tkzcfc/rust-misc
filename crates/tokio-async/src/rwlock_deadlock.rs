#[cfg(test)]
mod tests {
    #[test]
    /// https://rust-book.junmajinlong.com/ch100/06_task_state_sync.html
    fn tokio_rwlock_deadlock() {
        use std::sync::Arc;
        use tokio::{
            self,
            runtime::Runtime,
            sync::RwLock,
            time::{self, Duration},
        };

        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let lock = Arc::new(RwLock::new(0));

            let lock1 = lock.clone();
            tokio::spawn(async move {
                println!("try to read 1");
                let n = lock1.read().await;
                println!("n = {}", *n);

                // 此处不添加drop代码会导致死锁
                drop(n);

                time::sleep(Duration::from_secs(2)).await;

                println!("try to read 2");
                let nn = lock1.read().await;
                println!("nn = {}", *nn);
            });

            time::sleep(Duration::from_secs(1)).await;
            println!("try to write");
            let mut wn = lock.write().await;
            *wn = 2;
            println!("write ok");
            time::sleep(Duration::from_secs(2)).await;
        });
    }
}
