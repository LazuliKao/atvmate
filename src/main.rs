use atvmate::{
    global_device_manager::GlobalDeviceManager, operation_history::OperationHistory, web_service,
};
use poem::{Server, endpoint::StaticFilesEndpoint, listener::TcpListener};
use std::{env, net::SocketAddrV4, path::PathBuf, sync::Arc};

#[derive(Debug, PartialEq, Eq)]
struct ServerConfig {
    listen: String,
    port: u16,
    adb_server: Option<SocketAddrV4>,
    frontend_dir: PathBuf,
}

fn parse_args_from(args: impl Iterator<Item = String>) -> Result<ServerConfig, String> {
    let mut config = ServerConfig {
        listen: "0.0.0.0".to_string(),
        port: 8000,
        adb_server: None,
        frontend_dir: PathBuf::from("frontend/dist"),
    };
    let mut args = args;

    while let Some(arg) = args.next() {
        if matches!(arg.as_str(), "--help" | "-h") {
            return Err(
                "Usage: atvmate [--listen ADDRESS] [--port PORT] [--adb-server HOST:PORT] [--frontend-dir PATH]"
                    .to_string(),
            );
        }
        let value = args
            .next()
            .ok_or_else(|| format!("Missing value for {arg}"))?;
        match arg.as_str() {
            "--listen" => config.listen = value,
            "--port" => {
                config.port = value
                    .parse()
                    .map_err(|_| format!("Invalid port: {value}"))?;
            }
            "--adb-server" => {
                config.adb_server = Some(
                    value
                        .parse()
                        .map_err(|_| format!("Invalid ADB server address: {value}"))?,
                );
            }
            "--frontend-dir" => config.frontend_dir = PathBuf::from(value),
            _ => return Err(format!("Unknown argument: {arg}")),
        }
    }

    Ok(config)
}

fn parse_args() -> Result<ServerConfig, String> {
    parse_args_from(env::args().skip(1))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = parse_args().map_err(std::io::Error::other)?;
    let address = format!("{}:{}", config.listen, config.port);
    println!("Starting ATV Web Service...");

    let device_manager = Arc::new(GlobalDeviceManager::with_server_addr(config.adb_server));
    let app = web_service::app(device_manager.clone(), OperationHistory::new()).nest(
        "/",
        StaticFilesEndpoint::new(&config.frontend_dir).index_file("index.html"),
    );

    println!("Server running at http://{address}");
    println!("API documentation available at http://{address}/docs");

    Server::new(TcpListener::bind(address)).run(app).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{ServerConfig, parse_args_from};
    use std::path::PathBuf;

    #[test]
    fn parse_args_accepts_openwrt_runtime_options() {
        let config = parse_args_from(
            [
                "--listen",
                "192.168.1.1",
                "--port",
                "8080",
                "--adb-server",
                "127.0.0.1:5038",
                "--frontend-dir",
                "/usr/share/atvmate/frontend",
            ]
            .into_iter()
            .map(str::to_string),
        )
        .unwrap();

        assert_eq!(
            config,
            ServerConfig {
                listen: "192.168.1.1".to_string(),
                port: 8080,
                adb_server: Some("127.0.0.1:5038".parse().unwrap()),
                frontend_dir: PathBuf::from("/usr/share/atvmate/frontend"),
            }
        );
    }

    #[test]
    fn parse_args_rejects_unknown_options() {
        let error =
            parse_args_from(["--unknown", "value"].into_iter().map(str::to_string)).unwrap_err();

        assert_eq!(error, "Unknown argument: --unknown");
    }
}
