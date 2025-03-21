# Dev notes

* Some repositories are both template repositories and child repositories
  * Examples
    * `rust-public-lib-template`
* Some files must always be `git checkout --ours` during the merge
  * Examples
    * `README.md`
    * `Cargo.toml`
    * `Cargo.lock`
* Some files may not be present in the child repository
  * Examples
    * `Cargo.lock`
* Some child repositories have different branch naming rules
  * Examples
    * `~/workspace/personal` has `master` as the primary branch
    * `~/workspace/standard-traits` has `main` as the primary branch
  * Solutions
    * If target branch doesn't exist, exit with an error (don't create a new branch)
    * Autodetect the primary branch
    * Introduce `repoconf.toml` with a full configuration (which branch to merge into which)
