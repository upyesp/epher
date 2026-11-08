I1 = integral(@(x) exp(-x.^2), 0, 1);
I2 = integral(@(x) sin(x) .* exp(-x/3), 0, pi);
I3 = integral(@(x) log(x) ./ x, 1, 2);
r1 = fzero(@(x) cos(x) - x, [0, 1]);
r2 = fzero(@(x) x^3 - x - 1, [1, 2]);
r3 = fzero(@(x) exp(x) - 3*x, [1, 2]);
if ! isempty(getenv("BENCH_VERIFY_OUT"))
  fid = fopen(getenv("BENCH_VERIFY_OUT"), "w");
  fprintf(fid, "%.17e\n", I1, I2, I3, r1, r2, r3);
  fclose(fid);
  printf("VERIFY_WRITTEN 6\n");
end
