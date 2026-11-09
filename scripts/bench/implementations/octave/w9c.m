raw = fileread('/Users/pete/Developer/code/epher/scripts/bench/data/W9c-corpus.txt');
raw = regexprep(raw, "\n$", "");
lines = strsplit(raw, "\n");
n = numel(lines);
rev = cell(n, 1);
for i = 1:n
  ws = strsplit(lines{i}, " ");
  rev{i} = strjoin(fliplr(ws), " ");
endfor
joined = strjoin(rev, "\n");
bal = 0;
for i = 1:n
  d = 0;
  ok = true;
  for j = 1:length(lines{i})
    c = lines{i}(j);
    if c == "("
      d += 1;
    elseif c == ")"
      d -= 1;
      if d < 0
        ok = false;
        break;
      endif
    endif
  endfor
  if ok && d == 0
    bal += 1;
  endif
endfor
if ! isempty(getenv("BENCH_VERIFY_OUT"))
  fid = fopen("/tmp/bench-w9c-string.txt", "w");
  fputs(fid, joined);
  fclose(fid);
  fid = fopen(getenv("BENCH_VERIFY_OUT"), "w");
  fprintf(fid, "%d\n%d\n", bal, n);
  fclose(fid);
  printf("VERIFY_WRITTEN 2\n");
end
