// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file).

use accesskit::{ActionHandler, ActionRequest, TreeUpdate};
#[cfg(any(
    all(
        feature = "accesskit_unix",
        any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        )
    ),
    target_os = "windows"
))]
use std::sync::{Mutex, MutexGuard};
use winit::{
    event::WindowEvent,
    event_loop::EventLoopProxy,
    window::{Window, WindowId},
};

mod platform_impl;

#[derive(Debug)]
pub struct ActionRequestEvent {
    pub window_id: WindowId,
    pub request: ActionRequest,
}

struct WinitActionHandler<T: From<ActionRequestEvent> + Send + 'static> {
    window_id: WindowId,
    #[cfg(any(
        all(
            feature = "accesskit_unix",
            any(
                target_os = "linux",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd"
            )
        ),
        target_os = "windows"
    ))]
    proxy: Mutex<EventLoopProxy<T>>,
    #[cfg(not(any(
        all(
            feature = "accesskit_unix",
            any(
                target_os = "linux",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd"
            )
        ),
        target_os = "windows"
    )))]
    proxy: EventLoopProxy<T>,
}

impl<T: From<ActionRequestEvent> + Send + 'static> WinitActionHandler<T> {
    #[cfg(any(
        all(
            feature = "accesskit_unix",
            any(
                target_os = "linux",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd"
            )
        ),
        target_os = "windows"
    ))]
    fn new(window_id: WindowId, proxy: EventLoopProxy<T>) -> Self {
        Self {
            window_id,
            proxy: Mutex::new(proxy),
        }
    }
    #[cfg(not(any(
        all(
            feature = "accesskit_unix",
            any(
                target_os = "linux",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd"
            )
        ),
        target_os = "windows"
    )))]
    fn new(window_id: WindowId, proxy: EventLoopProxy<T>) -> Self {
        Self { window_id, proxy }
    }

    #[cfg(any(
        all(
            feature = "accesskit_unix",
            any(
                target_os = "linux",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd"
            )
        ),
        target_os = "windows"
    ))]
    fn proxy(&self) -> MutexGuard<'_, EventLoopProxy<T>> {
        self.proxy.lock().unwrap()
    }
    #[cfg(not(any(
        all(
            feature = "accesskit_unix",
            any(
                target_os = "linux",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd"
            )
        ),
        target_os = "windows"
    )))]
    fn proxy(&self) -> &EventLoopProxy<T> {
        &self.proxy
    }
}

impl<T: From<ActionRequestEvent> + Send + 'static> ActionHandler for WinitActionHandler<T> {
    fn do_action(&self, request: ActionRequest) {
        let event = ActionRequestEvent {
            window_id: self.window_id,
            request,
        };
        self.proxy().send_event(event.into()).ok();
    }
}

pub struct Adapter {
    adapter: platform_impl::Adapter,
}

impl Adapter {
    /// Creates a new AccessKit adapter for a winit window. This must be done
    /// before the window is shown for the first time. This means that you must
    /// use [`winit::window::WindowBuilder::with_visible`] to make the window
    /// initially invisible, then create the adapter, then show the window.
    pub fn new<T: From<ActionRequestEvent> + Send + 'static>(
        window: &Window,
        source: impl 'static + FnOnce() -> TreeUpdate + Send,
        event_loop_proxy: EventLoopProxy<T>,
    ) -> Self {
        let action_handler = WinitActionHandler::new(window.id(), event_loop_proxy);
        Self::with_action_handler(window, source, Box::new(action_handler))
    }

    /// Creates a new AccessKit adapter for a winit window. This must be done
    /// before the window is shown for the first time. This means that you must
    /// use [`winit::window::WindowBuilder::with_visible`] to make the window
    /// initially invisible, then create the adapter, then show the window.
    ///
    /// Use this if you need to provide your own AccessKit action handler
    /// rather than dispatching action requests through the winit event loop.
    /// Remember that an AccessKit action handler can be called on any thread,
    /// depending on the underlying AccessKit platform adapter.
    pub fn with_action_handler(
        window: &Window,
        source: impl 'static + FnOnce() -> TreeUpdate + Send,
        action_handler: platform_impl::ActionHandlerBox,
    ) -> Self {
        let adapter = platform_impl::Adapter::new(window, source, action_handler);
        Self { adapter }
    }

    #[must_use]
    pub fn on_event(&self, window: &Window, event: &WindowEvent) -> bool {
        self.adapter.on_event(window, event)
    }

    pub fn update(&self, update: TreeUpdate) {
        self.adapter.update(update)
    }

    pub fn update_if_active(&self, updater: impl FnOnce() -> TreeUpdate) {
        self.adapter.update_if_active(updater)
    }
}
