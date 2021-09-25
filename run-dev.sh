docker-compose up -d db
systemfd --no-pid -s http::8080 -- cargo watch -x run
