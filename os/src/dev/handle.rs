//! Lightweight handle types for shared ownership and weak parent references.
//!
//! Provide two complementary handle types:
//! - [Handle<T>] owns a strong reference to an object using [alloc::sync::Arc]. Use it where
//!   shared, long-lived ownership is required (for example device nodes).
//!   There should be only one [Handle<T>] instance to keep the lifecycle, 
//!   and other instances fetched by calling [HandleRef<T>::get_handle()] should be temporary.
//! - [HandleRef<T>] stores a weak reference ([alloc::sync::Weak]) and is suitable for parent
//!   pointers or other non-owning references that must not keep the target alive.
//!
//! Key guarantees and semantics:
//! - Call [Handle::create_ref] to derive a [HandleRef] from an existing strong [Handle].
//! - Call [HandleRef::get_handle] to attempt an upgrade; it returns [None] if the strong owner(s)
//!   have dropped the object. **Consumers must handle the [None] case explicitly.**
use alloc::{sync::Arc, sync::Weak};
use core::ops::Deref;

#[derive(Debug)]
/// Strong owning handle backed by [Arc<T>].
///
/// Use [Handle<T>] when multiple parts of the system need shared ownership of a value.
/// The inner value is reference-counted; cloning the handle increments the count.
/// Use [Handle<T>::create_ref] to produce a weak [HandleRef<T>] suitable for parent pointers.
pub struct Handle<T> {
    inner: Arc<T>,
}

impl<T> Deref for Handle<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &*self.inner
    }
}

impl<T> From<T> for Handle<T> {
    fn from(value: T) -> Self {
        Self {
            inner: Arc::new(value),
        }
    }
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> Handle<T> {
    /// Create a non-owning [HandleRef<T>] that refers to the same underlying object.
    ///
    /// The returned [HandleRef<T>] does not increment the strong reference count and
    /// must be upgraded with [HandleRef::get_handle] before use. Use this to store
    /// parent pointers or other back-references without preventing object drop.
    pub fn create_ref(&self) -> HandleRef<T> {
        HandleRef {
            inner: Arc::downgrade(&self.inner),
        }
    }
}

#[derive(Debug)]
/// Weak (non-owning) handle backed by [Weak<T>].
///
/// A [HandleRef<T>] represents an optional reference to an object which may be destroyed
/// independently of the referrers. Use [HandleRef<T>::get_handle] to attempt to obtain a strong [Handle<T>].
pub struct HandleRef<T> {
    inner: Weak<T>,
}

impl<T> Clone for HandleRef<T>{
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone() }
    }
}

impl<T> HandleRef<T> {
    /// Attempt to upgrade the weak reference into a strong [Handle<T>].
    ///
    /// Return `Some(Handle<T>)` if the target is still alive, otherwise return `None`.
    /// **Always check the result** before dereferencing the returned handle.
    pub fn get_handle(&self) -> Option<Handle<T>> {
        match Weak::upgrade(&self.inner) {
            Some(arc) => Some(Handle { inner: arc }),
            None => None,
        }
    }
}
