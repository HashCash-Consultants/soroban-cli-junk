# Contributing to soroban-cli

Thanks for taking the time to improve soroban-cli!

The following is a set of guidelines for contributions and may change over time.
Feel free to suggest improvements to this document in a pull request.We want to make it as easy as possible to contribute changes that help the Hcnet network grow and
thrive. There are a few guidelines that we ask contributors to follow so that we can merge your
changes quickly.

## Getting Started

* Make sure you have a [GitHub account](https://github.com/signup/free).
* Create a GitHub issue for your contribution, assuming one does not already exist.
  * Clearly describe the issue including steps to reproduce if it is a bug.
* Fork the repository on GitHub.

### Minor Changes

#### Documentation

For small changes to comments and documentation, it is not
always necessary to create a new GitHub issue. In this case, it is
appropriate to start the first line of a commit with 'doc' instead of
an issue number.

## Finding things to work on

The first place to start is always looking over the current GitHub issues for the project you are
interested in contributing to. Issues marked with [help wanted][help-wanted] are usually pretty
self-contained and a good place to get started.

Hcnet.org also uses these same GitHub issues to keep track of what we are working on. If you see
any issues that are assigned to a particular person or have the `in progress` label, that means
someone is currently working on that issue this issue in the next week or two.

Of course, feel free to create a new issue if you think something needs to be added or fixed.


## Making Changes

* Fork the soroban-cli repo to your own Github account

* List the current configured remote repository for your fork. Your git remote
should initially look like this. 
   ```
   $ git remote -v
   > origin  https://github.com/YOUR_USERNAME/soroban-cli.git (fetch)
   > origin  https://github.com/YOUR_USERNAME/soroban-cli.git (push)
   ```

* Set the `hcnet/soroban-cli` repo as new remote upstream repository that will
sync with your fork. 
  ```
  git remote add upstream https://github.com/hcnet/soroban-cli.git
  ```

* Verify the new upstream repository you've specified for your fork.
  ```
  $ git remote -v
  > origin    https://github.com/YOUR_USERNAME/soroban-cli.git (fetch)
  > origin    https://github.com/YOUR_USERNAME/soroban-cli.git (push)
  > upstream  https://github.com/hcnet/soroban-cli.git (fetch)
  > upstream  https://github.com/hcnet/soroban-cli.git (push)
  ```

* Add git hooks for commits and pushes so that checks run before pushing:
  ```
  ./install_githooks.sh
  ```

* Create a topic branch for your changes in your local repo. When you push you should be able
to create PR based on upstream hcnet/soroban-cli.

* Make sure you have added the necessary tests for your changes and make sure all tests pass.


## Submitting Changes

* All content, comments, pull requests and other contributions must comply with the
  [Hcnet Code of Conduct][coc].
* Push your changes to a topic branch in your fork of the repository.
* Submit a pull request to the repo in the Hcnet organization.
  * Include a descriptive [commit message][commit-msg].
  * Changes contributed via pull request should focus on a single issue at a time.
  * Rebase your local changes against the master branch. Resolve any conflicts that arise.


At this point you're waiting on us. We like to at least comment on pull requests within three
business days (typically, one business day). We may suggest some changes, improvements or
alternatives.

# Additional Resources

* #dev-discussion channel on [Discord](https://discord.gg/BYPXtmwX)

This document is inspired by:

[help-wanted]: https://github.com/hcnet/soroban-cli/contribute 
[commit-msg]: https://github.com/erlang/otp/wiki/Writing-good-commit-messages
[coc]: https://github.com/hcnet/.github/blob/master/CODE_OF_CONDUCT.md
