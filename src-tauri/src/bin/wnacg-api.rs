use std::{collections::HashMap, net::SocketAddr, path::PathBuf, process::Stdio, sync::Arc};

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, get_service},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
    sync::RwLock,
};
use tower_http::{
    cors::CorsLayer,
    services::{ServeDir, ServeFile},
};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    cli_path: String,
    tasks: Arc<RwLock<HashMap<String, DownloadTask>>>,
    task_store_path: Arc<PathBuf>,
}

#[derive(Debug, Deserialize, Clone)]
struct CommonQuery {
    api_domain: Option<String>,
    proxy: Option<String>,
    download_dir: Option<String>,
    config: Option<String>,
}

#[derive(Debug, Deserialize)]
struct KeywordSearchQuery {
    q: String,
    page: Option<i64>,
    api_domain: Option<String>,
    proxy: Option<String>,
    download_dir: Option<String>,
    config: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TagSearchQuery {
    tag: String,
    page: Option<i64>,
    api_domain: Option<String>,
    proxy: Option<String>,
    download_dir: Option<String>,
    config: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
struct DownloadQuery {
    target: String,
    format: Option<String>,
    img_concurrency: Option<usize>,
    img_interval_sec: Option<u64>,
    use_original_filename: Option<bool>,
    api_domain: Option<String>,
    proxy: Option<String>,
    download_dir: Option<String>,
    config: Option<String>,
}

#[derive(Debug, Serialize)]
struct CommandResponse {
    ok: bool,
    exit_code: i32,
    command: Vec<String>,
    stdout: String,
    stderr: String,
    data: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DownloadTask {
    id: String,
    target: String,
    status: String,
    title: Option<String>,
    cover: Option<String>,
    total_pages: Option<i64>,
    completed_pages: usize,
    error: Option<String>,
    zip_path: Option<String>,
    stdout: Vec<String>,
    stderr: Vec<String>,
    created_at: String,
    updated_at: String,
    finished_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ComicInfo {
    id: i64,
    title: String,
    cover: String,
    image_count: i64,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let host = std::env::var("WNACG_API_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("WNACG_API_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(3000);
    let cli_path = std::env::var("WNACG_CLI_PATH").unwrap_or_else(|_| "wnacg-cli".to_string());
    let web_dist_path = std::env::var("WNACG_WEB_DIST_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/usr/local/share/wnacg-web"));
    let task_store_path = std::env::var("WNACG_TASK_STORE_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/data/wnacg-tasks.json"));
    let tasks = Arc::new(RwLock::new(load_tasks(&task_store_path).await.unwrap_or_default()));

    let api_router = Router::new()
        .route("/health", get(health))
        .route("/search/keyword", get(search_keyword))
        .route("/search/tag", get(search_tag))
        .route("/comic", get(get_comic))
        .route("/download", get(start_download))
        .route("/download/start", get(start_download))
        .route("/tasks", get(list_tasks))
        .route("/tasks/{id}", get(get_task));
    let static_index = web_dist_path.join("index.html");
    let static_service = get_service(
        ServeDir::new(&web_dist_path).not_found_service(ServeFile::new(static_index)),
    );

    let app_state = AppState {
        cli_path,
        tasks,
        task_store_path: Arc::new(task_store_path),
    };
    let app = Router::new()
        .nest("/api", api_router.clone())
        .merge(api_router)
        .fallback_service(static_service)
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    let addr: SocketAddr = format!("{host}:{port}").parse()?;
    println!("wnacg-api listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health() -> impl IntoResponse {
    Json(json!({
        "ok": true,
        "commands": [
            "GET /search/keyword?q=<keyword>&page=1",
            "GET /search/tag?tag=<tag>&page=1",
            "GET /comic?target=<id-or-url>",
            "GET /download?target=<id-or-url>",
            "GET /download/start?target=<id-or-url>",
            "GET /tasks",
            "GET /tasks/<id>"
        ]
    }))
}

async fn search_keyword(
    State(state): State<AppState>,
    Query(query): Query<KeywordSearchQuery>,
) -> impl IntoResponse {
    let mut args = vec![
        "search".to_string(),
        "--json".to_string(),
        "--page".to_string(),
        query.page.unwrap_or(1).to_string(),
        "keyword".to_string(),
        query.q,
    ];
    push_common_args(
        &mut args,
        CommonQuery {
            api_domain: query.api_domain,
            proxy: query.proxy,
            download_dir: query.download_dir,
            config: query.config,
        },
    );
    run_cli_json(&state.cli_path, args).await
}

async fn search_tag(
    State(state): State<AppState>,
    Query(query): Query<TagSearchQuery>,
) -> impl IntoResponse {
    let mut args = vec![
        "search".to_string(),
        "--json".to_string(),
        "--page".to_string(),
        query.page.unwrap_or(1).to_string(),
        "tag".to_string(),
        query.tag,
    ];
    push_common_args(
        &mut args,
        CommonQuery {
            api_domain: query.api_domain,
            proxy: query.proxy,
            download_dir: query.download_dir,
            config: query.config,
        },
    );
    run_cli_json(&state.cli_path, args).await
}

async fn get_comic(State(state): State<AppState>, Query(query): Query<DownloadQuery>) -> impl IntoResponse {
    let mut args = vec!["comic".to_string(), "--json".to_string(), query.target];
    push_common_args(
        &mut args,
        CommonQuery {
            api_domain: query.api_domain,
            proxy: query.proxy,
            download_dir: query.download_dir,
            config: query.config,
        },
    );
    run_cli_json(&state.cli_path, args).await
}

async fn start_download(
    State(state): State<AppState>,
    Query(query): Query<DownloadQuery>,
) -> impl IntoResponse {
    let task_id = Uuid::new_v4().to_string();
    let now = now_string();

    let comic_info = fetch_comic_info(&state.cli_path, &query).await.ok();
    let task = DownloadTask {
        id: task_id.clone(),
        target: query.target.clone(),
        status: "downloading".to_string(),
        title: comic_info.as_ref().map(|comic| comic.title.clone()),
        cover: comic_info.as_ref().map(|comic| comic.cover.clone()),
        total_pages: comic_info.as_ref().map(|comic| comic.image_count),
        completed_pages: 0,
        error: None,
        zip_path: None,
        stdout: Vec::new(),
        stderr: Vec::new(),
        created_at: now.clone(),
        updated_at: now,
        finished_at: None,
    };

    state
        .tasks
        .write()
        .await
        .insert(task_id.clone(), task.clone());
    persist_tasks(&state).await;

    let state_for_task = state.clone();
    tokio::spawn(async move {
        run_download_task(state_for_task, task_id, query).await;
    });

    (StatusCode::OK, Json(task)).into_response()
}

async fn list_tasks(State(state): State<AppState>) -> impl IntoResponse {
    let mut tasks = state.tasks.read().await.values().cloned().collect::<Vec<_>>();
    tasks.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Json(json!({ "tasks": tasks }))
}

async fn get_task(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    match state.tasks.read().await.get(&id).cloned() {
        Some(task) => (StatusCode::OK, Json(json!(task))).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({ "ok": false, "error": format!("task `{id}` not found") })),
        )
            .into_response(),
    }
}

async fn run_download_task(state: AppState, task_id: String, query: DownloadQuery) {
    let args = build_download_args(query);
    let mut command = Command::new(&state.cli_path);
    command
        .args(&args)
        .env("WNACG_CLI_DISABLE_TASK_PROXY", "1")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(err) => {
            update_task(&state, &task_id, |task| {
                task.status = "failed".to_string();
                task.error = Some(format!("启动下载任务失败: {err}"));
                task.finished_at = Some(now_string());
                task.updated_at = now_string();
            })
            .await;
            return;
        }
    };

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    let state_stdout = state.clone();
    let task_stdout = task_id.clone();
    let stdout_handle = tokio::spawn(async move {
        if let Some(stdout) = stdout {
            let mut lines = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                handle_stdout_line(&state_stdout, &task_stdout, &line).await;
            }
        }
    });

