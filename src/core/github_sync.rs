//! GitHub 同步客户端（REST API）

use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::time::Duration;

use crate::core::error::{AppError, Result};
use crate::core::todo::TodoStatus;

#[derive(Debug, Clone, Deserialize)]
pub struct GithubIssue {
    pub number: i64,
    pub title: String,
    pub state: String,
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub pull_request: Option<serde_json::Value>,
}

pub struct GithubSyncClient {
    client: reqwest::Client,
    owner: String,
    repo: String,
    rest_base_url: String,
    graphql_url: String,
}

const REST_PAGE_SIZE: i32 = 100;
const MAX_RETRIES: usize = 3;
const RETRY_BASE_MS: u64 = 250;

impl GithubSyncClient {
    pub fn new(token: &str, owner: &str, repo: &str) -> Result<Self> {
        Self::new_with_base_urls(
            token,
            owner,
            repo,
            "https://api.github.com",
            "https://api.github.com/graphql",
        )
    }

    pub fn new_with_base_urls(
        token: &str,
        owner: &str,
        repo: &str,
        rest_base_url: &str,
        graphql_url: &str,
    ) -> Result<Self> {
        if token.trim().is_empty() {
            return Err(AppError::Authentication(
                "GitHub token 为空，无法执行同步".to_string(),
            ));
        }

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::USER_AGENT,
            reqwest::header::HeaderValue::from_static("pomoflow-rs"),
        );
        headers.insert(
            reqwest::header::ACCEPT,
            reqwest::header::HeaderValue::from_static("application/vnd.github+json"),
        );
        headers.insert(
            reqwest::header::AUTHORIZATION,
            reqwest::header::HeaderValue::from_str(&format!("Bearer {}", token))
                .map_err(|e| AppError::Authentication(format!("无效 token: {e}")))?,
        );
        headers.insert(
            reqwest::header::HeaderName::from_static("x-github-api-version"),
            reqwest::header::HeaderValue::from_static("2022-11-28"),
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .map_err(|e| AppError::Network(format!("创建 GitHub 客户端失败: {e}")))?;

        Ok(Self {
            client,
            owner: owner.to_string(),
            repo: repo.to_string(),
            rest_base_url: rest_base_url.trim_end_matches('/').to_string(),
            graphql_url: graphql_url.to_string(),
        })
    }

    pub async fn get_issue(&self, issue_number: i64) -> Result<GithubIssue> {
        let url = format!(
            "{}/repos/{}/{}/issues/{}",
            self.rest_base_url,
            self.owner, self.repo, issue_number
        );
        let response = self
            .client
            .get(url)
            .send_with_retry()
            .await
            .map_err(|e| AppError::Network(format!("获取 issue 失败: {e}")))?;

        self.map_issue_response(response).await
    }

    pub async fn update_issue(
        &self,
        issue_number: i64,
        title: Option<&str>,
        status: Option<&TodoStatus>,
    ) -> Result<GithubIssue> {
        let url = format!(
            "{}/repos/{}/{}/issues/{}",
            self.rest_base_url,
            self.owner, self.repo, issue_number
        );

        let state = status.map(|s| match s {
            TodoStatus::Done => "closed",
            TodoStatus::Todo | TodoStatus::InProgress => "open",
        });

        let mut body = serde_json::Map::new();
        if let Some(title) = title {
            body.insert("title".to_string(), serde_json::Value::String(title.to_string()));
        }
        if let Some(state) = state {
            body.insert("state".to_string(), serde_json::Value::String(state.to_string()));
        }

        let response = self
            .client
            .patch(url)
            .json(&body)
            .send_with_retry()
            .await
            .map_err(|e| AppError::Network(format!("更新 issue 失败: {e}")))?;

        self.map_issue_response(response).await
    }

