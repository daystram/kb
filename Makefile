CARGO:=$(shell which cargo)

.PHONY: build
build:
	${CARGO} build

.PHONY: lint
lint:
	${CARGO} clippy

.PHONY: run_u2f
run_u2f:
	${CARGO} run_u2f

.PHONY: run_probe_left
run_probe_left:
	${CARGO} run_probe -- --probe ${LEFT_PROBE}

.PHONY: run_probe_right
run_probe_right:
	${CARGO} run_probe -- --probe ${RIGHT_PROBE}
