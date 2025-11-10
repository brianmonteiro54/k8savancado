IMAGE ?= aula07/mini-karpenter-rs:latest

.PHONY: build run deploy undeploy

build:
	docker build -t $(IMAGE) ./rust-controller

run:
	cargo run --release --manifest-path ./rust-controller/Cargo.toml

deploy:
	kubectl apply -f k8s/00-namespace.yaml
	kubectl apply -f k8s/01-serviceaccount.yaml
	kubectl apply -f k8s/02-clusterrole.yaml
	kubectl apply -f k8s/03-clusterrolebinding.yaml
	sed 's|aula07/mini-karpenter-rs:latest|$(IMAGE)|' k8s/04-deployment.yaml | kubectl apply -f -

undeploy:
	kubectl delete -f k8s/04-deployment.yaml --ignore-not-found
	kubectl delete -f k8s/03-clusterrolebinding.yaml --ignore-not-found
	kubectl delete -f k8s/02-clusterrole.yaml --ignore-not-found
	kubectl delete -f k8s/01-serviceaccount.yaml --ignore-not-found
	kubectl delete -f k8s/00-namespace.yaml --ignore-not-found
