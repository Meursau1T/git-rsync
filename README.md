# git-rsync
## usage
Run this command in your project folder:
```
cargo run --manifest-path ~/your_path/rsync-git/Cargo.toml -- -u username@ip -l local_path -r remote_path
```
If you need preview changes, add `-p n` to the command.
