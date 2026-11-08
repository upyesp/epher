source('/Users/pete/Developer/code/epher/scripts/bench/implementations/octave/__astro.m');
V = zeros(20, 1);
V(1) = round(__sunrise(2027, 11, 8, -60, -122.42));
V(2) = round(__sunrise(2027, 4, 30, -53.7, -122.42));
V(3) = round(__sunrise(2026, 8, 22, -47.4, -122.42));
V(4) = round(__sunrise(2026, 10, 7, -41.1, -122.42));
V(5) = round(__sunrise(2027, 8, 13, -34.7, -122.42));
V(6) = round(__sunrise(2027, 10, 30, -28.4, -122.42));
V(7) = round(__sunrise(2027, 2, 17, -22.1, -122.42));
V(8) = round(__sunrise(2026, 3, 6, -15.8, -122.42));
V(9) = round(__sunrise(2026, 6, 29, -9.5, -122.42));
V(10) = round(__sunrise(2026, 12, 26, -3.2, -122.42));
V(11) = round(__sunrise(2026, 1, 24, 3.2, -122.42));
V(12) = round(__sunrise(2026, 1, 23, 9.5, -122.42));
V(13) = round(__sunrise(2027, 4, 3, 15.8, -122.42));
V(14) = round(__sunrise(2026, 12, 12, 22.1, -122.42));
V(15) = round(__sunrise(2026, 1, 15, 28.4, -122.42));
V(16) = round(__sunrise(2026, 4, 19, 34.7, -122.42));
V(17) = round(__sunrise(2026, 7, 22, 41.1, -122.42));
V(18) = round(__sunrise(2027, 9, 25, 47.4, -122.42));
V(19) = round(__sunrise(2026, 12, 21, 53.7, -122.42));
V(20) = round(__sunrise(2027, 9, 7, 60, -122.42));
if ! isempty(getenv("BENCH_VERIFY_OUT"))
  fid = fopen(getenv("BENCH_VERIFY_OUT"), "w");
  fprintf(fid, "%d\n", round(V));
  fclose(fid);
  printf("VERIFY_WRITTEN 20\n");
end
