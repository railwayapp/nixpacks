use actix_multipart::Multipart;
use actix_web::dev::ServerHandle;
use actix_web::error::ParseError;
use actix_web::{
    middleware, rt, web, App as ActixApp, Error as ActixError, HttpResponse, HttpServer,
};
use futures_util::stream::StreamExt;

use anyhow::Result;
// use actix_web::http::ContentEncoding;
// use actix_web::http::header::ContentDisposition;
// use actix_web::middleware::{Compress, Logger};
// use analysis_engine::enums::Log;
// use analysis_engine::taxonomy;
use flate2::read::GzDecoder;
// use flate2::write::GzEncoder;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::{sync::mpsc, thread, time};
use tar::Archive;

#[derive(Debug, Clone)]
pub struct DockerImageFileReceiver {
    save_to: PathBuf,
    server_handle: Option<ServerHandle>,
}

impl DockerImageFileReceiver {
    pub fn new(save_to: PathBuf) -> DockerImageFileReceiver {
        DockerImageFileReceiver {
            save_to,
            server_handle: None,
        }
    }

    // pub async fn stop(self) {
    //     if self.server_handle.is_some() {
    //         self.server_handle.unwrap().stop(true).await
    //     }
    // }

    pub fn start(self) {
        let (tx, rx) = mpsc::channel();

        println!("spawning thread for server");
        thread::spawn(move || {
            let server_future = DockerImageFileReceiver::run_app(tx, self.save_to);
            rt::System::new().block_on(server_future)
        });

        // let server_handle = rx.recv().unwrap();

        // println!("waiting 10 seconds");
        // thread::sleep(time::Duration::from_secs(10));

        // Send a stop signal to the server, waiting for it to exit gracefully
        // println!("stopping server");
        // rt::System::new().block_on(server_handle.stop(true));
    }

    async fn run_app(tx: mpsc::Sender<ServerHandle>, save_to: PathBuf) -> std::io::Result<()> {
        let save_to_data = web::Data::new(save_to.clone());

        std::env::set_var("RUST_LOG", "info");
        std::env::set_var("RUST_BACKTRACE", "1");

        println!("Starting Nixpacks HTTP server");

        let server = HttpServer::new(move || {
            ActixApp::new()
                .app_data(save_to_data.clone())
                .wrap(middleware::Logger::default())
                .service(web::resource("/upload").route(web::post().to(DockerImageFileReceiver::upload)))
                .service(web::resource("/get").route(web::get().to(|| async { "Hello World!" })))
        })
        .bind(("127.0.0.1", 8080))?
        .run();

        // Send server handle back to the main thread
        // let _ = tx.send(server.handle());

        server.await
    }

    pub async fn start2(mut self) -> std::io::Result<()> {
        std::fs::create_dir_all(self.save_to.clone())?;

        let save_to_data = web::Data::new(self.save_to.clone());
        std::env::set_var("RUST_LOG", "info");
        std::env::set_var("RUST_BACKTRACE", "1");

        let srv = HttpServer::new(move || {
            ActixApp::new()
                .app_data(save_to_data.clone())
                .wrap(middleware::Logger::default())
                .service(web::resource("/").route(web::post().to(DockerImageFileReceiver::upload)))
                .service(web::resource("/get").route(web::get().to(|| async { "Hello World!" })))
        })
        .bind(("127.0.0.1", 8080))?
        .run();

        self.server_handle = Some(srv.handle());

        srv.await
    }

    #[allow(dead_code)]
    async fn upload(
        mut payload: Multipart,
        data: web::Data<PathBuf>,
    ) -> Result<HttpResponse, ActixError> {
        let save_to = data;
        println!("upload 1111111111");

        while let Some(item) = payload.next().await {
            let mut byte_stream_field = item?;
            println!("upload 1");

            let filename = byte_stream_field
                .content_disposition()
                .get_filename()
                .ok_or_else(|| ParseError::Incomplete)?;


            let filepath = save_to.join(sanitize_filename::sanitize(&filename));

            println!("upload filepath {}", filepath.display().to_string());
            // File::create is a blocking operation, so use a thread pool
            let in_path = PathBuf::from(&filepath);
            let mut f: File = web::block(|| File::create(in_path)).await??;
            while let Some(chunk) = byte_stream_field.next().await {
                let data = chunk?;
                // Writing a file is also a blocking operation, so use a thread pool
                f = web::block(move || f.write_all(&data).map(|_| f)).await??;
            }

            web::block(move || f.flush()).await??;

            // DockerImageFileReceiver::decompress_file(filepath)?
        }
        Ok(HttpResponse::Ok().into())

        // Send the output zip file back to the client
    }

    #[allow(dead_code)]
    pub fn decompress_file(filepath: PathBuf) -> Result<(), ActixError> {
        let file = File::open(filepath)?;
        let mut archive = Archive::new(GzDecoder::new(file));
        archive
            .entries()?
            .filter_map(|e| e.ok())
            .map(|mut entry| -> Result<PathBuf> {
                let path = entry.path()?.to_path_buf();
                println!("Ahmed: {}", path.display().to_string());

                entry.unpack(&path)?;
                Ok(path)
            })
            .filter_map(|e| e.ok())
            .for_each(|x| println!("> {}", x.display()));
        Ok(())
    }
}
