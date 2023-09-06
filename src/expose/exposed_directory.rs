use core::sync::atomic::{AtomicPtr, AtomicU8};

use alloc::{borrow::Cow, collections::BTreeMap, sync::Arc};
use bytedata::StringData;

// TODO: complete this file

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DirWarmup {
    /// Warmup the directory lazily. Files will be added to the static cache but will not be recomputed when changed on the file system until the first time they've been requested.
    #[default]
    Warm,
    /// Warmup the directory eagerly. All files will be added to the static cache and will be recomputed when changed on the file system.
    Hot,
    /// Do not warmup the directory. No files will be added to the static cache until requested.
    Cold,
}

pub trait ExposeFilterTrait: core::any::Any + Send + Sync + 'static {
    fn filter_map_file<'s, 'a: 's, 'b: 's>(
        &'s self,
        base: &'a str,
        path: &'b str,
    ) -> Option<StringData<'s>>;
    fn filter_map_dir<'s, 'a: 's, 'b: 's>(
        &'s self,
        base: &'a str,
        path: &'b str,
    ) -> Option<StringData<'s>>;
    fn as_any(&'_ self) -> &'_ (dyn core::any::Any + Sync + Send);
}

impl ExposeFilterTrait for ExposeFilter {
    fn filter_map_file<'s, 'a: 's, 'b: 's>(
        &'s self,
        base: &'a str,
        path: &'b str,
    ) -> Option<StringData<'s>> {
        self.filter.filter_map_file(base, path)
    }
    fn filter_map_dir<'s, 'a: 's, 'b: 's>(
        &'s self,
        base: &'a str,
        path: &'b str,
    ) -> Option<StringData<'s>> {
        self.filter.filter_map_dir(base, path)
    }
    fn as_any(&'_ self) -> &'_ (dyn core::any::Any + Sync + Send) {
        self
    }
}

impl ExposeFilterTrait for fn(&str) -> bool {
    fn filter_map_file<'s, 'a: 's, 'b: 's>(
        &'s self,
        _base: &'a str,
        path: &'b str,
    ) -> Option<StringData<'s>> {
        if (self)(path) {
            Some(StringData::from_borrowed(path))
        } else {
            None
        }
    }
    fn filter_map_dir<'s, 'a: 's, 'b: 's>(
        &'s self,
        _base: &'a str,
        path: &'b str,
    ) -> Option<StringData<'s>> {
        if (self)(path) {
            Some(StringData::from_borrowed(path))
        } else {
            None
        }
    }
    fn as_any(&'_ self) -> &'_ (dyn core::any::Any + Sync + Send) {
        self
    }
}

impl ExposeFilterTrait for for<'a> fn(&'a str) -> Option<&'a str> {
    fn filter_map_file<'s, 'a: 's, 'b: 's>(
        &'s self,
        _base: &'a str,
        path: &'b str,
    ) -> Option<StringData<'s>> {
        let path = (self)(path)?;
        Some(StringData::from_borrowed(path))
    }
    fn filter_map_dir<'s, 'a: 's, 'b: 's>(
        &'s self,
        _base: &'a str,
        path: &'b str,
    ) -> Option<StringData<'s>> {
        let path = (self)(path)?;
        Some(StringData::from_borrowed(path))
    }
    fn as_any(&'_ self) -> &'_ (dyn core::any::Any + Sync + Send) {
        self
    }
}

impl<F: Fn(&str, &str) -> bool + Send + Sync + 'static> ExposeFilterTrait for F {
    fn filter_map_file<'s, 'a: 's, 'b: 's>(
        &'s self,
        base: &'a str,
        path: &'b str,
    ) -> Option<StringData<'s>> {
        if (self)(base, path) {
            Some(StringData::from_borrowed(path))
        } else {
            None
        }
    }
    fn filter_map_dir<'s, 'a: 's, 'b: 's>(
        &'s self,
        base: &'a str,
        path: &'b str,
    ) -> Option<StringData<'s>> {
        if (self)(base, path) {
            Some(StringData::from_borrowed(path))
        } else {
            None
        }
    }
    fn as_any(&'_ self) -> &'_ (dyn core::any::Any + Sync + Send) {
        self
    }
}