    pub async fn list_issues_since(&self, since: Option<&str>) -> Result<Vec<GithubIssue>> {
        let url = format!(
            "{}/repos/{}/{}/issues",
            self.rest_base_url, self.owner, self.repo
        );
        let mut page = 1_i32;
        let mut all_issues = Vec::new();

        loop {
            let mut req = self.client.get(&url).query(&[
                ("state", "all"),
                ("per_page", &REST_PAGE_SIZE.to_string()),
                ("page", &page.to_string()),
            ]);
            if let Some(since) = since.filter(|v| !v.trim().is_empty()) {
                req = req.query(&[("since", since)]);
            }

            let response = req
                .send_with_retry()
                .await
                .map_err(|e| AppError::Network(format!("拉取 issue 列表失败: {e}")))?;

            let status = response.status();
            if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN
            {
                let body = response.text().await.unwrap_or_default();
                return Err(AppError::Authentication(format!(
                    "GitHub 鉴权失败: HTTP {} {}",
                    status, body
                )));
            }
            if !status.is_success() {
                let body = response.text().await.unwrap_or_default();
                return Err(AppError::Network(format!(
                    "GitHub API 请求失败: HTTP {} {}",
                    status, body
                )));
            }

            let issues = response
                .json::<Vec<GithubIssue>>()
                .await
                .map_err(|e| AppError::Network(format!("解析 GitHub 响应失败: {e}")))?;
            let size = issues.len();
            all_issues.extend(issues.into_iter().filter(|issue| issue.pull_request.is_none()));

            if is_last_rest_page(size) {
                break;
            }
            page += 1;
        }

        Ok(all_issues)
    }

    pub async fn update_project_item_status(
        &self,
        project_number: i64,
        issue_number: i64,
        status_name: &str,
    ) -> Result<()> {
        if project_number <= 0 {
            return Err(AppError::Validation("project_number 必须为正整数".to_string()));
        }
        if issue_number <= 0 {
            return Err(AppError::Validation("issue_number 必须为正整数".to_string()));
        }
        if status_name.trim().is_empty() {
            return Err(AppError::Validation(
                "project status 不能为空".to_string(),
            ));
        }

        let project_number_i32 = i32::try_from(project_number)
            .map_err(|_| AppError::Validation("project_number 超出范围".to_string()))?;
        let field_ctx = self
            .find_project_item_status_context(project_number_i32, issue_number, status_name)
            .await?;

        let mutation = r#"
            mutation UpdateProjectStatus(
              $projectId: ID!,
              $itemId: ID!,
              $fieldId: ID!,
              $optionId: String!
            ) {
              updateProjectV2ItemFieldValue(
                input: {
                  projectId: $projectId,
                  itemId: $itemId,
                  fieldId: $fieldId,
                  value: { singleSelectOptionId: $optionId }
                }
              ) {
                projectV2Item { id }
              }
            }
        "#;
        let variables = serde_json::json!({
            "projectId": field_ctx.project_id,
            "itemId": field_ctx.item_id,
            "fieldId": field_ctx.field_id,
            "optionId": field_ctx.option_id,
        });
        let _ = self.github_graphql(mutation, variables).await?;
        Ok(())
    }

