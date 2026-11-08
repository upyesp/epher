D = load('/Users/pete/Developer/code/epher/scripts/bench/data/W4-samples.txt');
n = rows(D);
s = 0.0;
for i = 1:n
  s += D(i);
endfor
mu = s / n;
ss = 0.0;
for i = 1:n
  ss += (D(i) - mu)^2;
endfor
sd = sqrt(ss / (n - 1));
S = sort(D);
md = 0.0;
if mod(n, 2) == 1
  md = S((n + 1) / 2);
else
  md = (S(n / 2) + S(n / 2 + 1)) / 2;
endif
h = (n - 1) * 0.1;
lo = floor(h) + 1;
p10 = S(lo) + (h - floor(h)) * (S(lo + 1) - S(lo));
h = (n - 1) * 0.9;
lo = floor(h) + 1;
p90 = S(lo) + (h - floor(h)) * (S(lo + 1) - S(lo));
sxy = 0.0;
sxx = 0.0;
for i = 1:n
  sxy += (i - (n + 1) / 2) * (D(i) - mu);
  sxx += (i - (n + 1) / 2)^2;
endfor
slope = sxy / sxx;
intercept = mu - slope * (n + 1) / 2;
syy = ss;
r = sxy / sqrt(sxx * syy);
if ! isempty(getenv("BENCH_VERIFY_OUT"))
  fid = fopen(getenv("BENCH_VERIFY_OUT"), "w");
  fprintf(fid, "%.17e\n", mu, sd, md, p10, p90, slope, intercept, r);
  fclose(fid);
  printf("VERIFY_WRITTEN 8\n");
end
