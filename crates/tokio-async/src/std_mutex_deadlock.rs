#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::thread;
    use tokio::{
        self,
        runtime::Runtime,
        time::{sleep, Duration},
    };
    async fn work_std_mutex(mtx: &std::sync::Mutex<i32>) {
        println!("std mutex lock");
        {
            let mut v = mtx.lock().unwrap();
            println!("std mutex locked");

            sleep(Duration::from_millis(100)).await;
            *v += 1;
        }
        println!("std mutex unlock")
    }

    async fn work_tokio_mutex(mtx: &tokio::sync::Mutex<i32>) {
        println!("tokio mutex lock");
        {
            let mut v = mtx.lock().await;
            println!("tokio mutex locked");
            sleep(Duration::from_millis(100)).await;
            *v += 1;
        }
        println!("tokio mutex unlock")
    }

    #[test]
    #[should_panic(expected = "死锁发生")]
    /// 使用std::sync::Mutex在 work_std_mutex 函数实现这种情况下会导致死锁
    fn std_mutex_deadlock() {
        let test_completed = Arc::new(std::sync::Mutex::new(false));
        let test_completed_clone = Arc::clone(&test_completed);

        thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                let mtx = std::sync::Mutex::new(0);
                tokio::join!(work_std_mutex(&mtx), work_std_mutex(&mtx));
                *test_completed_clone.lock().unwrap() = true;
            });
        });

        // 等待足够长时间
        thread::sleep(Duration::from_secs(3));

        let completed = *test_completed.lock().unwrap();
        if !completed {
            panic!("死锁发生：std::sync::Mutex 在异步环境中导致死锁");
        }
    }

    #[test]
    /// 使用tokio::sync::Mutex在同样的逻辑下不会导致死锁
    fn tokio_mutex_deadlock() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let mtx = tokio::sync::Mutex::new(0);
            tokio::join!(work_tokio_mutex(&mtx), work_tokio_mutex(&mtx));
            println!("tokio_mutex_deadlock run ok! {}", *mtx.lock().await);
        });
    }
}
