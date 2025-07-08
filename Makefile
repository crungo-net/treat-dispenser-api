export $(shell xargs < .env)
IMAGE_NAME=treat-dispenser-api
REGISTRY=harbor.crungo.net/scout
TAG=dev
FULL_IMAGE=$(REGISTRY)/$(IMAGE_NAME):$(TAG)

.PHONY: build push run run-debug clean kill-containers

build:
	./build-image-local.sh $(FULL_IMAGE)

push: build
	docker push $(FULL_IMAGE)

run: build
	docker run ---rm $(FULL_IMAGE)

run-debug: kill-containers build
	docker run -it -p 3500:3500 -e DISPENSER_API_TOKEN=supersecret -e RUST_LOG=debug \
	--rm ${FULL_IMAGE}

run-latest-debug: kill-containers clean
	docker run -it -p 3500:3500 -e DISPENSER_API_TOKEN=supersecret -e RUST_LOG=debug \
	--rm ${REGISTRY}/${IMAGE_NAME}:latest

build-pi:
	./build-pi4-binary.sh

clean:
	- buildctl prune --all
	- docker rmi ${FULL_IMAGE}
	- docker rmi ${REGISTRY}/${IMAGE_NAME}:latest
	- docker image prune -f


kill-containers:
	@echo "Killing all containers..."
	docker ps -q | xargs -r docker kill