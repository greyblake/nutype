//! Tools that facilitates reporting issues on Github.
//! With some refactoring it can be extracted into its own crate.

pub fn build_github_link_with_issue(issue: &Issue) -> String {
    let builder = GithubIssueBuilder::new("greyblake/nutype");
    let url = builder.render_url(issue);
    format!("\nClick the following link to report the issue:\n\n{url}\n\n")
}

pub enum Issue {
    ArbitraryGeneratedInvalidValue { inner_type: String },
}

struct GithubIssueBuilder {
    // Repo ID on Github, for example "greyblake/nutype"
    repo_path: String,
}

impl GithubIssueBuilder {
    fn new(repo_path: &str) -> Self {
        Self {
            repo_path: repo_path.to_owned(),
        }
    }

    fn render_url(&self, issue: &Issue) -> String {
        let RenderedIssue { title, body } = render_issue(issue);

        let encoded_title = urlencoding::encode(&title);
        let encoded_body = urlencoding::encode(&body);
        let repo_path = &self.repo_path;
        format!(
            "https://github.com/{repo_path}/issues/new?title={encoded_title}&body={encoded_body}&labels=bug"
        )
    }
}

struct RenderedIssue {
    title: String,
    body: String,
}

fn render_issue(issue: &Issue) -> RenderedIssue {
    match issue {
        Issue::ArbitraryGeneratedInvalidValue {
            inner_type: type_name,
        } => RenderedIssue {
            title: format!("Arbitrary generates an invalid value for {}", type_name),
            body: "
Having my type defined as:

```rs
// Put the definition of your type with #[nutype] macro here
```

I got a panic when I tried to generate a value with Arbitrary.
"
            .to_owned(),
        },
    }
}
