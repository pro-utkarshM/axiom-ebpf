use core::ptr;
use core::pin::Pin;
use alloc::boxed::Box;
use crate::mcore::mtask::process::Process;
use crate::mcore::mtask::scheduler::global::GlobalTaskQueue;
use crate::mcore::mtask::task::Task;

pub struct TaskCleanup;

impl TaskCleanup {
    pub fn init() {
        log::info!("TaskCleanup: initializing...");
        let task = Task::create_new(Process::root(), Self::run, ptr::null_mut())
            .expect("should be able to create task cleanup");
        log::info!("TaskCleanup: task created, enqueuing...");
        GlobalTaskQueue::enqueue(Box::pin(task));
        log::info!("TaskCleanup: initialized");
    }

    pub fn enqueue(_task: Pin<Box<Task>>) {
        // TODO: implement actual cleanup queue
        // For now, we just let the task be dropped if it's finished,
        // but we need to be careful about where it's dropped.
        log::trace!("TaskCleanup: received zombie task");
    }

    extern "C" fn run(_arg: *mut core::ffi::c_void) {
        log::info!("TaskCleanup: running");
        loop {
            // TODO: implement task cleanup logic
            #[cfg(target_arch = "x86_64")]
            x86_64::instructions::hlt();
            #[cfg(target_arch = "aarch64")]
            unsafe {
                core::arch::asm!("wfi");
            }
        }
    }
}
