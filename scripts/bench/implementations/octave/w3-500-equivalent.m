source('/Users/pete/Developer/code/epher/scripts/bench/implementations/octave/__luhelpers.m');
A = load('/Users/pete/Developer/code/epher/scripts/bench/data/W3-A500.txt');
B = load('/Users/pete/Developer/code/epher/scripts/bench/data/W3-B500.txt');
b = load('/Users/pete/Developer/code/epher/scripts/bench/data/W3-A500-b.txt');
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
  fprintf(fid, "%.17e\n%.17e\n%.17e\n%.17e\n%.17e\n", dA, dB, tr, sxa, resid);
  fclose(fid);
  printf("VERIFY_WRITTEN 5\n");
end
