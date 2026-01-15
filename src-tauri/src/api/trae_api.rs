use anyhow::{anyhow, Result};
use reqwest::{header, Client};
use serde_json::json;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

use super::types::*;

const API_BASE_US: &str = "https://api-us-east.trae.ai";
const API_BASE_SG: &str = "https://api-sg-central.trae.ai";
const API_BASE_UG: &str = "https://ug-normal.trae.ai";

/// Trae API 客户端
pub struct TraeApiClient {
    client: Client,
    cookies: String,
    jwt_token: Option<String>,
    api_base: String,  // 动态 API 端点
}

impl TraeApiClient {
    /// 创建新的 API 客户端（使用 Cookies）
    pub fn new(cookies: &str) -> Result<Self> {
        let client = Client::builder()
            .build()?;

        // 清理 Cookie 字符串：移除换行符、多余空格
        let cleaned_cookies = cookies
            .lines()
            .map(|line| line.trim())
            .collect::<Vec<_>>()
            .join("")
            .replace("  ", " ");

        // 从 cookies 中检测区域
        let api_base = Self::detect_api_base_from_cookies(&cleaned_cookies);

        Ok(Self {
            client,
            cookies: cleaned_cookies,
            jwt_token: None,
            api_base,
        })
    }

    /// 创建新的 API 客户端（使用 Token）
    pub fn new_with_token(token: &str) -> Result<Self> {
        let client = Client::builder()
            .build()?;

        // 从 Token 中解析区域信息，默认尝试多个端点
        let api_base = API_BASE_SG.to_string(); // 默认使用新加坡，因为大多数亚洲用户

        Ok(Self {
            client,
            cookies: String::new(),
            jwt_token: Some(token.to_string()),
            api_base,
        })
    }

    /// 从 Cookies 中检测 API 端点
    fn detect_api_base_from_cookies(cookies: &str) -> String {
        // 检查 store-idc 或 trae-target-idc
        if cookies.contains("store-idc=useast") || cookies.contains("trae-target-idc=useast") {
            API_BASE_US.to_string()
        } else if cookies.contains("store-idc=alisg") || cookies.contains("trae-target-idc=alisg") {
            API_BASE_SG.to_string()
        } else {
            // 默认使用新加坡
            API_BASE_SG.to_string()
        }
    }

    /// 尝试多个 API 端点获取数据
    async fn try_api_endpoints<T, F, Fut>(&self, path: &str, request_fn: F) -> Result<T>
    where
        F: Fn(String) -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        // 先尝试当前设置的端点
        let url = format!("{}{}", self.api_base, path);
        match request_fn(url).await {
            Ok(result) => return Ok(result),
            Err(_) => {}
        }

        // 如果失败，尝试其他端点
        let other_base = if self.api_base == API_BASE_SG {
            API_BASE_US
        } else {
            API_BASE_SG
        };