    async fn map_issue_response(&self, response: reqwest::Response) -> Result<GithubIssue> {
        let status = response.status();
        if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN
        {
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::Authentication(format!(
                "GitHub 鉴权失败: HTTP {} {}",
                status, body
            )));
        }
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::Network(format!(
                "GitHub API 请求失败: HTTP {} {}",
                status, body
            )));
        }

        response
            .json::<GithubIssue>()
            .await
            .map_err(|e| AppError::Network(format!("解析 GitHub 响应失败: {e}")))
    }

    async fn github_graphql(
        &self,
        query: &str,
        variables: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let response = self
            .client
            .post(&self.graphql_url)
            .json(&serde_json::json!({
                "query": query,
                "variables": variables,
            }))
            .send_with_retry()
            .await
            .map_err(|e| AppError::Network(format!("GraphQL 请求失败: {e}")))?;

        let status = response.status();
        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AppError::Network(format!("解析 GraphQL 响应失败: {e}")))?;

        if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN
        {
            return Err(AppError::Authentication(format!(
                "GitHub GraphQL 鉴权失败: HTTP {} {}",
                status, body
            )));
        }
        if !status.is_success() {
            return Err(AppError::Network(format!(
                "GitHub GraphQL 请求失败: HTTP {} {}",
                status, body
            )));
        }
        if let Some(errors) = body.get("errors") {
            return Err(AppError::Network(format!("GitHub GraphQL 返回错误: {errors}")));
        }
        Ok(body
            .get("data")
            .cloned()
            .ok_or_else(|| AppError::Network("GraphQL 响应缺少 data 字段".to_string()))?)
    }

    async fn find_project_item_status_context(
        &self,
        project_number: i32,
        issue_number: i64,
        status_name: &str,
    ) -> Result<ProjectStatusContext> {
        let query = r#"
            query ProjectStatusContext(
              $owner: String!,
              $projectNumber: Int!,
              $issueNumber: Int!,
              $after: String
            ) {
              org: organization(login: $owner) {
                projectV2(number: $projectNumber) {
                  id
                  field: field(name: "Status") {
                    ... on ProjectV2SingleSelectField {
                      id
                      options { id name }
                    }
                  }
                  items(first: 100, after: $after) {
                    pageInfo {
                      hasNextPage
                      endCursor
                    }
                    nodes {
                      id
                      content {
                        ... on Issue { number }
                      }
                    }
                  }
                }
              }
              usr: user(login: $owner) {
                projectV2(number: $projectNumber) {
                  id
                  field: field(name: "Status") {
                    ... on ProjectV2SingleSelectField {
                      id
                      options { id name }
                    }
                  }
                  items(first: 100, after: $after) {
                    pageInfo {
                      hasNextPage
                      endCursor
                    }
                    nodes {
                      id
                      content {
                        ... on Issue { number }
                      }
                    }
                  }
                }
              }
            }
        "#;
        let mut after: Option<String> = None;
        let mut project_meta: Option<(String, String, String)> = None;

        loop {
            let data = self
                .github_graphql(
                    query,
                    serde_json::json!({
                        "owner": self.owner,
                        "projectNumber": project_number,
                        "issueNumber": issue_number,
                        "after": after,
                    }),
                )
                .await?;

            let project = data
                .get("org")
                .and_then(|v| v.get("projectV2"))
                .or_else(|| data.get("usr").and_then(|v| v.get("projectV2")))
                .ok_or_else(|| AppError::NotFound("未找到目标 GitHub Project".to_string()))?;

            if project_meta.is_none() {
                let project_id = project
                    .get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| AppError::NotFound("Project 缺少 id".to_string()))?
                    .to_string();

                let field = project
                    .get("field")
                    .ok_or_else(|| AppError::NotFound("Project 中未找到 Status 字段".to_string()))?;
                let field_id = field
                    .get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| AppError::NotFound("Status 字段缺少 id".to_string()))?
                    .to_string();
                let option_id = field
                    .get("options")
                    .and_then(|v| v.as_array())
                    .and_then(|options| {
                        options.iter().find_map(|option| {
                            let name = option.get("name").and_then(|v| v.as_str())?;
                            if name == status_name {
                                option
                                    .get("id")
                                    .and_then(|v| v.as_str())
                                    .map(|v| v.to_string())
                            } else {
                                None
                            }
                        })
                    })
                    .ok_or_else(|| {
                        AppError::NotFound(format!("Status 字段中不存在选项: {}", status_name))
                    })?;
                project_meta = Some((project_id, field_id, option_id));
            }

            if let Some(item_id) = extract_project_item_id(project, issue_number) {
                let (project_id, field_id, option_id) =
                    project_meta.clone().expect("project meta initialized");
                return Ok(ProjectStatusContext {
                    project_id,
                    item_id,
                    field_id,
                    option_id,
                });
            }

            let has_next = project
                .get("items")
                .and_then(|v| v.get("pageInfo"))
                .and_then(|v| v.get("hasNextPage"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !has_next {
                break;
            }
            after = project
                .get("items")
                .and_then(|v| v.get("pageInfo"))
                .and_then(|v| v.get("endCursor"))
                .and_then(|v| v.as_str())
                .map(|v| v.to_string());
        }

        Err(AppError::NotFound(format!(
            "Project 中未找到 issue #{} 对应的 item",
            issue_number
        )))
    }
}

struct ProjectStatusContext {
    project_id: String,
    item_id: String,
    field_id: String,
    option_id: String,
}

fn extract_project_item_id(project: &serde_json::Value, issue_number: i64) -> Option<String> {
    project
        .get("items")
        .and_then(|v| v.get("nodes"))
        .and_then(|v| v.as_array())
        .and_then(|nodes| {
            nodes.iter().find_map(|item| {
                let number = item
                    .get("content")
                    .and_then(|v| v.get("number"))
                    .and_then(|v| v.as_i64());
                if number == Some(issue_number) {
                    item.get("id")
                        .and_then(|v| v.as_str())
                        .map(|v| v.to_string())
                } else {
                    None
                }
            })
        })
}

trait RequestBuilderRetryExt {
    async fn send_with_retry(self) -> std::result::Result<reqwest::Response, reqwest::Error>;
}

