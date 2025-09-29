#[cfg(test)]
mod tests {
    use std::time::Duration;

    #[test]
    ///
    /// | 特性                  | 标准库 `std::sync::Mutex`                                    | `parking_lot::Mutex`                                         |
    /// | :-------------------- | :----------------------------------------------------------- | :----------------------------------------------------------- |
    /// | **性能**              | 相对较慢。因为它基于操作系统原生的互斥锁，每次加锁/解锁都涉及系统调用，上下文切换开销大。 | **极快**。在用户态实现，使用特定的原子操作和线程挂起策略，避免了系统调用。无竞争时通常只需一条原子指令。 |
    /// | **内存占用**          | 较大。在 Linux 上通常至少 40 字节，因为需要存储系统互斥体的所有信息。 | **极小**。只有一个 `usize`（通常是 4 或 8 字节），与一个 `AtomicUsize` 大小相同。 |
    /// | **Poisoning（中毒）** | **有**。如果一个线程在持有锁时发生 panic，锁会“中毒”，后续线程尝试加锁会得到 `Err(PoisonError)`。这是一种安全机制，防止数据处于不一致状态时被访问。 | **默认无**。如果一个线程 panic，锁会被直接释放。这基于一个设计权衡：在复杂应用中，从中毒错误中恢复非常困难，通常程序会选择直接终止。`parking_lot` 认为“中毒”实用性不高，反而增加了性能开销和代码复杂度。 |
    /// | **公平性**            | 通常由操作系统调度决定，不保证公平。                         | 默认是**不公平锁**。这可以进一步提高吞吐量，因为刚释放锁的线程再次获取锁的概率很高，避免了上下文切换。但也提供了**公平锁**（`parking_lot::FairMutex` 或 `Mutex::<T>::lock_fair`）。 |
    /// | **特性丰富度**        | 基础功能。                                                   | **功能丰富**。提供： • `try_lock`：非阻塞尝试。 • `try_lock_for` / `try_lock_until`：带超时的尝试。 • `get_mut`：获取可变引用（无需锁）。 • `is_locked`：检查锁是否正被持有。 • `into_inner`：消耗 Mutex，取出内部值。 |
    /// | **与系统集成**        | **好**。因为它就是系统锁的包装，所以可以与其它语言或系统库中使用的同一个锁进行交互（尽管不推荐）。 | **无**。纯 Rust 实现，不与系统锁交互。                       |
    /// | **NoStd 支持**        | 不支持。                                                     | 支持。通过禁用默认 features 可以在 `no_std` 环境中使用。     |
    ///
    /// ###
    fn parking_lot_mutex_test1() {
        // 标准库
        let counter = std::sync::Mutex::new(0);
        {
            let mut guard = counter.lock().unwrap(); // 必须解包 Result
            *guard += 1;
        } // guard 离开作用域，锁自动释放

        // parking_lot
        let mut counter = parking_lot::Mutex::new(0);
        {
            let mut guard = counter.lock(); // 直接获得守卫，无需解包
            *guard += 1;
        } // guard 离开作用域，锁自动释放
        *counter.get_mut() += 2; // 无需锁，直接获取可变引用
        assert_eq!(*counter.lock(), 3);
    }

    #[test]
    fn parking_lot_mutex_test2() {
        use std::sync::Arc;
        use std::thread;

        let counter = Arc::new(parking_lot::Mutex::new(0));

        // 子线程的 counter
        let counter_clone = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            thread::sleep(Duration::from_millis(10));
            let mut guard = counter_clone.lock();
            *guard += 1;
            println!("child thread done, counter = {}", *guard);
        });

        // 主线程也通过锁来修改
        {
            let mut guard = counter.lock();
            thread::sleep(Duration::from_millis(500));
            *guard += 1;
            println!("main thread done, counter = {}", *guard);
        }

        handle.join().unwrap(); // 等待子线程结束
        println!("final counter = {}", *counter.lock());
        assert_eq!(*counter.lock(), 2);
    }
}
