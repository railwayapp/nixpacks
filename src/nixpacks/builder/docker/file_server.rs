// use actix_multipart::Multipart;
use actix_web::http::header::HeaderValue;
use actix_web::{
    middleware, rt, web, App as ActixApp, Error as ActixError, HttpRequest, HttpResponse,
    HttpServer,
};
use futures_util::stream::StreamExt;

use anyhow::Result;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::thread;

#[derive(Debug, Clone)]
pub struct FileServer {
    data: FileServerData,
}

#[derive(Debug, Clone)]
pub struct FileServerData {
    save_to: PathBuf,
    access_token: String,
    host: String,
    port: u16,
}

impl FileServer {
    pub fn new(save_to: PathBuf, access_token: String) -> FileServer {
        FileServer {
            data: FileServerData {
                save_to,
                access_token,
                host: "0.0.0.0".to_string(),
                port: 8080,
            },
        }
    }

    pub fn start(self) {
        thread::spawn(move || {
            let server_future = FileServer::run_app(self.data);
            match rt::System::new().block_on(server_future) {
                Err(e) => println!("File server error: {}", e),
                _ => ()
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
        let mut counter = 0;

        while let Some(chunk) = payload.next().await {
            let data = chunk?;
            f = web::block(move || f.write_all(&data).map(|_| f)).await??;

            counter = counter + 1;
            // println!("Save chunk count: {}", counter)
            // let mut byte_stream_field = item?;

            // let filename = byte_stream_field
            //     .content_disposition()
            //     .get_filename()
            //     .ok_or(ParseError::Incomplete)?;

            // while let Some(chunk) = payload.next().await {
            //     let data = chunk?;
            //     f = web::block(move || f.write_all(&data).map(|_| f)).await??;
            // }
        }
        web::block(move || f.flush()).await??;

        Ok(HttpResponse::Ok().into())
    }
}
