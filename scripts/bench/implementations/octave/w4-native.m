D = load('/Users/pete/Developer/code/epher/scripts/bench/data/W4-samples.txt');
n = rows(D);
mu = mean(D);
sd = std(D);
md = median(D);
S = sort(D);
h = (n - 1) * 0.1;
lo = floor(h) + 1;
p10 = S(lo) + (h - floor(h)) * (S(lo + 1) - S(lo));
h = (n - 1) * 0.9;
lo = floor(h) + 1;
p90 = S(lo) + (h - floor(h)) * (S(lo + 1) - S(lo));
xs = (1:n)';
pf = polyfit(xs, D, 1);
slope = pf(1);
intercept = pf(2);
r = corr(xs, D);
if ! isempty(getenv("BENCH_VERIFY_OUT"))
  fid = fopen(getenv("BENCH_VERIFY_OUT"), "w");
  fprintf(fid, "%.17e\n", mu, sd, md, p10, p90, slope, intercept, r);
  fclose(fid);
  printf("VERIFY_WRITTEN 8\n");
end
