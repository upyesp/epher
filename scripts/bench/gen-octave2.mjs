#!/usr/bin/env node
// Stage 4 GNU Octave implementation generator (w3-w9). W1/W2 stay in
// gen-octave.mjs. Native track uses Octave builtins; equivalent track
// mirrors the same algorithm structure with explicit loops (plan rule 1).
// Gate values are written one per line to $BENCH_VERIFY_OUT by the -verify
// variants; the timed variants are the same computation without output.
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { join } from 'node:path';

const BENCH = new URL('./', import.meta.url).pathname;
const DATA = join(BENCH, 'data');
const OUT = join(BENCH, 'implementations', 'octave');
mkdirSync(OUT, { recursive: true });

const emit = (name, text) => writeFileSync(join(OUT, name), text.trim() + '\n');

// ---------- W3 ----------
for (const n of [100, 500]) {
  emit(`w3-${n}-native.m`, `
A = load('${join(DATA, `W3-A${n}.txt`)}');
B = load('${join(DATA, `W3-B${n}.txt`)}');
b = load('${join(DATA, `W3-A${n}-b.txt`)}');
P = A * B;
dA = det(A);
dB = det(B);
tr = trace(P);
x = A \\ b;
sxa = sum(abs(x));
resid = sum(abs(A * x - b));
if ! isempty(getenv("BENCH_VERIFY_OUT"))
  fid = fopen(getenv("BENCH_VERIFY_OUT"), "w");
  fprintf(fid, "%.17e\\n%.17e\\n%.17e\\n%.17e\\n%.17e\\n", dA, dB, tr, sxa, resid);
  fclose(fid);
  printf("VERIFY_WRITTEN 5\\n");
end
`);
  emit(`w3-${n}-equivalent.m`, `
source('${join(OUT, '__luhelpers.m')}');
A = load('${join(DATA, `W3-A${n}.txt`)}');
B = load('${join(DATA, `W3-B${n}.txt`)}');
b = load('${join(DATA, `W3-A${n}-b.txt`)}');
n2 = rows(A);
P = zeros(n2, n2);
for i = 1:n2
  for j = 1:n2
    s = 0.0;
    for k = 1:n2
      s += A(i,k) * B(k,j);
    endfor
    P(i,j) = s;
  endfor
endfor
% LU with partial pivoting, explicit loops (same algorithm as the referee)
[M, sign] = __lu_det(A);
dA = sign * exp(__lu_logabs(M));
[M2, sign2] = __lu_det(B);
dB = sign2 * exp(__lu_logabs(M2));
tr = 0.0;
for i = 1:n2
  tr += P(i,i);
endfor
% solve A x = b by explicit Gaussian elimination with pivoting
x = __lu_solve(A, b);
sxa = 0.0;
for i = 1:n2
  sxa += abs(x(i));
endfor
resid = 0.0;
for i = 1:n2
  s = 0.0;
  for k = 1:n2
    s += A(i,k) * x(k);
  endfor
  resid += abs(s - b(i));
endfor
if ! isempty(getenv("BENCH_VERIFY_OUT"))
  fid = fopen(getenv("BENCH_VERIFY_OUT"), "w");
  fprintf(fid, "%.17e\\n%.17e\\n%.17e\\n%.17e\\n%.17e\\n", dA, dB, tr, sxa, resid);
  fclose(fid);
  printf("VERIFY_WRITTEN 5\\n");
end
`);
}
// helper functions shared by the equivalent track
emit(`__luhelpers.m`, `
1; % script marker so source() defines these functions in the caller
function [L, s] = __lu_det(A)
  n = rows(A);
  L = A;
  s = 1;
  for k = 1:n
    piv = k;
    best = abs(L(k,k));
    for i = k+1:n
      if abs(L(i,k)) > best
        best = abs(L(i,k));
        piv = i;
      endif
    endfor
    if piv != k
      tmp = L(k,:); L(k,:) = L(piv,:); L(piv,:) = tmp;
      s = -s;
    endif
    for i = k+1:n
      f = L(i,k) / L(k,k);
      for j = k:n
        L(i,j) -= f * L(k,j);
      endfor
    endfor
  endfor
endfunction

function v = __lu_logabs(L)
  n = rows(L);
  v = 0.0;
  for k = 1:n
    v += log(abs(L(k,k)));
  endfor
endfunction

function x = __lu_solve(A, b)
  n = rows(A);
  M = A;
  bb = b;
  for k = 1:n
    piv = k;
    best = abs(M(k,k));
    for i = k+1:n
      if abs(M(i,k)) > best
        best = abs(M(i,k));
        piv = i;
      endif
    endfor
    if piv != k
      tmp = M(k,:); M(k,:) = M(piv,:); M(piv,:) = tmp;
      tv = bb(k); bb(k) = bb(piv); bb(piv) = tv;
    endif
    for i = k+1:n
      f = M(i,k) / M(k,k);
      bb(i) -= f * bb(k);
      for j = k:n
        M(i,j) -= f * M(k,j);
      endfor
    endfor
  endfor
  x = zeros(n, 1);
  for i = n:-1:1
    s = bb(i);
    for j = i+1:n
      s -= M(i,j) * x(j);
    endfor
    x(i) = s / M(i,i);
  endfor
endfunction
`);
// b files (A * ones)
for (const n of [100, 500]) {
  const A = readFileSync(join(DATA, `W3-A${n}.txt`), 'utf8').trim().split('\n').map((l) => l.split(' ').map(Number));
  const b = A.map((r) => r.reduce((a, c) => a + c, 0));
  writeFileSync(join(DATA, `W3-A${n}-b.txt`), b.map((v) => v.toFixed(6)).join('\n') + '\n');
}

