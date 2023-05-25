Reddit Exporter
===

Simple Rust project to login to a Reddit account and export its saved
posts/comments in a JSON format understood by ArchiveBox's GenericJson parser.

Setup
```console
# Install Rust - https://rustup.rs
cargo build --release
```

Usage with ArchiveBox:

```console
echo "<PASSWORD>" | target/release/reddit-export -u <USERNAME> --stdin -v > reddit_saved.json

archivebox add --update --depth 0 --parser json < reddit_saved.json
# Or in K8s
< reddit_saved.json kubectl --context <K8S_CONTEXT> -n archivebox exec -i archivebox-0 -- su archivebox -c "archivebox add --update --depth 0 --parser json"
```