        let url = format!("{}{}", other_base, path);
        request_fn(url).await
    }

    /// 构建请求头（仅使用 Token，不需要 Cookies）
    fn build_headers_token_only(&self) -> Result<header::HeaderMap> {
        let mut headers = header::HeaderMap::new();
        headers.insert(header::CONTENT_TYPE, "application/json".parse()?);
        headers.insert(header::ACCEPT, "application/json, text/plain, */*".parse()?);
        headers.insert(header::ORIGIN, "https://www.trae.ai".parse()?);
        headers.insert(header::REFERER, "https://www.trae.ai/".parse()?);
        headers.insert(
            header::USER_AGENT,
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36".parse()?,
        );

        if let Some(token) = &self.jwt_token {
            let auth_value = header::HeaderValue::from_bytes(
                format!("Cloud-IDE-JWT {}", token).as_bytes()
            ).map_err(|e| anyhow!("Token 格式错误: {}", e))?;
            headers.insert(header::AUTHORIZATION, auth_value);
        }

        Ok(headers)
    }

    /// 通过 Token 获取用户信息（从 entitlement 接口获取 user_id）
    pub async fn get_user_info_by_token(&self) -> Result<TokenUserInfo> {
        // 先解析 JWT Token 获取基本信息
        let token = self.jwt_token.as_ref().ok_or_else(|| anyhow!("Token 不存在"))?;
        let jwt_data = Self::parse_jwt_token(token)?;

        // 尝试多个 API 端点
        let headers = self.build_headers_token_only()?;
        let endpoints = [&self.api_base, API_BASE_SG, API_BASE_US];

        let mut last_error = anyhow!("所有 API 端点都失败");

        for base in endpoints.iter() {
            let url = format!("{}/trae/api/v1/pay/user_current_entitlement_list", base);

            let response = self
                .client
                .post(&url)
                .headers(headers.clone())
                .json(&json!({"require_usage": true}))
                .send()
                .await;

            match response {
                Ok(resp) if resp.status().is_success() => {
                    match resp.json::<EntitlementListResponse>().await {
                        Ok(data) => {
                            let user_id_from_api = data.user_entitlement_pack_list
                                .first()
                                .map(|p| p.entitlement_base_info.user_id.clone())
                                .unwrap_or_else(|| jwt_data.user_id.clone());

                            let user_detail = self.get_user_info_with_token().await.ok();

                            return Ok(TokenUserInfo {
                                user_id: user_id_from_api,
                                tenant_id: jwt_data.tenant_id,
                                screen_name: user_detail.as_ref().map(|u| u.screen_name.clone()),
                                avatar_url: user_detail.as_ref().and_then(|u| if u.avatar_url.is_empty() { None } else { Some(u.avatar_url.clone()) }),
                                email: user_detail.as_ref().and_then(|u| u.non_plain_text_email.clone()),
                            });
                        }
                        Err(e) => {
                            last_error = anyhow!("解析响应失败: {}", e);
                        }
                    }
                }
                Ok(resp) => {
                    last_error = anyhow!("API 返回错误: {}", resp.status());
                }
                Err(e) => {
                    last_error = anyhow!("请求失败: {}", e);
                }
            }
        }

        Err(last_error)
    }

    /// 尝试用 Token 调用 GetUserInfo 接口
    async fn get_user_info_with_token(&self) -> Result<UserInfoResult> {
        let url = format!("{}/cloudide/api/v3/trae/GetUserInfo", API_BASE_UG);
        let headers = self.build_headers_token_only()?;

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(&json!({"IfWebPage": true}))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("获取用户信息失败: {}", response.status()));
        }

        let data: GetUserInfoResponse = response.json().await?;
        Ok(data.result)
    }

    /// 解析 JWT Token 获取用户信息
    fn parse_jwt_token(token: &str) -> Result<JwtPayload> {
        // JWT 格式: header.payload.signature
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err(anyhow!("无效的 JWT Token 格式"));
        }

        // 解码 payload 部分（第二部分）
        let payload_b64 = parts[1];
        // JWT 使用 base64url 编码，需要处理 padding
        let padding = (4 - payload_b64.len() % 4) % 4;
        let padded = format!("{}{}", payload_b64, "=".repeat(padding));
        // 替换 base64url 字符为标准 base64
        let standard_b64 = padded.replace('-', "+").replace('_', "/");

        let payload_bytes = BASE64.decode(&standard_b64)
            .map_err(|e| anyhow!("解码 JWT payload 失败: {}", e))?;

        let payload_str = String::from_utf8(payload_bytes)
            .map_err(|e| anyhow!("JWT payload 不是有效的 UTF-8: {}", e))?;

        let payload: JwtPayloadRaw = serde_json::from_str(&payload_str)
            .map_err(|e| anyhow!("解析 JWT payload 失败: {}", e))?;

        Ok(JwtPayload {
            user_id: payload.data.id,
            tenant_id: payload.data.tenant_id,
        })
    }

    /// 构建请求头
    fn build_headers(&self, with_auth: bool) -> Result<header::HeaderMap> {
        let mut headers = header::HeaderMap::new();
        headers.insert(header::CONTENT_TYPE, "application/json".parse()?);
        headers.insert(header::ACCEPT, "application/json, text/plain, */*".parse()?);

        // 使用 from_bytes 来处理包含特殊字符的 Cookie
        let cookie_value = header::HeaderValue::from_bytes(self.cookies.as_bytes())
            .map_err(|e| anyhow!("Cookie 格式错误: {}", e))?;
        headers.insert(header::COOKIE, cookie_value);

        headers.insert(header::ORIGIN, "https://www.trae.ai".parse()?);
        headers.insert(header::REFERER, "https://www.trae.ai/".parse()?);
        headers.insert(
            header::USER_AGENT,
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36".parse()?,
        );

        if with_auth {
            if let Some(token) = &self.jwt_token {
                let auth_value = header::HeaderValue::from_bytes(
                    format!("Cloud-IDE-JWT {}", token).as_bytes()
                ).map_err(|e| anyhow!("Token 格式错误: {}", e))?;
                headers.insert(header::AUTHORIZATION, auth_value);
            }
        }

        Ok(headers)
    }

    /// 获取用户 Token
    pub async fn get_user_token(&mut self) -> Result<UserTokenResult> {
        let url = format!("{}/cloudide/api/v3/common/GetUserToken", self.api_base);
        let headers = self.build_headers(false)?;

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("获取 Token 失败: {}", response.status()));
        }

        let data: GetUserTokenResponse = response.json().await?;
        self.jwt_token = Some(data.result.token.clone());
        Ok(data.result)
    }

    /// 获取用户信息
    pub async fn get_user_info(&self) -> Result<UserInfoResult> {
        let url = format!("{}/cloudide/api/v3/trae/GetUserInfo", API_BASE_UG);
        let headers = self.build_headers(false)?;

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(&json!({"IfWebPage": true}))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("获取用户信息失败: {}", response.status()));
        }

        let data: GetUserInfoResponse = response.json().await?;
        Ok(data.result)
    }

    /// 获取用户配额和使用量
    pub async fn get_entitlement_list(&self) -> Result<EntitlementListResponse> {
        let url = format!("{}/trae/api/v1/pay/user_current_entitlement_list", self.api_base);
        let headers = self.build_headers(true)?;

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(&json!({"require_usage": true}))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("获取配额信息失败: {}", response.status()));
        }

        let data: EntitlementListResponse = response.json().await?;
        Ok(data)
    }

    /// 查询使用记录
    pub async fn query_usage(
        &self,
        start_time: i64,
        end_time: i64,
        page_size: i32,
        page_num: i32,
    ) -> Result<UsageQueryResponse> {
        let url = format!(
            "{}/trae/api/v1/pay/query_user_usage_group_by_session",
            self.api_base
        );
        let headers = self.build_headers(true)?;

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(&json!({
                "start_time": start_time,
                "end_time": end_time,
                "page_size": page_size,
                "page_num": page_num
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("查询使用记录失败: {}", response.status()));
        }

        let data: UsageQueryResponse = response.json().await?;
        Ok(data)
    }

    /// 获取使用量汇总（简化版，用于前端展示）
    pub async fn get_usage_summary(&mut self) -> Result<UsageSummary> {
        // 确保有 token
        if self.jwt_token.is_none() {
            self.get_user_token().await?;
        }

        let entitlements = self.get_entitlement_list().await?;
        Self::parse_entitlements_to_summary(entitlements)
    }

    /// 通过 Token 获取使用量汇总
    pub async fn get_usage_summary_by_token(&self) -> Result<UsageSummary> {
        let headers = self.build_headers_token_only()?;
        let endpoints = [&self.api_base, API_BASE_SG, API_BASE_US];

        let mut last_error = anyhow!("所有 API 端点都失败");

        for base in endpoints.iter() {
            let url = format!("{}/trae/api/v1/pay/user_current_entitlement_list", base);
            println!("[DEBUG] Trying API endpoint: {}", url);

            let response = self
                .client
                .post(&url)
                .headers(headers.clone())
                .json(&json!({"require_usage": true}))
                .send()
                .await;

            match response {
                Ok(resp) if resp.status().is_success() => {
                    let response_text = resp.text().await?;
                    println!("[DEBUG] API Response from {}: {}", base, response_text);

                    match serde_json::from_str::<EntitlementListResponse>(&response_text) {
                        Ok(entitlements) => {
                            let summary = Self::parse_entitlements_to_summary(entitlements)?;
                            println!("[DEBUG] Parsed Summary: fast_request_limit={}, extra_fast_request_limit={}",
                                summary.fast_request_limit, summary.extra_fast_request_limit);
                            return Ok(summary);
                        }
                        Err(e) => {
                            last_error = anyhow!("解析响应失败: {}", e);
                        }
                    }
                }
                Ok(resp) => {
                    println!("[DEBUG] API {} returned error: {}", base, resp.status());
                    last_error = anyhow!("API 返回错误: {}", resp.status());
                }
                Err(e) => {
                    println!("[DEBUG] API {} request failed: {}", base, e);
                    last_error = anyhow!("请求失败: {}", e);
                }
            }
        }

        Err(last_error)
    }

    /// 解析配额信息为使用量汇总
    fn parse_entitlements_to_summary(entitlements: EntitlementListResponse) -> Result<UsageSummary> {
        let mut summary = UsageSummary::default();

        for pack in entitlements.user_entitlement_pack_list {
            let base = &pack.entitlement_base_info;
            let usage = &pack.usage;
            let quota = &base.quota;

            // 判断是否是额外礼包（product_type == 2）
            if base.product_type == 2 {
                // Extra Package
                summary.extra_fast_request_limit = quota.premium_model_fast_request_limit;
                // 使用 premium_model_fast_amount 作为实际使用量
                summary.extra_fast_request_used = usage.premium_model_fast_amount;
                summary.extra_fast_request_left =
                    summary.extra_fast_request_limit as f64 - summary.extra_fast_request_used;
                summary.extra_expire_time = base.end_time;

                // 尝试获取礼包名称
                if let Some(pkg_extra) = &base.product_extra.package_extra {
                    if pkg_extra.package_source_type == 6 {
                        summary.extra_package_name = "2026 Anniversary Treat".to_string();
                    }
                }
            } else {
                // Free/Pro Plan
                summary.plan_type = if base.product_id == 0 {
                    "Free".to_string()
                } else {
                    "Pro".to_string()
                };
                summary.reset_time = base.end_time;

                summary.fast_request_limit = quota.premium_model_fast_request_limit;
                // 使用 premium_model_fast_amount 作为实际使用量
                summary.fast_request_used = usage.premium_model_fast_amount;
                summary.fast_request_left =
                    summary.fast_request_limit as f64 - summary.fast_request_used;

                summary.slow_request_limit = quota.premium_model_slow_request_limit;
                // 使用 premium_model_slow_amount 作为实际使用量
                summary.slow_request_used = usage.premium_model_slow_amount;
                summary.slow_request_left =
                    summary.slow_request_limit as f64 - summary.slow_request_used;

                summary.advanced_model_limit = quota.advanced_model_request_limit;
                // 使用 advanced_model_amount 作为实际使用量
                summary.advanced_model_used = usage.advanced_model_amount;
                summary.advanced_model_left =
                    summary.advanced_model_limit as f64 - summary.advanced_model_used;

                summary.autocomplete_limit = quota.auto_completion_limit;
                // 使用 auto_completion_amount 作为实际使用量
                summary.autocomplete_used = usage.auto_completion_amount;
                summary.autocomplete_left =
                    summary.autocomplete_limit as f64 - summary.autocomplete_used;
            }
        }

        Ok(summary)
    }

    /// 查询礼包状态
    pub async fn query_birthday_bonus(&self) -> Result<bool> {
        let url = format!("{}/trae/api/v1/pay/query_birthday_bonus", self.api_base);
        let headers = self.build_headers_token_only()?;

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("查询礼包状态失败: {}", response.status()));
        }

        let data: serde_json::Value = response.json().await?;

        // 返回是否已领取
        Ok(data["bonus_claimed"].as_bool().unwrap_or(false))
    }

    /// 领取礼包
    pub async fn claim_birthday_bonus(&self) -> Result<()> {
        let url = format!("{}/trae/api/v1/pay/claim_birthday_bonus", self.api_base);
        let headers = self.build_headers_token_only()?;

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("领取礼包失败: {}", response.status()));
        }

        Ok(())
    }
}
