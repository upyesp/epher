# epher-Benutzerhandbuch

Willkommen! epher ist ein programmierbarer, skriptfähiger Taschenrechner. Du
kannst ihn für eine schnelle Berechnung nutzen oder eigene Funktionen und
kleine Programme aufbauen — und alles ist in acht Sprachen verfügbar.

Dieses Handbuch richtet sich an komplette Einsteiger. Es beginnt mit der
einfachsten möglichen Berechnung und steigert sich bis zur vollen Kraft der
Sprache. Jedes Beispiel zeigt, was du eintippst und was epher antwortet.

Es gibt fünf Arten, epher zu nutzen — wähle, was zu dir passt:

| Version | Was es ist | Am besten, wenn |
|---|---|---|
| **Befehlszeile** (CLI) | Textbefehle in einem Terminal | Du lebst im Terminal und magst Skripte |
| **REPL** | Eine interaktive `epher`-Sitzung am Prompt `epher>` | Du willst schnelles Hin und Her, ohne das Terminal zu verlassen |
| **Terminal-Oberfläche** (TUI) | Ein Vollbild-Programm im Terminal | Du willst eine Terminal-App mit Graphen und Verlauf auf dem Bildschirm |
| **Desktop-App** | Ein normales Desktop-Programm mit eigenem Fenster | Du willst eine normale Anwendung |
| **Web-App** (PWA) | Läuft in deinem Browser, installierbar, funktioniert offline | Du willst den schnellsten Start; keine Installation |

Die Desktop-App, die Befehlszeile, das REPL und die Terminal-Oberfläche
sind ein Programm: Ein einziger Download installiert den Befehl `epher`,
der alle vier kann. Die Web-App ist die Ausnahme — sie braucht überhaupt
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

1. `!` Fakultät
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

Potenzen können gebrochen sein — `2 ^ 0.5` ist die Quadratwurzel aus 2:

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

Du kannst eine Variable jederzeit ändern — sie behält ihren Wert, bis du
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
die `Ans`-Taste eines Taschenrechners — praktisch für Kettenrechnungen:

```epher
2 + 3
ans * 2
```

```text
5
10
```

### 1.6 Konstanten: Namen, die sich nie ändern

Eine *Konstante* ist ein Name für einen Wert, der sich nie ändert — wie
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

und dasselbe gilt, wenn du dieselbe Konstante zweimal definierst:

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

> epher hat keine Textwerte — beide Zweige eines `if` müssen Zahlen sein
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

Das berühmteste Beispiel — die Fibonacci-Zahlen:

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

> Der Körper einer Funktion ist ein einzelner Ausdruck — eine Zeile. Kombiniere
> stattdessen mehrere Berechnungen mit `;` in einem Skript (nächster Abschnitt).

### 1.11 Skripte: mehrere Anweisungen auf einmal

Ein *Skript* sind mehrere Anweisungen, verbunden mit `;` — oder mit
Zeilenumbrüchen, die genau dasselbe bedeuten — die nacheinander
ausgeführt werden:

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
ebenfalls überall — auch in der Einmal-Befehlszeile (Abschnitt 4.1).

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

Zurück geht es mit **bin(x)**, **oct(x)** und **hex(x)** — die
präfixbehaftete Schreibweise einer ganzen Zahl, direkt wieder einsetzbar:

```epher
hex(255)
bin(10)
```

```text
0xff
0b1010
```

### 1.13 Eingebaute Funktionen

epher hat die Funktionen eines wissenschaftlichen Taschenrechners, nach
Familien gruppiert.

Trigonometrie arbeitet in Bogenmaß (Radiant) — nutze `deg` und `rad` zum
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
| `bin(x)` / `oct(x)` / `hex(x)` | Schreibweise mit Präfix in Basis 2 / 8 / 16 | `hex(255)` | `0xff` |

Sie lassen sich wie alles andere kombinieren:

```epher
min(sqrt(16), 5)
```

