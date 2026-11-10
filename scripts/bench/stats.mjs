// Shared statistics for the calculator benchmark (plan: min, median, MAD,
// p95 from the committed raw samples; no trimming; outliers beyond 10x the
// median are flagged and the run repeated once).
export function median(sorted) {
  const n = sorted.length;
  if (n === 0) return NaN;
  const mid = Math.floor(n / 2);
  return n % 2 ? sorted[mid] : (sorted[mid - 1] + sorted[mid]) / 2;
}

export function summarize(samplesMs) {
  const sorted = [...samplesMs].sort((a, b) => a - b);
  const med = median(sorted);
  const abs = sorted.map((s) => Math.abs(s - med)).sort((a, b) => a - b);
  const mad = median(abs);
  // Nearest-rank p95.
  const p95 = sorted[Math.min(sorted.length - 1, Math.ceil(0.95 * sorted.length) - 1)];
  const outliers = samplesMs
    .map((s, i) => ({ s, i }))
    .filter(({ s }) => s > 10 * med)
    .map(({ i }) => i);
  return {
    samples: samplesMs.length,
    units: "ms",
    min: sorted[0],
    median: med,
    mad,
    p95,
    max: sorted[sorted.length - 1],
    outlier_indices: outliers,
  };
}

// Plan rule 5 buckets, decided by the measured duration of a warm-up pass:
// under 1 s -> 30 samples; under 60 s -> 7; otherwise 3.
export function bucketSamples(passSeconds) {
  if (passSeconds < 1) return 30;
  if (passSeconds < 60) return 7;
  return 3;
}
