#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;
    use tokio::runtime::Runtime;

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


    #[test]
    #[should_panic(expected = "死锁发生")]
    fn tokio_rwlock_read_write_conflict_deadlock() {
        let test_completed = Arc::new(std::sync::Mutex::new(false));
        let test_completed_clone = Arc::clone(&test_completed);
        thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                let value = tokio::sync::RwLock::new(true);
                // 这种写法不会导致死锁
                if *value.read().await {
                    *value.write().await = false;
                }
                println!("value 1 = {}", *value.read().await);

                let gv = value.read().await;
                if *gv == false {
                    // 此处不添加drop代码会导致死锁
                    // drop(gv);
                    *value.write().await = true;
                }
                println!("value 2 = {}", *value.read().await);
            });
            *test_completed_clone.lock().unwrap() = true;
        });

        // 等待足够长时间
        thread::sleep(Duration::from_secs(3));

        let completed = *test_completed.lock().unwrap();
        if !completed {
            panic!("死锁发生：RwLock 在读取锁未释放的情况下，去进行写操作导致死锁");
        }
    }
}
