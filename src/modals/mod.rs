pub mod modal;
mod list_modal;
mod overlay_modal;
mod restricted_modal;
mod platform_detail_modal;
mod settings_modal;
pub use modal::{display_modal, default_modal_action, Modal, ModalSize, ModalContent, ModalResult, render_modal, MessageModalContent};
pub use list_modal::{ListItem, ListModal};
pub use overlay_modal::OverlayModal;
pub use restricted_modal::{SetRestrictedModal, VerifyRestrictedModal};
pub use settings_modal::{SettingsModal, on_settings_close};
pub use platform_detail_modal::{PlatformDetailModal, on_add_platform_close, on_platform_found_close, on_game_found_close, on_update_application_close};
