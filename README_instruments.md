# cargo instruments invocation

## solve

### time template, release build

```sh
cd solve
cargo instruments --template "time" --bin solve --release -- 888 100 75 50 25 10 9
```

### alloc template, release build

```sh
cd solve
cargo instruments --template "alloc" --bin solve --release -- 888 100 75 50 25 10 9
```

## solve_all

### time template, release build with 5 second timeout

```sh
cd solve_all
cargo instruments --template "time" --bin solve_all --release --time-limit 5000
```
