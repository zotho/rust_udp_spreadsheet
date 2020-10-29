# Rust UDP spreadsheet

## Usage
```bash
cargo run -- --help
```
Run tests:
```bash
cargo test -- --test-threads=1
```
Setup database URL:
```bash
cargo run -- --db mysql://zotho:zotho@localhost:3306/rust
```
Populate table with example data:
```bash
cargo run -- --populate
```

## GUI Usage example
In first instance:
```
DB: mysql://zotho:zotho@localhost:3306/rust
Bind socket: 127.0.0.1:10001
Connect socket: 127.0.0.1:10000
[*] Send
```

In second instance:
```
Bind socket: 127.0.0.1:10000
Connect socket: 127.0.0.1:10001
[*] Recieve
```

### Table
Start editing: double click or Enter
Save cell: Enter

Columns:
- number (INTEGER)
- text (TEXT)

Editable only in send mode