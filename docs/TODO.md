# TODO

## Workflow

* Pick the first unfinished (not checked off) task in docs/TODO.md
* Implement the task
* Validate with `cargo check --quiet --message-format=short`
* Run `cargo fmt`
* Check status with `jj st` and commit changes with `jj commit -m '...'`
* Check off the task item in docs/TODO.md

If instructed to work on multiple tasks:
Run as a sparse agent orchestrator.
Spawn a subagent that follows the workflow in docs/TODO.md for ONE task.
Instruct subagent to only give a minimal output of "DONE" when finished.
Repeatedly spawn subagents until they report that no more work is available.

## Tasks

- [x] Remove Home and blog from navbar

- [x] Improve settings gear icon used in navbar, current one sucks.
      Also, settings icon should always be visible, even on mobile bulma layouts

- [x] Extend src/store/mod.rs trait Store with a config load and write system
      Must define a config type in src/store/types.rs
      FS store should store config also in platform compatible dir, like
      .config/devbuddy on linux
      For now, only add a github_token: Option<String> config
      Should be used at startup in github client construction if specified.
      if not specified, should use auto-detection as currently done

- [x] improve store sharing:
      currently home.rs constructs a store.
      instead, the store should be initialized in main.rs and shared with
      the whole app through the dioxus context system 
       use_context_provider(|| mytype); +  let mut signal: Signal<mytype> = use_context();
       Define a type DynStore = Arc<dyn Store>; to use.
       define a use_store() helper function to easily retrieve the store, based on use_context();
       Then update home.rs to use the shared store!

- [x] extend views/settings.rs page with a github section
      should show : if token was auto-detected, show source of token
      (env var, Github cli ('gh auth token'), or custom config)
      this will require tracking where the token came from, and for the
      auto-detection to return the source
      define a GithubTokenSource enum with appropriate values
      use the use_store() helper to retrieve the store

- [x] extend the src/source/github.rs client with a simple method to validate
      the active token, eg by retrieving basic user context and returning it.

- [ ] Extend views/settings.rs page with a form to configure a custom github token
      place form in a new src/components/github_config_form.rs and use in settings
      view. must persist config through the store!
      use the use_store() helper to retrieve the store!
      the form should validate the token using the github client user context
      retrieval, and only persist the new config when token is valid!

- [ ] Extend github client in src/source/github.rs with a method to retrieve
      status for a specific CI run, want to know number of: total jobs, 
      in progress, failed, succeeded

- [ ] Implement monitoring of CI runs for PRs in home.rs
      if a PR is detected as having an active CI run,
      show it at the top, in a separate section,
      with a loading spinner (bulma button .is-loading), and a counter for
      succeeded, failed , in progress. use github client method to retrieve status
      active job runs should be monitored outside of the regular fetch update
      ticker, with a separate timer interval (defined as a separate constant)
      eg 15 seconds.
      
- [ ] better notifications: improve the notification system 
      in src/notify/mod.rs, define a 'trait Notifier' that abstracts
      notification sending
      add one implementor based on the already used dioxus_sdk_notification
      crate - see src/views/home.rs for usage, put it in a notification/sdk.rs 
      submodule!

      then define a type DynNotifier = Arc<dyn Notifier>, and a helper
      build_notifier() function in nofitication/mod.rs

- [ ] update home.rs to use the new abstract notifier system, see 
      src/notifiy/mod.rs and DynNotifier

- [ ] Extend the notification system in src/notify/mod.rs
      add an additional trait Notifier implementation for linux in notification/linux.rs
      it should use the notify-rust crate, which is already used internally
      by the dioxus-sdk-notification crate.
      but we want to expose richer semantics!

      extend to Notifier trait with semantics for: updating exisitng notifications,
      getting notified if notifications are dismissed, adding action buttons
      to notifications.
      These systems should be optional/additive over the baseline provided by the
      sdk Notifier impl

      The Notifier trait should have a way to check for what is supported or not,
      eg by returning a NotifierSupport struct with bool fields for various systems


- [ ] basic pr notificat

- [ ] better github pr CI run notifications 
      for the active PR CI runs tracked in home.rs, implement a notification mechanism
      when an active CI run is detected , send a notification
      if the Notifier backend supports it, update the notification when
      the status changes (fails, succeeds, when in progress update job counter (in progres, succeeded, failed)
      notifications when jobs finish (fail, 
