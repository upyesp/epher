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
  fprintf(fid, "%.17e\n", I1, I2, I3, r1, r2, r3);
  fclose(fid);
  printf("VERIFY_WRITTEN 6\n");
end
