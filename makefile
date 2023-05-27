binary_name = shapeless-blog
remote_host = jacky@drshapeless.com

.PHONY: build run
build:
	cargo build --release

run:
	./target/release/shapeless-blog

deploy: build
	scp ./target/release/$(binary_name) $(remote_host)':shapeless-blog/'
	scp ./init/shapeless-blog.init $(remote_host)':shapeless-blog/'
	ssh $(remote_host) 'zsh' < ./scripts/deploy.sh

build_and_run: build run

.PHONY: add_test_token
add_test_token:
	psql shapeless-blog -c "INSERT INTO tokens (user_id, token, expired_time) VALUES (1, 'verygoodtoken' , CURRENT_TIMESTAMP + INTERVAL '1 year')"

.PHONY: sqlx sqlx/migrate sqlx/init sqlx/drop sqlx/reset
sqlx:
	psql shapeless-blog

sqlx/migrate:
	sqlx migrate add -r $(name)

sqlx/init:
	sqlx database create
	sqlx migrate run

sqlx/drop:
	yes | sqlx database drop

sqlx/reset: sqlx/drop sqlx/init add_test_token
