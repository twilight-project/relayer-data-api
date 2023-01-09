#!/bin/bash

# TODO: check if initted/wait
#docker run --rm -d -p 5434:5432 -e POSTGRES_USER=relayer -e POSTGRES_PASSWORD=relayer -e POSTGRES_DATABASE=test postgres


DATABASE_URL=postgres://relayer:relayer@localhost:5434/test diesel database reset
