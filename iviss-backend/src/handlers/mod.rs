pub mod audit;
pub mod auth;
pub mod controls;
pub mod health;
pub mod ocr;
pub mod organizations;
pub mod stats;
pub mod submissions;
pub mod users;
pub mod vehicles;

pub mod list_control {
    pub use super::controls::*;
}

pub mod organization_management {
    pub use super::organizations::*;
}

pub mod pending_submission {
    pub use super::submissions::*;
}

pub mod photo {
    pub use super::ocr::{__path_photo_plate, photo_plate};
}

pub mod scan {
    pub use super::ocr::{__path_scan_plate, scan_plate};
}

pub mod search_vehicle {
    pub use super::vehicles::{
        __path_search_vehicle, __path_search_vehicle_v1, search_vehicle, search_vehicle_v1,
        validate_plate_format,
    };
}

pub mod user_management {
    pub use super::users::{
        __path_delete_user, __path_get_user, __path_list_org_users, __path_list_organizations,
        __path_list_users, __path_provision_org_user, __path_provision_user,
        __path_resend_activation_code, __path_resend_org_admin_password, __path_restart_session,
        __path_terminate_session, __path_update_user, delete_user, get_user, list_org_users,
        list_organizations, list_users, provision_org_user, provision_user, resend_activation_code,
        resend_org_admin_password, restart_session, terminate_session, update_user,
    };
}
