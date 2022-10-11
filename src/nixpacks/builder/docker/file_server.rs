// use actix_multipart::Multipart;
use actix_web::http::header::HeaderValue;
use actix_web::{
    middleware, rt, web, App as ActixApp, Error as ActixError, HttpRequest, HttpResponse,
    HttpServer,
};
use anyhow::Result;
use futures_util::stream::StreamExt;
use portpicker::pick_unused_port;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::thread;

use super::incremental_cache::IncrementalCacheDirs;
use uuid::Uuid;

const NIXPACKS_SERVER_HOST: &str = "host.docker.internal";
const NIXPACKS_SERVER_LISTEN_TO_IP: &str = "0.0.0.0";

#[derive(Debug, Clone)]
pub struct FileServer {}

#[derive(Debug, Clone, Default)]
pub struct FileServerConfig {
    pub listen_to_ip: String,
    pub port: u16,
    pub access_token: String,
    pub upload_url: String,
    pub files_dir: PathBuf,
}

impl FileServer {
    pub fn start(self, incremental_cache_dirs: &IncrementalCacheDirs) -> FileServerConfig {
        let port = self.get_free_port();

        let config = FileServerConfig {
            files_dir: incremental_cache_dirs.uploads_dir.clone(),
            access_token: Uuid::new_v4().to_string(),
            listen_to_ip: NIXPACKS_SERVER_LISTEN_TO_IP.to_string(),
            port,
            upload_url: format!("http://{}:{}/upload/", NIXPACKS_SERVER_HOST, port),
        };

        let server_config = config.clone();
        thread::spawn(move || {
            let server_future = FileServer::run_app(server_config);
            if let Err(e) = rt::System::new().block_on(server_future) {
                println!("File server error: {}", e);
            }
        });

        config
    }

    fn get_free_port(&self) -> u16 {
        for _ in 1..3 {
            // try 2 times
            if let Some(port) = pick_unused_port() {
                return port;
            }
        }

        // last try to get free port then fail if no ports available
        pick_unused_port().expect("No ports available")
    }

    async fn run_app(data: FileServerConfig) -> std::io::Result<()> {
        let server_config = web::Data::new(data.clone());
        let server = HttpServer::new(move || {
            ActixApp::new()
                .app_data(server_config.clone())
                .wrap(middleware::Logger::default())
                .service(
                    web::resource("/health")
                        .route(web::get().to(|| async { "Nixpacks HTTP server is up & running!" })),
                )
                .service(
                    web::resource("/upload/{filename}").route(web::put().to(FileServer::upload)),
                )
        })
        .bind((data.listen_to_ip, data.port))?
        .run();

        server.await
    }

    fn has_valid_access_token(token: Option<&HeaderValue>, access_token: &str) -> bool {
        if let Some(header) = token {
            match header.to_str() {
                Ok(value) => value == access_token,
                _ => false,
            }
        } else {
            false
        }
    }

    #[allow(dead_code)]
    async fn upload(
        mut payload: web::Payload,
        path: web::Path<String>,
        req: HttpRequest,
        data: web::Data<FileServerConfig>,
    ) -> Result<HttpResponse, ActixError> {
        if !FileServer::has_valid_access_token(req.headers().get("t"), &data.access_token) {
            return Ok(HttpResponse::Unauthorized().into());
        }

        let filename = path.into_inner();
        let filepath = data.files_dir.join(sanitize_filename::sanitize(&filename));

        let in_path = PathBuf::from(&filepath);
        let mut f: File = web::block(|| File::create(in_path)).await??;

        while let Some(chunk) = payload.next().await {
            let data = chunk?;
            f = web::block(move || f.write_all(&data).map(|_| f)).await??;
        }
        web::block(move || f.flush()).await??;

        Ok(HttpResponse::Ok().into())
    }
}
