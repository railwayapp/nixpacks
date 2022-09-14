use actix_multipart::Multipart;
use actix_web::error::ParseError;
use actix_web::{
    middleware, rt, web, App as ActixApp, Error as ActixError, HttpResponse, HttpServer,
};
use futures_util::stream::StreamExt;

use anyhow::Result;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::thread;

#[derive(Debug, Clone)]
pub struct DockerImageFileReceiver {
    save_to: PathBuf,
}

impl DockerImageFileReceiver {
    pub fn new(save_to: PathBuf) -> DockerImageFileReceiver {
        DockerImageFileReceiver { save_to }
    }

    pub fn start(self) {
        thread::spawn(move || {
            let server_future = DockerImageFileReceiver::run_app(self.save_to);
            rt::System::new().block_on(server_future)
        });
    }

    async fn run_app(save_to: PathBuf) -> std::io::Result<()> {
        let save_to_data = web::Data::new(save_to.clone());

        std::env::set_var("RUST_LOG", "info");
        std::env::set_var("RUST_BACKTRACE", "1");

        let server = HttpServer::new(move || {
            ActixApp::new()
                .app_data(save_to_data.clone())
                .wrap(middleware::Logger::default())
                .service(
                    web::resource("/upload").route(web::post().to(DockerImageFileReceiver::upload)),
                )
        })
        .bind(("127.0.0.1", 8080))?
        .run();

        server.await
    }

    #[allow(dead_code)]
    async fn upload(
        mut payload: Multipart,
        data: web::Data<PathBuf>,
    ) -> Result<HttpResponse, ActixError> {
        let save_to = data;

        while let Some(item) = payload.next().await {
            let mut byte_stream_field = item?;

            let filename = byte_stream_field
                .content_disposition()
                .get_filename()
                .ok_or(ParseError::Incomplete)?;

            let filepath = save_to.join(sanitize_filename::sanitize(&filename));
            let in_path = PathBuf::from(&filepath);
            let mut f: File = web::block(|| File::create(in_path)).await??;
            while let Some(chunk) = byte_stream_field.next().await {
                let data = chunk?;
                f = web::block(move || f.write_all(&data).map(|_| f)).await??;
            }

            web::block(move || f.flush()).await??;
        }
        Ok(HttpResponse::Ok().into())
    }
}
