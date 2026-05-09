we want to implement a storage system for this app, so update items can be stored in a pluggable storage backend.
This will be used  , among other things, for sending desktop notifications on changes.

do the tasks below. for each task, use a separate subagent to accomplish the goal.
you purely aare the coordinator driving subagents

instruct subagent to finish changes, then make sure project compiles
(cargo check -F desktop) and run `cargo fmt`.
Then they should commit usiing jj (NOT GIT!!!). `jj st` to show status,
`jj commit -m 'msg'` to commit, with a descriptive, good commit message.

- 1. define a canonical data model in src/store/types.rs .
     it should have definitions for generic items. the two currently defined
     items reflect what is used, github review requests and users github pull
     requests. should hava generic item enum with variants for the two existing
     items, though we will add more in the future.
     note: items may retrieve additional data, like "ignore", "ignore_until", etc,
     so prepare for that in the data model. items should also have a retrieved_at field denoting
     when data was fetched.dioxus_sdk_notification. probaly want struct Item
     and enum ItemKind {}

- 2. refactor home.rs to use the canonical data model instead. the data retrieved
     from the github client must be mapped into the canonical model immediately
     after retrieval

- 3. in store/mod.rs, implement a Store trait using async-trait, for now just
     two methods, one for for loading persisted items, and one writing stored
     items (Vec<Item> in or out)

- 4. in store/fs.rs, implement an implementation for the store trait,
     that simply stores data in jsonline files. storage dir should be a config
     param for the store, but should default to platform-native default dirs,
     like XDG on linux (application name: devbuddy)

- 5. update home.rs to first load data from storage, then trigger a refetch
     (but show stored data right away) . when data is loaded, persist back to
     storage . check if anything actually changed before storing!

- 6. implement automatic refetches on a timer. declare constant,
     use 90 seconds for now. coroutine can spawn a timer! also , ui should show
     data age next to refresh button. make sure code in the coroutine is clean
     for either user-triggered or timer triggered fetches.

- 7. using dioxus_sdk_notification, implement desktop notifications on
     item changes / new items / items removed. use dioxus_sdk_notification crate. 

