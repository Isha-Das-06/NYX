use std::collections::HashMap;
use std::mem;
use parking_lot::Mutex;
use indexmap::IndexSet;

pub trait GcTrace {
    fn trace(&self, visitor: &mut GcVisitor);
    fn size(&self) -> usize;
}

pub struct GcVisitor {
    pub marked: IndexSet<usize>,
}

impl GcVisitor {
    pub fn new() -> Self {
        Self {
            marked: IndexSet::new(),
        }
    }

    pub fn mark_object<T: GcTrace>(&mut self, obj: &GcObject<T>) {
        let ptr = (obj.as_ptr() as usize);
        if !self.marked.contains(&ptr) {
            self.marked.insert(ptr);
            unsafe {
                obj.value.trace(self);
            }
        }
    }
}

pub struct GcObject<T> {
    pub value: T,
    pub marked: bool,
}

impl<T> GcObject<T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            marked: false,
        }
    }

    pub fn as_ptr(&self) -> *const T {
        &self.value as *const T
    }
}

pub struct GarbageCollector {
    objects: HashMap<usize, Box<dyn GcTraceable>>,
    roots: IndexSet<usize>,
    allocation_count: usize,
    gc_threshold: usize,
    total_allocated: usize,
}

impl GarbageCollector {
    pub fn new() -> Self {
        Self {
            objects: HashMap::new(),
            roots: IndexSet::new(),
            allocation_count: 0,
            gc_threshold: 1000,
            total_allocated: 0,
        }
    }

    pub fn allocate<T: GcTrace + 'static>(&mut self, value: T) -> GcPtr<T> {
        let object = Box::new(GcObject::new(value));
        let ptr = Box::into_raw(object) as usize;
        
        self.objects.insert(ptr, unsafe {
            Box::from_raw(ptr as *mut GcObject<T>) as Box<dyn GcTraceable>
        });
        
        self.allocation_count += 1;
        self.total_allocated += mem::size_of::<T>();
        
        if self.allocation_count >= self.gc_threshold {
            self.collect_garbage();
        }
        
        GcPtr {
            ptr: ptr as *mut T,
            gc: self as *mut GarbageCollector as *mut (),
        }
    }

    pub fn add_root(&mut self, ptr: usize) {
        self.roots.insert(ptr);
    }

    pub fn remove_root(&mut self, ptr: usize) {
        self.roots.remove(&ptr);
    }

    pub fn collect_garbage(&mut self) {
        let mut visitor = GcVisitor::new();
        
        // Mark phase
        for &root_ptr in &self.roots {
            if let Some(object) = self.objects.get(&root_ptr) {
                object.trace(&mut visitor);
            }
        }
        
        // Sweep phase
        let mut to_remove = Vec::new();
        for (ptr, object) in &self.objects {
            if !visitor.marked.contains(ptr) {
                to_remove.push(*ptr);
            }
        }
        
        for ptr in to_remove {
            if let Some(object) = self.objects.remove(&ptr) {
                self.total_allocated -= object.size();
            }
        }
        
        self.allocation_count = 0;
    }

    pub fn get_stats(&self) -> GcStats {
        GcStats {
            total_objects: self.objects.len(),
            total_allocated: self.total_allocated,
            roots: self.roots.len(),
        }
    }
}

pub trait GcTraceable: Send + Sync {
    fn trace(&self, visitor: &mut GcVisitor);
    fn size(&self) -> usize;
}

impl<T: GcTrace> GcTraceable for GcObject<T> {
    fn trace(&self, visitor: &mut GcVisitor) {
        visitor.mark_object(self);
    }

    fn size(&self) -> usize {
        mem::size_of::<T>()
    }
}

pub struct GcPtr<T> {
    ptr: *mut T,
    gc: *mut (),
}

impl<T> GcPtr<T> {
    pub fn get(&self) -> &T {
        unsafe { &*self.ptr }
    }

    pub fn get_mut(&mut self) -> &mut T {
        unsafe { &mut *self.ptr }
    }

    pub fn as_raw(&self) -> *mut T {
        self.ptr
    }
}

impl<T> Clone for GcPtr<T> {
    fn clone(&self) -> Self {
        Self {
            ptr: self.ptr,
            gc: self.gc,
        }
    }
}

impl<T> Drop for GcPtr<T> {
    fn drop(&mut self) {
        unsafe {
            let gc = self.gc as *mut GarbageCollector;
            (*gc).remove_root(self.ptr as usize);
        }
    }
}

#[derive(Debug, Clone)]
pub struct GcStats {
    pub total_objects: usize,
    pub total_allocated: usize,
    pub roots: usize,
}

// Implement GcTrace for primitive types
impl GcTrace for i64 {
    fn trace(&self, _visitor: &mut GcVisitor) {}
    fn size(&self) -> usize { mem::size_of::<i64>() }
}

impl GcTrace for f64 {
    fn trace(&self, _visitor: &mut GcVisitor) {}
    fn size(&self) -> usize { mem::size_of::<f64>() }
}

impl GcTrace for bool {
    fn trace(&self, _visitor: &mut GcVisitor) {}
    fn size(&self) -> usize { mem::size_of::<bool>() }
}

impl GcTrace for String {
    fn trace(&self, _visitor: &mut GcVisitor) {}
    fn size(&self) -> usize { mem::size_of::<String>() + self.len() }
}

impl<T: GcTrace> GcTrace for Vec<T> {
    fn trace(&self, visitor: &mut GcVisitor) {
        for item in self {
            item.trace(visitor);
        }
    }

    fn size(&self) -> usize {
        mem::size_of::<Vec<T>>() + self.len() * mem::size_of::<T>()
    }
}

impl<T: GcTrace> GcTrace for Option<T> {
    fn trace(&self, visitor: &mut GcVisitor) {
        if let Some(value) = self {
            value.trace(visitor);
        }
    }

    fn size(&self) -> usize {
        mem::size_of::<Option<T>>() + 
            match self {
                Some(value) => value.size(),
                None => 0,
            }
    }
}

impl<K: GcTrace, V: GcTrace> GcTrace for HashMap<K, V> {
    fn trace(&self, visitor: &mut GcVisitor) {
        for (key, value) in self {
            key.trace(visitor);
            value.trace(visitor);
        }
    }

    fn size(&self) -> usize {
        mem::size_of::<HashMap<K, V>>() + 
            self.len() * (mem::size_of::<K>() + mem::size_of::<V>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_allocation() {
        let mut gc = GarbageCollector::new();
        let ptr = gc.allocate(42i64);
        assert_eq!(*ptr.get(), 42);
    }

    #[test]
    fn test_garbage_collection() {
        let mut gc = GarbageCollector::new();
        {
            let _ptr = gc.allocate(42i64);
            gc.collect_garbage();
            let stats = gc.get_stats();
            assert_eq!(stats.total_objects, 1);
        }
        gc.collect_garbage();
        let stats = gc.get_stats();
        assert_eq!(stats.total_objects, 0);
    }
}
