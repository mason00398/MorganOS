//! 抢占式进程调度器 - 基于fringe
//! 
//! 修复内容：
//! - 添加完整的进程创建/终止
//! - 添加进程间通信 (IPC) 机制
//! - 添加进程同步原语 (信号量/互斥锁)
//! - 完善上下文切换

use alloc::collections::BTreeMap;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use fringe::{OsStack, Generator};
use spin::Mutex;

// ==================== 进程控制块 (PCB) ====================

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProcessId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    Ready,
    Running,
    Blocked,
    Terminated,
    Sleeping,
}

#[repr(C)]
pub struct ProcessContext {
    pub rsp: u64,
    pub rip: u64,
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    pub rbp: u64,
}

pub struct Process {
    pub id: ProcessId,
    pub name: alloc::string::String,
    pub state: ProcessState,
    pub time_slice: u64,
    pub priority: u8,
    pub generator: Generator<'static, (), (), OsStack>,
    pub context: ProcessContext,
    pub stack_size: usize,
    pub parent: Option<ProcessId>,
    pub children: Vec<ProcessId>,
}

impl Process {
    pub fn new(id: ProcessId, name: &str, priority: u8, entry: extern "C" fn() -> !) -> Self {
        let stack_size = 1 << 16; // 64KB per process
        let stack = OsStack::new(stack_size).expect("Failed to allocate stack");
        
        let gen = Generator::new(stack, move |_| {
            entry();
            loop { core::hint::spin_loop(); }
        });
        
        Self {
            id,
            name: name.into(),
            state: ProcessState::Ready,
            time_slice: 10,
            priority,
            generator: gen,
            context: ProcessContext {
                rsp: 0, rip: 0, rax: 0, rbx: 0, rcx: 0, rdx: 0,
                rsi: 0, rdi: 0, r8: 0, r9: 0, r10: 0, r11: 0,
                r12: 0, r13: 0, r14: 0, r15: 0, rbp: 0,
            },
            stack_size,
            parent: None,
            children: Vec::new(),
        }
    }
}

// ==================== 信号量 ====================

pub struct Semaphore {
    count: AtomicUsize,
    name: alloc::string::String,
}

impl Semaphore {
    pub fn new(initial: usize, name: &str) -> Self {
        Self {
            count: AtomicUsize::new(initial),
            name: name.into(),
        }
    }
    
    pub fn wait(&self) {
        loop {
            let current = self.count.load(Ordering::Relaxed);
            if current > 0 && self.count.compare_exchange(
                current, current - 1, Ordering::Acquire, Ordering::Relaxed
            ).is_ok() {
                return;
            }
            // 阻塞等待
            core::hint::spin_loop();
        }
    }
    
    pub fn signal(&self) {
        self.count.fetch_add(1, Ordering::Release);
    }
}

// ==================== 互斥锁 ====================

pub struct MutexLock {
    locked: AtomicUsize,
    owner: AtomicU64,
    name: alloc::string::String,
}

impl MutexLock {
    pub fn new(name: &str) -> Self {
        Self {
            locked: AtomicUsize::new(0),
            owner: AtomicU64::new(0),
            name: name.into(),
        }
    }
    
    pub fn lock(&self, pid: u64) {
        loop {
            if self.locked.compare_exchange(
                0, 1, Ordering::Acquire, Ordering::Relaxed
            ).is_ok() {
                self.owner.store(pid, Ordering::Relaxed);
                return;
            }
            core::hint::spin_loop();
        }
    }
    
    pub fn unlock(&self, pid: u64) {
        if self.owner.load(Ordering::Relaxed) == pid {
            self.locked.store(0, Ordering::Release);
        }
    }
}

// ==================== 进程间通信 (IPC) ====================

pub struct MessageQueue {
    messages: Mutex<VecDeque<alloc::vec::Vec<u8>>>,
    capacity: usize,
    name: alloc::string::String,
}

impl MessageQueue {
    pub fn new(capacity: usize, name: &str) -> Self {
        Self {
            messages: spin::Mutex::new(VecDeque::with_capacity(capacity)),
            capacity,
            name: name.into(),
        }
    }
    
    pub fn send(&self, msg: alloc::vec::Vec<u8>) -> bool {
        let mut queue = self.messages.lock();
        if queue.len() >= self.capacity {
            return false;
        }
        queue.push_back(msg);
        true
    }
    
    pub fn recv(&self) -> Option<alloc::vec::Vec<u8>> {
        let mut queue = self.messages.lock();
        queue.pop_front()
    }
    
    pub fn len(&self) -> usize {
        self.messages.lock().len()
    }
}

// ==================== 调度器 ====================