```text
4
```

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
| Binär, oktal, hexadezimal | `0b…`, `0o…`, `0x…` | `0xFF + 0b1` |
| Basisschreibweise | `bin(x)`, `oct(x)`, `hex(x)` | `hex(255)` |

## 2. Die Web-App (PWA)

### 2.1 Sie öffnen

Die Web-App liegt unter:

```text
https://epher.org/pwa/
```

Keine Installation nötig — sie funktioniert in jedem modernen Browser auf
Computer, Telefon oder Tablet.

Dieses Handbuch ist auch in die App eingebaut: öffne **Help → User guide**
in der Menüleiste (tippe auf einem Telefon auf **☰**), um es in der App in
der aktuell eingestellten Sprache zu lesen. Tippe ein beliebiges Beispiel
in diesem Handbuch an, um es ins Eingabefeld zu laden.

### 2.2 Deine erste Berechnung

1. Klicke auf das Textfeld (es ist beim Laden der Seite bereits fokussiert).
2. Tippe einen Ausdruck, zum Beispiel `2 + 3 * 4`.
3. Drücke **Enter** oder klicke auf den Button **=**.

Das Ergebnis erscheint in großer Schrift unter dem Feld. Alles aus
Kapitel 1 funktioniert hier, einschließlich Variablen, Funktionen und
Skripten.

### 2.3 Verlauf

Jede Berechnung wird zur Verlaufsliste unter dem Ergebnis hinzugefügt,
damit du zurückscrollen und sehen kannst, was du gemacht hast. Die neuesten
Einträge erscheinen oben, und der Button **Clear history** über der Liste
leert sie. Der Verlauf bleibt erhalten, solange die Seite offen ist.

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
eigener Farbe — die Kurven sind alle durchgezogen, und die Legende
und die Beschriftungen unterscheiden sie ohne Farbe.
`graph clear` leert den Plot — und ein Button **Clear graph** oben im
Graph-Panel macht dasselbe für Kurven und 3D-Flächen zusammen. Die TUI
behält den Befehl in ihrem **Graph**-Menü.

Ganz oben im Graph-Bereich, neben **Clear graph** und **Copy SVG**,
blendest du in der Optionsleiste die Liste der besonderen Punkte aus, die
hervorgehobenen Punkte im Plot selbst — und stellst mit dem Regler
**Linienstärke** die Dicke der gezeichneten Linien ein.

```epher
graph x ^ 2
graph x ^ 3
```

Punkte, an denen der Ausdruck keinen Wert hat (zum Beispiel eine Division
durch null), werden übersprungen und hinterlassen eine Lücke in der
Kurve — und ein Sprung, der eigentlich eine senkrechte Asymptote ist,
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

**Verfolgen:** Bewege den Zeiger über den Plot — oder fokussiere ihn und
drücke die Pfeiltasten — und der nächstgelegene Punkt auf einer Kurve
wird markiert, seine Koordinaten werden unter dem Plot angezeigt.

**Besondere Punkte:** Nach jedem graph-Befehl findet epher die Nullstellen
und Extrempunkte jeder Kurve und die Schnittpunkte zwischen Kurven,
markiert sie im Plot und listet sie darunter auf:

```text
root (-1, 0)   minimum (0, 0)   root (1, 0)
```

**Tabellen:** Der Befehl `table` druckt eine Wertetabelle (Zeilen, an
denen der Ausdruck keinen Wert hat, bleiben leer):

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
Plot erscheint ein Schieberegler — ziehe ihn (oder bewege ihn mit den
Pfeiltasten) und jede Kurve wird neu gezeichnet:

```epher
const a = 1
graph a * x ^ 2
```

**SVG kopieren** kopiert den aktuellen Plot als eigenständiges SVG-Bild
zum Einfügen in Dokumente — die Farben sind eingebaut, es sieht überall
gleich aus. Der Regler **Linienstärke** ganz unten im Bereich stellt
ein, wie dick jede gezeichnete Linie erscheint.

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
vorn — so animieren Taschenrechner üblicherweise: Du animierst einen
Parameter, und alles, was ihn verwendet, bewegt sich mit. Drücke die Taste erneut, um zu
pausieren.

