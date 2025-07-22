# rust_ogc_features_server

An OGC API server written in Rust.

## API Endpoints

The server will eventually adhere to the OGC API - Features specification. For now, it is a work in progress.
It is documented in the [OGC API - Features](https://www.ogc.org/standards/ogcapi-features/) website.

An OGC API - Features server is a RESTful API that allows you to manage and query geospatial data.

The server currently supports the following features:

- Connect and fetch data from a PostGIS database.
- Expose aggregated data as a GeoJSON feature collection.
- Expose individual pieces of data as a GeoJSON feature.
- Expose a Swagger UI for the API.


It exposes the following endpoints:

- `/`: Landing page for the API.
- `/conformance`: Information about the standards conformance.
- `/collections`: List of available feature collections.
- `/collections/{collection_id}`: Details of a specific feature collection.
- `/collections/{collection_id}/items`: GeoJSON features for a specific collection.
- `/collections/{collection_id}/items/{id}`: A single GeoJSON feature.
- `/swagger-ui`: Swagger UI for the API.

## Configuration

The application is configured through a `config.toml` file. Here is an example configuration file:

```toml
title = "my-ogc-server"
description = "An OGC API server written in Rust"
# The base URL of the server. This is used to generate the links in the API.
url_base = "http://localhost:3000"

# A list of feature collections to expose.
[collections.my_collection]
table = "my_table"
id_column = "id"
geometry_column = "geom"
properties = ["property1", "property2"]
```

The application also requires a `.env` file with the following mandatory variable to connect to the database:

```
DATABASE_URL=postgres://user:password@host:port/database
```

## How to Run

1.  Create a `config.toml` file.
2.  Create a `.env` file with the `DATABASE_URL`.
3.  Install the dependencies and run the application:

```bash
cargo run
```

The application will be available at `http://localhost:3000` by default. You can change the port by setting the `PORT` environment variable.

## How to Run with Docker

1.  Create a `config.toml` file.
2.  Create a `.env` file with the `DATABASE_URL`.
3.  Build and run the Docker container:

```bash
docker build -t rust_ogc_features_server .
docker run -p 3000:3000 --env-file .env rust_ogc_features_server
```

The application will be available at `http://localhost:3000`.