impl RequestBuilderRetryExt for reqwest::RequestBuilder {
    async fn send_with_retry(self) -> std::result::Result<reqwest::Response, reqwest::Error> {
        let mut attempt = 0_usize;
        let mut builder = self;
        loop {
            let Some(next_builder) = builder.try_clone() else {
                return builder.send().await;
            };
            match builder.send().await {
                Ok(resp) if should_retry_response(resp.status(), attempt) =>
                {
                    attempt += 1;
                    tokio::time::sleep(next_retry_delay(attempt)).await;
                    builder = next_builder;
                }
                Ok(resp) => return Ok(resp),
                Err(err) if should_retry_error(attempt) => {
                    attempt += 1;
                    tokio::time::sleep(next_retry_delay(attempt)).await;
                    builder = next_builder;
                    let _ = err;
                }
                Err(err) => return Err(err),
            }
        }
    }
}

fn is_last_rest_page(size: usize) -> bool {
    size < REST_PAGE_SIZE as usize
}

fn should_retry_response(status: reqwest::StatusCode, attempt: usize) -> bool {
    status.is_server_error() && attempt + 1 < MAX_RETRIES
}

fn should_retry_error(attempt: usize) -> bool {
    attempt + 1 < MAX_RETRIES
}

fn next_retry_delay(attempt: usize) -> Duration {
    Duration::from_millis(RETRY_BASE_MS * attempt as u64)
}

#[cfg(test)]
mod tests {
    use super::{
        extract_project_item_id, is_last_rest_page, next_retry_delay, should_retry_error,
        should_retry_response, GithubSyncClient, MAX_RETRIES, REST_PAGE_SIZE, RETRY_BASE_MS,
    };
    use std::collections::VecDeque;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    #[test]
    fn rest_page_boundary_detection_works() {
        assert!(is_last_rest_page(0));
        assert!(is_last_rest_page((REST_PAGE_SIZE - 1) as usize));
        assert!(!is_last_rest_page(REST_PAGE_SIZE as usize));
    }

    #[test]
    fn retry_response_rules_match_limits() {
        assert!(should_retry_response(reqwest::StatusCode::INTERNAL_SERVER_ERROR, 0));
        assert!(!should_retry_response(
            reqwest::StatusCode::BAD_REQUEST,
            0
        ));
        assert!(!should_retry_response(
            reqwest::StatusCode::INTERNAL_SERVER_ERROR,
            MAX_RETRIES - 1
        ));
    }

    #[test]
    fn retry_error_rules_match_limits() {
        assert!(should_retry_error(0));
        assert!(should_retry_error(MAX_RETRIES - 2));
        assert!(!should_retry_error(MAX_RETRIES - 1));
    }

    #[test]
    fn retry_delay_scales_linearly() {
        assert_eq!(next_retry_delay(1).as_millis() as u64, RETRY_BASE_MS);
        assert_eq!(next_retry_delay(2).as_millis() as u64, RETRY_BASE_MS * 2);
    }

    #[test]
    fn extract_project_item_id_finds_matching_issue() {
        let project = serde_json::json!({
            "items": {
                "nodes": [
                    { "id": "PVTI_1", "content": { "number": 41 } },
                    { "id": "PVTI_2", "content": { "number": 42 } }
                ]
            }
        });
        assert_eq!(extract_project_item_id(&project, 42), Some("PVTI_2".to_string()));
        assert_eq!(extract_project_item_id(&project, 43), None);
    }

