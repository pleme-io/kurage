use crate::api::types::{
    Agent, AgentList, ArtifactList, Conversation, DeleteResponse, FollowupRequest, LaunchRequest,
    MeResponse, ModelList, RepoList, StopResponse,
};
use crate::error::{KurageError, Result};

/// HTTP client for the Cursor Cloud Agents API.
#[derive(Debug, Clone)]
pub struct CursorCloudClient {
    inner: reqwest::Client,
    base_url: String,
    api_key: String,
}

impl CursorCloudClient {
    /// Create a new client. Auth uses HTTP Basic with `api_key` as username, empty password.
    pub fn new(base_url: &str, api_key: &str) -> Result<Self> {
        let inner = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .user_agent("pleme-io/kurage 0.1.0")
            .build()
            .map_err(KurageError::Request)?;

        Ok(Self {
            inner,
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: api_key.to_string(),
        })
    }

    fn url(&self, path: &str) -> String {
        format!("{}/{}", self.base_url, path.trim_start_matches('/'))
    }

    async fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T> {
        let resp = self
            .inner
            .get(&self.url(path))
            .basic_auth(&self.api_key, Option::<&str>::None)
            .send()
            .await
            .map_err(KurageError::Request)?;
        Self::handle_response(resp).await
    }

    async fn post<B: serde::Serialize, T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let resp = self
            .inner
            .post(&self.url(path))
            .basic_auth(&self.api_key, Option::<&str>::None)
            .json(body)
            .send()
            .await
            .map_err(KurageError::Request)?;
        Self::handle_response(resp).await
    }

    async fn post_empty<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T> {
        let resp = self
            .inner
            .post(&self.url(path))
            .basic_auth(&self.api_key, Option::<&str>::None)
            .send()
            .await
            .map_err(KurageError::Request)?;
        Self::handle_response(resp).await
    }

    async fn delete<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T> {
        let resp = self
            .inner
            .delete(&self.url(path))
            .basic_auth(&self.api_key, Option::<&str>::None)
            .send()
            .await
            .map_err(KurageError::Request)?;
        Self::handle_response(resp).await
    }

    async fn handle_response<T: serde::de::DeserializeOwned>(
        resp: reqwest::Response,
    ) -> Result<T> {
        let status = resp.status().as_u16();
        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(KurageError::Api { status, body });
        }
        let text = resp.text().await.map_err(KurageError::Request)?;
        serde_json::from_str(&text).map_err(KurageError::Json)
    }

    // ── Public API methods ──────────────────────────────────────────────────

    pub async fn launch(&self, req: &LaunchRequest) -> Result<Agent> {
        self.post("/agents", req).await
    }

    pub async fn list(&self, limit: u32) -> Result<AgentList> {
        self.get(&format!("/agents?limit={limit}")).await
    }

    pub async fn status(&self, id: &str) -> Result<Agent> {
        self.get(&format!("/agents/{id}")).await
    }

    pub async fn logs(&self, id: &str) -> Result<Conversation> {
        self.get(&format!("/agents/{id}/conversation")).await
    }

    pub async fn stop(&self, id: &str) -> Result<StopResponse> {
        self.post_empty(&format!("/agents/{id}/stop")).await
    }

    pub async fn followup(&self, id: &str, req: &FollowupRequest) -> Result<Agent> {
        self.post(&format!("/agents/{id}/followup"), req).await
    }

    pub async fn delete_agent(&self, id: &str) -> Result<DeleteResponse> {
        self.delete(&format!("/agents/{id}")).await
    }

    pub async fn artifacts(&self, id: &str) -> Result<ArtifactList> {
        self.get(&format!("/agents/{id}/artifacts")).await
    }

    pub async fn models(&self) -> Result<ModelList> {
        self.get("/models").await
    }

    pub async fn repos(&self) -> Result<RepoList> {
        self.get("/repositories").await
    }

    pub async fn me(&self) -> Result<MeResponse> {
        self.get("/me").await
    }
}
