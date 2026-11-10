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
  fprintf(fid, "%d\n", s);
  fclose(fid);
  printf("VERIFY_WRITTEN 1\n");
end
