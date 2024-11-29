.PHONY: docker-images docker-image-linux docker-image-osx

DOCKER_IMAGE := IntelliXLabs/iwasm
DOCKER_TAG := 0.0.1
USER_ID := $(shell id -u)
USER_GROUP := $(shell id -g)

docker-image-alpine:
	docker build . -t $(DOCKER_IMAGE):$(DOCKER_TAG)-alpine -f docker/Dockerfile.alpine

docker-image-linux:
	docker build . -t $(DOCKER_IMAGE):$(DOCKER_TAG)-linux -f docker/Dockerfile.linux

docker-image-osx:
	docker build . -t $(DOCKER_IMAGE):$(DOCKER_TAG)-osx -f docker/Dockerfile.osx

docker-images: 
	make docker-image-osx 
	make docker-image-linux 
	make docker-image-alpine

# docker-publish:
# 	docker push $(DOCKER_IMAGE):$(DOCKER_TAG)-alpine
# 	docker push $(DOCKER_IMAGE):$(DOCKER_TAG)-linux
# 	docker push $(DOCKER_IMAGE):$(DOCKER_TAG)-osx

# Creates a release build in a containerized build environment of the static library for Alpine Linux (.a)
release-alpine:
	rm -rf lib_iwasm/target/release
	rm -rf lib_iwasm/target/x86_64-unknown-linux-musl/release
	docker run --rm -u $(USER_ID):$(USER_GROUP) -v $(shell pwd):/code/iwasm $(DOCKER_IMAGE):$(DOCKER_TAG)-alpine build_alpine.sh

# Creates a release build in a containerized build environment of the shared library for glibc Linux (.so)
release-linux:
	rm -rf lib_iwasm/target/release
	rm -rf lib_iwasm/target/x86_64-unknown-linux-gnu/release
	docker run --rm -u $(USER_ID):$(USER_GROUP) -v $(shell pwd):/code/iwasm $(DOCKER_IMAGE):$(DOCKER_TAG)-linux build_linux.sh

# Creates a release build in a containerized build environment of the shared library for macOS (.dylib)
release-osx:
	rm -rf lib_iwasm/target/release
	rm -rf lib_iwasm/target/x86_64-apple-darwin/release
	rm -rf lib_iwasm/target/aarch64-apple-darwin/release
	docker run --rm -u $(USER_ID):$(USER_GROUP) -v $(shell pwd):/code/iwasm $(DOCKER_IMAGE):$(DOCKER_TAG)-osx build_osx.sh

releases:
	make release-alpine
	make release-linux
	make release-osx