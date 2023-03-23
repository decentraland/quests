#!/bin/bash

# replace this with your own protoc
protoc \
		--plugin=../node_modules/.bin/protoc-gen-ts_proto \
		--ts_proto_opt=esModuleInterop=true,returnObservable=false,outputServices=generic-definitions \
		--ts_proto_out="$(pwd)" -I="$(pwd)" \
		"$(pwd)/quests.proto"
