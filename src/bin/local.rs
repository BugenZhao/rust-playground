use std::alloc::{GlobalAlloc, Layout};
use std::sync::atomic::{AtomicUsize, Ordering};

use tokio::task_local;

pub struct TaskLocalAllocator;

static GLOBAL_ALLOC: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct TaskLocalBytesAllocated(Option<&'static AtomicUsize>);

impl TaskLocalBytesAllocated {
    pub const fn null() -> Self {
        TaskLocalBytesAllocated(None)
    }

    pub fn add(&self, val: usize) {
        if let Some(bytes) = self.0 {
            bytes.fetch_add(val, Ordering::Relaxed);
        }
    }

    pub unsafe fn add_unchecked(&self, val: usize) {
        self.0.unwrap_unchecked().fetch_add(val, Ordering::Relaxed);
    }

    pub fn sub(&self, val: usize) {
        if let Some(bytes) = self.0 {
            bytes.fetch_sub(val, Ordering::Relaxed);
        }
    }
}

impl Default for TaskLocalBytesAllocated {
    fn default() -> Self {
        TaskLocalBytesAllocated(Some(Box::leak(Box::new(AtomicUsize::new(0)))))
    }
}

unsafe impl Send for TaskLocalBytesAllocated {}

impl TaskLocalBytesAllocated {
    pub fn val(&self) -> usize {
        self.0.as_ref().unwrap().load(Ordering::Relaxed)
    }
}

task_local! {
    pub static BYTES_ALLOCATED: TaskLocalBytesAllocated;
}

fn wrap_layout(layout: Layout) -> (Layout, usize) {
    let (wrapped_layout, offset) = Layout::new::<TaskLocalBytesAllocated>()
        .extend(layout)
        .expect("wrapping layout overflow");
    let wrapped_layout = wrapped_layout.pad_to_align();

    (wrapped_layout, offset)
}

struct TaskLocalAlloc;

unsafe impl GlobalAlloc for TaskLocalAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let (wrapped_layout, offset) = wrap_layout(layout);

        BYTES_ALLOCATED
            .try_with(|&bytes| {
                bytes.add_unchecked(layout.size());
                let ptr = GLOBAL_ALLOC.alloc(wrapped_layout);
                *ptr.cast() = bytes;
                ptr.wrapping_add(offset)
            })
            .unwrap_or_else(|_| {
                let ptr = GLOBAL_ALLOC.alloc(wrapped_layout);
                *ptr.cast() = TaskLocalBytesAllocated::null();
                ptr.wrapping_add(offset)
            })
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let (wrapped_layout, offset) = wrap_layout(layout);
        let ptr = ptr.wrapping_sub(offset);

        let bytes: TaskLocalBytesAllocated = *ptr.cast();
        bytes.sub(layout.size());

        GLOBAL_ALLOC.dealloc(ptr, wrapped_layout);
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let (wrapped_layout, offset) = wrap_layout(layout);

        BYTES_ALLOCATED
            .try_with(|&bytes| {
                bytes.add_unchecked(layout.size());
                let ptr = GLOBAL_ALLOC.alloc_zeroed(wrapped_layout);
                *ptr.cast() = bytes;
                ptr.wrapping_add(offset)
            })
            .unwrap_or_else(|_| {
                let ptr = GLOBAL_ALLOC.alloc_zeroed(wrapped_layout);
                *ptr.cast() = TaskLocalBytesAllocated::null();
                ptr.wrapping_add(offset)
            })
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let (wrapped_layout, offset) = wrap_layout(layout);
        let ptr = ptr.wrapping_sub(offset);

        let bytes: TaskLocalBytesAllocated = *ptr.cast();
        bytes.add(new_size);
        bytes.sub(layout.size());

        let ptr = GLOBAL_ALLOC.realloc(ptr, wrapped_layout, new_size);
        if ptr.is_null() {
            ptr
        } else {
            *ptr.cast() = bytes;
            ptr.wrapping_add(offset)
        }
    }
}

#[global_allocator]
static TASK_LOCAL_ALLOC: TaskLocalAlloc = TaskLocalAlloc;

fn main() {
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async { 233 });
}
