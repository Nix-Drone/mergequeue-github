
![light-complex (3)](https://github.com/trunk-io/mergequeue/assets/1265982/ded3489b-eef7-482f-b94f-0d944c1d93ce)

### Welcome

This repository is used to demonstrate the performance characteristics of a merge graph under different simulated loads. 

#### How does it work
The load imparted onto the connected queue is controlled by the `demo.toml` file in the .config folder. The PR Factory workflow
is set on a cron schedule to call trunk-pr-generator which is then responsible for generating the pull requests and enqueueing them.

The configuration system allows for setting the desired load on the queue, the flake rate and the interdependence element of the pull requests.

```toml
# parallelqueue - will push deps information to the service to take advantage of trunk merge dynamic parallel queues
# singlequeue - single traditional queueu
mode = "singlequeue"

# the frequency at which a pull request will fail under testing in the merge queue. flake_rate will not affect
# testing of the PR before it enters the queue
flake_rate = 0.01

# How long the PR should take to test. This is accomplished by sleeping 
sleep_for = "60s"

# the max number of deps in this repo. setting to 3 for example will cause
# files in alpha, bravo, charlie deps to be changed
max_deps = 2

# the max number of deps per PR to edit. this will control how interconnected prs
max_impacted_deps = 2

# the target number of pull requests to open and enqueue per hour
pull_requests_per_hour = 6
```
