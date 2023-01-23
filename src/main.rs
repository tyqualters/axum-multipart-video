use std::{fs::{File}, net::SocketAddr, path::PathBuf, io::{Write, Read}};

use axum::{Router, routing::{get, post}, http::{StatusCode, header, HeaderValue}, extract::{Path, DefaultBodyLimit, Multipart}, response::{IntoResponse, Response}, body::{self, Empty, Full, Bytes}};

use include_dir::{include_dir, Dir};

use axum_client_ip::ClientIp;

use nanoid::nanoid;

use substring::Substring;

static STATIC_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/public_html");
static UPLOAD_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/uploads");

static GIBIBYTE: usize = 1_073_741_824; // Bytes

fn generate_video_name() -> String {
    let mut video_name: String;
    loop {
        video_name = nanoid!() + ".mp4";
        if validate_video_not_exists(&video_name) {
            break;
        }
    }
    video_name
}

fn validate_video_not_exists(filename: &String) -> bool {
    match UPLOAD_DIR.get_file(filename) {
        None => true,
        Some(_) => false,
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app_upload: Router = Router::new()
        .route("/", post(upload))
        .layer(DefaultBodyLimit::max(GIBIBYTE));

    let app: Router = Router::new()
        .route("/", get(handle_static))
        .route("/*path", get(handle_static))
        .route("/video/*path", get(handle_static_videos))
        .nest_service("/upload", app_upload)
        .fallback(send_404);

    let port: u16 = 3344;

    let address = std::net::SocketAddr::from(([0, 0, 0, 0], port));

    println!("Hosting on http://0.0.0.0:{}", port);

    axum::Server::bind(&address)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .expect("Failed to bind server to address");

    Ok(())
}

pub async fn handle_static(ClientIp(client): ClientIp, path: Option<Path<String>>) -> impl IntoResponse {
    let path = path.unwrap_or(Path(String::from("index.html"))).trim_start_matches('/').to_string();

    let mime_type = mime_guess::from_path(path.clone()).first_or_text_plain();

    println!("Serving {} public_html/{} (MIME type: {})", client.to_string(), path, mime_type.to_string());

    match STATIC_DIR.get_file(path.clone()) {
        None => {
            println!("Requested resource public_html/{} not found", path);
            Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(body::boxed(Empty::new()))
            .unwrap()
        },
        Some(file) => Response::builder()
            .status(StatusCode::OK)
            .header(
                header::CONTENT_TYPE,
                HeaderValue::from_str(mime_type.as_ref()).unwrap(),
            )
            .body(body::boxed(Full::from(file.contents())))
            .unwrap(),
    }
}

pub async fn handle_static_videos(ClientIp(client): ClientIp, path: Option<Path<String>>) -> impl IntoResponse {
    let path = path.unwrap_or(Path(String::from("not_exists.mp4"))).trim_start_matches('/').to_string();

    let mime_type = mime_guess::from_path(path.clone()).first_or_text_plain();

    println!("Serving {} uploads/{} (MIME type: {})", client.to_string(), path, mime_type.to_string());

    // Axum / include_dir cache the file contents, so while this would otherwise be correct..
    //      this is the incorrect approach for a dynamic directory.
    //
    //
    // match UPLOAD_DIR.get_file(path.clone()) {
    //     None => {
    //         println!("Requested resource upload/{} not found", path);
    //         Response::builder()
    //         .status(StatusCode::NOT_FOUND)
    //         .body(body::boxed(Empty::new()))
    //         .unwrap()
    //     },
    //     Some(file) => Response::builder()
    //         .status(StatusCode::OK)
    //         .header(
    //             header::CONTENT_TYPE,
    //             HeaderValue::from_str(mime_type.as_ref()).unwrap(),
    //         )
    //         .body(body::boxed(Full::from(file.contents())))
    //         .unwrap(),
    // }

    let cargo_manifest_dir: String = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    let video_path = PathBuf::from(cargo_manifest_dir).join("uploads").join(path.clone());
    let video_path: &std::path::Path = &video_path.as_path();

    match video_path.exists() {
        true => {
            let mut file_contents = Vec::new();
            let mut file = File::open(&video_path).expect("Unable to open video file");
            file.read_to_end(&mut file_contents).expect("Unable to read video file");
            return Response::builder()
                .status(StatusCode::OK)
                .header(
                    header::CONTENT_TYPE,
                    HeaderValue::from_str(mime_type.as_ref()).unwrap(),
                )
                .body(body::boxed(Full::from(file_contents)))
                .unwrap()
        },
        false => {
            println!("Requested resource upload/{} not found", path);
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(body::boxed("Video does not exist.".to_owned()))
                .unwrap()
        },
    }

}

async fn upload(mut multipart: Multipart) -> impl IntoResponse {

    let cargo_manifest_dir: String = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let mut video_name: Option<String> = Option::None;

    while let Some(field) = multipart.next_field().await.unwrap() {
        let name: String = field.name().unwrap().to_string();
        if name == "file" {
            let data: Bytes = field.bytes().await.unwrap();
            println!("File received with size of {} bytes", data.len());
            video_name = Some(generate_video_name());
            let mut file: File = File::create(PathBuf::from(cargo_manifest_dir).join("uploads").join(video_name.clone().unwrap())).unwrap();
            match file.write_all(&data) {
                Ok(_) => {
                    println!("File uploaded successfully.");
                },
                Err(err) => {
                    eprintln!("{}", err);
                    return Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(body::boxed("Failed to save video to disk.".to_owned()))
                    .unwrap()
                },
            };
            let _ = file.flush();
            let _ = file.sync_all();
            break;
        }
    }

    let video_name: String = video_name.unwrap_or(String::from("[[Something went wrong]]"));

    Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(body::boxed(format!("Video uploaded successfully: ?watch={}", video_name.substring(0, video_name.len() - 4))))
            .unwrap()
}

async fn send_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "404 Not Found")
}