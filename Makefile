.PHONY flamegraph-view
flamegraph-view:
	perf script -i perf.data | inferno-collapse-perf | flamelens

.PHONY flamegraph
flamegraph:
	cargo flamegraph