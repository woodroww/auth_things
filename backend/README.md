sudo docker build --tag yogamat . 
sudo docker run -d yogamat
sudo docker stop container_id

## OAuth crate
https://crates.io/crates/oauth2
https://docs.rs/oauth2/4.3.0/oauth2/
https://github.com/ramosbugs/oauth2-rs

## FusionAuth
https://fusionauth.io/docs/v1/tech/oauth/endpoints

### why doesn't this work
sudo docker run -env APP_APPLICATION__CLIENT_ID -env APP_APPLICATION__CLIENT_SECRET -d yogamat

sudo docker run --env-file ./.env -d yogamat
