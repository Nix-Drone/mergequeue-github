use crate::process::gh;

pub struct GitHub;

impl GitHub {
    pub fn comment(pr: &str, body: &str) -> String {
        gh(&["pr", "comment", pr, "--body", body])
    }

    pub fn close(pr: &str) -> String {
        gh(&["pr", "close", pr])
    }
}
