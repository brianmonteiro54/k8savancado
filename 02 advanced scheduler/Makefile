SHELL := /bin/bash
CLUSTER ?= aula02
IMAGE := aula02-scheduler-demo:demo

.PHONY: kind build load setup deploy all clean

kind:
	cd kind && ./create-cluster.sh $(CLUSTER)

build:
	cd rust && docker build -t $(IMAGE) .

load:
	kind load docker-image $(IMAGE) --name $(CLUSTER)

setup:
	cd k8s && kubectl apply -f 00-namespace.yaml -f 01-priorityclasses.yaml && ./02-node-labels-and-taints.sh

deploy:
	cd k8s && kubectl apply -f 10-stable-app-deploy.yaml -f 11-noisy-canary.yaml -f 13-lowprio-filler.yaml -f 12-critical-api.yaml -f 14-besteffort-worker.yaml -f 15-burstable-worker.yaml -f 20-affinity-demo.yaml

all: kind build load setup deploy

clean:
	kind delete cluster --name $(CLUSTER) || true
