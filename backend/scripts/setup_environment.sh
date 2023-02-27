# source this file
export POSTGRES_USER=postgres
export POSTGRES_PASSWORD=password
export POSTGRES_PORT=5431
export POSTGRES_DB=users
export DATABASE_URL=postgres://${POSTGRES_USER}:${POSTGRES_PASSWORD}@localhost:${POSTGRES_PORT}/${POSTGRES_DB}
