use axum::{
    routing::{get, post},
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Json, Redirect},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use tower_http::services::ServeDir;

// 定義資料模型 (與 Pydantic 的 Post 相似)
#[derive(Debug, Serialize, Deserialize, Clone)]
struct Post {
    id: Option<usize>,
    title: String,
    body: String,
}

// 使用 Arc<RwLock<...>> 來處理執行緒安全的共享狀態
type SharedState = Arc<RwLock<Vec<Post>>>;

#[tokio::main]
async fn main() {
    // 初始資料
    let posts = Arc::new(RwLock::new(vec![
        Post { id: Some(0), title: "aaa".to_string(), body: "aaaaa".to_string() },
        Post { id: Some(1), title: "bbb".to_string(), body: "bbbbb".to_string() },
    ]));

    let app = Router::new()
    // 1. 處理根目錄跳轉
    .route("/", get(|| async { Redirect::temporary("/static/index.html") }))
    // 2. API 路由
    .route("/list", get(list_posts))
    .route("/post/:id", get(show_post))
    .route("/post", post(create_post))
    // 3. 關鍵修正：將 /static 開頭的請求映射到 static 資料夾
    .nest_service("/static", ServeDir::new("static"))
    .with_state(posts);

    // 啟動伺服器
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8000").await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

// --- Handler 實作 ---

async fn list_posts(axum::extract::State(state): axum::extract::State<SharedState>) -> impl IntoResponse {
    let posts = state.read().unwrap();
    Json(posts.clone())
}

async fn show_post(
    axum::extract::State(state): axum::extract::State<SharedState>,
    Path(id): Path<usize>,
) -> impl IntoResponse {
    let posts = state.read().unwrap();
    
    if id >= posts.len() {
        return Err((StatusCode::NOT_FOUND, "Invalid post id"));
    }
    
    Ok(Json(posts[id].clone()))
}

async fn create_post(
    axum::extract::State(state): axum::extract::State<SharedState>,
    Json(mut new_post): Json<Post>,
) -> impl IntoResponse {
    let mut posts = state.write().unwrap();
    
    new_post.id = Some(posts.len());
    posts.push(new_post.clone());
    
    (StatusCode::CREATED, Json(new_post))
}