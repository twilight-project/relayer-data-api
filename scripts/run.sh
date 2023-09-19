#!/bin/bash
set -e

function exit_now() {
	exit 0
}

trap exit_now SIGINT SIGKILL SIGABRT

if [ $1 == "archiver" ]
then
	/app/archiver
elif [ $1 == "api" ]
then
	/app/api
elif [ $1 == "auth" ]
then
	/app/auth
else
	echo "Invalid command: ${1}"
fi
