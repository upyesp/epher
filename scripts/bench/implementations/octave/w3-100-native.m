A = load('/Users/pete/Developer/code/epher/scripts/bench/data/W3-A100.txt');
B = load('/Users/pete/Developer/code/epher/scripts/bench/data/W3-B100.txt');
b = load('/Users/pete/Developer/code/epher/scripts/bench/data/W3-A100-b.txt');
P = A * B;
dA = det(A);
dB = det(B);
tr = trace(P);
x = A \ b;
sxa = sum(abs(x));
resid = sum(abs(A * x - b));
if ! isempty(getenv("BENCH_VERIFY_OUT"))
  fid = fopen(getenv("BENCH_VERIFY_OUT"), "w");
  fprintf(fid, "%.17e\n%.17e\n%.17e\n%.17e\n%.17e\n", dA, dB, tr, sxa, resid);
  fclose(fid);
  printf("VERIFY_WRITTEN 5\n");
end
