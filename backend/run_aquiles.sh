export APP_ENVIRONMENT=aquiles
# use fusion
export APP_APPLICATION__CLIENT_ID=4462657c-8ced-48a6-8aac-08bdbf3423f9
export APP_APPLICATION__CLIENT_SECRET=glVBoidvxgYjNDg83OSgM-_RXQv1lhAuiI3Kxph--Ys

export POSTGRES_USER=matt
export POSTGRES_PASSWORD=password
export POSTGRES_PORT=5432
export POSTGRES_DB=yogamat
export DATABASE_URL=postgres://${POSTGRES_USER}:${POSTGRES_PASSWORD}@aquiles.local:${POSTGRES_PORT}/${POSTGRES_DB}

cargo run --bin server
