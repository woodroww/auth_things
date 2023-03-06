export APP_ENVIRONMENT=imac
source .env
export POSTGRES_USER=matt
export POSTGRES_PASSWORD=password
export POSTGRES_PORT=5431
export POSTGRES_DB=yogamat
export DATABASE_URL=postgres://${POSTGRES_USER}:${POSTGRES_PASSWORD}@aquiles.local:${POSTGRES_PORT}/${POSTGRES_DB}
cargo run --bin server
