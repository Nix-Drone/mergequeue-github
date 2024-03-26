import logging

from utils import create_test_pull_request, wait_until_check_run_passes

IMPACTED_TARGETS_CHECK_NAME = "Compute Impacted Targets"


class DemoPr:
    def __init__(self, repo, targets_to_impact, fail=False):
        self.repo = repo
        self.targets_to_impact = targets_to_impact
        self.fail = fail
        self.p_r = ""

    def create(self):
        logging.info(
            "Creating PR afftecting target%s %s",
            "s" if len(self.targets_to_impact) == 1 else "",
            ", ".join(self.targets_to_impact),
        )
        self.p_r = create_test_pull_request(self.repo, "main", self.targets_to_impact)
        logging.info("Created - PR#%s", self.p_r.number)

    def poll_until_targets_uploaded(self):
        if self.p_r == "":
            raise AssertionError(
                "create must be called on DemoPr before poll_until_targets_uploaded"
            )

        commit = self.repo.get_commit(sha=self.p_r.head.sha)

        logging.info(
            "Polling until %s completes on PR #%s (SHA: %s)",
            IMPACTED_TARGETS_CHECK_NAME,
            self.p_r.number,
            self.p_r.head.sha,
        )
        wait_until_check_run_passes(commit, IMPACTED_TARGETS_CHECK_NAME)
        logging.info(
            "%s completed on PR #%s (SHA: %s)",
            IMPACTED_TARGETS_CHECK_NAME,
            self.p_r.number,
            self.p_r.head.sha,
        )

    def enqueue(self):
        if self.p_r == "":
            raise AssertionError("create must be called on DemoPr before enqueue")
        logging.info("Enqueueing PR #%s with '/trunk merge' comment", self.p_r.number)
        self.p_r.create_issue_comment("/trunk merge")
