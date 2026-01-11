mod types;

use anyhow::{Context, Result};
pub use types::*;

// GraphQL query constants
const MAX_REPOS: u32 = 100;
const MAX_LANGUAGES_PER_REPO: u32 = 10;
const MAX_MERGED_PRS: u32 = 100;
const AVATAR_SIZE: u32 = 32;

/// Fetch GitHub user data via GraphQL API
pub async fn fetch_user_data(
    client: &reqwest::Client,
    token: &str,
    include_private: bool,
) -> Result<UserData> {
    let privacy = if include_private {
        ""
    } else {
        ", privacy: PUBLIC"
    };

    let query = format!(
        r#"
        query {{
            viewer {{
                login
                contributionsCollection {{
                    totalCommitContributions
                    restrictedContributionsCount
                }}
                repositories(first: {}, ownerAffiliations: OWNER{}, orderBy: {{field: STARGAZERS, direction: DESC}}) {{
                    nodes {{
                        stargazerCount
                        forkCount
                        isFork
                        languages(first: {}, orderBy: {{field: SIZE, direction: DESC}}) {{
                            edges {{
                                size
                                node {{
                                    name
                                    color
                                }}
                            }}
                        }}
                    }}
                }}
                pullRequests(first: 1) {{
                    totalCount
                }}
                issues(first: 1) {{
                    totalCount
                }}
                mergedPullRequests: pullRequests(first: {}, states: MERGED, orderBy: {{field: CREATED_AT, direction: DESC}}) {{
                    totalCount
                    nodes {{
                        repository {{
                            name
                            owner {{
                                login
                                avatarUrl(size: {})
                            }}
                            stargazerCount
                        }}
                    }}
                }}
            }}
        }}
    "#,
        MAX_REPOS, privacy, MAX_LANGUAGES_PER_REPO, MAX_MERGED_PRS, AVATAR_SIZE
    );

    let body = serde_json::json!({
        "query": query
    });

    let response: GraphQLResponse<UserData> = client
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

    response.data.context("No data in response")
}
