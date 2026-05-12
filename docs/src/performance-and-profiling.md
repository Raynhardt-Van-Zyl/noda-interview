# Performance and Profiling

The detailed benchmark history lives in the repository root:

- [METRICS.md](../../METRICS.md)

Normal release builds use Cargo's default release settings. Profiling uses a
separate Cargo profile that keeps optimizations enabled and preserves line-table
debug info.

Generate a flamegraph:

```bash
scripts/profile.sh csv flamegraph
scripts/profile.sh ndjson flamegraph
```

Generate a `perf` report:

```bash
scripts/profile.sh csv perf
scripts/profile.sh ndjson perf
```

Profiling artifacts are written under `target/profiling/`.

CI performance checks use a smaller generated fixture:

```bash
scripts/ci_performance.sh
```

That script is a broad regression gate rather than a precise benchmark. The
longer benchmark notes remain in `METRICS.md`.
