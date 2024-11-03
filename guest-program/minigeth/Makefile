SHELL := /bin/bash

build: minigeth_mips minigeth_prefetch mipsevm
.PHONY: build

minigeth_mips:
	chmod +x mipsevm/build.sh
	cd mipsevm && ./build.sh
.PHONY: minigeth_mips

minigeth_prefetch:
	cd minigeth && go build
.PHONY: minigeth_prefetch

mipsevm:
	cd mipsevm && go build
.PHONY: mipsevm

# contracts: nodejs
#	npx hardhat compile
# .PHONY: contracts

#nodejs:
#	if [ -x "$$(command -v pnpm)" ]; then \
#		pnpm install; \
#	else \
#		npm install; \
#	fi
#.PHONY: nodejs

# Must be a definition and not a rule, otherwise it gets only called once and
# not before each test as we wish.
define clear_cache
	rm -rf /tmp/cannon
	mkdir -p /tmp/cannon
endef

clear_cache:
	$(call clear_cache)
.PHONY: clear_cache

test_challenge:
	$(call clear_cache)
	# Build preimage cache for block 13284469
	minigeth/go-ethereum 13284469
	# Generate initial (generic) MIPS memory checkpoint and final checkpoint for
	# block 13284469.
	mipsevm/mipsevm 13284469
	#npx hardhat test test/challenge_test.js
.PHONY: test_challenge

test_minigeth:
	$(call clear_cache)
	# Check that minigeth is able to validate the given transactions.
	# run block 13284491 (0 tx)
	minigeth/go-ethereum 13284491
	# run block 13284469 (few tx)
	minigeth/go-ethereum 13284469
	# block 13284053 (deletion)
	minigeth/go-ethereum 13284053
	# run block 13303075 (uncles)
	minigeth/go-ethereum 13303075
.PHONY: test_minigeth

#test_contracts:
#	$(call clear_cache)
#	npx hardhat test
#.PHONY: test_contracts

test: test_challenge test_minigeth
.PHONY: test

clean:
	rm -f minigeth/go-ethereum
	rm -f mipsevm/mipsevm
	rm -f mipsevm/minigeth
	rm -rf artifacts
.PHONY: clean

mrproper: clean
	rm -rf cache
	rm -rf node_modules
	rm -rf mipigo/venv
	rm -rf unicorn/build
.PHONY:  mrproper
