import logging
import time

from github import Auth, Github

import constants
from demo_pr import DemoPr

logging.basicConfig(level="INFO")

auth = Auth.Token(constants.GH_ACCESS_TOKEN)

g = Github(auth=auth)

repo = g.get_repo(constants.REPO)

LANE_COUNT = 3
ROWS_COUNT = 3

rows = []
for _ in range(ROWS_COUNT):
    row = []
    for y in range(LANE_COUNT):
        p_r = DemoPr(repo, [str(y + 1)])
        p_r.create()
        row.append(p_r)
    rows.append(row)

for row in rows:
    for pr in row:
        pr.poll_until_targets_uploaded()

for row in rows:
    for pr in row:
        pr.enqueue()
    time.sleep(5)

# clean_repo(repo)
