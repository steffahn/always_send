#![no_std]

#![cfg_attr(docsrs, feature(doc_cfg))]

//! [![crates.io]](https://crates.io/crates/always_send)
//! [![github]](https://github.com/steffahn/always_send)
//! [![MIT / Apache 2.0 licensed]](https://github.com/steffahn/always_send#License)
//!
//! Wrapper type to check `Send` only on construction, so `rustc` isn’t confused.
//!
//! For more context, please refer to the documentation of [`always_send::AlwaysSend`][AlwaysSend].
//!
//! [github]: https://img.shields.io/badge/github-steffahn/always__send-yellowgreen.svg
//! [crates.io]: https://img.shields.io/crates/v/always_send.svg
//! [MIT / Apache 2.0 licensed]: https://img.shields.io/crates/l/always_send.svg
//! [docs.rs]: https://docs.rs/always_send/badge.svg
//! [unsafe forbidden]: https://img.shields.io/badge/unsafe-forbidden-success.svg

mod safe {
    use core::marker::PhantomData;
    use core::pin::Pin;

    /// Transparent wrapper type around some [`Send`] contents.
    ///
    /// This type only requires `T: Send` on construction, so it cannot
    /// safely be instantiated for non-`Send` inner types `T`.
    ///
    /// This then allows it to implement an unconditional implementation
    /// for `AlwaysSend<T>: Send` itself. This can be very useful to work around
    /// certain cases of compiler-limitations where the attempt of tracking
    /// the `Send` auto trait fails with surprising error messages such as
    /// ```plain
    /// error: implementation of `FnOnce` is not general enough
    ///   --> src/main.rs:…:…
    ///    |
    ///  … |     tokio::spawn(async move {
    ///    |     ^^^^^^^^^^^^ implementation of `FnOnce` is not general enough
    ///    |
    ///    = note: closure with signature `fn(&'0 …) -> …` must implement `FnOnce<(&…,)>`, for any lifetime `'0`...
    ///    = note: ...but it actually implements `FnOnce<(&…,)>`
    /// ```
    /// appearing when calling `tokio::spawn` on an (otherwise functional) future.
    ///
    /// Compare for example [rust-lang/rust#89976](https://github.com/rust-lang/rust/issues/89976)
    /// and some Rust forum threads \[[(1)](https://users.rust-lang.org/t/buffer-unordered-non-send-when-used-with-references-and-closures/122354?u=steffahn),
    /// [(2)](https://users.rust-lang.org/t/implementation-of-trait-is-not-general-enough-when-used-inside-tokio-spawn/122490?u=steffahn)]
    /// for more conrete examples.
    ///
    /// A known possible workaround was to convert a relevant problematic future (or stream)
    /// into a boxed, type-erased version of itself. The future (or stream) that works
    /// is typically one close to where the closure (or other kind of value) -- which
    /// the trait bound (e.g. `FnOnce`) that the compiler error complained about belongs to -- was
    /// wrapped up into a future (or stream).
    ///
    /// This workaround is easiest with the extensions traits from the `futures` crate,
    /// because you just add a call to `.boxed()` in the right place, producing
    /// a `Pin<Box<dyn Future<…> + Send>>` (or `Stream`) without much additional typing.
    ///
    /// It turns out: The only actually relevant property
    /// of these boxed, type-erased futures/streams --
    /// which made them an effective workaround for this compiler bug & error --
    /// is that they implement `Send` unconditionally.
    ///
    /// This crate offers a similarly convenient API through its own extension traits
    /// `FutureExt` and `StreamExt` (the latter requires the `stream` feature).
    /// So just adding some call(s) to [`.always_send()`][super::FutureExt::always_send]
    /// in the right place(s) might solve your issue ;-)
    ///
    /// Note that this struct features an *invariant* type parameter `T`,
    /// so that subtyping coercions can not later invalidate the `T: Send` check
    /// from when the wrapped value was constructed.
    #[repr(transparent)]
    pub struct AlwaysSend<T> {
        /// The inner value is publicly accessible, and there is no [`Drop`] implementation
        /// so you can have full access to it.
        ///
        /// For this reasons, we also don't provides any getter methods, or `.into_inner()`.
        pub inner: T,
        marker: PhantomData<fn() -> *mut T>,
    }

    /// This is the main feature, an implementation of `Send` *without* reqiring `T: Send`.
    unsafe impl<T> Send for AlwaysSend<T> {}

    /// This wrapper offers structural pinning of the [`inner`][AlwaysSend::inner] field.
    impl<T: Unpin> Unpin for AlwaysSend<T> {}