/// A filter that can be used to limit exposure of files and directories.
#[derive(Clone)]
#[repr(transparent)]
pub struct ExposeFilter {
    filter: Arc<dyn ExposeFilterTrait>,
}

impl ExposeFilter {
    /// Create a new filter from a trait object.
    pub fn new(filter: impl ExposeFilterTrait + 'static) -> Self {
        let typeid = {
            let filter = filter.as_any();
            filter.type_id()
        };
        if core::any::TypeId::of::<ExposeFilter>() == typeid {
            let filter = core::mem::ManuallyDrop::new(filter);
            return unsafe {
                core::ptr::read(filter.as_any().downcast_ref::<ExposeFilter>().unwrap())
            };
        }
        ExposeFilter {
            filter: Arc::new(filter),
        }
    }
    /// Create a filter that exposes all files and directories except hidden files and directories.
    /// Hidden objects are entries that starts with `.`.
    pub fn not_hidden() -> Self {
        fn not_hidden(path: &str) -> bool {
            !path.starts_with('.') && !path.ends_with('~') && !path.contains("/.")
        }
        static FILT_NOT_HIDDEN: AtomicPtr<Arc<dyn ExposeFilterTrait>> =
            AtomicPtr::new(core::ptr::null_mut());
        let filter = FILT_NOT_HIDDEN.load(core::sync::atomic::Ordering::Relaxed);
        let filter = if filter.is_null() {
            let filter: std::sync::Arc<(dyn ExposeFilterTrait + 'static)> =
                Arc::new(not_hidden as fn(&str) -> bool);
            let filterp = Box::into_raw(Box::new(filter.clone()));
            if FILT_NOT_HIDDEN
                .compare_exchange(
                    core::ptr::null_mut(),
                    filterp,
                    core::sync::atomic::Ordering::Relaxed,
                    core::sync::atomic::Ordering::Relaxed,
                )
                .is_err()
            {
                core::mem::drop(unsafe { Box::from_raw(filterp) });
            }
            filter
        } else {
            unsafe { (*filter).clone() }
        };
        ExposeFilter { filter }
    }
}

type FileEntry = (
    AtomicU8,
    AtomicU8,
    parking_lot::RwLock<Arc<super::super::std::StdHttpFile>>,
);

pub struct ExposedDirectory {
    warmup: DirWarmup,
    web_path: Cow<'static, str>,
    file_path: Cow<'static, str>,
    files: parking_lot::RwLock<BTreeMap<Cow<'static, str>, FileEntry>>,
    nested: parking_lot::RwLock<BTreeMap<Cow<'static, str>, ExposedDirectory>>,
    filter: ExposeFilter,
}

impl ExposedDirectory {
    pub fn new_blocking(
        warmup: DirWarmup,
        web_path: impl Into<Cow<'static, str>>,
        file_path: impl Into<Cow<'static, str>>,
        filter: impl ExposeFilterTrait,
    ) -> std::io::Result<Self> {
        let web_path = web_path.into();
        let file_path = file_path.into();
        let filter = ExposeFilter::new(filter);
        let mut files = BTreeMap::new();
        let mut nested = BTreeMap::new();
        if matches!(warmup, DirWarmup::Hot | DirWarmup::Warm) {
            let mut walker = std::fs::read_dir(file_path.as_ref())?;
            while let Some(entry) = walker.next().and_then(|entry| entry.ok()) {
                let path = entry.path();
                if path.is_file() {
                    // TODO: files.insert(endpoint, file_entry);
                    1
                } else if path.is_dir() {
                    // TODO: nested.insert(endpoint, ExposedDirectory::new_blocking(warmup, endpoint, file_path)?);
                    2
                } else {
                    continue;
                };
            }
        }
        Ok(ExposedDirectory {
            warmup,
            web_path,
            file_path,
            files: parking_lot::RwLock::new(files),
            nested: parking_lot::RwLock::new(nested),
            filter,
        })
    }
}
