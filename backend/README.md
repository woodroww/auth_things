## OAuth crate
https://crates.io/crates/oauth2
https://docs.rs/oauth2/4.3.0/oauth2/
https://github.com/ramosbugs/oauth2-rs

## FusionAuth
https://fusionauth.io/docs/v1/tech/oauth/endpoints

## Docker fun
### build it
sudo docker build --tag yogamat . 
### run
#### why doesn't this work
sudo docker run -env APP_APPLICATION__CLIENT_ID -env APP_APPLICATION__CLIENT_SECRET -d yogamat
#### but this works
sudo docker run --env-file ./.env -d -p 3000:3000 yogamat
### stop a running container
sudo docker stop container_id
