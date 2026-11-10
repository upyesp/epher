1; % script marker so source() defines these functions in the caller
function [L, s] = __lu_det(A)
  n = rows(A);
  L = A;
  s = 1;
  for k = 1:n
    piv = k;
    best = abs(L(k,k));
    for i = k+1:n
      if abs(L(i,k)) > best
        best = abs(L(i,k));
        piv = i;
      endif
    endfor
    if piv != k
      tmp = L(k,:); L(k,:) = L(piv,:); L(piv,:) = tmp;
      s = -s;
    endif
    for i = k+1:n
      f = L(i,k) / L(k,k);
      for j = k:n
        L(i,j) -= f * L(k,j);
      endfor
    endfor
  endfor
endfunction

function v = __lu_logabs(L)
  n = rows(L);
  v = 0.0;
  for k = 1:n
    v += log(abs(L(k,k)));
  endfor
endfunction

function x = __lu_solve(A, b)
  n = rows(A);
  M = A;
  bb = b;
  for k = 1:n
    piv = k;
    best = abs(M(k,k));
    for i = k+1:n
      if abs(M(i,k)) > best
        best = abs(M(i,k));
        piv = i;
      endif
    endfor
    if piv != k
      tmp = M(k,:); M(k,:) = M(piv,:); M(piv,:) = tmp;
      tv = bb(k); bb(k) = bb(piv); bb(piv) = tv;
    endif
    for i = k+1:n
      f = M(i,k) / M(k,k);
      bb(i) -= f * bb(k);
      for j = k:n
        M(i,j) -= f * M(k,j);
      endfor
    endfor
  endfor
  x = zeros(n, 1);
  for i = n:-1:1
    s = bb(i);
    for j = i+1:n
      s -= M(i,j) * x(j);
    endfor
    x(i) = s / M(i,i);
  endfor
endfunction