// ---------- W4 ----------
emit(`w4-native.m`, `
D = load('${join(DATA, 'W4-samples.txt')}');
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
  fprintf(fid, "%.17e\\n", mu, sd, md, p10, p90, slope, intercept, r);
  fclose(fid);
  printf("VERIFY_WRITTEN 8\\n");
end
`);
emit(`w4-equivalent.m`, `
D = load('${join(DATA, 'W4-samples.txt')}');
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
  fprintf(fid, "%.17e\\n", mu, sd, md, p10, p90, slope, intercept, r);
  fclose(fid);
  printf("VERIFY_WRITTEN 8\\n");
end
`);

// ---------- W5 ----------
emit(`w5-native.m`, `
I1 = integral(@(x) exp(-x.^2), 0, 1);
I2 = integral(@(x) sin(x) .* exp(-x/3), 0, pi);
I3 = integral(@(x) log(x) ./ x, 1, 2);
r1 = fzero(@(x) cos(x) - x, [0, 1]);
r2 = fzero(@(x) x^3 - x - 1, [1, 2]);
r3 = fzero(@(x) exp(x) - 3*x, [1, 2]);
if ! isempty(getenv("BENCH_VERIFY_OUT"))
  fid = fopen(getenv("BENCH_VERIFY_OUT"), "w");
  fprintf(fid, "%.17e\\n", I1, I2, I3, r1, r2, r3);
  fclose(fid);
  printf("VERIFY_WRITTEN 6\\n");
end
`);
emit(`w5-equivalent.m`, `
% Simpson with 8192 intervals (loop form) and bisection to 1e-13 bracket
n = 8192;
I1 = 0.0; I2 = 0.0; I3 = 0.0;
for k = 0:n
  x = k * (1.0 / n);
  w = 1;
  if mod(k, 2) == 1
    w = 4;
  endif
  if k > 0 && k < n
    if w == 1
      w = 2;
    endif
  endif
  I1 += w * exp(-x^2);
  x2 = k * (pi / n);
  I2 += w * sin(x2) * exp(-x2/3);
  x3 = 1 + k * (1.0 / n);
  I3 += w * log(x3) / x3;
endfor
I1 = I1 * (1.0 / n) / 3;
I2 = I2 * (pi / n) / 3;
I3 = I3 * (1.0 / n) / 3;
lo = 0; hi = 1;
while hi - lo > 1e-13
  m = (lo + hi) / 2;
  fm = cos(m) - m;
  fh = cos(hi) - hi;
  if (fm < 0 && fh < 0) || (fm >= 0 && fh >= 0)
    hi = m;
  else
    lo = m;
  endif
endwhile
r1 = (lo + hi) / 2;
lo = 1; hi = 2;
while hi - lo > 1e-13
  m = (lo + hi) / 2;
  fm = m^3 - m - 1;
  fh = hi^3 - hi - 1;
  if (fm < 0 && fh < 0) || (fm >= 0 && fh >= 0)
    hi = m;
  else
    lo = m;
  endif
endwhile
r2 = (lo + hi) / 2;
lo = 1; hi = 2;
while hi - lo > 1e-13
  m = (lo + hi) / 2;
  fm = exp(m) - 3*m;
  fh = exp(hi) - 3*hi;
  if (fm < 0 && fh < 0) || (fm >= 0 && fh >= 0)
    hi = m;
  else
    lo = m;
  endif
endwhile
r3 = (lo + hi) / 2;
if ! isempty(getenv("BENCH_VERIFY_OUT"))
  fid = fopen(getenv("BENCH_VERIFY_OUT"), "w");
  fprintf(fid, "%.17e\\n", I1, I2, I3, r1, r2, r3);
  fclose(fid);
  printf("VERIFY_WRITTEN 6\\n");
end
`);

