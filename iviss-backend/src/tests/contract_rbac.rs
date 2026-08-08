use super::stats_handler_tests::{seed_admin_user, seed_org_admin, seed_test_data, setup_test_app};
use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use tower::ServiceExt;

struct RbacCase<'a> {
    name: &'a str,
    path: &'a str,
    token: Option<&'a str>,
    expected_status: StatusCode,
}

async fn get(app: Router, path: &str, token: Option<&str>) -> StatusCode {
    let mut request = Request::builder().uri(path);
    if let Some(token) = token {
        request = request.header("Authorization", format!("Bearer {token}"));
    }

    app.oneshot(request.body(Body::empty()).expect("valid request"))
        .await
        .expect("router must handle request")
        .status()
}

#[tokio::test]
async fn contract_rbac_route_role_matrix() {
    let (app, db, _cache, private_key, _public_key, _pg) = setup_test_app().await;
    let (_admin_id, admin_token) = seed_admin_user(&db, &private_key).await;
    let (_org_id, _org_admin_id, org_admin_token) =
        seed_org_admin(&db, &private_key, "RBAC Organization").await;
    let (_agent_org_id, _agent_id, _device_id, agent_token) =
        seed_test_data(&db, &private_key).await;

    let cases = [
        RbacCase {
            name: "public health endpoint",
            path: "/api/v1/health",
            token: None,
            expected_status: StatusCode::OK,
        },
        RbacCase {
            name: "admin endpoint without credentials",
            path: "/api/v1/admin/stats",
            token: None,
            expected_status: StatusCode::UNAUTHORIZED,
        },
        RbacCase {
            name: "agent on admin endpoint",
            path: "/api/v1/admin/stats",
            token: Some(&agent_token),
            expected_status: StatusCode::FORBIDDEN,
        },
        RbacCase {
            name: "org admin on admin endpoint",
            path: "/api/v1/admin/stats",
            token: Some(&org_admin_token),
            expected_status: StatusCode::OK,
        },
        RbacCase {
            name: "admin on org-admin endpoint",
            path: "/api/v1/org-admin/stats",
            token: Some(&admin_token),
            expected_status: StatusCode::FORBIDDEN,
        },
        RbacCase {
            name: "org admin on org-admin endpoint",
            path: "/api/v1/org-admin/stats",
            token: Some(&org_admin_token),
            expected_status: StatusCode::OK,
        },
        RbacCase {
            name: "protected agent endpoint without credentials",
            path: "/api/v1/controls",
            token: None,
            expected_status: StatusCode::UNAUTHORIZED,
        },
        RbacCase {
            name: "agent on protected endpoint",
            path: "/api/v1/controls",
            token: Some(&agent_token),
            expected_status: StatusCode::OK,
        },
    ];

    for case in cases {
        let status = get(app.clone(), case.path, case.token).await;
        assert_eq!(status, case.expected_status, "{}", case.name);
    }
}
