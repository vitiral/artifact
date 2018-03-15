.PHONY: echo build latest build-latest run repl reactor test
PWD=$(shell pwd)
VERSION=0.18.0
USER=$(shell whoami)
APP=elm
ME=sleepyfox

echo:
	@echo Docker hub username: $(ME)
	@echo Unix username: $(USER)
	@echo Application name: $(APP)

build:
	docker build --build-arg user=$(USER) \
	  -t $(ME)/$(APP):$(VERSION) .

latest:
	docker tag $(ME)/$(APP):$(VERSION) $(ME)/$(APP):latest

build-latest: build latest

run:
	docker run -it -p "8000:8000" \
	-v $(PWD):/home/$(APP) \
	$(ME)/$(APP):$(VERSION) bash

repl:
	docker run -it --rm \
	-v $(PWD):/home/$(APP) \
	$(ME)/$(APP):$(VERSION) elm-repl

reactor:
	docker run -it --rm -p 8000:8000 \
	-v $(PWD):/home/$(APP) \
	$(ME)/$(APP):$(VERSION) elm-reactor --address=0.0.0.0

test:
	docker run -it --rm \
	-v $(PWD):/home/$(APP) \
	$(ME)/$(APP):$(VERSION) ???

push: build-latest
	docker push $(ME)/$(APP):$(VERSION)
	docker push $(ME)/$(APP):latest
