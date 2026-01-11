use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GraphQLResponse<T> {
    pub data: Option<T>,
    pub errors: Option<Vec<GraphQLError>>,
}

#[derive(Debug, Deserialize)]
pub struct GraphQLError {
    pub message: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PageInfo {
    pub has_next_page: bool,
    pub end_cursor: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UserData {
    pub viewer: Option<User>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub login: String,
    pub contributions_collection: ContributionsCollection,
    pub repositories: RepositoryConnection,
    pub pull_requests: PullRequestConnection,
    pub issues: IssueConnection,
    pub merged_pull_requests: MergedPullRequestConnection,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContributionsCollection {
    pub total_commit_contributions: u32,
    pub restricted_contributions_count: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryConnection {
    pub nodes: Vec<Repository>,
    pub page_info: PageInfo,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Repository {
    pub stargazer_count: u32,
    pub fork_count: u32,
    pub is_fork: bool,
    pub languages: Option<LanguageConnection>,
}

#[derive(Debug, Deserialize)]
pub struct LanguageConnection {
    pub edges: Vec<LanguageEdge>,
}

#[derive(Debug, Deserialize)]
pub struct LanguageEdge {
    pub size: u64,
    pub node: Language,
}

#[derive(Debug, Deserialize)]
pub struct Language {
    pub name: String,
    pub color: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestConnection {
    pub total_count: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueConnection {
    pub total_count: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MergedPullRequestConnection {
    pub total_count: u32,
    pub nodes: Vec<MergedPullRequest>,
    pub page_info: PageInfo,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MergedPullRequest {
    pub repository: PullRequestRepo,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestRepo {
    pub name: String,
    pub owner: RepoOwner,
    pub stargazer_count: u32,
}

#[derive(Debug, Deserialize)]
pub struct RepoOwner {
    pub login: String,
    #[serde(rename = "avatarUrl")]
    pub avatar_url: String,
}
