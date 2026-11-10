% Calculator benchmark W9a, GNU Octave, equivalent-algorithm track: the
% classic flag-array sieve expressed with explicit loops, the same loop
% structure a tool without index assignment would have to follow.
% Boundary: the driver times run('this file') from outside; the span covers
% file read + parse + evaluation of the sieve.
N = 10000;
flags = true(1, N);
flags(1) = false;
for p = 2 : floor(sqrt(N))
  if flags(p)
    for k = p * p : p : N
      flags(k) = false;
    end
  end
end
count = sum(flags);
printf('W9A_COUNT %d\n', count);
