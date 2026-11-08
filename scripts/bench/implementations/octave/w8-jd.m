source('/Users/pete/Developer/code/epher/scripts/bench/implementations/octave/__astro.m');
V = zeros(20, 1);
V(1) = round(__jd(1948, 5, 20, 15) * 100000);
V(2) = round(__jd(2085, 2, 13, 19) * 100000);
V(3) = round(__jd(2060, 4, 3, 1.5) * 100000);
V(4) = round(__jd(2041, 7, 19, 14) * 100000);
V(5) = round(__jd(2083, 10, 16, 19.5) * 100000);
V(6) = round(__jd(1973, 4, 3, 16.5) * 100000);
V(7) = round(__jd(1978, 9, 18, 23.5) * 100000);
V(8) = round(__jd(1931, 9, 26, 11) * 100000);
V(9) = round(__jd(2088, 8, 7, 0) * 100000);
V(10) = round(__jd(2041, 8, 25, 19) * 100000);
V(11) = round(__jd(1985, 7, 16, 20) * 100000);
V(12) = round(__jd(1912, 7, 16, 6.5) * 100000);
V(13) = round(__jd(1963, 6, 5, 9) * 100000);
V(14) = round(__jd(1975, 11, 18, 23.5) * 100000);
V(15) = round(__jd(2003, 4, 28, 8) * 100000);
V(16) = round(__jd(1934, 11, 20, 15.5) * 100000);
V(17) = round(__jd(2028, 5, 20, 21.5) * 100000);
V(18) = round(__jd(1916, 3, 21, 18) * 100000);
V(19) = round(__jd(1991, 12, 12, 12) * 100000);
V(20) = round(__jd(2030, 7, 11, 8) * 100000);
if ! isempty(getenv("BENCH_VERIFY_OUT"))
  fid = fopen(getenv("BENCH_VERIFY_OUT"), "w");
  fprintf(fid, "%d\n", round(V));
  fclose(fid);
  printf("VERIFY_WRITTEN 20\n");
end