// ---------- W7 ----------
{
  const lines = readFileSync(join(DATA, 'W7-complex.txt'), 'utf8').trim().split('\n');
  // Octave complex literals match epher's spelling (a+bi)
  const body = lines.map((l, i) => `V(${i + 1}) = ${l};`).join('\n');
  emit(`w7.m`, `
V = complex(zeros(200, 1));
${body}
if ! isempty(getenv("BENCH_VERIFY_OUT"))
  fid = fopen(getenv("BENCH_VERIFY_OUT"), "w");
  for k = 1:200
    fprintf(fid, "%.17e\\n%.17e\\n", real(V(k)), imag(V(k)));
  endfor
  fclose(fid);
  printf("VERIFY_WRITTEN 400\\n");
end
`);
}
// ---------- W8 ----------
const dates = JSON.parse(readFileSync(join(DATA, 'W8-dates.json'), 'utf8'));
emit(`__astro.m`, `
1; % script marker so source() defines these functions in the caller
function jd = __jd(y, m, d, hr)
  if m <= 2
    y -= 1;
    m += 12;
  endif
  a = floor(y / 100);
  b = 2 - a + floor(a / 4);
  jd = floor(365.25 * (y + 4716)) + floor(30.6001 * (m + 1)) + d + b - 1524.5 + hr / 24;
endfunction

function [i] = __phase(jd)
  T = (jd - 2451545.0) / 36525;
  Lp = mod(218.3164477 + 481267.88123421 * T - 0.0015786 * T^2, 360);
  D = mod(297.8501921 + 445267.1114034 * T - 0.0018819 * T^2, 360);
  Mm = mod(134.9633964 + 477198.8675055 * T + 0.0087414 * T^2, 360);
  F = mod(93.2720950 + 483202.0175233 * T - 0.0036539 * T^2, 360);
  Ms = mod(357.52911 + 35999.05029 * T - 0.0001537 * T^2, 360);
  L0 = mod(280.46646 + 36000.76983 * T + 0.0003032 * T^2, 360);
  C = (1.914602 - 0.004817 * T) * sind(Ms) + 0.019993 * sind(2 * Ms);
  ls = mod(L0 + C, 360);
  lon = Lp + 6.288774 * sind(Mm) + 1.274027 * sind(2*D - Mm) + 0.658314 * sind(2*D) ...
    + 0.213618 * sind(2*Mm) - 0.185116 * sind(Ms) - 0.114332 * sind(2*F) ...
    + 0.058793 * sind(2*D - 2*Mm) + 0.057066 * sind(2*D - Ms - Mm) ...
    + 0.053322 * sind(2*D + Mm) + 0.045758 * sind(2*D - Ms);
  lat = 5.128122 * sind(F) + 0.280602 * sind(Mm + F) + 0.277693 * sind(Mm - F) ...
    + 0.173237 * sind(2*D - F) + 0.055377 * sind(2*D - Mm + F);
  dist = 385000.56 - 20905.355 * cosd(Mm) - 3699.111 * cosd(2*D - Mm) ...
    - 2955.968 * cosd(2*D) - 569.925 * cosd(2*Mm);
  eps = 23.439291 - 0.0130042 * T;
  % moon RA/dec
  lam = lon;
  am = atan2d(sind(lam) * cosd(eps) - tand(lat) * sind(eps), cosd(lam));
  dm = asind(sind(lat) * cosd(eps) + cosd(lat) * sind(eps) * sind(lam));
  % sun RA/dec
  asun = atan2d(cosd(eps) * sind(ls), cosd(ls));
  dsun = asind(sind(eps) * sind(ls));
  Rs = 1.000001018 - 0.016708634 * cosd(Ms) - 0.000139897 * cosd(2 * Ms);
  cospsi = sind(dm) * sind(dsun) + cosd(dm) * cosd(dsun) * cosd(am - asun);
  psi = acosd(max(-1, min(1, cospsi)));
  Rkm = Rs * 149597870.7;
  i = atan2d(Rkm * sind(psi), dist - Rkm * cosd(psi));
endfunction

function minutes = __sunrise(y, m, d, lat, lon)
  jd0 = __jd(y, m, d, 0);
  N = jd0 - 2451544.5 + 0.0008;
  Jstar = N - lon / 360;
  M = mod(357.5291 + 0.98560028 * Jstar, 360);
  C = 1.9148 * sind(M) + 0.02 * sind(2*M) + 0.0003 * sind(3*M);
  lam = mod(M + C + 180 + 102.9372, 360);
  Jtransit = 2451545.0 + Jstar + 0.0053 * sind(M) - 0.0069 * sind(2*lam);
  sindec = sind(lam) * sind(23.44);
  dec = asind(sindec);
  cosHa = (sind(-0.833) - sind(lat) * sindec) / (cosd(lat) * cosd(dec));
  if cosHa > 1 || cosHa < -1
    minutes = -77777;
    return;
  endif
  Ha = acosd(cosHa);
  Jrise = Jtransit - Ha / 360;
  minutes = (Jrise - jd0) * 1440;
endfunction
`);
{
  const dl = dates.map((d) => `V(${dates.indexOf(d) + 1}) = round(__phase(__jd(${d.y}, ${d.m}, ${d.d}, 12)) * 100);`).join('\n');
  emit(`w8-lunar.m`, `
source('${join(OUT, '__astro.m')}');
V = zeros(100, 1);
${dl}
if ! isempty(getenv("BENCH_VERIFY_OUT"))
  fid = fopen(getenv("BENCH_VERIFY_OUT"), "w");
  fprintf(fid, "%d\\n", round(V));
  fclose(fid);
  printf("VERIFY_WRITTEN 100\\n");
end
`);
  const pairs = JSON.parse(readFileSync(join(DATA, 'W8-sunrise.json'), 'utf8'));
  const pl = pairs.map((p, i) => `V(${i + 1}) = round(__sunrise(${p.y}, ${p.m}, ${p.d}, ${p.lat}, ${p.lon}));`).join('\n');
  emit(`w8-sunrise.m`, `
source('${join(OUT, '__astro.m')}');
V = zeros(20, 1);
${pl}
if ! isempty(getenv("BENCH_VERIFY_OUT"))
  fid = fopen(getenv("BENCH_VERIFY_OUT"), "w");
  fprintf(fid, "%d\\n", round(V));
  fclose(fid);
  printf("VERIFY_WRITTEN 20\\n");
end
`);
  const jds = JSON.parse(readFileSync(join(DATA, 'W8-jd.json'), 'utf8'));
  const jl = jds.map((r, i) => `V(${i + 1}) = round(__jd(${r.y}, ${r.m}, ${r.d}, ${r.hr}) * 100000);`).join('\n');
  emit(`w8-jd.m`, `
source('${join(OUT, '__astro.m')}');
V = zeros(20, 1);
${jl}
if ! isempty(getenv("BENCH_VERIFY_OUT"))
  fid = fopen(getenv("BENCH_VERIFY_OUT"), "w");
  fprintf(fid, "%d\\n", round(V));
  fclose(fid);
  printf("VERIFY_WRITTEN 20\\n");
end
`);
}