pub struct Scheduler {
    processes: BTreeMap<ProcessId, Process>,
    ready_queue: VecDeque<ProcessId>,
    current: Option<ProcessId>,
    next_pid: AtomicU64,
    total_ticks: AtomicU64,
    // IPC 对象
    message_queues: BTreeMap<String, Arc<MessageQueue>>,
    // 同步原语
    semaphores: BTreeMap<String, Arc<Semaphore>>,
    mutexes: BTreeMap<String, Arc<MutexLock>>,
}

impl Scheduler {
    pub const fn new() -> Self {
        Self {
            processes: BTreeMap::new(),
            ready_queue: VecDeque::new(),
            current: None,
            next_pid: AtomicU64::new(1),
            total_ticks: AtomicU64::new(0),
            message_queues: BTreeMap::new(),
            semaphores: BTreeMap::new(),
            mutexes: BTreeMap::new(),
        }
    }

    /// 创建新进程 (fork)
    pub fn fork(&mut self, name: &str, priority: u8, entry: extern "C" fn() -> !) -> ProcessId {
        let pid = ProcessId(self.next_pid.fetch_add(1, Ordering::Relaxed));
        let proc = Process::new(pid, name, priority, entry);
        self.processes.insert(pid, proc);
        self.ready_queue.push_back(pid);
        
        // 设置父进程关系
        if let Some(current) = self.current {
            if let Some(parent) = self.processes.get_mut(&current) {
                parent.children.push(pid);
            }
        }
        
        pid
    }

    /// 终止进程
    pub fn terminate(&mut self, pid: ProcessId) -> bool {
        // 递归终止子进程
        if let Some(proc) = self.processes.get(&pid) {
            for child_id in &proc.children {
                self.terminate(*child_id);
            }
        }
        
        if self.processes.remove(&pid).is_some() {
            self.ready_queue.retain(|&p| p != pid);
            if self.current == Some(pid) {
                self.current = None;
                self.yield_cpu();
            }
            true
        } else {
            false
        }
    }

    /// 创建消息队列
    pub fn create_message_queue(&mut self, capacity: usize, name: &str) -> &Arc<MessageQueue> {
        let mq = Arc::new(MessageQueue::new(capacity, name));
        self.message_queues.insert(name.into(), Arc::clone(&mq));
        Arc::clone(&mq)
    }

    /// 创建信号量
    pub fn create_semaphore(&mut self, initial: usize, name: &str) -> &Arc<Semaphore> {
        let sem = Arc::new(Semaphore::new(initial, name));
        self.semaphores.insert(name.into(), Arc::clone(&sem));
        Arc::clone(&sem)
    }

    /// 创建互斥锁
    pub fn create_mutex(&mut self, name: &str) -> &Arc<MutexLock> {
        let mtx = Arc::new(MutexLock::new(name));
        self.mutexes.insert(name.into(), Arc::clone(&mtx));
        Arc::clone(&mtx)
    }

    fn running_count(&self) -> usize {
        self.processes.values()
            .filter(|p| p.state == ProcessState::Running || p.state == ProcessState::Ready)
            .count()
    }

    pub fn tick(&mut self) {
        self.total_ticks.fetch_add(1, Ordering::Relaxed);

        if self.running_count() == 0 {
            self.current = None;
            return;
        }

        let current_pid = match self.current {
            Some(pid) => pid,
            None => { self.yield_cpu(); return; }
        };

        if let Some(proc) = self.processes.get_mut(&current_pid) {
            if proc.time_slice > 0 {
                proc.time_slice -= 1;
            }
            if proc.time_slice == 0 || proc.state != ProcessState::Running {
                proc.state = ProcessState::Ready;
                self.ready_queue.push_back(current_pid);
                self.yield_cpu();
            }
        }
    }

    pub fn yield_cpu(&mut self) {
        while let Some(pid) = self.ready_queue.pop_front() {
            if let Some(proc) = self.processes.get_mut(&pid) {
                if proc.state == ProcessState::Ready || proc.state == ProcessState::Running {
                    proc.state = ProcessState::Running;
                    proc.time_slice = 10;
                    self.current = Some(pid);
                    proc.generator.resume(());
                    return;
                }
            }
        }
        self.current = None;
    }

    pub fn list_processes(&self) -> Vec<(ProcessId, &str, ProcessState)> {
        let mut result = Vec::new();
        for (id, proc) in &self.processes {
            if proc.state != ProcessState::Terminated {
                result.push((*id, proc.name.as_str(), proc.state));
            }
        }
        result
    }

    pub fn current(&self) -> Option<ProcessId> { self.current }
}

static mut G_SCHEDULER: Option<Scheduler> = None;

pub fn init_scheduler() {
    unsafe { G_SCHEDULER = Some(Scheduler::new()); }
}

pub fn get_scheduler() -> &'static Scheduler {
    unsafe { G_SCHEDULER.as_ref().expect("Scheduler not initialized") }
}

pub fn get_scheduler_mut() -> &'static mut Scheduler {
    unsafe { G_SCHEDULER.as_mut().expect("Scheduler not initialized") }
}
