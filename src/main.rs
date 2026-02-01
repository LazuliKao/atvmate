use poem::{listener::TcpListener, Route, Server, endpoint::StaticFilesEndpoint};
use poem_openapi::OpenApiService;
use std::sync::Arc;
use atvmate::{global_device_manager::GlobalDeviceManager, web_service::ApiService};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting ATV Web Service...");

    let device_manager = Arc::new(GlobalDeviceManager::new());
    let api_service = ApiService::new(device_manager.clone());

    let api_service = OpenApiService::new(api_service, "ATV Remote Control", "1.0")
        .server("http://127.0.0.1:8000");

    let ui = api_service.swagger_ui();
    let spec = api_service.spec_endpoint();
    let app = Route::new()
        .nest("/api", api_service)
        .nest("/docs", ui)
        .at("/api-docs/openapi.json", spec)
        .nest("/", StaticFilesEndpoint::new("frontend/dist").index_file("index.html"));

    println!("Server running at http://127.0.0.1:8000");
    println!("API documentation available at http://127.0.0.1:8000/docs");

    Server::new(TcpListener::bind("0.0.0.0:8000"))
        .run(app)
        .await?;

    Ok(())
}
