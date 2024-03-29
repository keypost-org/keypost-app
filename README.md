# Keypost App

#### This serves as the main user facing application for authenticated users. Security is the number one priority!

### Configuration
 - To override the default port, export `ROCKET_PORT`
 - Verbose mode for development, set `ROCKET_LOG` to `debug`
 - You can also consider adding a [`Rocket.toml` file](https://github.com/SergioBenitez/Rocket/blob/36c1570c614e3b9c1ff6a33f0ebd3c94b440e2cc/site/guide/9-configuration.md#rockettoml).

### Development
 - Run [db-init.sh](https://github.com/keypost-org/keypost-app/blob/master/scripts/db-init.sh)
 - To create a database migration, `diesel migration generate <name-of-db-actions-you-want-to-do>`
 - To run migration(s), `diesel migration run` (to run `down.sql` and then `up.sql`, run `diesel migration redo`)
 - To start with a clean database, run `diesel database reset`
 - Additional `diesel` documentation can be found [here](https://diesel.rs/guides/) and examples [here](https://github.com/diesel-rs/diesel/tree/master/examples/postgres)

## Web

### Setup
 - Built based on https://webpack.js.org/guides/getting-started/
```
cd static
npm install webpack webpack-cli --save-dev
npm install --save-dev html-webpack-plugin
npm install --save lodash
npm install --save-dev style-loader css-loader
```

### Build
```
npm run build
```

#### Docker
```
docker build . -t keypost-app:latest
docker run --env DATABASE_URL --env KEYPOST_DATABASE_PSSWD keypost-app:latest
```

### Deploy