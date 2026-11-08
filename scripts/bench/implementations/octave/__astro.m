1; % script marker so source() defines these functions in the caller
function jd = __jd(y, m, d, hr)
  if m <= 2
    y -= 1;
    m += 12;
  endif
  a = floor(y / 100);
  b = 2 - a + floor(a / 4);
  jd = floor(365.25 * (y + 4716)) + floor(30.6001 * (m + 1)) + d + b - 1524.5 + hr / 24;
endfunction

function [i] = __phase(jd)
  T = (jd - 2451545.0) / 36525;
  Lp = mod(218.3164477 + 481267.88123421 * T - 0.0015786 * T^2, 360);
  D = mod(297.8501921 + 445267.1114034 * T - 0.0018819 * T^2, 360);
  Mm = mod(134.9633964 + 477198.8675055 * T + 0.0087414 * T^2, 360);
  F = mod(93.2720950 + 483202.0175233 * T - 0.0036539 * T^2, 360);
  Ms = mod(357.52911 + 35999.05029 * T - 0.0001537 * T^2, 360);
  L0 = mod(280.46646 + 36000.76983 * T + 0.0003032 * T^2, 360);
  C = (1.914602 - 0.004817 * T) * sind(Ms) + 0.019993 * sind(2 * Ms);
  ls = mod(L0 + C, 360);
  lon = Lp + 6.288774 * sind(Mm) + 1.274027 * sind(2*D - Mm) + 0.658314 * sind(2*D) ...
    + 0.213618 * sind(2*Mm) - 0.185116 * sind(Ms) - 0.114332 * sind(2*F) ...
    + 0.058793 * sind(2*D - 2*Mm) + 0.057066 * sind(2*D - Ms - Mm) ...
    + 0.053322 * sind(2*D + Mm) + 0.045758 * sind(2*D - Ms);
  lat = 5.128122 * sind(F) + 0.280602 * sind(Mm + F) + 0.277693 * sind(Mm - F) ...
    + 0.173237 * sind(2*D - F) + 0.055377 * sind(2*D - Mm + F);
  dist = 385000.56 - 20905.355 * cosd(Mm) - 3699.111 * cosd(2*D - Mm) ...
    - 2955.968 * cosd(2*D) - 569.925 * cosd(2*Mm);
  eps = 23.439291 - 0.0130042 * T;
  % moon RA/dec
  lam = lon;
  am = atan2d(sind(lam) * cosd(eps) - tand(lat) * sind(eps), cosd(lam));
  dm = asind(sind(lat) * cosd(eps) + cosd(lat) * sind(eps) * sind(lam));
  % sun RA/dec
  asun = atan2d(cosd(eps) * sind(ls), cosd(ls));
  dsun = asind(sind(eps) * sind(ls));
  Rs = 1.000001018 - 0.016708634 * cosd(Ms) - 0.000139897 * cosd(2 * Ms);
  cospsi = sind(dm) * sind(dsun) + cosd(dm) * cosd(dsun) * cosd(am - asun);
  psi = acosd(max(-1, min(1, cospsi)));
  Rkm = Rs * 149597870.7;
  i = atan2d(Rkm * sind(psi), dist - Rkm * cosd(psi));
endfunction

function minutes = __sunrise(y, m, d, lat, lon)
  jd0 = __jd(y, m, d, 0);
  N = jd0 - 2451544.5 + 0.0008;
  Jstar = N - lon / 360;
  M = mod(357.5291 + 0.98560028 * Jstar, 360);
  C = 1.9148 * sind(M) + 0.02 * sind(2*M) + 0.0003 * sind(3*M);
  lam = mod(M + C + 180 + 102.9372, 360);
  Jtransit = 2451545.0 + Jstar + 0.0053 * sind(M) - 0.0069 * sind(2*lam);
  sindec = sind(lam) * sind(23.44);
  dec = asind(sindec);
  cosHa = (sind(-0.833) - sind(lat) * sindec) / (cosd(lat) * cosd(dec));
  if cosHa > 1 || cosHa < -1
    minutes = -77777;
    return;
  endif
  Ha = acosd(cosHa);
  Jrise = Jtransit - Ha / 360;
  minutes = (Jrise - jd0) * 1440;
endfunction
