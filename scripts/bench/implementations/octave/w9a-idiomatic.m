% Calculator benchmark W9a, GNU Octave, idiomatic track: the vectorized
% flag-array sieve, Octave's natural style (logical array operations).
% Boundary: the driver times run('this file') from outside; the span covers
% file read + parse + evaluation of the sieve.
N = 10000;
flags = true(1, N);
flags(1) = false;
p = 2;
while p * p <= N
  flags(p * p : p : N) = false;
  p = p + 1;
end
count = sum(flags);
printf('W9A_COUNT %d\n', count);
