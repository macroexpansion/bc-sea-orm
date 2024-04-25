# BC ORM

## Generate entity with `sea-orm-cli`
```bash
cargo install sea-orm-cli
docker compose -f ./docker-compose.gen-sea-orm-entity.yaml up -d
sea-orm-cli generate entity -u postgres://username:password123@localhost:5432/bc -o src/entity --with-serde serialize
```
