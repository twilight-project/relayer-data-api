#!/bin/bash

DBID=$(docker ps -f name=test-db -q)

function kill_db() {
	docker stop ${DBID}
}

function start_db() {
	docker run --rm -d -p 5434:5432 --name test-db -e POSTGRES_USER=relayer -e POSTGRES_PASSWORD=relayer -e POSTGRES_DATABASE=test postgres
}

if [ -z "${DBID}" ]
then
	echo "Test db not up"
else
	echo "tdb up"
	kill_db
fi

start_db

timeout 90s bash -c "until docker exec test-db pg_isready ; do sleep 5 ; done"

DATABASE_URL=postgres://relayer:relayer@localhost:5434/test diesel database reset
