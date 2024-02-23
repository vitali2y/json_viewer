### `json_viewer`

`TUI`-based `JSON` Viewer

In the future it's expected that everyone can expand it according to their needs with using AKA commands.

Examples of usage:

Testing of simple `JSON` record:
```
echo '{ "ticket": { "state": { "list": [ { "id": 3391511, "customer_id": 3783698 } ] } }}' | \
    cargo r --
```

Opening of all `JSON` records from issue file:
```
TS="2402211324"; cat issues-$TS.log | awk 'BEGIN { b = "[" } { split($0, a, " log: "); b = b""a[2]"," } END { print substr(b, 0, length(b) - 1)"]" }' | \
    ./target/debug/json_viewer
```

Opening just a second `JSON` record from the back of issue file:
```
TS="2402211324"; N=2; tail -n $N issues-$TS.log | head -n 1 | awk '{ split($0, a, " log: "); print a[2] }' | \
    ./target/debug/json_viewer
```