// ---------- W9 ----------
emit(`w9b.m`, `
x = 1;
n = 0;
while abs(cos(x) - x) > 1e-12 && n < 1000
  x = cos(x);
  n += 1;
endwhile
if ! isempty(getenv("BENCH_VERIFY_OUT"))
  fid = fopen(getenv("BENCH_VERIFY_OUT"), "w");
  fprintf(fid, "%.17e\\n%d\\n", x * 1e12, n);
  fclose(fid);
  printf("VERIFY_WRITTEN 2\\n");
end
`);
const corpus = readFileSync(join(DATA, 'W9c-corpus.txt'), 'utf8').trim();
emit(`w9c.m`, `
raw = fileread('${join(DATA, 'W9c-corpus.txt')}');
raw = regexprep(raw, "\\n$", "");
lines = strsplit(raw, "\\n");
n = numel(lines);
rev = cell(n, 1);
for i = 1:n
  ws = strsplit(lines{i}, " ");
  rev{i} = strjoin(fliplr(ws), " ");
endfor
joined = strjoin(rev, "\\n");
bal = 0;
for i = 1:n
  d = 0;
  ok = true;
  for j = 1:length(lines{i})
    c = lines{i}(j);
    if c == "("
      d += 1;
    elseif c == ")"
      d -= 1;
      if d < 0
        ok = false;
        break;
      endif
    endif
  endfor
  if ok && d == 0
    bal += 1;
  endif
endfor
if ! isempty(getenv("BENCH_VERIFY_OUT"))
  fid = fopen("/tmp/bench-w9c-string.txt", "w");
  fputs(fid, joined);
  fclose(fid);
  fid = fopen(getenv("BENCH_VERIFY_OUT"), "w");
  fprintf(fid, "%d\\n%d\\n", bal, n);
  fclose(fid);
  printf("VERIFY_WRITTEN 2\\n");
end
`);
{
  const dl = dates.map((d, i) => `V(${i + 1}) = round(__phase(__jd(${d.y}, ${d.m}, ${d.d}, 12)) * 100);`).join('\n');
  emit(`w9d.m`, `
source('${join(OUT, '__astro.m')}');
V = zeros(100, 1);
${dl}
if ! isempty(getenv("BENCH_VERIFY_OUT"))
  fid = fopen(getenv("BENCH_VERIFY_OUT"), "w");
  fprintf(fid, "%d\\n", round(V));
  fclose(fid);
  printf("VERIFY_WRITTEN 100\\n");
end
`);
}
emit(`w9e.m`, `
s = 0;
for i = 1:50000
  if mod(i, 3) == 0
    s += i;
  elseif mod(i, 7) == 0
    s -= i;
  else
    s += 1;
  endif
endfor
if ! isempty(getenv("BENCH_VERIFY_OUT"))
  fid = fopen(getenv("BENCH_VERIFY_OUT"), "w");
  fprintf(fid, "%d\\n", s);
  fclose(fid);
  printf("VERIFY_WRITTEN 1\\n");
end
`);
console.log('octave stage-4 implementations written');
