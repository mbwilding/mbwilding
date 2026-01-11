mod types;

use anyhow::{Context, Result};
use log::debug;
pub use types::*;

// GraphQL query constants
const MAX_PER_PAGE: u32 = 100;
const MAX_LANGUAGES_PER_REPO: u32 = 100;
const AVATAR_SIZE: u32 = 32;

/// Response type for the initial query (contains base user info + first page of data)
#[derive(Debug, serde::Deserialize)]
struct InitialUserData {
    viewer: Option<InitialUser>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct InitialUser {
    login: String,
    contributions_collection: ContributionsCollection,
    repositories: RepositoryConnection,
    pull_requests: PullRequestConnection,
    issues: IssueConnection,
    merged_pull_requests: MergedPullRequestConnection,
}

/// Response type for paginated repository queries
#[derive(Debug, serde::Deserialize)]
struct RepoPageData {
    viewer: Option<RepoPageViewer>,
}

#[derive(Debug, serde::Deserialize)]
struct RepoPageViewer {
    repositories: RepositoryConnection,
}

/// Response type for paginated merged PR queries
#[derive(Debug, serde::Deserialize)]
struct MergedPRPageData {
    viewer: Option<MergedPRPageViewer>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct MergedPRPageViewer {
    merged_pull_requests: MergedPullRequestConnection,
}

/// Fetch GitHub user data via GraphQL API
pub async fn fetch_user_data(
    client: &reqwest::Client,
    token: &str,
    include_private: bool,
) -> Result<UserData> {
    debug!("Fetching user data (include_private: {})", include_private);
    let privacy = if include_private {
        ""
    } else {
        ", privacy: PUBLIC"
    };

    // Fetch initial page with all data
    let initial_data = fetch_initial_page(client, token, &privacy).await?;
    let initial_user = initial_data.viewer.context("No viewer in response")?;

    // Paginate repositories if needed
    let all_repos = paginate_repositories(
        client,
        token,
        &privacy,
        initial_user.repositories.nodes,
        initial_user.repositories.page_info,
    )
    .await?;

    // Paginate merged PRs if needed
    let (all_merged_prs, merged_pr_total_count) = paginate_merged_prs(
        client,
        token,
        initial_user.merged_pull_requests.nodes,
        initial_user.merged_pull_requests.page_info,
        initial_user.merged_pull_requests.total_count,
    )
    .await?;

    // Construct the final UserData
    Ok(UserData {
        viewer: Some(User {
            login: initial_user.login,
            contributions_collection: initial_user.contributions_collection,
            repositories: RepositoryConnection {
                nodes: all_repos,
                page_info: PageInfo {
                    has_next_page: false,
                    end_cursor: None,
                },
            },
            pull_requests: initial_user.pull_requests,
            issues: initial_user.issues,
            merged_pull_requests: MergedPullRequestConnection {
                total_count: merged_pr_total_count,
                nodes: all_merged_prs,
                page_info: PageInfo {
                    has_next_page: false,
                    end_cursor: None,
                },
            },
        }),
    })
}

/// Fetch the initial page of data
async fn fetch_initial_page(
    client: &reqwest::Client,
    token: &str,
    privacy: &str,
) -> Result<InitialUserData> {
    let query = format!(
        r#"
        query {{
            viewer {{
                login
                contributionsCollection {{
                    totalCommitContributions
                    restrictedContributionsCount
                }}
                repositories(first: {max_per_page}, ownerAffiliations: OWNER{privacy}, orderBy: {{field: STARGAZERS, direction: DESC}}) {{
                    nodes {{
                        stargazerCount
                        forkCount
                        isFork
                        languages(first: {max_languages}, orderBy: {{field: SIZE, direction: DESC}}) {{
                            edges {{
                                size
                                node {{
                                    name
                                    color
                                }}
                            }}
                        }}
                    }}
                    pageInfo {{
                        hasNextPage
                        endCursor
                    }}
                }}
                pullRequests(first: 1) {{
                    totalCount
                }}
                issues(first: 1) {{
                    totalCount
                }}
                mergedPullRequests: pullRequests(first: {max_per_page}, states: MERGED, orderBy: {{field: CREATED_AT, direction: DESC}}) {{
                    totalCount
                    nodes {{
                        repository {{
                            name
                            owner {{
                                login
                                avatarUrl(size: {avatar_size})
                            }}
                            stargazerCount
                        }}
                    }}
                    pageInfo {{
                        hasNextPage
                        endCursor
                    }}
                }}
            }}
        }}
    "#,
        max_per_page = MAX_PER_PAGE,
        privacy = privacy,
        max_languages = MAX_LANGUAGES_PER_REPO,
        avatar_size = AVATAR_SIZE
    );

    execute_query(client, token, &query).await
}

/// Paginate through all repositories
async fn paginate_repositories(
    client: &reqwest::Client,
    token: &str,
    privacy: &str,
    mut all_repos: Vec<Repository>,
    mut page_info: PageInfo,
) -> Result<Vec<Repository>> {
    while page_info.has_next_page {
        let cursor = page_info
            .end_cursor
            .as_ref()
            .context("Missing cursor for next page")?;
        debug!("Fetching next page of repositories (cursor: {})", cursor);

        let query = format!(
            r#"
            query {{
                viewer {{
                    repositories(first: {max_per_page}, after: "{cursor}", ownerAffiliations: OWNER{privacy}, orderBy: {{field: STARGAZERS, direction: DESC}}) {{
                        nodes {{
                            stargazerCount
                            forkCount
                            isFork
                            languages(first: {max_languages}, orderBy: {{field: SIZE, direction: DESC}}) {{
                                edges {{
                                    size
                                    node {{
                                        name
                                        color
                                    }}
                                }}
                            }}
                        }}
                        pageInfo {{
                            hasNextPage
                            endCursor
                        }}
                    }}
                }}
            }}
        "#,
            max_per_page = MAX_PER_PAGE,
            cursor = cursor,
            privacy = privacy,
            max_languages = MAX_LANGUAGES_PER_REPO
        );

        let page_data: RepoPageData = execute_query(client, token, &query).await?;
        let viewer = page_data
            .viewer
            .context("No viewer in paginated response")?;

        all_repos.extend(viewer.repositories.nodes);
        page_info = viewer.repositories.page_info;

        debug!("Fetched {} total repositories so far", all_repos.len());
    }

    debug!("Finished fetching all {} repositories", all_repos.len());
    Ok(all_repos)
}

/// Paginate through all merged pull requests
async fn paginate_merged_prs(
    client: &reqwest::Client,
    token: &str,
    mut all_prs: Vec<MergedPullRequest>,
    mut page_info: PageInfo,
    total_count: u32,
) -> Result<(Vec<MergedPullRequest>, u32)> {
    while page_info.has_next_page {
        let cursor = page_info
            .end_cursor
            .as_ref()
            .context("Missing cursor for next page")?;
        debug!("Fetching next page of merged PRs (cursor: {})", cursor);

        let query = format!(
            r#"
            query {{
                viewer {{
                    mergedPullRequests: pullRequests(first: {max_per_page}, after: "{cursor}", states: MERGED, orderBy: {{field: CREATED_AT, direction: DESC}}) {{
                        nodes {{
                            repository {{
                                name
                                owner {{
                                    login
                                    avatarUrl(size: {avatar_size})
                                }}
                                stargazerCount
                            }}
                        }}
                        pageInfo {{
                            hasNextPage
                            endCursor
                        }}
                    }}
                }}
            }}
        "#,
            max_per_page = MAX_PER_PAGE,
            cursor = cursor,
            avatar_size = AVATAR_SIZE
        );

        let page_data: MergedPRPageData = execute_query(client, token, &query).await?;
        let viewer = page_data
            .viewer
            .context("No viewer in paginated response")?;

        all_prs.extend(viewer.merged_pull_requests.nodes);
        page_info = viewer.merged_pull_requests.page_info;

        debug!("Fetched {} total merged PRs so far", all_prs.len());
    }

    debug!("Finished fetching all {} merged PRs", all_prs.len());
    Ok((all_prs, total_count))
}

/// Execute a GraphQL query and return the parsed response
async fn execute_query<T: serde::de::DeserializeOwned>(
    client: &reqwest::Client,
    token: &str,
    query: &str,
) -> Result<T> {
    let body = serde_json::json!({
        "query": query
    });

    debug!("Sending GraphQL request to GitHub API");

    let response: GraphQLResponse<T> = client
        .post("https://api.github.com/graphql")
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "github-stats-generator")
        .json(&body)
        .send()
        .await?
        .json()
        .await?;

    if let Some(errors) = response.errors {
        let messages: Vec<_> = errors.iter().map(|e| e.message.as_str()).collect();
        anyhow::bail!("GraphQL errors: {}", messages.join(", "));
    }

    debug!("Successfully fetched data from GitHub API");

    response.data.context("No data in response")
}