    // this is because all constructors do require `T: Send`
    // and invariance ensures there is no way in which
    // the contained type `T` could change later
    impl<T: Send> AlwaysSend<T> {
        /// Wraps sendable type in the [`AlwaysSend<T>`] wrapper.
        pub fn new(inner: T) -> Self {
            Self {
                inner,
                marker: PhantomData,
            }
        }

        /// Wrap as `AlwaysSend` behind a reference.
        ///
        /// To go the other way, from `wrapped: &Always<T>` to `&T`,
        /// just access `&wrapped.inner`.
        pub fn from_ref(r: &T) -> &Self {
            // SAFETY: #[repr(transparent)]
            unsafe { &*(r as *const T as *const Self) }
        }

        /// Wrap as `AlwaysSend` behind a mutable reference.
        ///
        /// To go the other way, from `wrapped: &mut Always<T>` to `&mut T`,
        /// just access `&mut wrapped.inner`.
        pub fn from_mut(r: &mut T) -> &mut Self {
            // SAFETY: #[repr(transparent)]
            unsafe { &mut *(r as *mut T as *mut Self) }
        }

        /// Wrap as `AlwaysSend` behind a pinned immutable reference.
        ///
        /// To go the other way, see [`.inner_pin()`][Self::inner_pin].
        pub fn from_pin_ref(r: Pin<&T>) -> Pin<&Self> {
            // SAFETY: field is structurally pinned
            unsafe { r.map_unchecked(Self::from_ref) }
        }

        /// Wrap as `AlwaysSend` behind a pinned mutable reference.
        ///
        /// To go the other way, see [`.inner_pin_mut()`][Self::inner_pin_mut].
        pub fn from_pin_mut(r: Pin<&mut T>) -> Pin<&mut Self> {
            // SAFETY: field is structurally pinned
            unsafe { r.map_unchecked_mut(Self::from_mut) }
        }
    }
    impl<T> AlwaysSend<T> {
        /// Pinned access to <code>self.[inner][Self::inner]</code>.
        pub fn inner_pin(self: Pin<&Self>) -> Pin<&T> {
            // SAFETY: field is structurally pinned
            unsafe { self.map_unchecked(|this| &this.inner) }
        }

        /// Pinned mutable access to <code>self.[inner][Self::inner]</code>.
        pub fn inner_pin_mut(self: Pin<&mut Self>) -> Pin<&mut T> {
            // SAFETY: field is structurally pinned
            unsafe { self.map_unchecked_mut(|this| &mut this.inner) }
        }
    }
}
pub use safe::AlwaysSend;

// the below impls need no access to the implementation details, so
// we lifted them outside of the module
use core::future::Future;
use core::pin::Pin;

impl<T: Send> From<T> for AlwaysSend<T> {
    /// Wraps sendable type in the [`AlwaysSend<T>`] wrapper,
    /// like [`AlwaysSend::new`].
    fn from(value: T) -> AlwaysSend<T> {
        AlwaysSend::new(value)
    }
}

// Future, straightforward delegation
impl<F: Future> Future for AlwaysSend<F> {
    type Output = F::Output;

    fn poll(
        self: Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        self.inner_pin_mut().poll(cx)
    }
}

// stream behind an optional feature, since it's another dependency

#[cfg(feature = "stream")]
use futures_core::{Stream, FusedStream, FusedFuture};

#[cfg(feature = "stream")]
#[cfg_attr(docsrs, doc(cfg(feature = "stream")))]
impl<S: Stream> Stream for AlwaysSend<S> {
    type Item = S::Item;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Option<Self::Item>> {
        self.inner_pin_mut().poll_next(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

#[cfg(feature = "stream")]
#[cfg_attr(docsrs, doc(cfg(feature = "stream")))]
impl<S: FusedStream> FusedStream for AlwaysSend<S> {
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}

#[cfg(feature = "stream")]
#[cfg_attr(docsrs, doc(cfg(feature = "stream")))]
impl<F: FusedFuture> FusedFuture for AlwaysSend<F> {
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}

/// Convenience extension trait for easy construction
/// of the [`AlwaysSend`] wrapper for futures
/// in method chains.
pub trait FutureExt: Future + Send + Sized {
    fn always_send(self) -> AlwaysSend<Self> {
        AlwaysSend::new(self)
    }
}

impl<F: Future + Send> FutureExt for F {}

#[cfg(feature = "stream")]
#[cfg_attr(docsrs, doc(cfg(feature = "stream")))]
/// Convenience extension trait for easy construction
/// of the [`AlwaysSend`] wrapper for streams
/// in method chains.
pub trait StreamExt: Stream + Send + Sized {
    fn always_send(self) -> AlwaysSend<Self> {
        AlwaysSend::new(self)
    }
}

#[cfg(feature = "stream")]
#[cfg_attr(docsrs, doc(cfg(feature = "stream")))]
impl<S: Stream + Send> StreamExt for S {}

