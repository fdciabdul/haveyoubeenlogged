use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use askama::Template;
use glob::glob;
use serde::Deserialize;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::task;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    results: Vec<String>,
    query: String,
    folder_size: String,  
}

async fn index() -> impl Responder {
    let results = vec![];
    let query = "".to_string();
    let folder_size = calculate_folder_size("./DIR").await;
    let folder_size_str = format!("{:.2} GB", folder_size as f64 / 1_073_741_824.0); 
    let html = IndexTemplate { query, results, folder_size: folder_size_str }.render().unwrap();
    HttpResponse::Ok().content_type("text/html").body(html)
}

async fn search(query: web::Form<FormData>) -> impl Responder {
    let results = search_in_files(&query.text).await;
    let folder_size = calculate_folder_size("./DIR").await;
    let folder_size_str = format!("{:.2} GB", folder_size as f64 / 1_073_741_824.0); 
    let html = IndexTemplate { query: query.text.clone(), results, folder_size: folder_size_str }.render().unwrap();
    HttpResponse::Ok().content_type("text/html").body(html)
}

async fn calculate_folder_size(path: &str) -> u64 {
    let mut total_size = 0;
    let entries = glob(&format!("{}/**/*", path)).expect("Failed to read glob pattern");
    for entry in entries {
        if let Ok(path) = entry {
            if path.is_file() {
                if let Ok(metadata) = tokio::fs::metadata(&path).await {
                    total_size += metadata.len();
                }
            }
        }
    }
    total_size
}

async fn search_in_files(query: &str) -> Vec<String> {
    let mut results = vec![];
    let pattern = "./DIR/**/*.txt";
    let mut number = 0; 

    'outer: for entry in glob(&pattern).expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                
                let query_clone = query.to_string(); 
                let task = task::spawn(async move {
                    let mut result_vec = Vec::new();
                    if let Ok(file) = File::open(&path).await {
                        let reader = BufReader::new(file);
                        let mut lines = reader.lines();
                        while let Ok(Some(line)) = lines.next_line().await {
                            if line.contains(&query_clone) {
                                number += 1; 
                                let current_number = number; 
                                result_vec.push(format!("[{}] Found data: {}", current_number, line));
                                if result_vec.len() == 10 {
                                    break; 
                                }
                            }
                        }
                    }
                    result_vec
                });

                if let Ok(mut result) = task.await {
                    results.append(&mut result);
                    if results.len() >= 10 {
                        results.truncate(10); 
                        results.push("Hanya 10 hasil yang di akan ditampilkan untuk sekarang, jika ingin lebih silahkan ikuti link.".to_string());
                        break 'outer; 
                    }
                }
            },
            Err(e) => eprintln!("Error processing file: {:?}", e),
        }
    }

    if results.len() < 1 {
        results.push("Tidak ada data yang ditemukan.".to_string());
    }

    results
}






#[derive(Deserialize)]
struct FormData {
    text: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting server at http://127.0.0.1:8080/");
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .route("/search", web::post().to(search))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
