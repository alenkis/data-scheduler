default:
    @just --choose

clean:
    @rm -rf ./data
    @rm -rf {{ mongo-export-file }}
    @docker-compose down
    @docker-compose up --force-recreate -d

start:
    @docker-compose up -d

# TODO: get from .env
mongo-uri := "mongodb://mongoadmin:secret@localhost:27017/mydatabase?authSource=admin"
mongo-collection := "products"
mongo-import-file := "seed.json"
mongo-export-file := "products.out.json"
postgres-host := "localhost"
postgres-port := "5432"
postgres-db := "products"
postgres-user := "postgres"
postgres-password := "secret"

seed-mongo:
    @echo "Seeding MongoDB"
    @mongoimport \
        --uri {{ mongo-uri }} \
        --collection {{ mongo-collection }} \
        --file {{ mongo-import-file }} \
        --jsonArray

# First 5 days based on seed.json
start := "2021-12-01T00:00:00Z"
end := "2023-12-06T00:00:00Z"

run:
    @echo "Building data-scheduler..."
    @cargo build --release
    @echo "Running data-scheduler...\n"
    @./target/release/data-scheduler --config=config.yml
