import logging
import random
import re
import string

from tenacity import (
    retry,
    retry_if_not_result,
    stop_after_attempt,
    stop_after_delay,
    wait_random,
)

TEST_PR_PREFIX = "[Merge Test]"


def get_random_string(length):
    # choose from all lowercase letter
    letters = string.ascii_lowercase
    # trunk-ignore(bandit/B311)
    result_str = "".join(random.choice(letters) for i in range(length))
    return result_str


def create_test_pull_request(repo, source_branch, targets_to_impact):
    s_b = repo.get_branch(source_branch)
    t_b = f"branch_for_demo_{get_random_string(10)}"
    repo.create_git_ref(ref="refs/heads/" + t_b, sha=s_b.commit.sha)

    for target in targets_to_impact:
        repo.create_file(
            path=f"demo_workspace/target_{target}/files/{t_b}.txt",
            message="Create diff",
            content=get_random_string(100),
            branch=t_b,
        )

    pr_body = """
THIS PR IS MANAGED BY THE TRUNK MERGE DEMO SCRIPT.

It was openend for the purpose of demonstrating the merge graph, and either will be merged or eventually closed.

To force close this PR, run `python3 ./clean_repo.py
    """

    p_r = repo.create_pull(
        title=f"{TEST_PR_PREFIX} - PR impacting targets {', '.join(targets_to_impact)}",
        body=pr_body,
        head=t_b,
        base="main",
    )
    return p_r


@retry(
    wait=wait_random(min=5, max=10),
    stop=(stop_after_attempt(10) | stop_after_delay(90)),
    reraise=True,
    retry=retry_if_not_result(lambda result: result is True),
)
def wait_until_check_run_passes(commit, check_name):
    check_runs = commit.get_check_runs(check_name="Compute Impacted Targets")
    if check_runs.totalCount == 0:
        logging.debug(
            "No checks with name %s found on commit %s. Retrying.",
            check_name,
            commit.sha,
        )
        return False
    if check_runs.totalCount > 1:
        logging.error(
            "Too many check runs found on commit %s with name %s. Aborting.",
            commit.sha,
            check_name,
        )
        raise AssertionError(
            f"Too many check runs found for commit {commit.sha} with name {check_name}"
        )
    check_run = check_runs[0]
    if check_run.conclusion == "failure":
        raise AssertionError(
            f"Check run {check_name} failed when it should have passed"
        )

    logging.info(
        "%s status for commit %s = %s", check_name, commit.sha, check_run.conclusion
    )
    return check_run.conclusion == "success"


def get_content_file_paths_recursively(repo, dir):
    file_paths = []
    contents = repo.get_contents(dir)
    while contents:
        file_content = contents.pop(0)
        if file_content.type == "dir":
            contents.extend(repo.get_contents(file_content.path))
        else:
            file_paths.append(file_content.path)
    return file_paths


def clean_repo(repo):
    open_prs = repo.get_pulls(state="open")
    logging.info("Closing all PRs opened by test scripts on %s", repo.full_name)
    for p_r in open_prs:
        if p_r.title.startswith(TEST_PR_PREFIX):
            p_r.edit(state="closed")
    logging.info("Done closing open PRs on repo")

    logging.info("Resetting repo")

    # file_paths = get_content_file_paths_recursively(repo, "demo_workspace")
    # file_deletions = []
    # for file in file_paths:
    #     if re.search("target_")
