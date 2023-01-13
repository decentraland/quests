# Database

The database would be used from more than one service, so here we have some abstractions to reuse queries and setups. 

### Install sqlx-cli:

```bash
  $ cargo install sqlx-cli --no-default-features --features native-tls,postgres
```

### Crates

- db-core: Definitions and traits
- db-migrations: Migrations and run scripts
- db-sqlx: Implementation of db-core traits