Eine "Zeit"-Variable ist nur eine Konstante, die du animierst:

```epher
const t = 0
graph sin(x - t)
```

Wenn du den Schieberegler von t abspielst, wandert die Welle.
3D-Flächen animieren sich genauso — definiere zuerst eine Konstante und
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
deine App-Liste — sie öffnet sich sofort, auch ohne Internetverbindung.

### 2.6 Was die Web-App nicht kann

Die Web-App hält deine Arbeit in der aktuellen Sitzung: Sie wertet
Ausdrücke aus, zeichnet ihre Graphen (Abschnitt 2.4) und führt einen
Verlauf. Die Befehle **save**, **save script** und **language**
funktionieren in der Desktop-, Befehlszeilen- und Terminal-Version
(Kapitel 3, 4 und 5) — in der Web-App antworten sie mit einem Hinweis,
dass Speichern dort funktioniert. Der Verlauf wird zwischen Besuchen
nicht gespeichert.

## 3. Die Desktop-App

Die Desktop-App ist ein normales Fenster um dieselbe Web-App herum. Alles
aus Kapitel 2 gilt; der Unterschied liegt nur darin, wie du sie
installierst und startest.

### 3.1 Installieren

Lade von der epher-Website einen Installer für dein System herunter:

- **Windows:** führe `epher-windows-x86_64.exe` aus. Der Installer legt
  `epher` in deinen PATH — öffne ein neues CMD- oder PowerShell-Fenster
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

- **Linux (jede Distribution, auch Arch):** das AppImage — mach es
  ausführbar und starte es:

```sh
chmod +x epher-linux-x86_64.AppImage
./epher-linux-x86_64.AppImage
```

Jeder Installer enthält *ganz* epher — die Desktop-App, die Befehlszeile
(Kapitel 4) und die Terminal-Oberfläche (Kapitel 5) — als den einzigen
Befehl `epher`. Unter Linux legt das Paket `epher` in `/usr/bin` ab.

### 3.2 Verwenden

Starte epher wie jede andere Anwendung. Du bekommst ein Fenster mit
derselben Oberfläche wie die Web-App: tippe einen Ausdruck, drücke
**Enter** oder klicke auf **=**, und lies das Ergebnis. Graphen zeichnen
funktioniert auch hier — `graph x ^ 2` zeichnet im Fenster (Kapitel 2.4).
Das Fenster lässt sich frei skalieren. Die Menüleiste enthält
**Help → User guide** — dasselbe Handbuch wie diese Seite, mit antippbaren
Beispielen.

Du kannst es auch aus einem Terminal öffnen: ein bloßes `epher` (oder
`epher gui`) startet die Desktop-App. Verwende unter macOS den Button
**Install the epher command** in der App, um `epher` in den PATH deines
Terminals zu legen.

### 3.3 Speicherung: ein Speicher, gemeinsam mit CLI und TUI

Die Desktop-App teilt ihren Speicher mit der Befehlszeilen- und der
Terminal-Version. Funktionen, Konstanten, Skripte, Verlauf und die
Sprachpräferenz leben an einem Ort — `~/.epher` auf deinem Computer (oder
`EPHER_STORE_DIR`, Kapitel 4.6) — und alles, was in einer Version
gespeichert wurde, ist in den anderen verfügbar:

```text
def area(w, h) = w * h
save area
```

Definiere `area` in der Desktop-App, speichere sie mit `save`, schließe
das Fenster — dann öffne die CLI und `area(3, 4)` funktioniert einfach.
Andersherum geht es auch: Funktionen und Skripte, die du in der CLI oder
TUI gespeichert hast, sind beim Öffnen des Desktop-Fensters schon da,
einschließlich Variablen, die gespeicherte Skripte gesetzt haben. Die
Befehle `save`, `save script` und `language` aus Kapitel 4 funktionieren
hier genau gleich.

