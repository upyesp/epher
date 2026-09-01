# epher-Benutzerhandbuch

Willkommen! epher ist ein programmierbarer, skriptfähiger Taschenrechner. Du
kannst ihn für eine schnelle Berechnung nutzen oder eigene Funktionen und
kleine Programme aufbauen. Alles ist in acht Sprachen verfügbar.

Dieses Handbuch richtet sich an komplette Einsteiger. Es beginnt mit der
einfachsten möglichen Berechnung und steigert sich bis zur vollen Kraft der
Sprache. Jedes Beispiel zeigt, was du eintippst und was epher antwortet.

Es gibt fünf Arten, epher zu nutzen. Wähle, was zu dir passt:

| Version | Was es ist | Am besten, wenn |
|---|---|---|
| **Befehlszeile** (CLI) | Textbefehle in einem Terminal | Du lebst im Terminal und magst Skripte |
| **REPL** | Eine interaktive `epher`-Sitzung am Prompt `epher>` | Du willst schnelles Hin und Her, ohne das Terminal zu verlassen |
| **Terminal-Oberfläche** (TUI) | Ein Vollbild-Programm im Terminal | Du willst eine Terminal-App mit Graphen und Verlauf auf dem Bildschirm |
| **Desktop-App** | Ein normales Desktop-Programm mit eigenem Fenster | Du willst eine normale Anwendung |
| **Web-App** (PWA) | Läuft in deinem Browser, installierbar, funktioniert offline | Du willst den schnellsten Start; keine Installation |

Die Desktop-App, die Befehlszeile, das REPL und die Terminal-Oberfläche
sind ein Programm: Ein einziger Download installiert den Befehl `epher`,
der alle vier kann. Die Web-App ist die Ausnahme: Sie braucht überhaupt
keinen Download.

Alle fünf Versionen verstehen genau dieselbe Sprache. Lerne sie einmal,
nutze sie überall.

## 1. Die Sprache epher

Dieses Kapitel lehrt die Sprache, die alle Versionen von epher teilen. In
der Web-App oder der Desktop-App gibst du einen Ausdruck ein und drückst
**Enter** (oder klickst auf den Button **=**). In der CLI startest du die
Sitzung mit `epher repl` und tippst nach dem Prompt `epher>`. In der TUI
(`epher tui`) tippst du einfach und drückst **Enter**. In der CLI kannst du
auch `epher "expression"` schreiben, um einen Ausdruck direkt auszuwerten.

### 1.1 Deine erste Berechnung

Tippe dies:

```epher
2 + 3 * 4
```

epher antwortet:

```text
14
```

Die Multiplikation wird vor der Addition ausgeführt, genau wie in der
Mathematik. Diese Regel heißt *Operatorrangfolge*.

### 1.2 Reihenfolge der Operationen

Die vollständige Rangfolge, von der stärksten zur schwächsten:

1. `!` Fakultät und `%` Prozent (beide nachgestellt)
2. `^` Potenz
3. `*` und `/` Multiplikation und Division
4. `+` und `-` Addition und Subtraktion

Nutze Klammern, um die Reihenfolge zu ändern:

```epher
(2 + 3) * 4
```

```text
20
```

Der Operator `^` berechnet Potenzen und arbeitet von rechts nach links:

```epher
2 ^ 10
```

```text
1024
```

```epher
2 ^ 3 ^ 2
```

```text
512
```

(`2 ^ 3 ^ 2` bedeutet `2 ^ (3 ^ 2)`, also `2 ^ 9` = 512.)

Potenzen können gebrochen sein. `2 ^ 0.5` ist die Quadratwurzel aus 2:

```epher
2 ^ 0.5
```

```text
1.4142135623730951
```

Subtraktion und Division arbeiten von links nach rechts:

```epher
10 - 3 - 2
```

```text
5
```

Das Zeichen `%` ist ein nachgestellter Operator und bedeutet „durch 100 geteilt“: `5%` ist 0.05. Es schaut sich die Operatoren darum herum nie an, darum ist `200 + 10%` gleich 200.1. Um 200 um 10% zu erhöhen, schreib die Multiplikation aus:

```epher
200 * (1 + 10%)
```

```text
220
```


### 1.3 Die besonderen Zahlen pi, e, tau und phi

Die berühmten Konstanten sind eingebaut:

```epher
pi
```

```text
3.141592653589793
```

```epher
2 * pi
```

```text
6.283185307179586
```

```epher
e
```

```text
2.718281828459045
```

Zwei weitere: `tau` ist eine volle Umdrehung (2 pi), und `phi` ist der
Goldene Schnitt:

```epher
tau
```

```text
6.283185307179586
```

```epher
phi
```

```text
1.618033988749895
```

### 1.4 Vergleichen und Logik

Du kannst Zahlen vergleichen. Das Ergebnis ist entweder `true` oder
`false`:

| Vergleich | Bedeutung |
|---|---|
| `a > b` | a ist größer als b |
| `a < b` | a ist kleiner als b |
| `a >= b` | a ist größer oder gleich b |
| `a <= b` | a ist kleiner oder gleich b |
| `a == b` | a ist gleich b (beachte das doppelte `=`) |
| `a != b` | a ist ungleich b |

```epher
3 > 2
```

```text
true
```

```epher
1 != 2
```

```text
true
```

Kombiniere Vergleiche mit `and`, `or` und `not`:

```epher
3 > 2 and 2 < 3
```

```text
true
```

```epher
not 3 > 2
```

```text
false
```

### 1.5 Variablen

Gib einem Wert mit einem einzelnen `=` einen Namen:

```epher
x = 5
```

```text
5
```

epher gibt dir den Wert zurück. Ab jetzt kann `x` überall verwendet
werden:

```epher
x ^ 2
```

```text
25
```

Du kannst eine Variable jederzeit ändern. Sie behält ihren Wert, bis du
sie änderst:

```epher
x = x + 1
```

```text
6
```

> Namen dürfen Buchstaben und Unterstriche enthalten, wie `radius` oder
> `my_total`. Sie dürfen keine Leerzeichen enthalten und nicht mit einer Zahl beginnen.

Die besondere Variable `ans` enthält immer die vorherige Antwort, wie
die `Ans`-Taste eines Taschenrechners, praktisch für Kettenrechnungen:

```epher
2 + 3
ans * 2
```

```text
5
10
```

### 1.6 Konstanten: Namen, die sich nie ändern

Eine *Konstante* ist ein Name für einen Wert, der sich nie ändert, wie
das eingebaute `pi`, aber von dir gewählt. Definiere eine mit `const`:

```epher
const tax = 0.2
```

```text
0.2
```

Verwende sie überall dort, wo eine Zahl stehen kann:

```epher
100 * (1 + tax)
```

```text
120
```

Der Wert ist fest: ihn mit `=` zu ändern ist ein Fehler,

```epher
tax = 0.25
```

```text
error: cannot assign to constant tax
```

und dasselbe gilt, wenn du sie mit einem anderen Wert neu definierst:

```epher
const tax = 0.25
```

```text
error: constant already defined: tax
```

Konstanten unterscheiden sich von Variablen noch in einer weiteren
Hinsicht: Wie `pi` funktionieren sie innerhalb deiner eigenen Funktionen.

```epher
const g = 9.81
```

```text
9.81
```

```epher
def weight(m) = m * g
```

```epher
weight(80)
```

```text
784.8000000000001
```

Speichere eine Konstante für künftige Sitzungen mit `save tax`, genau wie
eine Funktion (Kapitel 4.4).

> Eine Variable und eine Konstante können nicht denselben Namen haben:
> nach `const tax = 0.2` ist `tax = ...` immer ein Fehler. Wähle einen
> frischen Namen oder starte eine neue Sitzung.

### 1.7 Entscheidungen mit if

`if` wählt zwischen zwei Werten:

```epher
if 3 > 2 then 10 else 20
```

```text
10
```

Die Form ist immer `if condition then value_if_true else value_if_false`.
Der `else`-Teil ist Pflicht.

Ein nützlicheres Beispiel mit einer Variablen:

```epher
price = 100
if price > 50 then 2 else 1
```

```text
2
```

> epher hat keine Textwerte: beide Zweige eines `if` müssen Zahlen sein
> (oder Ergebnisse von Vergleichen).

### 1.8 Schleifen mit while

`while` wiederholt eine Anweisung, solange eine Bedingung gilt:

```epher
x = 0; while x < 5 do x = x + 1; x
```

```text
5
```

Lies das Skript so: *starte x bei 0; solange x kleiner als 5 ist, addiere 1
zu x; zeige dann x.* Das Ergebnis ist 5, weil die Schleife fünfmal lief.

> **Sicherheitsnetz:** epher stoppt jede Schleife nach 100.000 Schritten und zeigt
> `error: step limit exceeded`. Das schützt dich vor Schleifen, die nie
> enden würden. Wenn du das siehst, ist deine Bedingung vermutlich nie falsch geworden.

### 1.9 Eigene Funktionen mit def

Eine Funktion ist eine Berechnung mit einem Namen und Parametern:

```epher
def f(x) = x ^ 2
```

Dann verwende sie:

```epher
f(7)
```

```text
49
```

Funktionen können mehrere Parameter annehmen:

```epher
def area(w, h) = w * h
area(3, 4)
```

```text
12
```

Du kannst auch eine Funktion ohne Parameter definieren:

```epher
def answer() = 42
answer()
```

```text
42
```

### 1.10 Rekursion: eine Funktion, die sich selbst aufruft

Das berühmteste Beispiel sind die Fibonacci-Zahlen:

```epher
def fib(n) = if n <= 1 then n else fib(n - 1) + fib(n - 2)
```

```epher
fib(10)
```

```text
55
```

`fib(10)` ist die 10. Fibonacci-Zahl. Die Funktion ruft sich selbst mit
kleineren Argumenten auf, bis sie `n <= 1` erreicht. Das funktioniert,
weil die Form `if ... then ... else ...` nur den Zweig berechnet, den sie
braucht.

> Der Körper einer Funktion ist ein einzelner Ausdruck, eine Zeile. Kombiniere
> stattdessen mehrere Berechnungen mit `;` in einem Skript (nächster Abschnitt).

### 1.11 Skripte: mehrere Anweisungen auf einmal

