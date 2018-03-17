use std::ffi::CStr;
use std::sync::Arc;

use sys;

use ::State;
use apps::AppId;

pub enum NotificationPosition {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

/// Access to the steam utils interface
pub struct Utils {
    pub(crate) _state: Arc<State>,
    pub(crate) inner: *mut sys::ISteamUtils,
}

impl Utils {
    /// Returns whether the user currently has the app with the given
    /// ID currently installed.
    ///
    /// This does not mean the user owns the game.
    pub fn app_id(&self) -> AppId {
        unsafe {
            AppId(sys::SteamAPI_ISteamUtils_GetAppID(self.inner))
        }
    }

    /// Returns the language the steam client is currently
    /// running in.
    ///
    /// Generally you want `Apps::current_game_language` instead of this
    pub fn ui_language(&self) -> String {
        unsafe {
            let lang = sys::SteamAPI_ISteamUtils_GetSteamUILanguage(self.inner);
            let lang = CStr::from_ptr(lang);
            lang.to_string_lossy().into_owned()
        }
    }

    /// Sets the position on the screen where popups from the steam overlay
    /// should appear and display themselves in.
    pub fn set_overlay_notification_position(&self, position: NotificationPosition) {
        unsafe {
            let position = match position {
                NotificationPosition::TopLeft => sys::NotificationPosition::TopLeft,
                NotificationPosition::TopRight => sys::NotificationPosition::TopRight,
                NotificationPosition::BottomLeft => sys::NotificationPosition::BottomLeft,
                NotificationPosition::BottomRight => sys::NotificationPosition::BottomRight,
            };
            sys::SteamAPI_ISteamUtils_SetOverlayNotificationPosition(self.inner, position);
        }
    }
}