    let state_stderr = state.clone();
    let task_stderr = task_id.clone();
    let stderr_handle = tokio::spawn(async move {
        if let Some(stderr) = stderr {
            let mut lines = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                update_task(&state_stderr, &task_stderr, |task| {
                    task.stderr.push(line.clone());
                    task.updated_at = now_string();
                })
                .await;
            }
        }
    });

    let status = child.wait().await.ok();
    let _ = stdout_handle.await;
    let _ = stderr_handle.await;

    update_task(&state, &task_id, |task| {
        task.updated_at = now_string();
        task.finished_at = Some(now_string());

        if status.is_some_and(|status| status.success()) {
            task.status = "success".to_string();
            if let Some(total_pages) = task.total_pages {
                task.completed_pages = total_pages as usize;
            }
            if task.title.is_none() {
                task.title = Some(task.target.clone());
            }
        } else {
            task.status = "failed".to_string();
            if task.error.is_none() {
                let last_error = task
                    .stderr
                    .last()
                    .cloned()
                    .or_else(|| task.stdout.last().cloned())
                    .unwrap_or_else(|| "下载失败".to_string());
                task.error = Some(last_error);
            }
        }
    })
    .await;
}

async fn handle_stdout_line(state: &AppState, task_id: &str, line: &str) {
    update_task(state, task_id, |task| {
        task.stdout.push(line.to_string());
        if let Some((completed, total)) = parse_progress(line) {
            task.completed_pages = completed;
            task.total_pages = Some(total as i64);
        }
        if let Some(zip_path) = line.strip_prefix("zipped to ") {
            task.zip_path = Some(zip_path.trim().to_string());
        }
        if let Some(downloaded_to) = line.strip_prefix("downloaded to ") {
            task.zip_path = Some(downloaded_to.trim().to_string());
        }
        task.updated_at = now_string();
    })
    .await;
}

async fn update_task<F>(state: &AppState, task_id: &str, mut updater: F)
where
    F: FnMut(&mut DownloadTask),
{
    let mut changed = false;
    {
        let mut tasks = state.tasks.write().await;
        if let Some(task) = tasks.get_mut(task_id) {
            updater(task);
            changed = true;
        }
    }
    if changed {
        persist_tasks(state).await;
    }
}

