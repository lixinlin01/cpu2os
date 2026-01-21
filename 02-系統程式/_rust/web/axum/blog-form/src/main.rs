use axum::{
    extract::{Path, State, Form},
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::sync::Arc;
use tera::{Context, Tera};
use tower_sessions::{MemoryStore, Session, SessionManagerLayer};

// --- 資料模型 ---
#[derive(Debug, Serialize, sqlx::FromRow)]
struct User {
    id: i32,
    username: String,
    password: String, // 實際開發請務必加密
    email: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct Post {
    id: i32,
    username: String,
    title: String,
    body: String,
}

#[derive(Deserialize)]
struct PostForm {
    title: String,
    body: String,
}

#[derive(Deserialize)]
struct AuthForm {
    username: String,
    password: String,
    email: Option<String>,
}

// --- 全域狀態 ---
struct AppState {
    db: SqlitePool,
    templates: Tera,
}

#[tokio::main]
async fn main() {
    // 1. 初始化資料庫
    let db_url = "sqlite:blog.db";
    let pool = SqlitePoolOptions::new()
        .connect(db_url)
        .await
        .expect("無法連接資料庫");

    // 建立資料表 (對應 FastAPI 的 create_all)
    sqlx::query("CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY, username TEXT UNIQUE, password TEXT, email TEXT)").execute(&pool).await.unwrap();
    sqlx::query("CREATE TABLE IF NOT EXISTS posts (id INTEGER PRIMARY KEY, username TEXT, title TEXT, body TEXT)").execute(&pool).await.unwrap();

    // 2. 初始化模板
    let mut tera = Tera::new("templates/**/*").expect("模板路徑錯誤");
    tera.full_reload().unwrap();

    let app_state = Arc::new(AppState { db: pool, templates: tera });

    // 3. Session 設定
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false); // 開發環境設為 false

    // 4. 定義路由
    let app = Router::new()
        .route("/", get(list_posts))
        .route("/signup", get(signup_ui).post(signup))
        .route("/login", get(login_ui).post(login))
        .route("/logout", get(logout))
        .route("/post/new", get(new_post_ui))
        .route("/post", post(create_post))
        .route("/post/:id", get(show_post))
        .layer(session_layer)
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8000").await.unwrap();
    println!("Server running on http://127.0.0.1:8000");
    axum::serve(listener, app).await.unwrap();
}

// --- 處理函式 (Handlers) ---

async fn list_posts(State(state): State<Arc<AppState>>, session: Session) -> impl IntoResponse {
    let posts = sqlx::query_as::<_, Post>("SELECT * FROM posts").fetch_all(&state.db).await.unwrap();
    let user: Option<serde_json::Value> = session.get("user").await.unwrap();

    let mut ctx = Context::new();
    ctx.insert("posts", &posts);
    ctx.insert("user", &user);

    let rendered = state.templates.render("list.html", &ctx).unwrap();
    Html(rendered)
}

async fn signup_ui(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    Html(state.templates.render("signup.html", &Context::new()).unwrap())
}

async fn signup(State(state): State<Arc<AppState>>, Form(form): Form<AuthForm>) -> impl IntoResponse {
    let res = sqlx::query("INSERT INTO users (username, password, email) VALUES (?, ?, ?)")
        .bind(&form.username)
        .bind(&form.password)
        .bind(&form.email.unwrap_or_default())
        .execute(&state.db)
        .await;

    match res {
        Ok(_) => Redirect::to("/login"),
        Err(_) => Redirect::to("/signup"), // 簡單處理註冊失敗
    }
}

async fn login_ui(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    Html(state.templates.render("login.html", &Context::new()).unwrap())
}

async fn login(State(state): State<Arc<AppState>>, session: Session, Form(form): Form<AuthForm>) -> impl IntoResponse {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = ? AND password = ?")
        .bind(&form.username)
        .bind(&form.password)
        .fetch_optional(&state.db)
        .await
        .unwrap();

    if let Some(u) = user {
        session.insert("user", serde_json::json!({"username": u.username})).await.unwrap();
        Redirect::to("/")
    } else {
        Redirect::to("/login")
    }
}

async fn logout(session: Session) -> impl IntoResponse {
    session.clear().await;
    Redirect::to("/")
}

async fn new_post_ui(State(state): State<Arc<AppState>>, session: Session) -> impl IntoResponse {
    let user: Option<serde_json::Value> = session.get("user").await.unwrap();
    if user.is_none() { return Redirect::to("/login").into_response(); }
    
    Html(state.templates.render("new_post.html", &Context::new()).unwrap()).into_response()
}

async fn create_post(State(state): State<Arc<AppState>>, session: Session, Form(form): Form<PostForm>) -> impl IntoResponse {
    let user: Option<serde_json::Value> = session.get("user").await.unwrap();
    if let Some(u) = user {
        let username = u["username"].as_str().unwrap();
        sqlx::query("INSERT INTO posts (username, title, body) VALUES (?, ?, ?)")
            .bind(username)
            .bind(&form.title)
            .bind(&form.body)
            .execute(&state.db)
            .await
            .unwrap();
        Redirect::to("/")
    } else {
        Redirect::to("/login")
    }
}

async fn show_post(State(state): State<Arc<AppState>>, Path(id): Path<i32>) -> impl IntoResponse {
    let post = sqlx::query_as::<_, Post>("SELECT * FROM posts WHERE id = ?")
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .unwrap();

    if let Some(p) = post {
        let mut ctx = Context::new();
        ctx.insert("post", &p);
        Html(state.templates.render("show_post.html", &ctx).unwrap())
    } else {
        Html("Post Not Found".to_string())
    }
}