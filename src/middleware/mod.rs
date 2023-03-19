use axum::{body::Body, http::Request, middleware::Next, response::Response};
use axum_client_ip::InsecureClientIp;
use chrono::Utc;
use tracing::{event, Level};

use crate::error::HTTPResult;

pub fn clone_value_from_task_local<T>(value: &T) -> T
where
    T: Clone,
{
    value.clone()
}

tokio::task_local! {
    pub static STARTED_AT: i64;
    pub static CLIENT_IP: String;
}

pub async fn entry<B>(req: Request<B>, next: Next<B>) -> Response {
    // 设置请求处理开始时间
    STARTED_AT
        .scope(Utc::now().timestamp_millis(), async { next.run(req).await })
        .await
}

pub async fn access_log(
    InsecureClientIp(ip): InsecureClientIp,
    req: Request<Body>,
    next: Next<Body>,
) -> HTTPResult<Response> {
    CLIENT_IP
        .scope(ip.to_string(), async {
            let start_at = STARTED_AT.with(clone_value_from_task_local);
            let uri = req.uri().to_string();
            let method = req.method().to_string();

            let resp = next.run(req).await;

            let status = resp.status().as_u16();

            let cost = Utc::now().timestamp_millis() - start_at;
            event!(
                Level::INFO,
                category = "accessLog",
                ip = ip.to_string(),
                method,
                uri,
                status,
                cost,
            );

            Ok(resp)
        })
        .await
}
