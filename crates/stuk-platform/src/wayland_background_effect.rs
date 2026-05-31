use std::ffi::{c_char, c_uint, c_void};
use std::ptr;
use std::sync::Arc;

use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};
use winit::window::Window;

use crate::{WindowBackgroundEffect, WindowOptions};

#[cfg(target_os = "linux")]
#[path = "wayland_background_effect_protocol.rs"]
mod wayland_background_effect_protocol;
#[cfg(target_os = "linux")]
use wayland_background_effect_protocol::*;

#[cfg(target_os = "linux")]
pub(crate) fn request(window: &Arc<dyn Window>, options: &WindowOptions) -> Option<WaylandEffect> {
    if !options.transparent || options.background_effect == WindowBackgroundEffect::None {
        debug("skipped: transparent window with a background effect was not requested");
        return None;
    }
    let Some(display) = wayland_display(window) else {
        debug("skipped: native window is not backed by a Wayland display");
        return None;
    };
    let Some(surface) = wayland_surface(window) else {
        debug("skipped: native window is not backed by a Wayland surface");
        return None;
    };
    debug("requesting ext_background_effect_v1");
    unsafe {
        ExtBackgroundEffect::bind(
            display,
            surface,
            options.background_effect,
            options.width as i32,
            options.height as i32,
        )
    }
}

#[cfg(not(target_os = "linux"))]
pub(crate) fn request(
    _window: &Arc<dyn Window>,
    _options: &WindowOptions,
) -> Option<WaylandEffect> {
    None
}

#[derive(Debug)]
pub(crate) struct WaylandEffect {
    effect: *mut WlProxy,
    manager: *mut WlProxy,
    _manager_state: Box<ManagerState>,
}

impl Drop for WaylandEffect {
    fn drop(&mut self) {
        unsafe {
            if !self.effect.is_null() {
                wl_proxy_marshal_flags(self.effect, EFFECT_DESTROY, ptr::null(), 1, DESTROY_FLAG);
                self.effect = ptr::null_mut();
            }
            if !self.manager.is_null() {
                wl_proxy_marshal_flags(self.manager, MANAGER_DESTROY, ptr::null(), 1, DESTROY_FLAG);
                self.manager = ptr::null_mut();
            }
        }
    }
}

#[cfg(target_os = "linux")]
fn wayland_display(window: &Arc<dyn Window>) -> Option<*mut WlDisplay> {
    match window.display_handle().ok()?.as_raw() {
        RawDisplayHandle::Wayland(display) => Some(display.display.as_ptr().cast()),
        _ => None,
    }
}

#[cfg(target_os = "linux")]
fn wayland_surface(window: &Arc<dyn Window>) -> Option<*mut WlProxy> {
    match window.window_handle().ok()?.as_raw() {
        RawWindowHandle::Wayland(surface) => Some(surface.surface.as_ptr().cast()),
        _ => None,
    }
}

#[cfg(target_os = "linux")]
struct ExtBackgroundEffect;

