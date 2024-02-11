### `json_viewer`, TUI-based JSON Viewer

In the future it's expected that everyone can expand it according to their needs with using AKA commands.

Examples of usage:
```
echo '{ "ticket": { "state": { "list": [ { "id": 3391511, "customer_id": 3783698 } ] } }}' | \
    cargo r --
```
```
tail -n 1 ./issues-2402110451.log | awk '{ split($0, a, " log: "); print a[2] }' | \
    ./target/debug/json_viewer
```
