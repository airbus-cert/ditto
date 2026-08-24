NIX_CMD=nix develop --command bash -c

.PHONY: build-py
build-py:
	$(NIX_CMD) "cd pyditto && maturin build --profile release"

.PHONY: publish-py
publish-py: build-py
	$(NIX_CMD) "cd pyditto && maturin publish --profile release"
