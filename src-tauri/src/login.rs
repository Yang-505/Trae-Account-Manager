use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
use tokio::sync::{oneshot, Mutex};
use warp::Filter;

use crate::account::AccountManager;

pub async fn start_login_flow(
    app: AppHandle,
    state: Arc<Mutex<AccountManager>>,
) -> Result<(), String> {
    // 如果已有登录窗口，聚焦它
    if let Some(win) = app.get_webview_window("trae-login") {
        let _ = win.set_focus();
        return Ok(());
    }

    // 创建 oneshot channel 用于通知 warp 服务停止
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    let shutdown_tx = Arc::new(Mutex::new(Some(shutdown_tx)));

    let app_clone = app.clone();
    let state_clone = state.clone();

    // POST /callback — 接收 token 和 cookies
    let callback = warp::post()
        .and(warp::path("callback"))
        .and(warp::body::json())
        .and_then(move |body: serde_json::Value| {
            let app = app_clone.clone();
            let state = state_clone.clone();
            async move {
                let token = body["token"].as_str().unwrap_or("");
                if token.is_empty() {
                    return Ok::<_, warp::Rejection>(warp::reply::json(
                        &serde_json::json!({"status": "waiting"}),
                    ));
                }

                // 提取 cookies（如果有）
                let cookies = body["cookies"].as_str().map(|s| s.to_string());

                let mut manager = state.lock().await;
                match manager.add_account_by_token(token.to_string(), cookies).await {
                    Ok(account) => {
                        let _ = app.emit("login-success", &account.email);
                        // 延迟关闭窗口，让 warp 先返回响应
                        let app2 = app.clone();
                        tokio::spawn(async move {
                            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                            if let Some(win) = app2.get_webview_window("trae-login") {
                                let _ = win.close();
                            }
                        });
                        Ok(warp::reply::json(&serde_json::json!({"status": "ok"})))
                    }
                    Err(e) => {
                        let msg = e.to_string();
                        if msg.contains("已存在") {
                            let _ = app.emit("login-failed", "该账号已存在");
                            let app2 = app.clone();
                            tokio::spawn(async move {
                                tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                                if let Some(win) = app2.get_webview_window("trae-login") {
                                    let _ = win.close();
                                }
                            });
                        }
                        Ok(warp::reply::json(
                            &serde_json::json!({"status": "error", "message": msg}),
                        ))
                    }
                }
            }
        });

    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(vec!["POST"])
        .allow_headers(vec!["content-type"]);

    let routes = callback.with(cors);

    let (addr, server) =
        warp::serve(routes).bind_with_graceful_shutdown(([127, 0, 0, 1], 0), async {
            let _ = shutdown_rx.await;
        });
    let port = addr.port();

    tokio::spawn(server);

    // 注入 JS：Hook fetch/XHR 拦截 trae.ai 前端自身的 GetUserToken 请求响应
    // 注意：document.cookie 无法获取 HttpOnly cookies，所以这里只发送 token
    // 完整的 cookies 需要在 Rust 端通过 webview API 获取
    let init_script = format!(
        r#"
        (function() {{
            var __sent = false;
            var __callbackUrl = "http://127.0.0.1:{port}/callback";

            function sendToken(token) {{
                if (__sent || !token || token.length < 50) return;
                __sent = true;

                // 注意：document.cookie 只能获取非 HttpOnly cookies
                // 大部分认证 cookies（如 sessionid, sid_guard 等）是 HttpOnly 的，无法通过 JS 访问
                var cookies = document.cookie;

                console.log("[Trae Auto] 捕获到 Token，长度:", token.length);
                console.log("[Trae Auto] document.cookie 长度:", cookies.length);
                console.log("[Trae Auto] 注意：HttpOnly cookies 无法通过 JS 获取");

                var xhr = new XMLHttpRequest();
                xhr.open("POST", __callbackUrl, true);
                xhr.setRequestHeader("Content-Type", "application/json");
                xhr.send(JSON.stringify({{
                    token: token,
                    cookies: cookies || ""
                }}));
            }}

            function tryExtractToken(text) {{
                try {{
                    var data = typeof text === "string" ? JSON.parse(text) : text;
                    if (data && data.Result && data.Result.Token) {{
                        return data.Result.Token;
                    }}
                }} catch(e) {{}}
                return null;
            }}

            // Hook fetch
            var origFetch = window.fetch;
            window.fetch = function() {{
                var url = arguments[0];
                if (typeof url === "object" && url.url) url = url.url;
                var p = origFetch.apply(this, arguments);
                if (typeof url === "string" && url.indexOf("GetUserToken") !== -1) {{
                    p.then(function(resp) {{
                        return resp.clone().text();
                    }}).then(function(text) {{
                        var token = tryExtractToken(text);
                        if (token) sendToken(token);
                    }}).catch(function() {{}});
                }}
                return p;
            }};

            // Hook XMLHttpRequest
            var origOpen = XMLHttpRequest.prototype.open;
            var origSend = XMLHttpRequest.prototype.send;
            XMLHttpRequest.prototype.open = function(method, url) {{
                this.__url = url;
                return origOpen.apply(this, arguments);
            }};
            XMLHttpRequest.prototype.send = function() {{
                var self = this;
                if (self.__url && self.__url.indexOf("GetUserToken") !== -1) {{
                    self.addEventListener("load", function() {{
                        var token = tryExtractToken(self.responseText);
                        if (token) sendToken(token);
                    }});
                }}
                return origSend.apply(this, arguments);
            }};
        }})();
    "#,
        port = port
    );

    // 不使用 incognito 模式，以便能访问所有 cookies
    let window = WebviewWindowBuilder::new(
        &app,
        "trae-login",
        WebviewUrl::External("https://www.trae.ai".parse().unwrap()),
    )
    .title("登录 Trae 账号")
    .inner_size(500.0, 700.0)
    .center()
    .incognito(false)  // 改为 false，允许访问完整 cookies
    .initialization_script(&init_script)
    .build()
    .map_err(|e| e.to_string())?;

    // 监听窗口关闭，停止 warp 服务并通知前端
    let shutdown_on_close = shutdown_tx.clone();
    let app_for_close = app.clone();
    window.on_window_event(move |event| {
        if let tauri::WindowEvent::Destroyed = event {
            let shutdown = shutdown_on_close.clone();
            let app = app_for_close.clone();
            tauri::async_runtime::spawn(async move {
                if let Some(tx) = shutdown.lock().await.take() {
                    // shutdown 还在说明不是登录成功后关的窗口，是用户手动关的
                    let _ = app.emit("login-cancelled", ());
                    let _ = tx.send(());
                }
            });
        }
    });

    Ok(())
}
