#!/bin/bash
# https://www.digitalocean.com/community/tutorials/how-to-install-and-use-postgresql-on-ubuntu-20-04

sudo apt-get -y install postgresql postgresql-contrib libpq-dev
sudo -i -u postgres
createuser keypost --superuser
createdb keypost
exit
sudo adduser keypost
psql -d keypost
# ALTER ROLE keypost WITH PASSWORD 'changeme';

# https://diesel.rs/guides/getting-started
cargo install diesel_cli --no-default-features --features postgres --verbose
export KEYPOST_DATABASE_PSSWD=changme
export DATABASE_URL=postgres://keypost:${KEYPOST_DATABASE_PSSWD}@localhost/keypost
diesel setup