#[cfg(target_os = "linux")]
impl ExtBackgroundEffect {
    unsafe fn bind(
        display: *mut WlDisplay,
        surface: *mut WlProxy,
        _effect: WindowBackgroundEffect,
        width: i32,
        height: i32,
    ) -> Option<WaylandEffect> {
        if display.is_null() || surface.is_null() {
            return None;
        }

        let registry = unsafe {
            wl_proxy_marshal_flags(
                display.cast(),
                DISPLAY_GET_REGISTRY,
                &WL_REGISTRY_INTERFACE,
                wl_proxy_get_version(display.cast()),
                0,
                ptr::null::<c_void>(),
            )
        };
        if registry.is_null() {
            debug("failed to create wl_registry");
            return None;
        }

        let mut state = RegistryState::default();
        let add_listener = unsafe {
            wl_proxy_add_listener(
                registry.cast(),
                &REGISTRY_LISTENER as *const RegistryListener as *mut _,
                &mut state as *mut RegistryState as *mut c_void,
            )
        };
        if add_listener != 0 {
            debug("failed to add wl_registry listener");
            unsafe { wl_proxy_destroy(registry.cast()) };
            return None;
        }

        unsafe {
            wl_display_roundtrip(display);
            wl_display_roundtrip(display);
        }

        let Some(manager_name) = state.manager_name else {
            debug("ext_background_effect_manager_v1 was not advertised");
            unsafe { wl_proxy_destroy(registry.cast()) };
            return None;
        };
        let Some(compositor_name) = state.compositor_name else {
            debug("wl_compositor was not advertised");
            unsafe { wl_proxy_destroy(registry.cast()) };
            return None;
        };

        debug("binding ext_background_effect_manager_v1");
        let manager = unsafe {
            wl_proxy_marshal_flags(
                registry.cast(),
                REGISTRY_BIND,
                &EXT_BACKGROUND_EFFECT_MANAGER_V1_INTERFACE,
                1,
                0,
                manager_name,
                EXT_BACKGROUND_EFFECT_MANAGER_V1_INTERFACE.name,
                1_u32,
                ptr::null::<c_void>(),
            )
        };
        if manager.is_null() {
            debug("failed to bind ext_background_effect_manager_v1");
            unsafe { wl_proxy_destroy(registry.cast()) };
            return None;
        }
        debug("bound ext_background_effect_manager_v1");

        let mut manager_state = Box::<ManagerState>::default();
        let add_manager_listener = unsafe {
            wl_proxy_add_listener(
                manager.cast(),
                &MANAGER_LISTENER as *const ManagerListener as *mut _,
                manager_state.as_mut() as *mut ManagerState as *mut c_void,
            )
        };
        if add_manager_listener != 0 {
            debug("failed to add ext_background_effect_manager_v1 listener");
            unsafe { wl_proxy_destroy(registry.cast()) };
            unsafe { wl_proxy_destroy(manager.cast()) };
            return None;
        }
        unsafe {
            wl_display_roundtrip(display);
        }
        if !manager_state.supports_blur() {
            debug("ext_background_effect_manager_v1 does not advertise blur capability");
            unsafe { wl_proxy_destroy(registry.cast()) };
            unsafe { wl_proxy_destroy(manager.cast()) };
            return None;
        }

        let compositor = unsafe {
            wl_proxy_marshal_flags(
                registry.cast(),
                REGISTRY_BIND,
                &WL_COMPOSITOR_INTERFACE,
                1,
                0,
                compositor_name,
                WL_COMPOSITOR_INTERFACE.name,
                1_u32,
                ptr::null::<c_void>(),
            )
        };
        unsafe { wl_proxy_destroy(registry.cast()) };
        if compositor.is_null() {
            debug("failed to bind wl_compositor");
            unsafe { wl_proxy_destroy(manager.cast()) };
            return None;
        }

        let effect_proxy = unsafe {
            wl_proxy_marshal_flags(
                manager.cast(),
                MANAGER_GET_BACKGROUND_EFFECT,
                &EXT_BACKGROUND_EFFECT_SURFACE_V1_INTERFACE,
                1,
                0,
                ptr::null::<c_void>(),
                surface,
            )
        };
        if effect_proxy.is_null() {
            debug("failed to create ext_background_effect_surface_v1");
            unsafe { wl_proxy_destroy(compositor.cast()) };
            unsafe { wl_proxy_destroy(manager.cast()) };
            return None;
        }
        debug("created ext_background_effect_surface_v1");

        let region = unsafe {
            wl_proxy_marshal_flags(
                compositor,
                COMPOSITOR_CREATE_REGION,
                &WL_REGION_INTERFACE,
                1,
                0,
                ptr::null::<c_void>(),
            )
        };
        if region.is_null() {
            debug("failed to create blur wl_region");
            unsafe { wl_proxy_destroy(effect_proxy.cast()) };
            unsafe { wl_proxy_destroy(compositor.cast()) };
            unsafe { wl_proxy_destroy(manager.cast()) };
            return None;
        }

        unsafe {
            wl_proxy_marshal_flags(
                region,
                REGION_ADD,
                ptr::null(),
                1,
                0,
                0_i32,
                0_i32,
                width,
                height,
            );
            wl_proxy_marshal_flags(
                effect_proxy,
                EFFECT_SET_BLUR_REGION,
                ptr::null(),
                1,
                0,
                region,
            );
            wl_proxy_marshal_flags(region, REGION_DESTROY, ptr::null(), 1, DESTROY_FLAG);
            wl_proxy_destroy(compositor.cast());
            wl_proxy_marshal_flags(
                surface,
                SURFACE_COMMIT,
                ptr::null(),
                wl_proxy_get_version(surface),
                0,
            );
            wl_display_flush(display);
        }
        debug("committed ext_background_effect_surface_v1");

        Some(WaylandEffect {
            effect: effect_proxy,
            manager: manager.cast(),
            _manager_state: manager_state,
        })
    }
}