Ein *Skript* sind mehrere Anweisungen, verbunden mit `;` oder mit
Zeilenumbrüchen, die genau dasselbe bedeuten. Sie werden nacheinander
ausgeführt:

```epher
x = 10; y = x + 5; x + y
```

```text
25
```

Mit Skripten baust du kleine Programme: Variablen einrichten, Schleifen
laufen lassen und ein Endergebnis zeigen.

Zeilenumbrüche und `;` sind dasselbe Trennzeichen, und du kannst sie frei
mischen. Der Button **Kopieren** über einem mehrzeiligen Beispiel kopiert
das ganze Skript, und du kannst es direkt in epher einfügen: das
Eingabefeld in der Web-App und in der Desktop-App, die
Terminal-Oberfläche und `epher repl` führen alle jede Zeile der Reihe
nach aus, genau so, als hättest du sie eine nach der anderen getippt.
Mehrere Anweisungen mit `;` in einer Zeile zu verbinden, funktioniert
ebenfalls überall, auch in der Einmal-Befehlszeile (Abschnitt 4.1).


Skripte können **Kommentare** tragen - Notizen für dich, die epher überspringt, geschrieben wie in PHP. `//` oder `#` kommentiert bis zum Zeilenende; `/* ... */` kommentiert einen Block aus, über Zeilen hinweg oder inline zwischen Token:

```epher
// a small script with notes
r = 3 # radius in metres
area = /* pi r squared */ pi * r ^ 2
area
```
### 1.12 Exakte Ergebnisse: frac, dec und big

Normalerweise rechnet epher mit Dezimalzahlen wie ein Taschenrechner.
Manche Zahlen sehen exakt besser aus.

**frac(n, d)** erzeugt einen exakten Bruch:

```epher
1 / 3
```

```text
0.3333333333333333
```

```epher
frac(1, 3)
```

```text
1/3
```

Brüche bleiben bei Berechnungen exakt:

```epher
frac(1, 3) * 3
```

```text
1
```

**dec(x)** erzeugt eine exakte Dezimalzahl. Vergleiche diese beiden:

```epher
0.1 + 0.2
```

```text
0.30000000000000004
```

```epher
dec(0.1) + dec(0.2)
```

```text
0.3
```

Das erste Ergebnis ist der winzige Rundungsfehler, den jeder Computer bei
Dezimalzahlen macht. `dec()` beseitigt ihn.

**big(x)** erzeugt eine exakte ganze Zahl, für Werte, die zu groß für
einen Taschenrechner sind:

```epher
big(10 ^ 20)
```

```text
100000000000000000000
```

**Zahlensysteme** schreiben ganze Zahlen so, wie die Fachwelt sie notiert:
`0b` für binär, `0o` für oktal, `0x` für hexadezimal (das Präfix ändert
nur die Schreibweise, nie den Wert):

```epher
0b1010 + 0xFF
```

```text
265
```

Zurück geht es mit **bin(x)**, **oct(x)** und **hex(x)**. Sie liefern die
präfixbehaftete Schreibweise einer ganzen Zahl, direkt wieder einsetzbar:

```epher
hex(255)
bin(10)
```