> Die Web-App im Browser ist die eine Version, die diesen Speicher nicht
> nutzt — sie behält jede Sitzung für sich (Kapitel 2.6).

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
genauso. Alles aus Kapitel 1 ist verfügbar — Variablen, Funktionen,
Schleifen, alles — und die Zeilen teilen eine Sitzung, wie ein gepipetes
Skript (Abschnitt 4.2).

### 4.2 Gepipete Skripte

`epher -` liest Ausdrücke aus der Standardeingabe, Zeile für Zeile — so,
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
Anweisungen mit `;` verbinden — Zeilenumbrüche und `;` bedeuten überall
in epher dasselbe.

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

Konstanten speichern sich genauso — `save` auf den Namen der Konstante:

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
*tippst* — die Ausdruckssprache — ist immer dieselbe, egal welche Sprache
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
Terminal. Sie ist Teil desselben `epher`-Programms — starte sie mit:

```sh
epher tui
```

### 5.1 Der Bildschirm

Der Bildschirm ist in Panels unterteilt:

- **Ausdruck** — die Eingabezeile (oben).
- Das aktuelle **Ergebnis** direkt darunter.
- **Verlauf** — jede Zeile, die du eingegeben hast, mit ihrer Antwort.
- **Graph** — die Zeichnung aus dem Befehl `graph` (unten).
- Eine Hinweiszeile zeigt die Tastenkürzel.

### 5.2 Tasten

| Taste | Aktion |
|---|---|
| Tippen | zum Ausdruck hinzufügen |
| **Enter** | auswerten |
| **Esc** | die Eingabezeile leeren |
| **Ctrl+C** | beenden |
| **q** | beenden (wenn die Eingabe leer ist) |
| **Arrow keys** | die 3D-Ansicht drehen (wenn die Eingabe leer ist) |
| **Space** | die Animation starten/stoppen (wenn die Eingabe leer ist) |
| **F10** | die Menüs öffnen (Datei, Bearbeiten, Graph, Einstellungen, Hilfe) |
| **Tab** | das immer sichtbare Tastenfeld fokussieren (bzw. den Verlauf, vom Tastenfeld aus); Gruppen wechseln (**Esc** zurück zum Tippen) |
| **Ctrl+L** | den Verlauf leeren |

Die Gruppen des Tastenfelds enthalten jede Funktion, jede Konstante und
jeden Befehl der Sprache: **trig**, **fn**, **num**, **0x** und
**var** — die 0x-Gruppe enthält die exakten und Basis-Umwandlungen
(`frac`, `dec`, `big`, `bin`, `oct`, `hex`) und die Fakultät `!`. Die
Pfeiltasten bewegen die Markierung, **Enter** fügt das Token ein, und
**Tab** wechselt die Gruppen.

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

Du kannst jeden Ausdruck zeichnen, auch deine eigenen Funktionen —
definiere zuerst eine und zeichne sie dann:

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
Plot. Nach jedem graph-Befehl listet die TUI die besonderen Punkte —
Nullstellen, Extrempunkte und Schnittpunkte — unter dem Plot auf. Der
Befehl `table` (Abschnitt 2.4.2) funktioniert auch hier.

`graph3d x ^ 2 - y ^ 2` zeichnet eine 3D-Fläche als ASCII-Drahtgitter —
drehe sie mit den Pfeiltasten und drücke die Leertaste, um die Konstante
eines Schiebereglers zu animieren (Abschnitt 2.4.5).

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

- Das **installierte epher-Programm** — Desktop-App, CLI und TUI —
  speichert Funktionen, Skripte, Verlauf und die Sprachwahl lokal in
  `~/.epher` (oder `EPHER_STORE_DIR`). Nichts verlässt deinen Computer.
- Die **Web-App** behält nichts auf der Festplatte: Der Verlauf dauert
  nur, solange die Seite offen ist. Die Web-App kann offline arbeiten,
  weil die Seite selbst von deinem Browser gespeichert wird.

Alle fünf Versionen führen die Berechnung vollständig auf deinem Gerät
aus — nichts wird irgendwohin gesendet.
