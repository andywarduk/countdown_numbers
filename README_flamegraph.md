# cargo flamegraph invocation

## solve

### release build

```sh
CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph --open --bin solve -- 888 100 75 50 25 10 9
```

### debug build

```sh
cargo flamegraph --open --bin solve --dev -- 888 100 75 50 25 10 9
```