```text
0xff
```
0b1010
```

**exact(x)** rekonstruiert den exakten Bruch hinter einem Dezimalergebnis: jeder Wert mit einem guten Bruch mit kleinem Nenner wird als solcher angezeigt. Dieselbe Rekonstruktion steckt hinter der Standardanzeige der Apps, daher erscheint `1 / 3` meist direkt als `1/3`:

```epher
exact(0.3333333333333333)
exact(0.30000000000000004)
```

```text
1/3
3/10
```

Ein irrationaler Wert wie `pi` hat keinen guten Bruch, `exact()` lässt ihn daher unverändert.

Die Anzeigebefehle schreiben eine Zahl in anderer Notation. **scientific(x)** nutzt eine Ziffer vor dem Exponenten, **engineering(x)** Exponenten in Dreierschritten (die Mantisse bleibt zwischen 1 und 1000), und **grouped(x)** setzt dünne Leerzeichen als Tausendertrenner:

```epher
scientific(12345)
engineering(12345)
engineering(0.5)
grouped(1234567.89)
```

```text
1.2345e4
12.345e3
500e-3
1 234 567.89
```

Auch die Web-App und das TUI bieten diese als Anzeigeoptionen an (siehe Kapitel 2.2 und 5.2): exakte Brüche an/aus, Auto-/wissenschaftliche/technische Notation und Tausendertrenner. Die Optionen ändern nur die Darstellung; die Werte bleiben darunter gewöhnliche Dezimalzahlen.

### ### 1.13 Eingebaute Funktionen

epher hat die Funktionen eines wissenschaftlichen Taschenrechners, nach
Familien gruppiert.

Trigonometrie arbeitet in Bogenmaß (Radiant). Nutze `deg` und `rad` zum
Umrechnen:

| Funktion | Bedeutung | Beispiel | Ergebnis |
|---|---|---|---|
| `sin(x)`, `cos(x)`, `tan(x)` | trigonometrische Funktionen | `sin(pi / 2)` | `1` |
| `asin(x)`, `acos(x)`, `atan(x)` | inverse trigonometrische Funktionen | `atan(1)` | `0.7853981633974483` |
| `atan2(y, x)` | Winkel des Punkts (x, y) | `atan2(1, 1)` | `0.7853981633974483` |
| `deg(x)` | Bogenmaß → Grad | `deg(pi)` | `180` |
| `rad(x)` | Grad → Bogenmaß | `rad(180)` | `3.141592653589793` |
| `sinh(x)`, `cosh(x)`, `tanh(x)` | hyperbolische Funktionen | `sinh(1)` | `1.1752011936438014` |
| `asinh(x)`, `acosh(x)`, `atanh(x)` | inverse hyperbolische Funktionen | `acosh(1)` | `0` |

Potenzen, Wurzeln und Logarithmen (auf einem Taschenrechner ist `log` die
Basis 10):

| Funktion | Bedeutung | Beispiel | Ergebnis |
|---|---|---|---|
| `sqrt(x)` | Quadratwurzel | `sqrt(16)` | `4` |
| `cbrt(x)` | Kubikwurzel | `cbrt(-27)` | `-3` |
| `root(n, x)` | n-te Wurzel | `root(3, 8)` | `2` |
| `exp(x)` | e hoch x | `exp(1)` | `2.718281828459045` |
| `ln(x)` | natürlicher Logarithmus | `ln(e)` | `1` |
| `log(x)` | Logarithmus zur Basis 10 | `log(100)` | `2` |
| `log2(x)` | Logarithmus zur Basis 2 | `log2(8)` | `3` |
| `logb(b, x)` | Logarithmus zur Basis b | `logb(2, 8)` | `3` |
| `hypot(a, b)` | Hypotenuse | `hypot(3, 4)` | `5` |
| `5!` (auch `fact(n)`) | Fakultät | `5!` | `120` |

Runden, Vorzeichen und ganze Zahlen:

| Funktion | Bedeutung | Beispiel | Ergebnis |
|---|---|---|---|
| `abs(x)` | Betrag | `abs(-3)` | `3` |
| `floor(x)` / `ceil(x)` | abrunden / aufrunden | `floor(2.7)` | `2` |
| `round(x)` | nächstgelegene, ab halb von null weg | `round(2.5)` | `3` |
| `trunc(x)` | Nachkommastellen abschneiden | `trunc(-2.9)` | `-2` |
| `sign(x)` | -1, 0 oder 1 | `sign(-5)` | `-1` |
| `ncr(n, r)` | Kombinationen | `ncr(52, 5)` | `2598960` |
| `npr(n, r)` | Permutationen | `npr(5, 2)` | `20` |
| `gcd(a, b)` / `lcm(a, b)` | gemeinsame Teiler und Vielfache | `gcd(12, 18)` | `6` |
| `mod(a, b)` | Rest | `mod(7, 3)` | `1` |

Primzahlen und Teiler arbeiten mit ganzen Zahlen:

| Funktion | Bedeutung | Beispiel | Ergebnis |
|---|---|---|---|
| `isprime(n)` | wahr, wenn n eine Primzahl ist | `isprime(97)` | `true` |
| `nextprime(n)` / `prevprime(n)` | die nächsten Primzahlen | `nextprime(10)` | `11` |
| `factors(n)` | Primfaktorzerlegung | `factors(360)` |
| Listenliteral | `{…}` | `{1, 2, 3}` |
| Listenelement | `list[i]` (ab 1) | `{5, 6}[2]` |
| Listenstatistik | `mean(liste)`, `median(liste)`, … | `stdev(d)` |
| Listenform | `len(s)`, `sort(s)`, `mode(s)`, `range(s)`, `quartile(s, k)` | `quartile(d, 1)` |
| Lineare Regression | `linreg(xs, ys)` | `linreg(x, y)` |
| Normalverteilung | `normpdf` `normcdf` `invnorm` | `invnorm(0.975)` |
| t-Verteilung | `tpdf` `tcdf` `invt` | `invt(0.975, 10)` |
| Chi-Quadrat | `chi2pdf` `chi2cdf` `invchi2` | `chi2cdf(3.84, 1)` |
| Diskrete Verteilungen | `binompdf` `binomcdf` `poissonpdf` `poissoncdf` | `binomcdf(2, 10, 0.5)` |
| Tests und Intervalle | `ztest` `ttest` `zinterval` `tinterval` `chisq_gof` | `tinterval(d, 0.95)` |
| Datenplots | `graph scatter(xs, ys)` `histogram(data)` `boxplot(data)` | `graph boxplot(d)` |
| Zufallszahlen | `random()`, `random(a, b)`, `randint(a, b)`, `randseed(n)` | `randint(1, 6)` |
| Konstanten-Browser | Hilfe → Konstanten: alle eingebauten Konstanten, nach Gruppe | Hilfe → Konstanten |
| Größe | `5 m`, `60 mile/hr`, `1 km` | `2 m^2` |
| Umrechnen | `expr in Einheit` oder `expr -> Einheit` | `72 km/hr in m/s` |
| Vorsätze | `k M G T m µ n p` skalieren jede Einheit | `5 km`, `3 MPa`, `1 GHz` |
| Bitweises Und, Oder | `a & b`, `a \| b` | `0xFF & 0x0F` |
| Bitweises exklusives Oder | `a xor b` | `5 xor 3` |
| Bitweises Nicht | `~a` | `~0` |
| Verschiebungen | `a << n`, `a >> n` | `1 << 8` |
| Wortbreite | `bits(n)` — 8, 16, 32, 64 | `bits(8)` |
| Implizite Beziehung | `graph lhs == rhs` | `graph x^2 + y^2 == 1` |
| Matrix-Literal | `[[1, 2], [3, 4]]` | `[[1, 2], [3, 4]] * [[5, 6], [7, 8]]` |
| Matrixfunktionen | `det` `inv` `transpose` `trace` `dim` `ref` `rref` | `rref([[2, 1, 5], [1, -1, 1]])` | `2^3 * 3^2 * 5` |
| `totient(n)` | Eulersche Phi-Funktion | `totient(12)` | `4` |
| `ndivisors(n)` | Anzahl der Teiler | `ndivisors(360)` | `24` |
| `modpow(b, e, m)` | b hoch e, modulo m, exakt | `modpow(2, 10, 1000)` | `24` |


Statistik nimmt beliebig viele Argumente:

| Funktion | Bedeutung | Beispiel | Ergebnis |
|---|---|---|---|
| `sum(...)` / `product(...)` | Summen und Produkte | `sum(1, 2, 3)` | `6` |
| `mean(...)` | Mittelwert | `mean(1, 2, 3)` | `2` |
| `median(...)` | mittlerer Wert | `median(1, 2, 3, 4)` | `2.5` |
| `min(...)` / `max(...)` | kleinster / größter Wert | `max(4, 1, 3)` | `4` |
| `variance(...)` / `stdev(...)` | Streuung der Werte | `stdev(2, 4)` | `1` |

Die exakten Ebenen aus Abschnitt 1.12 bleiben:

| Funktion | Bedeutung | Beispiel | Ergebnis |
|---|---|---|---|
| `frac(n, d)` | exakter Bruch | `frac(1, 3)` | `1/3` |
| `dec(x)` | exakte Dezimalzahl | `dec(0.1)` | `0.1` |
| `big(x)` | exakte ganze Zahl | `big(10 ^ 20)` | `100000000000000000000` |
| Binär, oktal, hexadezimal | `0b…`, `0o…`, `0x…` | `0xFF + 0b1` |
| Basisschreibweise | `bin(x)`, `oct(x)`, `hex(x)` | `hex(255)` |
| Primzahlen | `isprime(n)`, `factors(n)`, … | `factors(360)` |
| `bin(x)` / `oct(x)` / `hex(x)` | Schreibweise mit Präfix in Basis 2 / 8 / 16 | `hex(255)` | `0xff` |

Sie lassen sich wie alles andere kombinieren:

```epher
min(sqrt(16), 5)
```

```text
4
```

Die physikalischen Konstanten verwenden SI-Einheiten, wie die astronomischen in Abschnitt 1.16:

| Name | Bedeutung | Wert |
|---|---|---|
| `G` | Newtons Gravitationskonstante | 6.6743e-11 |
| `gamma` | Euler-Mascheroni-Konstante | 0.5772156649015329 |
| `q_e` | Elementarladung | 1.602176634e-19 |
| `ev` | Elektronenvolt in Joule | 1.602176634e-19 |
| `eps_0` | Permittivität des Vakuums | 8.8541878128e-12 |
| `mu_0` | Permeabilität des Vakuums | 1.25663706212e-6 |
| `z_0` | Wellenwiderstand des Vakuums | 376.730313668 |
| `m_e` | Masse des Elektrons | 9.1093837139e-31 |
| `m_p` | Masse des Protons | 1.67262192595e-27 |
| `m_n` | Masse des Neutrons | 1.67492750056e-27 |
| `m_u` | Atomare Masseneinheit | 1.66053906892e-27 |
| `a_0` | Bohrscher Radius | 5.29177210544e-11 |
| `alpha` | Feinstrukturkonstante | 0.0072973525643 |
| `r_inf` | Rydberg-Konstante | 10973731.568160 |
| `mu_b` | Bohrsches Magneton | 9.2740100783e-24 |
| `n_a` | Avogadro-Konstante | 6.02214076e23 |
| `faraday` | Faraday-Konstante, C/mol | 96485.33212 |
| `r_gas` | Universelle Gaskonstante | 8.31446261815324 |
| `atm` | Standardatmosphäre in Pascal | 101325 |
| `wien` | Wiensche Wellenlängenkonstante | 0.002897771955 |
| `phi_0` | Magnetisches Flussquantum | 2.067833848e-15 |
| `m_P` | Planck-Masse | 2.176434e-8 |
| `l_P` | Planck-Länge | 1.616255e-35 |
| `t_P` | Planck-Zeit | 5.391247e-44 |
| `r_e` | klassischer Elektronenradius | 2.8179403205e-15 |
| `lambda_c` | Compton-Wellenlänge | 2.42631023867e-12 |
| `mu_n` | Kernmagneton | 5.050783699e-27 |


### 1.14 Fehler lesen

Wenn etwas schiefläuft, sagt es dir epher, statt zu raten:

```epher
1 / 0
```

```text
error: division by zero
```

```epher
sqrt(-4)
```

```text
error: domain error: sqrt of negative number -4
```

```epher
unknown_name
```

```text
error: unknown name: unknown_name
```

```epher
foo(1)
```

```text
error: unknown name: foo
```

Das letzte Beispiel ist wichtig: epher sagt dir genau, welchen Namen es
nicht kennt, damit du deinen Ausdruck korrigieren kannst.

### 1.15 Kurzreferenz

| Was | Syntax | Beispiel |
|---|---|---|
| Addieren, Subtrahieren, Multiplizieren, Dividieren | `+ - * /` | `7 / 2` |
| Potenz | `^` (von rechts nach links) | `2 ^ 10` |
| Fakultät | `!` (nachgestellt) | `5!` |
| Prozent | `%` (nachgestellt) | `200 * (1 + 10%)` |
| Klammern | `( )` | `(2 + 3) * 4` |
| Konstanten | `pi`, `e`, `tau`, `phi` | `2 * pi` |
| Wissenschaftliche Notation | `2.5e-3` | `6.02e23` |
| Vergleichen | `> < >= <= == !=` | `3 >= 2` |
| Logik | `and or not` | `a > 1 and a < 10` |
| Variable | `name = value` | `x = 5` |
| Konstante | `const name = value` | `const tax = 0.2` |
| Entscheidung | `if c then a else b` | `if x > 0 then 1 else -1` |
| Schleife | `while c do statement` | `while x < 5 do x = x + 1` |
| Funktion | `def name(params) = expr` | `def f(x) = x ^ 2` |
| Skript | Anweisungen, verbunden mit `;` oder Zeilenumbrüchen | `x = 1; x + 1` |
| Exakter Bruch | `frac(n, d)` | `frac(1, 3)` |
| Exakte Dezimalzahl | `dec(x)` | `dec(0.1) + dec(0.2)` |
| Exakte ganze Zahl | `big(x)` | `big(10 ^ 20)` |
| Bruch rekonstruieren | `exact(x)` | `exact(0.3333333333333333)` |
| Wissenschaftlich, technisch, gruppiert | `scientific(x)` `engineering(x)` `grouped(x)` | `engineering(12345)` |
| Imaginäre Einheit | `i`, oder ein Literal `4i` | `sqrt(-1)` |
| Komplexe Teile | `re(z)` `im(z)` `arg(z)` `conj(z)` `abs(z)` | `re(3 + 4i)` |
| Gleichung lösen | `solve lhs == rhs` | `solve x^2 == 9` |
| Numerische Ableitung | `derivative(expr, x)` | `derivative(x^2, 3)` |
| Bestimmtes Integral | `integral(expr, a, b)` | `integral(x^2, 0, 3)` |
| Binär, oktal, hexadezimal | `0b…`, `0o…`, `0x…` | `0xFF + 0b1` |
| Basisschreibweise | `bin(x)`, `oct(x)`, `hex(x)` | `hex(255)` |

### 1.16 Astronomie und das Sonnensystem

epher spricht Astronomie: Einheitssuffixe, physikalische Konstanten,
Kalender- und Zeitfunktionen sowie eine Live-Ephemeride für Sonne, Mond,
Planeten und Pluto. Alles funktioniert offline.

**Einheiten, die Astronomie sprechen.** Schreiben Sie eine Zahl gefolgt von
einem Einheitssuffix, und epher rechnet sofort in SI-Einheiten um:

| Suffix | Einheit | Rechnet um in |
|---|---|---|
| `AU` oder `au` | astronomische Einheit | Meter |
| `pc` | Parsec | Meter |
| `ly` | Lichtjahr | Meter |
| `deg` | Grad | Radiant |
| `arcmin`, `arcsec` | Bogenminute, Bogensekunde | Radiant |
| `min`, `hr`, `d`, `yr` | Minute, Stunde, Tag, julianisches Jahr | Sekunden |
| `Jy` | Jansky | W m-2 Hz-1 |

```epher
3.2 AU in m
```

```text
478713186240 m
```

```epher
sin(30 deg)
```

```text
0.5
```

Die Suffixe gehören zur Grammatik: Keine benutzerdefinierte Konstante kann
ändern, was `3.2 AU` bedeutet, und `h` bleibt die Planck-Konstante; Stunden
schreiben Sie als `hr`. Funktionen liefern Zahlen in natürlichen Einheiten;
ein Suffix wandelt eine Zahl in SI um, daher ist `mag2jy(20)` ein
Jansky-Wert und `mag2jy(20) Jy` derselbe Fluss in Watt pro Quadratmeter
Hertz.

**Astronomische Konstanten.** `au`, `pc`, `ly`, `c`, `g`, `h`, `h_bar`,
`k_b`, `sigma_sb`, `m_sun`, `r_sun`, `l_sun`, `m_earth`, `r_earth`, `m_moon`, `r_moon` wirken
wie `pi` und lassen sich wie jede Konstante überschatten.

**Datum und Zeit.** `jd(y, m, d [, hr])` und `mjd(...)` rechnen ein
Kalenderdatum in eine julianische Date um, `now()` liest den aktuellen
Zeitpunkt:

```epher
jd(2000, 1, 1, 12)
```

```text
2451545
```

`delta_t(jd)` ist die Korrektur TT - UT1, und `lst(jd, lon)` ist die
lokale Sternzeit in Stunden bei einem Ost-Längengrad in Grad.

**Stunden, Minuten und Sekunden.** `hms2deg(h, m, s)` rechnet Rektaszension
in Grad um, `dms2deg(d, m, s)` einen sexagesimalen Winkel, und
`deg2hms(x)` / `deg2dms(x)` schreiben einen Winkel als Text zurück:

```epher
deg2hms(90)
```

```text
6h 0m 0s
```

**Der Himmel, in Zahlen.** Jeder Zugriffsfunktion geben Sie eine
Körpersnummer: Merkur 1, Venus 2, Mars 4, Jupiter 5, Saturn 6, Uranus 7,
Neptun 8, Pluto 9, Sonne 10, Mond 11 (Die Erde ist 3, die Beobachterin,
niemals ein Ziel).

| Funktion | Bedeutung |
|---|---|
| `ra(b, jd)`, `decl(b, jd)` | geozentrische Rektaszension und Deklination (Grad) |
| `dist(b, jd)` | Entfernung in AE |
| `alt(b, jd, lat, lon)`, `az(b, jd, lat, lon)` | topozentrische Höhe und Azimut (Grad, wahr) |
| `rise(b, jd, lat, lon)`, `set(...)`, `transit(...)` | Ereignisse des lokalen Sonnentags, als julianische Dates |
| `mag(b, jd)` | scheinbare Helligkeit |
| `phase(b, jd)`, `illum(b, jd)` | Phasenwinkel (Grad) und beleuchteter Anteil |
| `diam(b, jd)` | scheinbarer Durchmesser (Grad) |

```epher
decl(10, jd(2000, 6, 21, 1.8))
```

```text
23.437882351
```

Breiten und Längen sind Grad, Ost positiv. Positionen sind geozentrisch,
außer ein Beobachter ist angegeben. Pluto rechnet mit einer genäherten
Bahn, ehrlich auf etwa eine Bogenminute, weit unter der Genauigkeit der
anderen Körper; Sonnen- und Mondfinsternisse sowie Konjunktionssuchen sind
nicht enthalten.

**Optik und Licht.** `kepler(M, e)` löst die Kepler-Gleichung,
`airmass(alt)` ist die sec(z)-Luftmasse, `dawes(d)` das Auflösungsvermögen
eines d-Millimeter-Objektivs in Bogensekunden, und `dist_mod(mu)` rechnet
ein Entfernungsmodul in Parsec um.

**Jahreszeiten.** `march_equinox(year)`, `june_solstice(year)`,
`september_equinox(year)` und `december_solstice(year)` liefern die
julianische Date jedes Jahreszeitenbeginns:

```epher
march_equinox(2000)
```

```text
2451623.8159797275
```

**Das Sonnensystem in 3D.** Der Befehl `solar3d` zeichnet das ganze
System: jede Bahn als Kurve, jeder Körper als beschrifteter Punkt, mit
einer Spur, die zeigt, wo er gerade war:

```epher
solar3d jd(2020, 7, 1)
```

Geben Sie die Zeit als Konstante an und drücken Sie die Wiedergabetaste,
um den Planeten zuzusehen: `const t = now(); solar3d t`. Ziehen Sie
mit der Maus oder nutzen Sie die Pfeiltasten zum Drehen, `clear` zum
Leeren und `solar3d save file.svg` zum Exportieren.

Die Ephemeride rechnet das Crate solar-ephemeris
(github.com/Protonmatter/sol), geprüft gegen JPL Horizons; Dank an seinen
Autor. Die Genauigkeit ist bogensekundenklassig für Sonne, Mond und
Planeten über etwa 5000 Jahre um die Gegenwart.

### 1.17 Komplexe Zahlen

epher rechnet automatisch mit komplexen Zahlen. Die imaginäre Einheit ist **i**, genau wie `pi`:

```epher
i ^ 2
sqrt(-1)
```

```text
-1
i
```

Schreiben Sie eine komplexe Zahl mit dem `i`-Suffix, ohne Multiplikationszeichen: `3 + 4i` ist ein Literal, `2.5i` funktioniert, ebenso die Basisliterale (`0xFFi`). Die übliche Arithmetik erweitert sich: Addieren, Subtrahieren, Multiplizieren, Dividieren und Potenzen funktionieren, und `i` folgt der normalen Rangfolge (`i ^ 2` bindet wie jede Potenz).

Auch die reellen Funktionen erweitern sich. Mit einem komplexen Argument rechnen sie in der komplexen Ebene; mit einem reellen Argument außerhalb ihres reellen Definitionsbereichs liefern sie das Hauptwert-Ergebnis statt eines Fehlers:

```epher
ln(-1)
asin(2)
exp(i * pi)
```

```text
3.141592653589793i
1.5707963267948966-1.3169578969248166i
-1+0.00000000000000012246467991473532i
```

(`exp(i * pi)` ist exakt `-1`; die letzten Ziffern sind das Rauschen von `sin(pi)` in der Arithmetik des Rechners.)

Vier Funktionen lesen die Teile einer komplexen Zahl, und `abs()` ist ihr Betrag:

```epher
re(3 + 4i)
im(3 + 4i)
arg(-1)
conj(3 - 4i)
abs(3 + 4i)
```

```text
3
4
3.141592653589793
3+4i
5
```

Nur-ganzzahlige Funktionen (`fact`, `gcd`, `floor`, `isprime`, ...) weisen komplexe Argumente mit einem Typfehler zurück.

### 1.18 Gleichungen lösen

**solve** findet die Nullstellen einer Gleichung in einer Variablen. Die Gleichung verwendet `==`:

```epher
solve x^2 == 5*x + 6
```

```text
x = -1, x = 6
```

Polynomiale Gleichungen (aus `+ - * ^` und Konstanten) liefern jede Nullstelle, reell und komplex:

```epher
solve x^2 == -1
solve x^2 + 2*x + 5 == 0
solve (x - 1)^2 == 0
```

```text
x = -i, x = i
x = -1-2i, x = -1+2i
x = 1
```

Die gesuchte Variable ist `x`, wenn sie vorkommt, sonst die einzige andere Variable. Konstanten und gebundene Variablen wirken als Parameter:

```epher
const k = 3
solve k*x == 12
```

```text
x = 4
```

Jede andere Gleichung wird numerisch über -100..100 abgetastet: Nullstellen werden über Vorzeichenwechsel eingeklammert, daher listet `solve sin(x) == 0.5` jede Nullstelle in diesem Bereich. Zwei ehrliche Einschränkungen: Eine Nullstelle, bei der die Funktion nur berührt (wie `x^2 == 0` über den numerischen Pfad), kann übersehen werden, und Gleichungen in mehreren ungebundenen Variablen sind ein Fehler.

### 1.19 Analysis: Ableitung und Integral

**derivative(expr, p)** ist die numerische Ableitung von `expr` an der Stelle `p`. Das erste Argument bleibt ein Ausdruck, und seine freie Variable ist die differenzierte:

```epher
derivative(x^2, 3)
derivative(sin(t), 0)
```

```text
6
1
```

Weil das Argument ein Ausdruck bleibt, ist die Ableitung grafisch darstellbar: `graph derivative(x^3 - x, x)` zeichnet die Steigungskurve.

**integral(expr, a, b)** ist das bestimmte Integral von `a` bis `b`, berechnet mit adaptiver Simpson-Quadratur:

```epher
integral(x^2, 0, 3)
integral(sin(x), 0, pi)
```

```text
9
2
```

`integral(x^2, 3, 0)` ist `-9` (das vorzeichenbehaftete Integral), und eine grafisch darstellbare obere Grenze funktioniert: `graph integral(x^2, 0, x)`.

Beide sind numerisch; die Ausdrücke müssen im Bereich reellwertig sein, und ein Ausdruck in mehreren Variablen ist ein Fehler.

### 1.20 Daten: Listen, Statistik und Regression

Eine Liste ist eine Zahlenreihe in geschweiften Klammern: `{1, 2, 3}`.
Die Elemente sind Ausdrücke, die leere Liste `{}` ist erlaubt, und eine
Liste wird wie jeder Wert an einen Namen gebunden:

```epher
d = {12, 15, 14, 16, 13, 15, 14, 17}
d[2]
len(d)
```

`list[i]` ist das i-te Element, 1-basiert wie ein Taschenrechner es
erwartet; ein Index außerhalb der Liste ist ein Fehler. Die Klammer
bindet enger als `^`, also ist `d[2]^2` gleich `(d[2])^2`.

Die Arithmetik über einer Liste ist elementweise; eine einzelne Zahl
wird auf jedes Element angewendet:

```epher
{1, 2, 3} * 2
{1, 2, 3} + 10
```

Zwei Listen müssen für `+ - * / ^` gleich lang sein. `==` und `!=`
vergleichen ganze Listen; Ordnungsvergleiche lehnen Listen ab.

Die Statistikfunktionen nehmen eine Liste als einziges Argument (die
Mehrfachargument-Form bleibt — `mean(1, 2, 3)` funktioniert weiter):
`sum product mean median mode variance stdev min max range`. Die
neuen Formfunktionen sind `len(liste)`, `sort(liste)` (aufsteigende
Kopie), `mode(liste)` (häufigster Wert, bei Gleichstand der kleinste),
`range(liste)` (größter minus kleinster Wert) und `quartile(liste, k)`
für k in 1..3 (Quartile nach TI-Art, Median der Hälften):

```epher
mean(d)
median(d)
quartile(d, 1)
```

**linreg(xs, ys)** passt die Ausgleichsgerade durch zwei gleich lange
Listen an und berichtet sie mit dem Korrelationskoeffizienten r:

```epher
linreg({1, 2, 3, 4}, {2.1, 4.2, 5.8, 8.1})
```

Die angepasste Gerade ist eine Anzeige wie die Lösungen von solve; das
Bild der Anpassung zeigt das Streudiagramm (Abschnitt 1.22).

### 1.21 Verteilungen und Hypothesentests

Die Wahrscheinlichkeitsfunktionen decken die Standardnormal-, die
t-, die Chi-Quadrat-, die Binomial- und die Poisson-Verteilung ab. Die
Normal-Familie nimmt ein oder drei Argumente — ein Argument ist die
Standardnormalverteilung:

```epher
normcdf(1.96)
invnorm(0.975)
normcdf(12, 10, 2)
```

`normpdf(x[, mu, sigma])`, `normcdf(x[, mu, sigma])`, `invnorm(p[,
mu, sigma])`; `tpdf(x, df)`, `tcdf(x, df)`, `invt(p, df)`;
`chi2pdf(x, df)`, `chi2cdf(x, df)`, `invchi2(p, df)`;
`binompdf(k, n, p)`, `binomcdf(k, n, p)`; `poissonpdf(k, lambda)`,
`poissoncdf(k, lambda)`. Die `inv*`-Funktionen beantworten die
umgekehrte Frage: `invt(0.975, 10)` ist der t-Wert, unter dem 97.5 %
der Masse liegt.

Die Tests nehmen eine Datenliste und melden die Prüfgröße und den
zweiseitigen p-Wert als Anzeigetext; die Intervalle melden
`(untere, obere)` auf dem von Ihnen genannten Niveau:

```epher
d = {12, 15, 14, 16, 13, 15, 14, 17}
ttest(d, 14)
tinterval(d, 0.95)
ztest(d, 14, 1.5)
chisq_gof({20, 30, 25, 25}, {25, 25, 25, 25})
```

`ttest(daten, mu0)` und `tinterval(daten, niveau)` verwenden die
Stichproben-Standardabweichung (n−1); `ztest(daten, mu0, sigma)` und
`zinterval(daten, sigma, niveau)` benötigen das bekannte sigma.
`chisq_gof(beobachtet, erwartet)` ist der Anpassungstest mit k−1
Freiheitsgraden. Die Ergebnisse sind Anzeigetexte: lesbar und
kopierbar, aber nicht rechnerisch weiterverwendbar.

### 1.22 Datenplots

Die Graph-Familie nimmt auch Listen: ein Streudiagramm, ein
Histogramm und ein Kastendiagramm. Ein Datenplot gehört wie ein
Sonnensystem allein zum Feld — der neueste Befehl gewinnt, und
`graph clear` leert es.

```epher
x = {1, 2, 3, 4, 5}
y = {2.1, 4.2, 5.8, 8.1, 9.9}
graph scatter(x, y)
```

```epher
graph histogram({1, 2, 2, 3, 3, 3, 4, 5})
```

```epher
graph boxplot({1, 2, 2, 3, 3, 3, 9})
```

**scatter(xs, ys)** zeichnet die Punkte und, ab zwei Punkten, die
Ausgleichsgerade mit der Beschriftung `y = a*x + b (r = …)` in der
Legende. **histogram(daten[, bins])** zeichnet ein
Häufigkeitshistogramm; die Klassenzahl ist optional (standardmäßig
nach Sturgess Regel) und muss eine ganze Zahl zwischen 1 und 50 sein.
**boxplot(daten)** zeichnet das Kastendiagramm: Minimum, Q1, Median,
Q3, Maximum, mit Antennen bis zu den Extremen. Das Fenster passt sich
immer den Daten an — die `from a to b`-Schlüsselwörter gelten nicht —
und das Bild exportiert und speichert wie jeder andere Plot.

### 1.23 Zufallszahlen

`random()` zieht eine gleichverteilte Zahl aus `[0, 1)`, `random(a, b)`
eine aus `[a, b)`, und `randint(a, b)` eine ganze Zahl aus dem
geschlossenen Bereich `[a, b]` — ein Würfelwurf:

```epher
randseed(7)
randint(1, 6)
```

```text
7
3
```

Die Folge ist reproduzierbar: `randseed(n)` setzt den Generator mit `n`
neu und meldet es, sodass derselbe Seed in jeder Sitzung und jeder
Oberfläche dieselben Ziehungen wiederholt.

### 1.24 Einheiten und Umrechnung

Eine Zahl mit einer Einheit dahinter wird zu einer *Größe*: dem Wert in
SI-Einheiten plus seinen Dimensionen. Die Einheitentabelle umfasst die
SI-Basis- und -abgeleiteten Einheiten (`m`, `s`, `kg`, `A`, `K`, `mol`,
`cd`, `Hz`, `N`, `Pa`, `J`, `W`, `C`, `V`, `F`, `ohm`, `S`, `Wb`, `T`,
`H`, `lm`, `lx`, `Bq`, `Gy`, `Sv`), die Alltagseinheiten (`min`, `hr`,
`d`, `yr`, `L`, `t`, `bar`, `atm`, `torr`, `psi`, `eV`, `mile`, `yd`,
`ft`, `inch`, `nmi`, `lb`, `oz`, `gal`, `qt`, `pt`, `mph`, `knot`) und
die Astronomie-Suffixe aus Abschnitt 1.16. Zusammengesetzte Einheiten
verkettet: `60 mile/hr` und `5 m/s^2` sind einzelne Einheiten.

```epher
60 mile/hr
```

```text
60 mile/hr
```

Die SI-Vorsätze skalieren jede davon: `k M G T m µ n p` sind Kilo,
Mega, Giga, Tera, Milli, Mikro, Nano, Piko — `5 km`, `3 MPa`, `1 GHz`
funktionieren alle, und `2 kg` ist das Kilogramm selbst.

Die Dimensionen werden geprüft: Addition oder Vergleich von Größen mit
verschiedenen Einheiten meldet einen Fehler, statt Meter und Sekunden
zu mischen:

```epher
5 m + 3 s
```

```text
error: dimension error: cannot add 5 m and 3 s
```

Die Arithmetik setzt die Dimensionen zusammen: `5 m * 3 m` ist
`15 m^2`, `(3 m)^2` ist `9 m^2`, `sqrt(4 m^2)` ist `2 m`, und ein
ganzer Ausdruck, dessen Dimensionen sich wegheben, ist wieder eine
gewöhnliche Zahl (`5 m / 5 m` ist `1`). Ergebnisse bevorzugen den
exakten abgeleiteten Namen, wenn die Dimensionen passen —
`5 kg * 3 m / 1 s^2` antwortet `15 N`.

**Umrechnung.** `expr in Einheit` (oder `expr -> Einheit`) zeigt eine
Größe in der genannten Einheit; die Dimensionen müssen übereinstimmen.
`in` bindet am lockersten der Operatoren, also rechnet
`5 m + 3 m in km` die ganze Summe um:

```epher
72 km/hr in m/s
```

```text
20 m/s
```

```epher
2 m^2 in cm^2
```

```text
20000 cm^2
```

Temperaturskalen (Celsius, Fahrenheit) sind hier keine Einheiten —
Kelvin schon, und `K` funktioniert wie jede andere.


### 1.25 Bitoperationen

Die Basisschreibweisen aus Abschnitt 1.13 sind dafür gemacht:
`0b101`, `0o17`, `0xFF`. Die Bitoperatoren arbeiten mit ganzen Zahlen
und antworten mit exakten ganzen Zahlen:

```epher
0xFF & 0x0F
```

```text
15
```

| Operator | Bedeutung |
|---|---|
| `a & b` | bitweises Und |
| `a \| b` | bitweises Oder |
| `a xor b` | bitweises exklusives Oder |
| `~a` | bitweises Nicht (Zweierkomplement) |
| `a << n` | links schieben (mal 2^n) |
| `a >> n` | rechts schieben, arithmetisch (durch 2^n, abrunden) |

Die Ergebnisse sind exakte `big`-Ganzzahlen, also behält `1 << 60`
jede Ziffer. Die Wortbreite ist standardmäßig 64 Bit: Ergebnisse
werden als vorzeichenbehaftetes Zweierkomplement gelesen, also ist
`~0` = -1 und `1 << 100` wickelt auf 0. `bits(n)` ändert die Wortbreite
auf 8, 16, 32 oder 64, und `bits()` meldet sie:

```epher
bits(8)
~0
```

```text
8
-1
```

Eine negative Verschiebung kehrt die Richtung um (`8 << -1` ist `4`).
Das boolesche `and` und `or` behalten ihre Bedeutung; `&` und `|` sind
die Bit-Schreibweisen.


### 1.26 Implizite Beziehungen

Eine Gleichung mit zwei Unbekannten wird als Kurve gezeichnet: die Graph-Familie tastet die Beziehung mit Marching Squares ab und zeichnet ihre Null-Kontur. Kreis, Parabel und senkrechte Gerade — je ein Befehl:

```epher
graph x^2 + y^2 == 1
```

```epher
graph y == x^2
```

```epher
graph x == 2
```

Die Beziehung wird über dem Quadrat aus `from a to b` (oder dem Standardfenster) abgetastet, also passt `graph x^2 + y^2 == 1 from -2 to 2` das Fenster an den Kreis. Alles, was eine Kurve kann, gilt auch hier: die Legende beschriftet die Gleichung, die Schieber animieren ihre Konstanten, und das Bild zoomt, verschiebt und exportiert wie jedes andere. Die Ungleichungsfüllungen (`y < …`, `y > …`) bleiben schattierte Kurven; eine Beziehung hat keine Punkte von Interesse.


### 1.27 Matrizen

Eine Matrix ist ein Zahlenraster, geschrieben als Zeilen aus Listen:
`[[1, 2], [3, 4]]` ist die 2×2-Matrix. `+` und `-` wirken elementweise
(gleiche Formen), `*` ist das Matrixprodukt, eine Zahl skaliert
elementweise, und `^` ist die ganzzahlige Matrixpotenz (`A ^ 0` ist
die Einheitsmatrix, Potenzen brauchen also quadratische Matrizen).
`M[2][1]` ist das Element in Zeile 2, Spalte 1 — Zeilen indizieren wie
Listen, ab 1.

```epher
[[1, 2], [3, 4]] * [[5, 6], [7, 8]]
```

```text
[[19, 22], [43, 50]]
```

Die Matrixfunktionen decken den Klassenraum-Bedarf: `det(M)` (nur
quadratisch), `inv(M)` (singuläre Matrizen sind ein Fehler),
`transpose(M)`, `trace(M)` (quadratisch), `dim(M)` (die Liste
`{Zeilen, Spalten}`) und `ref(M)` mit `rref(M)` für die
Zeilenreduktion. Lineare Gleichungssysteme lösen sich über rref auf
der erweiterten Matrix:

```epher
rref([[2, 1, 5], [1, -1, 1]])
```

```text
[[1, 0, 2], [0, 1, 1]]
```

Die Zeilen lesen `x = 2`, `y = 1` — die letzte Spalte der reduzierten
erweiterten Matrix. Exakte Brüche erscheinen in Matrizen wie in
Listen, also zeigt `inv([[1, 2], [3, 4]])` `[[-2, 1], [3/2, -1/2]]`.


## 2. Die Web-App (PWA)

### 2.1 Sie öffnen

Die Web-App liegt unter:

```text
https://epher.org/pwa/
```

Keine Installation nötig. Sie funktioniert in jedem modernen Browser auf
Computer, Telefon oder Tablet.

Dieses Handbuch ist auch in die App eingebaut: öffne **Help → User guide**
in der Menüleiste (tippe auf einem Telefon auf **☰**), um es in der App in
der aktuell eingestellten Sprache zu lesen. Tippe ein beliebiges Beispiel
in diesem Handbuch an, um es ins Eingabefeld zu laden. **Hilfe → Konstanten** öffnet den Konstanten-Browser: alle eingebauten Konstanten in Gruppen (Mathematik, Astronomie, Physik, Chemie), jede mit ihrem Wert und einer kurzen Beschreibung; tippe eine an, um ihren Namen ins Eingabefeld einzufügen, und das Suchfeld filtert die Liste.

### 2.2 Deine erste Berechnung

1. Klicke auf das Textfeld (es ist beim Laden der Seite bereits fokussiert).
2. Tippe einen Ausdruck, zum Beispiel `2 + 3 * 4`.
3. Drücke **Enter** oder klicke auf den Button **=**.

Das Ergebnis erscheint in großer Schrift unter dem Feld. Alles aus
Kapitel 1 funktioniert hier, einschließlich Variablen, Funktionen und
Skripten.

Während du einen Namen tippst, erscheint unter dem Feld eine Vorschlagsliste: die Pfeile bewegen die Markierung, **Enter** oder **Tab** übernimmt, **Esc** schließt, und ein Klick übernimmt, ohne die Tastatur zu verlassen. Jeder Vorschlag trägt eine kurze Beschreibung der Funktion oder Konstante. **F1** zeigt dieselbe Beschreibung für das Wort unter dem Cursor in der Hinweisleiste über dem Tastenfeld. Beginnt eine leere Eingabe mit einem Operator (`+ - * / ^ % !`), fügt epher `ans` ein, und die Zeile macht mit dem letzten Ergebnis weiter.

Das Menü **Einstellungen** (das Zahnrad-Symbol oder **☰ → Einstellungen** am Telefon) enthält drei Gruppen. **Design** und **Sprache** tun, was ihre Namen sagen. **Ergebnisse** bestimmt die Darstellung der Antworten: exakte Brüche (standardmäßig an, so wird `1 / 3` als `1/3` angezeigt), die Notation (Auto, wissenschaftlich oder technisch) und Tausendertrenner. Das sind reine Anzeigeoptionen; die Werte darunter bleiben gewöhnliche Zahlen.

### 2.3 Verlauf

Jede Berechnung wird zur Verlaufsliste unter dem Ergebnis hinzugefügt,
damit du zurückscrollen und sehen kannst, was du gemacht hast. Die neuesten
Einträge erscheinen oben, und das Mülleimer-Symbol neben der
Überschrift **Verlauf** leert sie (im Terminal: Strg+L oder ein Klick
auf dasselbe Symbol). Der Verlauf bleibt erhalten, solange die Seite offen
ist.

Jeder Eintrag liegt zwischen dünnen Trennlinien: ein einzeiliger Ausdruck ist eine Zeile, und ein mehrzeiliges Skript ist ein Eintrag, der alle seine Zeilen zeigt. Klicke einen Eintrag an, um ihn zurück ins Eingabefeld zu laden und erneut auszuführen.

### 2.4 Graphen zeichnen

Tippe `graph`, gefolgt von einem Ausdruck, und drücke **Enter**:

```epher
graph x ^ 2
```

epher zeichnet die Kurve y = f(x) von x = −10 bis x = 10 unterhalb des
Eingabefelds, auf einem Raster mit beschrifteten Achsen. Du kannst jeden
Ausdruck zeichnen, auch deine eigenen Funktionen:

```epher
def f(x) = x ^ 3
graph f(x)
```

Jede `graph`-Zeile fügt demselben Plot eine weitere Kurve hinzu, jede mit
eigener Farbe. Die Kurven sind alle durchgezogen, und die Legende
und die Beschriftungen unterscheiden sie ohne Farbe.
`graph clear` leert den Plot, und ein Button **Clear graph** oben im
Graph-Panel macht dasselbe für Kurven und 3D-Flächen zusammen. Die TUI
behält den Befehl in ihrem **Graph**-Menü.

Ganz oben im Graph-Bereich, neben **Clear graph** und **Copy SVG**,
blendest du in der Symbolleiste die Liste der besonderen Punkte und die
hervorgehobenen Punkte im Plot selbst aus. Direkt über jedem Plot liegt
eine Leiste mit Symbol-Reglern, die Worte stehen im jeweiligen
Hinweisfeld: Linienstärke (0 bis 4 in Schritten von 0.1 für
2D-Kurven, 0 bis 0.2 in Schritten von 0.01 für 3D-Flächen - nur
der Regler der gerade sichtbaren Art wird angezeigt, und jede Art merkt
sich ihren eigenen Wert), und bei 3D und dem Sonnensystem die horizontale
und vertikale Drehgeschwindigkeit sowie die Zoomgeschwindigkeit.
Jeder Eintrag in der Legende hat ein Kästchen, standardmäßig aktiviert:
Abhaken blendet die Kurve aus dem Plot, ihren besonderen Punkten und dem
SVG-Export aus.

```epher
graph x ^ 2
graph x ^ 3
```

Punkte, an denen der Ausdruck keinen Wert hat (zum Beispiel eine Division
durch null), werden übersprungen und hinterlassen eine Lücke in der
Kurve. Ein Sprung, der eigentlich eine senkrechte Asymptote ist,
wird nie als Verbindungslinie gezeichnet.

#### 2.4.1 Was du zeichnen kannst

Ein Definitionsbereich deiner Wahl:

```epher
graph sin(x) from 0 to 2*pi
```

Parametrische Kurven (t läuft von 0 bis 2π):

```epher
graph param 2*cos(t), 3*sin(t)
```

Polarkurven:

```epher
graph polar 1 + cos(theta)
```

Bereiche: `y <` schattiert die Fläche unter der Kurve, `y >` schattiert darüber:

```epher
graph y < x ^ 2
```
#### 2.4.2 Den Plot lesen

**Verfolgen:** Bewege den Zeiger über den Plot oder fokussiere ihn und
drücke die Pfeiltasten. Der nächstgelegene Punkt auf einer Kurve
wird markiert, seine Koordinaten werden unter dem Plot angezeigt.

**Besondere Punkte:** Nach jedem graph-Befehl findet epher die Nullstellen
und Extrempunkte jeder Kurve und die Schnittpunkte zwischen Kurven,
markiert sie im Plot und listet sie darunter auf:

```text
root (-1, 0)   minimum (0, 0)   root (1, 0)
```

**Tabellen:** Der Befehl `table` druckt eine Wertetabelle (Zeilen, an
denen der Ausdruck keinen Wert hat, bleiben leer):

Ein optionaler `derivative <ausdruck>`-Zusatz fügt eine dritte
Spalte hinzu, die numerische Ableitung dieses Ausdrucks an jeder
Stelle x:

```epher
table x ^ 2 from -2 to 2 points 5 derivative x ^ 2
```

```text
         x           y          y'
        -2           4          -4
        -1           1          -2
         0           0           0
         1           1           2
         2           4           4
