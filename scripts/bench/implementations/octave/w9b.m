x = 1;
n = 0;
while abs(cos(x) - x) > 1e-12 && n < 1000
  x = cos(x);
  n += 1;
endwhile
if ! isempty(getenv("BENCH_VERIFY_OUT"))
  fid = fopen(getenv("BENCH_VERIFY_OUT"), "w");
  fprintf(fid, "%.17e\n%d\n", x * 1e12, n);
  fclose(fid);
  printf("VERIFY_WRITTEN 2\n");
end
