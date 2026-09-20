//! Test utilities for API endpoint testing.
//!
//! Provides helpers to construct [`GlobalDeviceManager`] instances and poem
//! [`TestClient`]s for integration-style tests that exercise the HTTP layer
//! without requiring real ADB hardware.
//!
//! # Usage
//! ```no_run
//! use atvmate::tests::utils::*;
//! use poem::test::TestClient;
//!
//! #[tokio::main]
//! async fn example() {
//!     let mgr = test_device_manager(vec!["device-1"]);
//!     let client = TestClient::new(api_service(mgr));
//!     let resp = client.get("/devices").send().await;
//!     resp.assert_status_is_ok();
//! }
//! ```

use crate::global_device_manager::GlobalDeviceManager;
use crate::operation_history::OperationHistory;
use crate::web_service::{self, ApiService};
use poem::Route;
use poem_openapi::OpenApiService;
use std::sync::Arc;

/// Build a [`GlobalDeviceManager`] pre-populated with fake server devices.
///
/// Server-type devices are created via `ADBDevice::server()` which stores the
/// identifier but does **not** connect – making them safe for unit / API tests
/// that never invoke `shell_command`.
///
/// # Example
/// ```no_run
/// let mgr = atvmate::tests::utils::test_device_manager(vec!["device-1", "device-2"]);
/// assert_eq!(mgr.list_devices().len(), 2);
/// ```
pub fn test_device_manager(device_ids: Vec<&str>) -> Arc<GlobalDeviceManager> {
    let manager = Arc::new(GlobalDeviceManager::new());
    for id in device_ids {
        let serial = id.to_string();
        // `add_server_device` accepts `None` for the server address – no
        // connection is attempted.
        let _ = manager.add_server_device(None, &serial);
    }
    manager
}

/// Build an empty [`GlobalDeviceManager`] (no devices registered).
pub fn empty_device_manager() -> Arc<GlobalDeviceManager> {
    Arc::new(GlobalDeviceManager::new())
}

/// Create an [`OpenApiService`] wrapping an [`ApiService`] backed by the
/// provided device manager.
///
/// Pass the returned value to [`poem::test::TestClient::new`] to get a test
/// client. We return the service rather than the `TestClient` directly because
/// `TestClient` carries a generic endpoint type that depends on the service.
///
/// # Example
/// ```no_run
/// use atvmate::tests::utils::*;
/// use poem::test::TestClient;
///
/// #[tokio::main]
/// async fn example() {
///     let mgr = test_device_manager(vec!["living-room"]);
///     let client = TestClient::new(api_service(mgr));
///     let resp = client.get("/devices").send().await;
///     resp.assert_status_is_ok();
/// }
/// ```
pub fn api_service(manager: Arc<GlobalDeviceManager>) -> OpenApiService<ApiService, ()> {
    web_service::openapi_service(manager, OperationHistory::new())
}

/// Create the full HTTP app, including raw routes that sit alongside OpenAPI.
pub fn app(manager: Arc<GlobalDeviceManager>) -> Route {
    web_service::app(manager, OperationHistory::new())
}

/// Create an [`OpenApiService`] with an empty device manager (no pre-registered
/// devices). Convenient when the test only cares about add / discover flows.
pub fn api_service_empty() -> OpenApiService<ApiService, ()> {
    let mgr = empty_device_manager();
    api_service(mgr)
}

/// Convenience wrapper: create a service and return both it and a device id
/// for use in assertions.
pub fn api_service_with_one_device() -> (OpenApiService<ApiService, ()>, String) {
    let serial = "test-device-0".to_string();
    let mgr = test_device_manager(vec![&serial]);
    let device_id = format!("server:{}", serial);
    let svc = api_service(mgr);
    (svc, device_id)
}

// -- Unit tests for the utilities themselves ---------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_manager_has_no_devices() {
        let mgr = empty_device_manager();
        assert!(mgr.list_devices().is_empty());
    }

    #[test]
    fn test_manager_contains_expected_ids() {
        let mgr = test_device_manager(vec!["a", "b", "c"]);
        let mut ids = mgr.list_devices();
        ids.sort();
        assert_eq!(ids, vec!["server:a", "server:b", "server:c"]);
    }

    #[test]
    fn test_manager_remove_works() {
        let mgr = test_device_manager(vec!["x"]);
        mgr.remove_device("server:x").unwrap();
        assert!(mgr.list_devices().is_empty());
    }

    #[tokio::test]
    async fn api_service_builds_without_error() {
        let mgr = empty_device_manager();
        // Just verify the service constructs – we don't send requests here.
        let _svc = api_service(mgr);
    }
}