```

Die Tabellenzellen folgen den Ergebniseinstellungen: bei
eingeschalteten exakten Brüchen (Standard) zeigt sich ein Wert, der
ein einfacher Bruch ist, als solcher — `table x / 3 from 0 to 1
points 4` listet `1/3` statt `0.333`.
```epher
table x ^ 2 from -2 to 2 points 5
```

```text
         x           y
        -2           4
        -1           1
         0           0
         1           1
         2           4
```

#### 2.4.3 Schieberegler und Export

Definiere eine Konstante, verwende sie in einem Graphen, und unter dem
Plot erscheint ein Schieberegler. Ziehe ihn (oder bewege ihn mit den
Pfeiltasten) und jede Kurve wird neu gezeichnet:

```epher
const a = 1
graph a * x ^ 2
```

**SVG kopieren** kopiert den aktuellen Plot als eigenständiges SVG-Bild
zum Einfügen in Dokumente. Die Farben sind eingebaut, es sieht überall
gleich aus. **PNG speichern** speichert dasselbe Bild als Bitmap in doppelter Größe, damit die Kurven scharf bleiben; die Desktop-App fragt, wohin, die Web-App legt es in den Download-Ordner (oder fragt, wo der Browser es anbietet). Die Reglerzeilen und animierten Konstanten sitzen direkt
unter dem Plot, über der Liste der besonderen Punkte.

#### 2.4.4 3D-Flächen

`graph3d` zeichnet eine Fläche z = f(x, y) über einem quadratischen
Bereich (−5 bis 5 oder dein `from a to b`):

```epher
graph3d x ^ 2 - y ^ 2
```

Netzlinien, die näher bei dir liegen, werden kräftiger gezeichnet, sodass
die Form räumliche Tiefe bekommt. Mehrere `graph3d`-Zeilen überlagern
sich wie Kurven, und `graph3d clear` leert den Plot. Drehe die Ansicht
durch Ziehen oder fokussiere den Plot und verwende die Pfeiltasten. Die
TUI zeichnet dieselbe Fläche als ASCII-Drahtgitter; mit den Pfeiltasten
drehst du es.

#### 2.4.5 Animation

Jeder Schieberegler hat eine Wiedergabetaste. Sie lässt seine Konstante
den Bereich des Schiebereglers durchlaufen und beginnt danach wieder von
vorn. So animieren Taschenrechner üblicherweise: Du animierst einen
Parameter, und alles, was ihn verwendet, bewegt sich mit. Drücke die Taste erneut, um zu
pausieren.

Eine "Zeit"-Variable ist nur eine Konstante, die du animierst:

```epher
const t = 0
graph sin(x - t)
```

Wenn du den Schieberegler von t abspielst, wandert die Welle.
3D-Flächen animieren sich genauso. Definiere zuerst eine Konstante und
spiele dann ihren Schieberegler ab:

```epher
const a = 1
graph3d sin(a * (x ^ 2 + y ^ 2)) from -3 to 3
```

In der TUI startet und stoppt die Leertaste die Animation.

### 2.5 Installieren und offline nutzen

Die Web-App ist eine *progressive Web-App*: nach einem Besuch funktioniert
sie vollständig offline, und du kannst sie wie eine normale App
installieren.

- **Chrome, Edge oder Android:** klicke auf das Installieren-Symbol in der
  Adressleiste (oder *App installieren* im Browser-Menü) und bestätige.
- **iPhone / iPad (Safari):** tippe auf **Teilen** → **Zum Home-Bildschirm**.
- **Andere Browser:** suche im Menü nach *Installieren* oder *Zum
  Home-Bildschirm hinzufügen*.

Sobald sie installiert ist, starte sie über deinen Home-Bildschirm oder
deine App-Liste. Sie öffnet sich sofort, auch ohne Internetverbindung.

### 2.6 Was die Web-App nicht kann

Die Web-App hält deine Arbeit in der aktuellen Sitzung: Sie wertet
Ausdrücke aus, zeichnet ihre Graphen (Abschnitt 2.4) und führt einen
Verlauf. Die Befehle **save**, **save script** und **language**
funktionieren in der Desktop-, Befehlszeilen- und Terminal-Version
(Kapitel 3, 4 und 5). In der Web-App antworten sie mit einem Hinweis,
dass Speichern dort funktioniert. Der Verlauf wird zwischen Besuchen
nicht gespeichert.

## 3. Die Desktop-App

Die Desktop-App ist ein normales Fenster um dieselbe Web-App herum. Alles
aus Kapitel 2 gilt; der Unterschied liegt nur darin, wie du sie
installierst und startest.

### 3.1 Installieren

Lade von der epher-Website einen Installer für dein System herunter:

- **Windows:** führe `epher-windows-x86_64.exe` aus. Der Installer legt
  `epher` in deinen PATH. Öffne ein neues CMD- oder PowerShell-Fenster
  und `epher "2 + 2"` funktioniert. Da der Build nicht signiert ist, wähle
  beim ersten Start *Weitere Informationen* → *Trotzdem ausführen*.
- **macOS:** öffne `epher-macos-aarch64.dmg` und ziehe epher in Programme.
  Da der Build nicht signiert ist, braucht der erste Start einen
  Rechtsklick → **Öffnen**.
- **Linux (Debian/Ubuntu):** das Paket `.deb`

```sh
sudo apt install ./epher-linux-x86_64.deb
```

- **Linux (Fedora/RHEL):** das Paket `.rpm`

```sh
sudo dnf install ./epher-linux-x86_64.rpm
```

- **Linux (jede Distribution, auch Arch):** das AppImage. Mach es
  ausführbar und starte es:

```sh
chmod +x epher-linux-x86_64.AppImage
./epher-linux-x86_64.AppImage
```

Jeder Installer enthält *ganz* epher: die Desktop-App, die Befehlszeile
(Kapitel 4) und die Terminal-Oberfläche (Kapitel 5), als den einzigen
Befehl `epher`. Unter Linux legt das Paket `epher` in `/usr/bin` ab.

### 3.2 Verwenden

Starte epher wie jede andere Anwendung. Du bekommst ein Fenster mit
derselben Oberfläche wie die Web-App: tippe einen Ausdruck, drücke
**Enter** oder klicke auf **=**, und lies das Ergebnis. Graphen zeichnen
funktioniert auch hier. `graph x ^ 2` zeichnet im Fenster (Kapitel 2.4).
Das Fenster lässt sich frei skalieren. Die Menüleiste enthält
**Help → User guide**, dasselbe Handbuch wie diese Seite, mit antippbaren
Beispielen.

Du kannst es auch aus einem Terminal öffnen: ein bloßes `epher` (oder
`epher gui`) startet die Desktop-App. Verwende unter macOS den Button
**Install the epher command** in der App, um `epher` in den PATH deines
Terminals zu legen.

### 3.3 Speicherung: ein Speicher, gemeinsam mit CLI und TUI

Die Desktop-App teilt ihren Speicher mit der Befehlszeilen- und der
Terminal-Version. Funktionen, Konstanten, Skripte, Verlauf und die
Sprachpräferenz leben an einem Ort, `~/.epher` auf deinem Computer (oder
`EPHER_STORE_DIR`, Kapitel 4.6), und alles, was in einer Version
gespeichert wurde, ist in den anderen verfügbar:

```text
def area(w, h) = w * h
save area
```

Definiere `area` in der Desktop-App, speichere sie mit `save`, schließe
das Fenster. Dann öffne die CLI und `area(3, 4)` funktioniert einfach.
Andersherum geht es auch: Funktionen und Skripte, die du in der CLI oder
TUI gespeichert hast, sind beim Öffnen des Desktop-Fensters schon da,
einschließlich Variablen, die gespeicherte Skripte gesetzt haben. Die
Befehle `save`, `save script` und `language` aus Kapitel 4 funktionieren
hier genau gleich.

Befehle, die du in der CLI, der REPL, der TUI oder der Desktop-App
eingibst, landen alle im selben Verlauf, und die Sitzung wandert mit:
Variablen, die du zuweist, und der `ans`-Wert folgen dir von einer
Version zur nächsten. Der gemeinsame Speicher ist live: Wenn zwei
Versionen gleichzeitig geöffnet sind, erscheint eine Änderung in der
einen sofort in der anderen (die Desktop-App und die TUI beobachten den
Speicher und aktualisieren sich von selbst).

> Die Web-App im Browser ist die eine Version, die diesen Speicher nicht
> nutzt. Sie behält jede Sitzung für sich (Kapitel 2.6).

## 4. Die Befehlszeile (CLI)

Die CLI ist die Textseite desselben `epher`-Programms wie die Desktop-App.
Sie hat drei Modi: einmalige Auswertung, gepipete Skripte und eine
interaktive Sitzung für längere Arbeit.

Für Hilfe zu jeder Zeit führe `epher --help` aus (alle Befehle, mit
Beispielen) oder `epher help` (das vollständige Handbuch; bei
Linux-Paketen ist das die Seite `man epher`).

### 4.1 Einmalige Berechnungen

Gib den Ausdruck als Argument an:

```sh
epher "2 + 3 * 4"
```

```text
14
```

Du kannst alles aus Kapitel 1 machen, das ein einzelner Ausdruck ist:

```sh
epher "if 3 > 2 then 10 else 20"
```

```text
10
```

Ein Ausdruck, der mit einem Minuszeichen beginnt, funktioniert direkt:

```sh
epher "-2 + 5"
```

```text
3
```

Der Einmal-Modus ist für Skripte, von einem einzelnen Ausdruck bis zu
einem ganzen Programm. Der Wert jeder Anweisung wird in einer eigenen
Zeile ausgegeben:

```sh
epher "x = 10; x + 5"
```

```text
10
15
```

Anweisungen, verbunden mit Zeilenumbrüchen, funktionieren im Argument
genauso. Alles aus Kapitel 1 ist verfügbar: Variablen, Funktionen,
Schleifen, alles. Die Zeilen teilen eine Sitzung, wie ein gepipetes
Skript (Abschnitt 4.2).

### 4.2 Gepipete Skripte

`epher -` liest Ausdrücke aus der Standardeingabe, Zeile für Zeile, so
wie Skriptsprachen in Pipelines verwendet werden:

```sh
printf "x = 3\nx * 10\n" | epher -
```

```text
= 3
= 30
```

Alles aus Kapitel 1 funktioniert, und die Zeilen teilen eine Sitzung:
Eine Funktion, die in einer frühen Zeile definiert wurde, ist später
verfügbar, und `save` schreibt wie immer in denselben Speicher. Fehler
werden ausgegeben und das Skript läuft weiter. Eine Zeile kann mehrere
Anweisungen mit `;` verbinden. Zeilenumbrüche und `;` bedeuten überall
in epher dasselbe.


Eine Datei funktioniert genauso: `epher plots/sine.es` führt jede Zeile der Datei der Reihe nach aus und druckt jedes Ergebnis. Das Argument gilt als Datei, wenn es eine vorhandene Datei benennt und einen `.`, `/` oder `\` enthält - `epher x` wertet also weiterhin den Namen `x` aus.
### 4.3 Die interaktive Sitzung (REPL)

Starte sie mit `epher repl`:

```sh
epher repl
```

> Ein bloßes `epher` ohne Argumente öffnet die Desktop-App (Kapitel 3).

epher zeigt seinen Prompt und wartet:

```text
epher>
```

Tippe nun irgendetwas aus Kapitel 1, eine Zeile nach der anderen.
Variablen behalten ihre Werte zwischen den Zeilen:

```text
epher> x = 5
= 5
epher> x ^ 2
= 25
```

Der Befehl `table` (Abschnitt 2.4.2) druckt auch hier eine Wertetabelle:

```text
epher> table x ^ 2 from -2 to 2 points 5
         x           y
        -2           4
        -1           1
         0           0
         1  Auch `graph`-Zeilen funktionieren hier: Die Kurven sammeln sich über die
