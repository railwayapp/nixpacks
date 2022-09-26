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

use super::incremental_cache::IncrementalCacheConfig;

#[derive(Debug, Clone)]
pub struct FileServer {}

#[derive(Debug, Clone)]
pub struct FileServerData {
    save_to: PathBuf,
    access_token: String,
    host: String,
    port: u16,
}

impl FileServer {
    pub fn start(self, incremental_cache_config: &IncrementalCacheConfig) {
        let port: u16 = pick_unused_port().expect("No ports available");
        let data = FileServerData {
            save_to: incremental_cache_config.uploads_dir.clone(),
            access_token: incremental_cache_config.upload_server_access_token.clone(),
            host: "0.0.0.0".to_string(),
            port,
        };

        thread::spawn(move || {
            println!("Nixpacks Web server running at {}:{}", data.host, data.port);

            let server_future = FileServer::run_app(data);
            if let Err(e) = rt::System::new().block_on(server_future) {
                println!("File server error: {}", e);
            }
        });
    }

    async fn run_app(data: FileServerData) -> std::io::Result<()> {
        let save_to_data = web::Data::new(data.clone());
        let server = HttpServer::new(move || {
            ActixApp::new()
                .app_data(save_to_data.clone())
                .wrap(middleware::Logger::default())
                .service(
                    web::resource("/health")
                        .route(web::get().to(|| async { "Nixpacks HTTP server is up & running!" })),
                )
                .service(
                    web::resource("/upload/{filename}").route(web::put().to(FileServer::upload)),
                )
        })
        .bind((data.host, data.port))?
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
        data: web::Data<FileServerData>,
    ) -> Result<HttpResponse, ActixError> {
        if !FileServer::has_valid_access_token(req.headers().get("t"), &data.access_token) {
            return Ok(HttpResponse::Unauthorized().into());
        }

        let filename = path.into_inner();
        let filepath = data.save_to.join(sanitize_filename::sanitize(&filename));

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
