#!/bin/bash

### https://www.digitalocean.com/community/tutorials/how-to-install-and-use-postgresql-on-ubuntu-20-04

echo "This script is to be executed manually per section as needed (Linux only)."
exit 1

sudo apt-get -y install postgresql postgresql-contrib libpq-dev
sudo -i -u postgres
createuser keypost --superuser
createdb keypost

###############################################################
### BELOW COMMANDS ARE TO BE EXECUTED MANUALLY (Linux only) ###
###############################################################

sudo adduser keypost --no-create-home --disabled-login
sudo -su keypost
psql -d keypost # ALTER ROLE keypost WITH PASSWORD 'change-me-to-something-secret';

### https://diesel.rs/guides/getting-started
cargo install diesel_cli --no-default-features --features postgres --verbose
export KEYPOST_DATABASE_PSSWD=change-me-to-something-secret
export DATABASE_URL=postgres://keypost:${KEYPOST_DATABASE_PSSWD}@localhost/keypost
diesel migration run

### Only run below if the migrations directory is empty:
diesel setup 

### Now you'll have to find the most current database schema to recreate the database:
diesel migration generate keypost_schema

### Paste the most recent schema into up.sql in the "migrations/<timestamp>_keypost_schema" directory then run:
diesel migration run

### Become the keypost user:
sudo -u keypost

### Login to the database and verify the tables are created:
psql -d keypost

### Start the app and verify its functionality:
cargo run

### NOTES
#  - "The diesel_migrations crate provides the embed_migrations! macro, allowing you to embed migration scripts in 
#     the final binary. Once your code uses it, you can simply include embedded_migrations::run(&db_conn) at the 
#     start of your main function to run migrations every time the application starts."