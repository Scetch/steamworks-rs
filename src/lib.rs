#![allow(dead_code)]

extern crate libc;
extern crate steamworks_sys as sys;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate bitflags;

use std::collections::HashMap;
use std::sync::{ Arc, Mutex };

pub use error::SteamError;

use apps::Apps;
use callback::Callback;
use friends::Friends;
use matchmaking::Matchmaking;
use user::User;
use utils::Utils;

mod apps;
mod callback;
mod error;
mod friends;
mod matchmaking;
mod server;
mod user;
mod utils;

#[derive(Default)]
struct Callbacks {
    callbacks: Vec<*mut libc::c_void>,
    call_results: HashMap<sys::SteamAPICall, *mut libc::c_void>,
}

pub struct State {
    shutdown: fn(),
    callbacks: Mutex<Callbacks>,
}

impl Drop for State {
    fn drop(&mut self) {
        unsafe {
            let callbacks = self.callbacks.lock().unwrap();
            
            for cb in &callbacks.callbacks {
                sys::delete_rust_callback(*cb);
            }

            for cb in callbacks.call_results.values() {
                sys::delete_rust_callback(*cb);
            }
        }

        (self.shutdown)();
    }
}

unsafe impl Send for State {}
unsafe impl Sync for State {}

pub struct SteamApi {
    state: Arc<State>,
}

impl SteamApi {
    pub fn init() -> Result<Self, SteamError> {
        unsafe {
            if sys::SteamAPI_Init() == 0 {
                return Err(SteamError::InitFailed);
            }

            fn shutdown() {
                unsafe {
                    sys::SteamAPI_Shutdown();
                }
            }

            Ok(SteamApi {
                state: Arc::new(State {
                    shutdown: shutdown,
                    callbacks: Default::default(),
                })
            })
        }
    }

    /// Runs any currently pending callbacks
    ///
    /// This runs all currently pending callbacks on the current
    /// thread.
    ///
    /// This should be called frequently (e.g. once per a frame)
    /// in order to reduce the latency between recieving events.
    pub fn run_callbacks(&self) {
        unsafe {
            sys::SteamAPI_RunCallbacks();
        }
    }

    /// Registers the passed function as a callback for the
    /// given type.
    ///
    /// The callback will be run on the thread that `run_callbacks`
    /// is called when the event arrives.
    pub fn register_callback<C, F>(&self, f: F)
        where C: Callback,
              F: FnMut(C) + Send + Sync + 'static 
    {
        unsafe {
            callback::register_callback(&self.state, f, false);
        }
    }

    /// Returns an accessor to the steam utils interface
    pub fn utils(&self) -> Utils {
        unsafe {
            let inner = sys::steam_rust_get_utils();
            debug_assert!(!inner.is_null());
            Utils {
                _state: self.state.clone(),
                inner: inner,
            }
        }
    }

    /// Returns an accessor to the steam matchmaking interface
    pub fn matchmaking(&self) -> Matchmaking {
        unsafe {
            let inner = sys::steam_rust_get_matchmaking();
            debug_assert!(!inner.is_null());
            Matchmaking {
                state: self.state.clone(),
                inner: inner,
            }
        }
    }

    /// Returns an accessor to the steam apps interface
    pub fn apps(&self) -> Apps {
        unsafe {
            let inner = sys::steam_rust_get_apps();
            debug_assert!(!inner.is_null());
            Apps {
                _state: self.state.clone(),
                inner: inner,
            }
        }
    }

    /// Returns an accessor to the steam friends interface
    pub fn friends(&self) -> Friends {
        unsafe {
            let inner = sys::steam_rust_get_friends();
            debug_assert!(!inner.is_null());
            Friends {
                state: self.state.clone(),
                inner: inner,
            }
        }
    }

    /// Returns an accessor to the steam user interface
    pub fn user(&self) -> User {
        unsafe {
            let inner = sys::steam_rust_get_user();
            debug_assert!(!inner.is_null());
            User {
                _state: self.state.clone(),
                inner: inner,
            }
        }
    }
}

unsafe impl Send for SteamApi {}
unsafe impl Sync for SteamApi {}

/// A user's steam id
#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct SteamId(pub(crate) u64);

impl SteamId {
    /// Creates a `SteamId` from a raw 64 bit value.
    ///
    /// May be useful for deserializing steam ids from
    /// a network or save format.
    pub fn from_raw(id: u64) -> SteamId {
        SteamId(id)
    }

    /// Returns the raw 64 bit value of the steam id
    ///
    /// May be useful for serializing steam ids over a
    /// network or to a save format.
    pub fn raw(&self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use ::{ SteamApi, SteamId };
    use apps::AppId;
    use friends::{ PersonaStateChange, FriendFlags };
    
    #[test]
    fn basic_test() {
        let api = SteamApi::init().unwrap();

        api.register_callback(|p: PersonaStateChange| {
            println!("Got callback: {:?}", p);
        });

        let utils = api.utils();
        println!("Utils:");
        println!("AppId: {:?}", utils.app_id());
        println!("UI Language: {}", utils.ui_language());

        let apps = api.apps();
        println!("Apps");
        println!("IsInstalled(480): {}", apps.is_app_installed(AppId(480)));
        println!("InstallDir(480): {}", apps.app_install_dir(AppId(480)));
        println!("BuildId: {}", apps.app_build_id());
        println!("AppOwner: {:?}", apps.app_owner());
        println!("Langs: {:?}", apps.available_game_languages());
        println!("Lang: {}", apps.current_game_language());
        println!("Beta: {:?}", apps.current_beta_name());

        let friends = api.friends();
        println!("Friends");
        let list = friends.get_friends(FriendFlags::IMMEDIATE);
        println!("{:?}", list);
        for f in &list {
            println!("Friend: {:?} - {}({:?})", f.id(), f.name(), f.state());
            friends.request_user_information(f.id(), true);
        }
        friends.request_user_information(SteamId(76561198174976054), true);

        for _ in 0 .. 50 {
            api.run_callbacks();
            ::std::thread::sleep(::std::time::Duration::from_millis(100));
        }
    }
}