    #[tokio::test]
    async fn get_issue_retries_on_5xx_then_succeeds() {
        let responses = vec![
            mock_response(500, r#"{"message":"server error"}"#),
            mock_response(
                200,
                r#"{"number":42,"title":"Recovered","state":"open","updated_at":"2026-01-01T00:00:00Z"}"#,
            ),
        ];
        let (base_url, hits, handle) = spawn_mock_server(responses).await;
        let client = GithubSyncClient::new_with_base_urls(
            "github_pat_xxxxxxxxxxxx",
            "acme",
            "repo",
            &base_url,
            &format!("{}/graphql", base_url),
        )
        .expect("client init");

        let issue = client.get_issue(42).await.expect("get_issue should succeed");
        assert_eq!(issue.number, 42);
        assert_eq!(issue.title, "Recovered");
        assert_eq!(hits.load(Ordering::SeqCst), 2);
        handle.await.expect("server join");
    }

    #[tokio::test]
    async fn get_issue_fails_after_retry_exhausted() {
        let responses = vec![
            mock_response(500, r#"{"message":"server error"}"#),
            mock_response(500, r#"{"message":"server error"}"#),
            mock_response(500, r#"{"message":"server error"}"#),
        ];
        let (base_url, hits, handle) = spawn_mock_server(responses).await;
        let client = GithubSyncClient::new_with_base_urls(
            "github_pat_xxxxxxxxxxxx",
            "acme",
            "repo",
            &base_url,
            &format!("{}/graphql", base_url),
        )
        .expect("client init");

        let err = client.get_issue(42).await.expect_err("should fail after retries");
        let msg = err.to_string();
        assert!(msg.contains("GitHub API 请求失败") || msg.contains("获取 issue 失败"));
        assert_eq!(hits.load(Ordering::SeqCst), MAX_RETRIES);
        handle.await.expect("server join");
    }

    #[tokio::test]
    async fn update_project_status_paginates_until_item_found() {
        let first_page = r#"{
          "data": {
            "org": {
              "projectV2": {
                "id": "PVT_1",
                "field": {
                  "id": "PVTF_1",
                  "options": [
                    {"id": "OPT_TODO", "name": "Todo"},
                    {"id": "OPT_DONE", "name": "Done"}
                  ]
                },
                "items": {
                  "pageInfo": {"hasNextPage": true, "endCursor": "CURSOR_2"},
                  "nodes": [{"id": "PVTI_1", "content": {"number": 41}}]
                }
              }
            },
            "usr": null
          }
        }"#;
        let second_page = r#"{
          "data": {
            "org": {
              "projectV2": {
                "id": "PVT_1",
                "field": {
                  "id": "PVTF_1",
                  "options": [
                    {"id": "OPT_TODO", "name": "Todo"},
                    {"id": "OPT_DONE", "name": "Done"}
                  ]
                },
                "items": {
                  "pageInfo": {"hasNextPage": false, "endCursor": null},
                  "nodes": [{"id": "PVTI_2", "content": {"number": 42}}]
                }
              }
            },
            "usr": null
          }
        }"#;
        let mutation_ok = r#"{
          "data": {
            "updateProjectV2ItemFieldValue": {
              "projectV2Item": {"id":"PVTI_2"}
            }
          }
        }"#;

        let responses = vec![
            mock_response(200, first_page),
            mock_response(200, second_page),
            mock_response(200, mutation_ok),
        ];
        let (base_url, hits, handle) = spawn_mock_server(responses).await;
        let client = GithubSyncClient::new_with_base_urls(
            "github_pat_xxxxxxxxxxxx",
            "acme",
            "repo",
            &base_url,
            &format!("{}/graphql", base_url),
        )
        .expect("client init");

        client
            .update_project_item_status(1, 42, "Done")
            .await
            .expect("project status update should succeed");
        assert_eq!(hits.load(Ordering::SeqCst), 3);
        handle.await.expect("server join");
    }

    fn mock_response(status: u16, body: &str) -> MockResponse {
        MockResponse {
            status,
            body: body.to_string(),
            content_type: "application/json",
        }
    }

    async fn spawn_mock_server(
        responses: Vec<MockResponse>,
    ) -> (String, Arc<AtomicUsize>, tokio::task::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind mock server");
        let addr = listener.local_addr().expect("local addr");
        let queue = Arc::new(Mutex::new(VecDeque::from(responses)));
        let hit_count = Arc::new(AtomicUsize::new(0));
        let queue_clone = Arc::clone(&queue);
        let hit_count_clone = Arc::clone(&hit_count);

        let handle = tokio::spawn(async move {
            loop {
                let has_more = {
                    let guard = queue_clone.lock().expect("lock queue");
                    !guard.is_empty()
                };
                if !has_more {
                    break;
                }

                let (mut socket, _) = listener.accept().await.expect("accept");
                let mut buf = [0_u8; 8192];
                let _ = socket.read(&mut buf).await;
                hit_count_clone.fetch_add(1, Ordering::SeqCst);

                let response = {
                    let mut guard = queue_clone.lock().expect("lock queue");
                    guard.pop_front().expect("mock response")
                };

                let status_line = match response.status {
                    200 => "200 OK",
                    400 => "400 Bad Request",
                    401 => "401 Unauthorized",
                    403 => "403 Forbidden",
                    404 => "404 Not Found",
                    429 => "429 Too Many Requests",
                    500 => "500 Internal Server Error",
                    502 => "502 Bad Gateway",
                    503 => "503 Service Unavailable",
                    _ => "520 Unknown Error",
                };
                let payload = format!(
                    "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    status_line,
                    response.content_type,
                    response.body.len(),
                    response.body
                );
                let _ = socket.write_all(payload.as_bytes()).await;
                let _ = socket.shutdown().await;
            }
        });

        (format!("http://{}", addr), hit_count, handle)
    }

    struct MockResponse {
        status: u16,
        body: String,
        content_type: &'static str,
    }
}
