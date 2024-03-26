from github import Auth, Github

import constants
from utils import clean_repo

auth = Auth.Token(constants.GH_ACCESS_TOKEN)

g = Github(auth=auth)

repo = g.get_repo(constants.REPO)

clean_repo(repo)