#[cfg(target_os = "linux")]
fn debug(message: &str) {
    if std::env::var_os("STUK_WAYLAND_EFFECT_DEBUG").is_some() {
        eprintln!("stuk wayland background effect: {message}");
    }
}

#[cfg(target_os = "linux")]
#[derive(Default)]
struct RegistryState {
    manager_name: Option<u32>,
    compositor_name: Option<u32>,
}

#[cfg(target_os = "linux")]
#[derive(Debug, Default)]
struct ManagerState {
    capabilities: u32,
}

#[cfg(target_os = "linux")]
impl ManagerState {
    fn supports_blur(&self) -> bool {
        self.capabilities & MANAGER_CAPABILITY_BLUR != 0
    }
}

#[cfg(target_os = "linux")]
unsafe extern "C" fn registry_global(
    data: *mut c_void,
    _registry: *mut WlRegistry,
    name: u32,
    interface: *const c_char,
    _version: u32,
) {
    if data.is_null() || interface.is_null() {
        return;
    }
    let state = unsafe { &mut *(data.cast::<RegistryState>()) };
    let interface = unsafe { std::ffi::CStr::from_ptr(interface) };
    if interface.to_bytes() == MANAGER_INTERFACE_NAME {
        state.manager_name = Some(name);
    } else if interface.to_bytes() == COMPOSITOR_INTERFACE_NAME {
        state.compositor_name = Some(name);
    }
}

#[cfg(target_os = "linux")]
unsafe extern "C" fn registry_global_remove(
    _data: *mut c_void,
    _registry: *mut WlRegistry,
    _name: u32,
) {
}

#[cfg(target_os = "linux")]
#[repr(C)]
struct RegistryListener {
    global: unsafe extern "C" fn(*mut c_void, *mut WlRegistry, u32, *const c_char, u32),
    global_remove: unsafe extern "C" fn(*mut c_void, *mut WlRegistry, u32),
}

#[cfg(target_os = "linux")]
static REGISTRY_LISTENER: RegistryListener = RegistryListener {
    global: registry_global,
    global_remove: registry_global_remove,
};

#[cfg(target_os = "linux")]
unsafe impl Sync for RegistryListener {}

#[cfg(target_os = "linux")]
unsafe extern "C" fn manager_capabilities(
    data: *mut c_void,
    _manager: *mut WlProxy,
    flags: c_uint,
) {
    if data.is_null() {
        return;
    }
    let state = unsafe { &mut *(data.cast::<ManagerState>()) };
    state.capabilities = flags;
}

#[cfg(target_os = "linux")]
#[repr(C)]
struct ManagerListener {
    capabilities: unsafe extern "C" fn(*mut c_void, *mut WlProxy, c_uint),
}

#[cfg(target_os = "linux")]
static MANAGER_LISTENER: ManagerListener = ManagerListener {
    capabilities: manager_capabilities,
};

#[cfg(target_os = "linux")]
unsafe impl Sync for ManagerListener {}