async fn persist_tasks(state: &AppState) {
    let tasks = state.tasks.read().await.clone();
    let path = state.task_store_path.as_ref().clone();
    if let Some(parent) = path.parent() {
        if let Err(err) = tokio::fs::create_dir_all(parent).await {
            eprintln!("failed to create task store dir `{}`: {err}", parent.display());
            return;
        }
    }
    let content = match serde_json::to_vec_pretty(&tasks) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("failed to serialize tasks: {err}");
            return;
        }
    };
    if let Err(err) = tokio::fs::write(&path, content).await {
        eprintln!("failed to persist tasks to `{}`: {err}", path.display());
    }
}

async fn load_tasks(path: &PathBuf) -> anyhow::Result<HashMap<String, DownloadTask>> {
    if !path.exists() {
        return Ok(HashMap::new());
    }
    let content = tokio::fs::read(path).await?;
    Ok(serde_json::from_slice(&content)?)
}

fn parse_progress(line: &str) -> Option<(usize, usize)> {
    let line = line.strip_prefix('[')?;
    let (progress, _) = line.split_once(']')?;
    let (completed, total) = progress.split_once('/')?;
    Some((completed.parse().ok()?, total.parse().ok()?))
}

fn build_download_args(query: DownloadQuery) -> Vec<String> {
    let mut args = vec!["download".to_string(), query.target];
    if let Some(format) = query.format {
        args.push("--format".to_string());
        args.push(format);
    }
    if let Some(img_concurrency) = query.img_concurrency {
        args.push("--img-concurrency".to_string());
        args.push(img_concurrency.to_string());
    }
    if let Some(img_interval_sec) = query.img_interval_sec {
        args.push("--img-interval-sec".to_string());
        args.push(img_interval_sec.to_string());
    }
    if query.use_original_filename.unwrap_or(false) {
        args.push("--use-original-filename".to_string());
    }
    push_common_args(
        &mut args,
        CommonQuery {
            api_domain: query.api_domain,
            proxy: query.proxy,
            download_dir: query.download_dir,
            config: query.config,
        },
    );
    args
}

fn push_common_args(args: &mut Vec<String>, query: CommonQuery) {
    if let Some(api_domain) = query.api_domain {
        args.push("--api-domain".to_string());
        args.push(api_domain);
    }
    if let Some(proxy) = query.proxy {
        args.push("--proxy".to_string());
        args.push(proxy);
    }
    if let Some(download_dir) = query.download_dir {
        args.push("--download-dir".to_string());
        args.push(download_dir);
    }
    if let Some(config) = query.config {
        args.push("--config".to_string());
        args.push(config);
    }
}

async fn fetch_comic_info(cli_path: &str, query: &DownloadQuery) -> anyhow::Result<ComicInfo> {
    let mut args = vec!["comic".to_string(), "--json".to_string(), query.target.clone()];
    push_common_args(
        &mut args,
        CommonQuery {
            api_domain: query.api_domain.clone(),
            proxy: query.proxy.clone(),
            download_dir: query.download_dir.clone(),
            config: query.config.clone(),
        },
    );
    let response = run_cli(cli_path, args).await?;
    if !response.ok {
        anyhow::bail!(response.stderr);
    }
    serde_json::from_str(&response.stdout).map_err(Into::into)
}

async fn run_cli_json(cli_path: &str, args: Vec<String>) -> axum::response::Response {
    let response = run_cli(cli_path, args).await;
    match response {
        Ok(mut payload) => {
            payload.data = serde_json::from_str::<Value>(&payload.stdout).ok();
            let status = if payload.ok {
                StatusCode::OK
            } else {
                StatusCode::BAD_GATEWAY
            };
            (status, Json(payload)).into_response()
        }
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CommandResponse {
                ok: false,
                exit_code: -1,
                command: vec![],
                stdout: String::new(),
                stderr: err.to_string(),
                data: None,
            }),
        )
            .into_response(),
    }
}

async fn run_cli_text(cli_path: &str, args: Vec<String>) -> axum::response::Response {
    let response = run_cli(cli_path, args).await;
    match response {
        Ok(payload) => {
            let status = if payload.ok {
                StatusCode::OK
            } else {
                StatusCode::BAD_GATEWAY
            };
            (status, Json(payload)).into_response()
        }
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CommandResponse {
                ok: false,
                exit_code: -1,
                command: vec![],
                stdout: String::new(),
                stderr: err.to_string(),
                data: None,
            }),
        )
            .into_response(),
    }
}

async fn run_cli(cli_path: &str, args: Vec<String>) -> anyhow::Result<CommandResponse> {
    let output = Command::new(cli_path)
        .args(&args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await?;

    Ok(CommandResponse {
        ok: output.status.success(),
        exit_code: output.status.code().unwrap_or(-1),
        command: std::iter::once(cli_path.to_string())
            .chain(args.iter().cloned())
            .collect(),
        stdout: String::from_utf8_lossy(&output.stdout).trim().to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        data: None,
    })
}

fn now_string() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    now.to_string()
}
