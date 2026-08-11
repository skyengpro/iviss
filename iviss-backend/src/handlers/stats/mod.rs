mod admin;
mod agent;
mod org;
pub mod router;

pub use admin::__path_get_dashboard_stats;
pub use admin::get_dashboard_stats;
pub use agent::{
    __path_get_activity_feed, __path_get_control_activity, __path_get_recent_alerts,
    __path_get_top_agents,
};
pub use agent::{get_activity_feed, get_control_activity, get_recent_alerts, get_top_agents};
pub use org::{
    __path_get_org_activity_feed, __path_get_org_control_activity, __path_get_org_dashboard_stats,
    __path_get_org_recent_alerts, __path_get_org_top_agents,
};
pub use org::{
    get_org_activity_feed, get_org_control_activity, get_org_dashboard_stats,
    get_org_recent_alerts, get_org_top_agents,
};