Zeilen, und `graph save plot.svg` schreibt dasselbe SVG-Bild, das die
Schaltfläche **SVG kopieren** der Web-App liefert. `graph3d
save datei.svg` speichert eine 3D-Fläche auf dieselbe Weise. Dieselben
graph-Zeilen gelten auch in Einzeilern und gepipeten Skripten:
`epher "graph sin(x); graph save plot.svg"` ist ein fertiger Plot in
einem Befehl.

         1
         2           4
```

Jede Antwort wird als `= result` angezeigt. Zum Verlassen tippe `quit`
(oder `exit`):

```text
epher> quit
```

Dein Verlauf wird gemerkt: Wenn du das nächste Mal `epher repl` ausführst,
sind die Zeilen der vorherigen Sitzung noch da.


Der Befehl `load` führt ein Skript aus - einen Dateipfad oder den Namen eines mit `save script` gespeicherten Skripts - Zeile für Zeile, genau als hättest du es eingetippt:

```text
epher> load plots/sine.es
epher> load my_setup
```
### 4.4 Funktionen, Konstanten und Skripte speichern

Definiere eine Funktion und speichere sie dann:

```text
epher> def fib(n) = if n <= 1 then n else fib(n - 1) + fib(n - 2)
epher> save fib
saved fib
```

Der Befehl `save fib` legt die Funktion auf der Festplatte ab. Wenn du die
Sitzung das nächste Mal startest, ist `fib` bereits definiert:

```text
epher> fib(10)
= 55
```

Konstanten speichern sich genauso. `save` auf den Namen der Konstante:

```text
epher> const tax = 0.2
= 0.2
epher> save tax
saved tax
```

Um ein ganzes Skript zu speichern (die letzte Zeile, die du getippt
hast), verwende `save script`:

```text
epher> x = 0; while x < 5 do x = x + 1; x
= 5
epher> save script count_to_five
saved script count_to_five
```

Gespeicherte Skripte laufen beim Start von epher automatisch, sodass
alles, was sie definieren, für dich bereitsteht.


Du kannst ein gespeichertes Skript auch bei Bedarf mit `load count_to_five` laden, es als einfache Datei behalten und `load count_to_five.es` ausführen; `epher count_to_five.es` startet es direkt von der Kommandozeile (Abschnitt 4.2).
### 4.5 Die Sprache der Oberfläche ändern

Die Sprache der Oberfläche wird aus den Sprachen gewählt, die du auf
deinem Gerät eingestellt hast. Um sie zu überschreiben, tippe `language`,
gefolgt von einer dieser: `en`, `zh-CN`, `hi`, `es`, `fr`, `ar`, `de`,
`pt`:

```text
epher> language fr
language set to fr
```

Die Wahl wird für das nächste Mal gemerkt. Hinweis: Die Sprache, die du
*tippst*, die Ausdruckssprache, ist immer dieselbe, egal welche Sprache
die Oberfläche hat.

### 4.6 Wo deine Daten leben

Funktionen, Skripte, Verlauf und deine Sprachwahl werden in einem Ordner
auf deinem Computer gespeichert:

```text
~/.epher
```

Lösche diesen Ordner, um ganz von vorn zu beginnen. Um einen anderen Ort
zu verwenden, setze die Umgebungsvariable `EPHER_STORE_DIR`, bevor du
epher startest:

```sh
EPHER_STORE_DIR=/tmp/my-epher epher repl
```

## 5. Die Terminal-Oberfläche (TUI)

Die TUI ist eine Vollbild-Version der interaktiven Sitzung, in deinem
Terminal. Sie ist Teil desselben `epher`-Programms. Starte sie mit:

```sh
epher tui
```

### 5.1 Der Bildschirm

Der Bildschirm ist in Panels unterteilt:

- **Ausdruck**: das Eingabefeld (oben). Shift+Enter beginnt eine neue
  Zeile; die Pfeiltasten oder ein Mausklick bewegen den Cursor im Text.
- Das aktuelle **Ergebnis** direkt darunter.
- **Verlauf**: jede Zeile, die du eingegeben hast, mit ihrer Antwort.
- **Graph**: die Zeichnung aus dem Befehl `graph` (unten).
- Eine Hinweiszeile zeigt die Tastenkürzel.

### 5.2 Tasten

| Taste | Aktion |
|---|---|
| Tippen | am Cursor zum Ausdruck hinzufügen |
| **Enter** | das ganze Skript auswerten (eine mehrzeilige Eingabe läuft als ein Verlaufseintrag) |
| **Shift+Enter** | eine neue Zeile beginnen |
| **← → ↑ ↓** | den Cursor bewegen (bei leerer Eingabe: die 3D-Ansicht drehen) |
| **Esc** | die Eingabezeile leeren |
| **F1** | die Funktion unter dem Cursor beschreiben (in der Ergebniszeile) |
| **Ctrl+C** | beenden |
| **q** | beenden (wenn die Eingabe leer ist) |
| **Arrow keys** | die 3D-Ansicht drehen (wenn die Eingabe leer ist) |
| **Space** | die Animation starten/stoppen (wenn die Eingabe leer ist) |
| **F10** | die Menüs öffnen (Datei, Bearbeiten, Graph, Einstellungen, Hilfe) |
| **Tab** | das immer sichtbare Tastenfeld fokussieren (bzw. den Verlauf, vom Tastenfeld aus); Gruppen wechseln (**Esc** zurück zum Tippen) |
| **Maus** | Menüs und Menüeinträge anklicken, Tastenfeld-Zellen und Bank-Registerkarten, Verlaufszeilen (lädt den Ausdruck); im Grafikfeld ziehen, um zu drehen (3D) bzw. zu verschieben (2D), das Rad zoomt, ein Doppelklick setzt die Ansicht zurück |
| **Ctrl+L** | den Verlauf leeren |

Das Menü **Hilfe** öffnet das eingebaute Handbuch, die Tastenfeld-Hilfe und einen Konstanten-Browser: die Konstanten in Gruppen, die Pfeile wählen eine Zeile, **Enter** fügt ihren Namen in den Ausdruck am Cursor ein, **Esc** schließt.

Die Gruppen des Tastenfelds enthalten jede Funktion, jede Konstante und
jeden Befehl der Sprache: **trig**, **fn**, **num**, **0x** und
**var**. Die 0x-Gruppe enthält die exakten und Basis-Umwandlungen
(`frac`, `dec`, `big`, `bin`, `oct`, `hex`) und die Fakultät `!`. Die
Pfeiltasten bewegen die Markierung, **Enter** fügt das Token ein, und
**Tab** wechselt die Gruppen. Ein Operator am Anfang einer leeren Zeile (oder vom Tastenfeld eingefügt) setzt `ans` davor, die Zeile macht also mit dem letzten Ergebnis weiter.

Das Menü **Einstellungen** bietet dieselben Ergebnisdarstellungs-Optionen wie die Web-App (exakte Brüche, Notation, Tausendertrenner), neben den Zeilen für Design und Sprache.

### 5.3 Graphen zeichnen

Tippe `graph`, gefolgt von einem Ausdruck, und drücke **Enter**:

```epher
graph x ^ 2
```

epher tastet die Kurve von x = −10 bis x = 10 ab und zeichnet sie als
ASCII-Plot in das Panel Graph; die Legende über dem Plot benennt, was
gezeichnet wird.

`graph clear` leert den Plot, und das **Graph**-Menü macht dasselbe; das
**Help**-Menü öffnet dieses Handbuch in der TUI (Pfeiltasten scrollen,
**Esc** schließt). Das **Settings**-Menü kann die besonderen Punkte unter
dem Plot ausblenden.

Du kannst jeden Ausdruck zeichnen, auch deine eigenen Funktionen.
Definiere zuerst eine und zeichne sie dann:

```epher
def f(x) = x ^ 3
graph f(x)
```

Jede `graph`-Zeile fügt dem Plot eine Kurve hinzu, gezeichnet mit ihrem
eigenen Symbol (`o`, `x`, `+`, `*`); `graph clear` leert den Plot.
Dieselbe Grammatik wie in der Web-App gilt: ein Definitionsbereich
(`graph sin(x) from 0 to 2*pi`), parametrische Kurven
(`graph param 2*cos(t), 3*sin(t)`), Polarkurven
(`graph polar 1 + cos(theta)`) und Bereiche (`graph y < x ^ 2` schattiert
die Fläche unter der Kurve).

Punkte, an denen der Ausdruck keinen Wert hat (zum Beispiel die Division
durch null), werden einfach übersprungen und hinterlassen eine Lücke im
Plot. Nach jedem graph-Befehl listet die TUI die besonderen Punkte
(Nullstellen, Extrempunkte und Schnittpunkte) unter dem Plot auf. Der
Befehl `table` (Abschnitt 2.4.2) funktioniert auch hier.

`graph3d x ^ 2 - y ^ 2` zeichnet eine 3D-Fläche als ASCII-Drahtgitter.
Drehe sie mit den Pfeiltasten, solange die Eingabe leer ist, und drücke
die Leertaste, um die Konstante eines Schiebereglers zu animieren
(Abschnitt 2.4.5). Die Hinweiszeile unten zeigt die Pfeil- und
Leertasten-Hinweise nur, solange eine 3D-Fläche oder eine animierbare
Kurve angezeigt wird.

`graph save plot.svg` schreibt den aktuellen Plot als dasselbe SVG-Bild,
das die Schaltfläche **SVG kopieren** der Web-App liefert; `graph3d save
datei.svg` speichert das 3D-Gitter aus dem Blickwinkel, den du gerade
siehst.

### 5.4 Speichern und Persistenz

Die TUI teilt ihren Speicher mit der CLI: alles, was in der einen
gespeichert ist, ist in der anderen verfügbar. Funktionen, Skripte,
Verlauf und die Sprachpräferenz leben in `~/.epher` (Kapitel 4.6), und
dieselben Befehle `save`, `save script` und `language` funktionieren hier.

## 6. Deine Daten und Privatsphäre

- Das **installierte epher-Programm** (Desktop-App, CLI und TUI)
  speichert Funktionen, Skripte, Verlauf und die Sprachwahl lokal in
  `~/.epher` (oder `EPHER_STORE_DIR`). Nichts verlässt deinen Computer.
- Die **Web-App** behält nichts auf der Festplatte: Der Verlauf dauert
  nur, solange die Seite offen ist. Die Web-App kann offline arbeiten,
  weil die Seite selbst von deinem Browser gespeichert wird.

Alle fünf Versionen führen die Berechnung vollständig auf deinem Gerät
aus. Nichts wird irgendwohin gesendet.
