# Guide de l'utilisateur de epher

Bienvenue ! epher est une calculatrice programmable et scriptable. Vous pouvez
l'utiliser pour un calcul rapide, ou construire vos propres fonctions et
petits programmes. Tout est disponible en huit langues.

Ce guide s'adresse aux débutants complets. Il commence par le calcul le plus
simple possible et monte jusqu'à toute la puissance du langage. Chaque
exemple montre ce que vous tapez et ce que epher répond.

Il y a cinq façons d'utiliser epher. Choisissez celle qui vous convient :

| Version | Ce que c'est | Quand la choisir |
|---|---|---|
| **Ligne de commande** (CLI) | Commandes texte dans un terminal | Vous vivez dans un terminal et aimez les scripts |
| **REPL** | Une session interactive `epher` à l'invite `epher>` | Pour des allers-retours rapides sans quitter le terminal |
| **Interface de terminal** (TUI) | Un programme plein écran dans le terminal | Pour une appli terminal avec graphiques et historique |
| **Application de bureau** | Un programme classique avec sa propre fenêtre | Pour une application classique |
| **Application web** (PWA) | Tourne dans votre navigateur, installable, fonctionne hors ligne | Pour démarrer au plus vite ; sans installation |

L'application de bureau, la ligne de commande, le REPL et l'interface de
terminal sont un seul programme : un unique téléchargement installe la
commande `epher`, qui fait les quatre. L'application web est l'exception :
aucun téléchargement n'est nécessaire.

Les cinq versions comprennent exactement le même langage. Apprenez-le une
fois, utilisez-le partout.

## 1. Le langage de epher

Ce chapitre enseigne le langage commun à toutes les versions de epher. Dans
l'application web ou de bureau, tapez une expression et appuyez sur
**Entrée** (ou cliquez sur le bouton **=**). Dans la CLI, lancez la session
avec `epher repl` et tapez après l'invite `epher>`. Dans la TUI
(`epher tui`), tapez et appuyez sur **Entrée**. Dans la CLI
vous pouvez aussi écrire `epher "expression"` pour évaluer directement une
expression.

### 1.1 Votre premier calcul

Tapez ceci :

```epher
2 + 3 * 4
```

epher répond :

```text
14
```

La multiplication se fait avant l'addition, exactement comme en
mathématiques. Cette règle s'appelle la *précédence des opérateurs*.

### 1.2 Ordre des opérations

L'ordre complet de précédence, du plus fort au plus faible :

1. `!` factorielle et `%` pourcentage (tous deux postfixes)
2. `^` puissance
3. `*` et `/` multiplication et division
4. `+` et `-` addition et soustraction

Utilisez des parenthèses pour changer l'ordre :

```epher
(2 + 3) * 4
```

```text
20
```

L'opérateur `^` calcule les puissances et fonctionne de droite à gauche :

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

(`2 ^ 3 ^ 2` signifie `2 ^ (3 ^ 2)`, c'est-à-dire `2 ^ 9` = 512.)

Les puissances peuvent être fractionnaires. `2 ^ 0.5` est la racine carrée
de 2 :

```epher
2 ^ 0.5
```

```text
1.41421356237
```

La soustraction et la division fonctionnent de gauche à droite :

```epher
10 - 3 - 2
```

```text
5
```

Le signe `%` est un opérateur postfixé qui signifie « divisé par 100 » : `5%` vaut 0.05. Il ne regarde jamais les opérateurs autour de lui, donc `200 + 10%` vaut 200.1. Pour augmenter 200 de 10%, écrivez la multiplication :

```epher
200 * (1 + 10%)
```

```text
220
```


### 1.3 Les nombres spéciaux pi, e, tau et phi

Les constantes célèbres sont intégrées :

```epher
pi
```

```text
3.14159265359
```

```epher
2 * pi
```

```text
6.28318530718
```

```epher
e
```

```text
2.71828182846
```

Deux autres : `tau` est un tour complet (2 pi) et `phi` est le nombre d'or :

```epher
tau
```

```text
6.28318530718
```

```epher
phi
```

```text
1.61803398875
```

### 1.4 Comparer et logique

Vous pouvez comparer des nombres. Le résultat est `true` (vrai) ou `false`
(faux) :

| Comparaison | Signification |
|---|---|
| `a > b` | a est plus grand que b |
| `a < b` | a est plus petit que b |
| `a >= b` | a est plus grand ou égal à b |
| `a <= b` | a est plus petit ou égal à b |
| `a == b` | a est égal à b (notez le double `=`) |
| `a != b` | a n'est pas égal à b |

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

Combinez les comparaisons avec `and`, `or` et `not` :

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

### 1.5 Variables

Donnez un nom à une valeur avec un seul `=` :

```epher
x = 5
```

```text
5
```

epher vous répète la valeur. Désormais, `x` peut être utilisé partout :

```epher
x ^ 2
```

```text
25
```

Vous pouvez changer une variable quand vous voulez. Elle garde sa valeur
jusqu'à ce que vous la changiez :

```epher
x = x + 1
```

```text
6
```

> Les noms peuvent contenir des lettres et des tirets bas, comme `radius` ou
> `my_total`. Ils ne peuvent pas contenir d'espaces ni commencer par un
> chiffre.

La variable spéciale `ans` contient toujours la réponse précédente,
comme la touche `Ans` d'une calculatrice de poche, pratique pour
enchaîner les calculs :

```epher
2 + 3
ans * 2
```

```text
5
10
```

### 1.6 Les constantes : des noms qui ne changent jamais

Une *constante* est un nom dont la valeur ne change jamais, comme le `pi`
intégré, mais choisi par vous. Définissez-en une avec `const` :

```epher
const tax = 0.2
```

```text
0.2
```

Utilisez-la partout où un nombre peut aller :

```epher
100 * (1 + tax)
```

```text
120
```

La valeur est figée : la changer avec `=` est une erreur,

```epher
tax = 0.25
```

```text
error: cannot assign to constant tax
```

et la redéfinir avec une valeur différente aussi :

```epher
const tax = 0.25
```

```text
error: constant already defined: tax
```

Les constantes diffèrent des variables sur un autre point : comme `pi`,
elles fonctionnent à l'intérieur de vos propres fonctions.

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
784.8
```

Enregistrez une constante pour les sessions futures avec `save tax`,
exactement comme une fonction (chapitre 4.4).

> Une variable et une constante ne peuvent pas porter le même nom : après
> `const tax = 0.2`, `tax = ...` est toujours une erreur. Choisissez un
> autre nom ou démarrez une nouvelle session.

### 1.7 Les décisions avec if

`if` choisit entre deux valeurs :

```epher
if 3 > 2 then 10 else 20
```

```text
10
```

La forme est toujours `if condition then valeur_si_vrai
else valeur_si_faux`. La partie `else` est obligatoire.

Un exemple plus utile avec une variable :

```epher
price = 100
if price > 50 then 2 else 1
```

```text
2
```

> epher n'a pas de valeurs texte : les deux branches d'un `if` doivent être
> des nombres (ou des résultats de comparaisons).

### 1.8 Les boucles avec while

`while` répète une instruction tant qu'une condition est vraie :

```epher
x = 0; while x < 5 do x = x + 1; x
```

```text
5
```

Lisez ce script ainsi : *commence x à 0 ; tant que x est inférieur à 5,
ajoute 1 à x ; puis affiche x.* Le résultat est 5 parce que la boucle s'est
exécutée cinq fois.

> **Filet de sécurité :** epher arrête toute boucle après 100 000 étapes et
> affiche `error: step limit exceeded`. Cela vous protège des boucles qui ne
> se termineraient jamais. Si vous le voyez, votre condition ne devenait
> probablement jamais fausse.

### 1.9 Vos propres fonctions avec def

Une fonction est un calcul avec un nom et des paramètres :

```epher
def f(x) = x ^ 2
```

Puis utilisez-la :

```epher
f(7)
```

```text
49
```

Les fonctions peuvent avoir plusieurs paramètres :

```epher
def area(w, h) = w * h
area(3, 4)
```

```text
12
```

Vous pouvez aussi définir une fonction sans paramètre :

```epher
def answer() = 42
answer()
```

```text
42
```

### 1.10 La récursivité : une fonction qui s'appelle elle-même

L'exemple le plus célèbre est les nombres de Fibonacci :

```epher
def fib(n) = if n <= 1 then n else fib(n - 1) + fib(n - 2)
```

```epher
fib(10)
```

```text
55
```

`fib(10)` est le dixième nombre de Fibonacci. La fonction s'appelle
elle-même avec des arguments plus petits jusqu'à atteindre `n <= 1`. Cela
fonctionne parce que la forme `if ... then ... else ...` ne calcule que la
branche dont elle a besoin.

> Le corps d'une fonction est une seule expression, une ligne. Combinez
> plutôt plusieurs calculs avec `;` dans un script (section suivante).

### 1.11 Les scripts : plusieurs instructions à la fois

Un *script* est plusieurs instructions reliées par `;` ou par des
retours à la ligne, qui signifient exactement la même chose, exécutées
l'une après l'autre :

```epher
x = 10; y = x + 5; x + y
```

```text
25
```

Les scripts sont la façon de construire de petits programmes : préparez des
variables, faites des boucles, et affichez un résultat final.

Les retours à la ligne et `;` sont le même séparateur, et vous pouvez les
mélanger librement. Le bouton **Copier** au-dessus d'un exemple de
plusieurs lignes copie tout le script, et vous pouvez le coller directement
dans epher : le champ de saisie de l'application web et de l'application de
bureau, l'interface de terminal et `epher repl` exécutent toutes les lignes
dans l'ordre, exactement comme si vous les aviez tapées une à une. Relier
plusieurs instructions avec `;` sur une seule ligne fonctionne aussi partout,
y compris sur la ligne de commande à usage unique (section 4.1).


Les scripts peuvent porter des **commentaires** - des notes pour vous qu'epher ignore, à la manière de PHP. `//` ou `#` commente jusqu'à la fin de la ligne ; `/* ... */` met un bloc en commentaire, sur plusieurs lignes ou entre deux jetons :

```epher
// a small script with notes
r = 3 # radius in metres
area = /* pi r squared */ pi * r ^ 2
area
```
### 1.12 Résultats exacts : frac, dec et big

Normalement epher calcule avec des nombres décimaux comme une
calculatrice de poche, et les résultats arrondissent à douze chiffres
significatifs comme le fait une calculatrice : `0.1 + 0.2` donne
`0.3`, jamais `0.30000000000000004`. Les fractions exactes sont
activées par défaut — un résultat ayant une bonne fraction à petit
dénominateur dont le développement décimal se répète s'affiche comme
tel. `1 / 3` s'affiche `1/3` sans le demander :

```epher
1 / 3
```

```text
1/3
```

Avec **fractions exactes désactivées** dans les paramètres des
résultats (chapitre 2.2), la même division affiche
`0.333333333333`. **frac(n, d)** crée une fraction exacte qui reste
exacte à travers les calculs :

```epher
frac(1, 3)
```

```text
1/3
```

Les fractions restent exactes à travers les calculs :

```epher
frac(1, 3) * 3
```

```text
1
```

**dec(x)** crée un nombre décimal exact. `0.1 + 0.2` affiche `0.3`
dans les deux cas — la différence est arithmétique :

```epher
0.1 * 3 - 0.3
dec(0.1) * 3 - dec(0.3)
```

```text
0.0000000000000000555111512313
0.0
```

Le résultat flottant porte la petite erreur d'arrondi que tout
ordinateur fait avec les nombres décimaux ; `dec()` garde le calcul
exact.

**big(x)** crée un nombre entier exact, pour les valeurs trop grandes pour
une calculatrice de poche :

```epher
big(10 ^ 20)
```

```text
100000000000000000000
```

**Les bases** écrivent les entiers comme le fait la communauté
mathématique : `0b` pour le binaire, `0o` pour l'octal, `0x` pour
l'hexadécimal (le préfixe change l'orthographe, jamais la valeur) :

```epher
0b1010 + 0xFF
```

```text
265
```

La conversion inverse se fait avec **bin(x)**, **oct(x)** et **hex(x)** :
il donne l'orthographe préfixée d'un nombre entier, prête à être réutilisée.

```epher
hex(255)
bin(10)
```

```text
0xff
```
0b1010
```

**exact(x)** reconstruit la fraction exacte derrière un résultat décimal : toute valeur ayant une bonne fraction à petit dénominateur s'affiche ainsi. C'est la même reconstruction que les applications utilisent par défaut, donc `1 / 3` s'affiche généralement directement `1/3` :

```epher
exact(0.3333333333333333)
exact(0.30000000000000004)
```

```text
1/3
3/10
```

Une valeur irrationnelle comme `pi` n'a pas de bonne fraction, `exact()` la laisse donc telle quelle.

Les verbes d'affichage écrivent un nombre dans une autre notation. **scientific(x)** utilise un chiffre avant l'exposant, **engineering(x)** des exposants par pas de trois (la mantisse reste entre 1 et 1000), et **grouped(x)** insère des espaces fines comme séparateurs de milliers :

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

L'application web et le TUI proposent aussi ces choix dans les paramètres (voir chapitres 2.2 et 5.2) : fractions exactes activées/désactivées, notation Auto/scientifique/ingénieur et séparateurs de milliers. Ces paramètres ne changent que l'affichage ; les valeurs restent des nombres décimaux ordinaires.

### ### 1.13 Fonctions intégrées

epher possède les fonctions d'une calculatrice scientifique, regroupées par
famille.

La trigonométrie travaille en radians. Utilisez `deg` et `rad` pour
convertir :

| Fonction | Signification | Exemple | Résultat |
|---|---|---|---|
| `sin(x)`, `cos(x)`, `tan(x)` | fonctions trigonométriques | `sin(pi / 2)` | `1` |
| `asin(x)`, `acos(x)`, `atan(x)` | trigonométrie inverse | `atan(1)` | `0.7853981633974483` |
| `atan2(y, x)` | angle du point (x, y) | `atan2(1, 1)` | `0.7853981633974483` |
| `deg(x)` | radians → degrés | `deg(pi)` | `180` |
| `rad(x)` | degrés → radians | `rad(180)` | `3.14159265359` |
| `sinh(x)`, `cosh(x)`, `tanh(x)` | fonctions hyperboliques | `sinh(1)` | `1.1752011936438014` |
| `asinh(x)`, `acosh(x)`, `atanh(x)` | hyperboliques inverses | `acosh(1)` | `0` |

Puissances, racines et logarithmes (sur une calculatrice `log` est en
base 10) :

| Fonction | Signification | Exemple | Résultat |
|---|---|---|---|
| `sqrt(x)` | racine carrée | `sqrt(16)` | `4` |
| `cbrt(x)` | racine cubique | `cbrt(-27)` | `-3` |
| `root(n, x)` | racine n-ième | `root(3, 8)` | `2` |
| `exp(x)` | e puissance x | `exp(1)` | `2.71828182846` |
| `ln(x)` | logarithme népérien | `ln(e)` | `1` |
| `log(x)` | logarithme base 10 | `log(100)` | `2` |
| `log2(x)` | logarithme base 2 | `log2(8)` | `3` |
| `logb(b, x)` | logarithme en base b | `logb(2, 8)` | `3` |
| `hypot(a, b)` | hypoténuse | `hypot(3, 4)` | `5` |
| `5!` (aussi `fact(n)`) | factorielle | `5!` | `120` |

Arrondis, signes et nombres entiers :

| Fonction | Signification | Exemple | Résultat |
|---|---|---|---|
| `abs(x)` | valeur absolue | `abs(-3)` | `3` |
| `floor(x)` / `ceil(x)` | arrondir en bas / en haut | `floor(2.7)` | `2` |
| `round(x)` | le plus proche, les demis s'éloignent de zéro | `round(2.5)` | `3` |
| `trunc(x)` | supprimer la partie décimale | `trunc(-2.9)` | `-2` |
| `sign(x)` | -1, 0 ou 1 | `sign(-5)` | `-1` |
| `ncr(n, r)` | combinaisons | `ncr(52, 5)` | `2598960` |
| `npr(n, r)` | permutations | `npr(5, 2)` | `20` |
| `gcd(a, b)` / `lcm(a, b)` | diviseurs et multiples communs | `gcd(12, 18)` | `6` |
| `mod(a, b)` | reste | `mod(7, 3)` | `1` |

Les nombres premiers et les diviseurs travaillent sur des entiers :

| Fonction | Signification | Exemple | Résultat |
|---|---|---|---|
| `isprime(n)` | vrai quand n est premier | `isprime(97)` | `true` |
| `nextprime(n)` / `prevprime(n)` | les premiers les plus proches | `nextprime(10)` | `11` |
| `factors(n)` | décomposition en facteurs premiers | `factors(360)` |
| Littéral de liste | `{…}` | `{1, 2, 3}` |
| Élément de liste | `list[i]` (à partir de 1) | `{5, 6}[2]` |
| Statistiques de liste | `mean(liste)`, `median(liste)`, … | `stdev(d)` |
| Forme de liste | `len(s)`, `sort(s)`, `mode(s)`, `range(s)`, `quartile(s, k)` | `quartile(d, 1)` |
| Régression linéaire | `linreg(xs, ys)` | `linreg(x, y)` |
| Famille normale | `normpdf` `normcdf` `invnorm` | `invnorm(0.975)` |
| Famille t | `tpdf` `tcdf` `invt` | `invt(0.975, 10)` |
| Famille khi-deux | `chi2pdf` `chi2cdf` `invchi2` | `chi2cdf(3.84, 1)` |
| Familles discrètes | `binompdf` `binomcdf` `poissonpdf` `poissoncdf` | `binomcdf(2, 10, 0.5)` |
| Tests et intervalles | `ztest` `ttest` `zinterval` `tinterval` `chisq_gof` | `tinterval(d, 0.95)` |
| Graphiques de données | `graph scatter(xs, ys)` `histogram(data)` `boxplot(data)` | `graph boxplot(d)` |
| Nombres aléatoires | `random()`, `random(a, b)`, `randint(a, b)`, `randseed(n)` | `randint(1, 6)` |
| Explorateur de constantes | Aide → Constantes : toutes les constantes, groupées | Aide → Constantes |
| Grandeur | `5 m`, `60 mile/hr`, `1 km` | `2 m^2` |
| Convertir | `expr in unité` ou `expr -> unité` | `72 km/hr in m/s` |
| Préfixes | `k M G T m µ n p` modifient toute unité | `5 km`, `3 MPa`, `1 GHz` |
| Et, ou binaires | `a & b`, `a \| b` | `0xFF & 0x0F` |
| Ou exclusif binaire | `a xor b` | `5 xor 3` |
| Non binaire | `~a` | `~0` |
| Décalages | `a << n`, `a >> n` | `1 << 8` |
| Taille de mot | `bits(n)` — 8, 16, 32, 64 | `bits(8)` |
| Relation implicite | `graph lhs == rhs` | `graph x^2 + y^2 == 1` |
| Littéral de matrice | `[[1, 2], [3, 4]]` | `[[1, 2], [3, 4]] * [[5, 6], [7, 8]]` |
| Fonctions matricielles | `det` `inv` `transpose` `trace` `dim` `ref` `rref` | `rref([[2, 1, 5], [1, -1, 1]])` |
| Solveur TVM | `tvm_n` `tvm_i` `tvm_pv` `tvm_pmt` `tvm_fv` | `tvm_pmt(360, 0.08/12, -100000, 0)` |
| VAN et TRI | `npv(rate, flows)` `irr(flows)` | `irr({-100, 60, 60})` |
| Amortissement | `amort(p, r, n, k)` | `amort(1000, 0.01, 12, 6)` |
| Intérêts | `simple_interest` `compound_interest` | `compound_interest(1000, 0.05, 2)` | `2^3 * 3^2 * 5` |
| `totient(n)` | indicatrice d'Euler | `totient(12)` | `4` |
| `ndivisors(n)` | nombre de diviseurs | `ndivisors(360)` | `24` |
| `modpow(b, e, m)` | b puissance e, modulo m, exact | `modpow(2, 10, 1000)` | `24` |


Les statistiques acceptent un nombre quelconque d'arguments :

| Fonction | Signification | Exemple | Résultat |
|---|---|---|---|
| `sum(...)` / `product(...)` | totaux | `sum(1, 2, 3)` | `6` |
| `mean(...)` | moyenne | `mean(1, 2, 3)` | `2` |
| `median(...)` | valeur centrale | `median(1, 2, 3, 4)` | `2.5` |
| `min(...)` / `max(...)` | le plus petit / le plus grand | `max(4, 1, 3)` | `4` |
| `variance(...)` / `stdev(...)` | dispersion des valeurs | `stdev(2, 4)` | `1` |

Les couches exactes de la section 1.12 restent :

| Fonction | Signification | Exemple | Résultat |
|---|---|---|---|
| `frac(n, d)` | fraction exacte | `frac(1, 3)` | `1/3` |
| `dec(x)` | décimal exact | `dec(0.1)` | `0.1` |
| `big(x)` | nombre entier exact | `big(10 ^ 20)` | `100000000000000000000` |
| Binaire, octal, hexa | `0b…`, `0o…`, `0x…` | `0xFF + 0b1` |
| Orthographe en base | `bin(x)`, `oct(x)`, `hex(x)` | `hex(255)` |
| Premiers | `isprime(n)`, `factors(n)`, … | `factors(360)` |
| `bin(x)` / `oct(x)` / `hex(x)` | orthographe préfixée en base 2 / 8 / 16 | `hex(255)` | `0xff` |

Elles se combinent comme tout le reste :

```epher
min(sqrt(16), 5)
```

```text
4
```

Les constantes physiques utilisent les unités SI, comme celles d'astronomie de la section 1.16 :

| Nom | Signification | Valeur |
|---|---|---|
| `G` | constante gravitationnelle de Newton | 6.6743e-11 |
| `gamma` | constante d'Euler-Mascheroni | 0.577215664902 |
| `q_e` | charge élémentaire | 1.602176634e-19 |
| `ev` | électronvolt, en joules | 1.602176634e-19 |
| `eps_0` | permitivité du vide | 8.8541878128e-12 |
| `mu_0` | perméabilité du vide | 1.25663706212e-6 |
| `z_0` | impédance du vide | 376.730313668 |
| `m_e` | masse de l'électron | 9.1093837139e-31 |
| `m_p` | masse du proton | 1.67262192595e-27 |
| `m_n` | masse du neutron | 1.67492750056e-27 |
| `m_u` | unité de masse atomique | 1.66053906892e-27 |
| `a_0` | rayon de Bohr | 5.29177210544e-11 |
| `alpha` | constante de structure fine | 0.0072973525643 |
| `r_inf` | constante de Rydberg | 10973731.568160 |
| `mu_b` | magnéton de Bohr | 9.2740100783e-24 |
| `n_a` | constante d'Avogadro | 6.02214076e23 |
| `faraday` | constante de Faraday, C/mol | 96485.33212 |
| `r_gas` | constante molaire des gaz | 8.31446261815 |
| `atm` | atmosphère standard, en pascals | 101325 |
| `wien` | constante de longueur d'onde de Wien | 0.002897771955 |
| `phi_0` | quantum de flux magnétique | 2.067833848e-15 |
| `m_P` | Planck-Masse | 2.176434e-8 |
| `l_P` | Planck-Länge | 1.616255e-35 |
| `t_P` | Planck-Zeit | 5.391247e-44 |
| `r_e` | klassischer Elektronenradius | 2.8179403205e-15 |
| `lambda_c` | Compton-Wellenlänge | 2.42631023867e-12 |
| `mu_n` | Kernmagneton | 5.050783699e-27 |


### 1.14 Lire les erreurs

Quand quelque chose ne va pas, epher vous le dit au lieu de deviner :

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

Le dernier exemple est important : epher vous dit exactement quel nom il ne
connaît pas, pour que vous puissiez corriger votre expression.

### 1.15 Référence rapide

| Quoi | Syntaxe | Exemple |
|---|---|---|
| Addition, soustraction, multiplication, division | `+ - * /` | `7 / 2` |
| Puissance | `^` (de droite à gauche) | `2 ^ 10` |
| Factorielle | `!` (postfixe) | `5!` |
| Pourcentage | `%` (postfixe) | `200 * (1 + 10%)` |
| Parenthèses | `( )` | `(2 + 3) * 4` |
| Constantes | `pi`, `e`, `tau`, `phi` | `2 * pi` |
| Notation scientifique | `2.5e-3` | `6.02e23` |
| Comparer | `> < >= <= == !=` | `3 >= 2` |
| Logique | `and or not` | `a > 1 and a < 10` |
| Variable | `name = value` | `x = 5` |
| Constante | `const name = value` | `const tax = 0.2` |
| Décision | `if c then a else b` | `if x > 0 then 1 else -1` |
| Boucle | `while c do statement` | `while x < 5 do x = x + 1` |
| Fonction | `def name(params) = expr` | `def f(x) = x ^ 2` |
| Script | instructions reliées par `;` ou des retours à la ligne | `x = 1; x + 1` |
| Fraction exacte | `frac(n, d)` | `frac(1, 3)` |
| Décimal exact | `dec(x)` | `dec(0.1) + dec(0.2)` |
| Nombre entier exact | `big(x)` | `big(10 ^ 20)` |
| Reconstruire une fraction | `exact(x)` | `exact(0.3333333333333333)` |
| Scientifique, ingénieur, groupé | `scientific(x)` `engineering(x)` `grouped(x)` | `engineering(12345)` |
| Unité imaginaire | `i`, ou un littéral `4i` | `sqrt(-1)` |
| Parties d'un complexe | `re(z)` `im(z)` `arg(z)` `conj(z)` `abs(z)` | `re(3 + 4i)` |
| Résoudre une équation | `solve lhs == rhs` | `solve x^2 == 9` |
| Dérivée numérique | `derivative(expr, x)` | `derivative(x^2, 3)` |
| Intégrale définie | `integral(expr, a, b)` | `integral(x^2, 0, 3)` |
| Binaire, octal, hexa | `0b…`, `0o…`, `0x…` | `0xFF + 0b1` |
| Orthographe en base | `bin(x)`, `oct(x)`, `hex(x)` | `hex(255)` |

### 1.16 Astronomie et système solaire

epher parle astronomie : suffixes d'unités, constantes physiques, fonctions
de calendrier et de temps, et une éphéméride en direct pour le Soleil, la
Lune, les planètes et Pluton. Tout fonctionne hors ligne.

**Des unités qui parlent astronomie.** Écrivez un nombre suivi d'un
suffixe d'unité et epher le convertit en unités SI sur-le-champ :

| Suffixe | Unité | Convertit en |
|---|---|---|
| `AU` ou `au` | unité astronomique | mètres |
| `pc` | parsec | mètres |
| `ly` | année-lumière | mètres |
| `deg` | degré | radians |
| `arcmin`, `arcsec` | minute et seconde d'arc | radians |
| `min`, `hr`, `d`, `yr` | minute, heure, jour, année julienne | secondes |
| `Jy` | jansky | W m-2 Hz-1 |

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

Les suffixes font partie de la grammaire : aucune constante utilisateur ne
peut changer ce que signifie `3.2 AU`, et `h` reste la constante de Planck ;
les heures s'écrivent `hr`. Les fonctions renvoient des comptages en
unités naturelles ; un suffixe convertit un comptage en SI, ainsi
`mag2jy(20)` est un comptage en janskys et `mag2jy(20) Jy` le même flux en
watts par mètre carré hertz.

**Constantes astronomiques.** `au`, `pc`, `ly`, `c`, `g`, `h`, `h_bar`,
`k_b`, `sigma_sb`, `m_sun`, `r_sun`, `l_sun`, `m_earth`, `r_earth`, `m_moon`, `r_moon`
fonctionnent comme `pi`, et vous pouvez les masquer par vos propres
constantes.

**Dates et temps.** `jd(y, m, d [, hr])` et `mjd(...)` convertissent une
date du calendrier en date julienne, `now()` lit l'instant présent :

```epher
jd(2000, 1, 1, 12)
```

```text
2451545
```

`delta_t(jd)` est la correction TT - UT1, et `lst(jd, lon)` est le temps
sidéral local en heures pour une longitude est en degrés.

**Heures, minutes et secondes.** `hms2deg(h, m, s)` convertit l'ascension
droite en degrés, `dms2deg(d, m, s)` un angle sexagésimal, et
`deg2hms(x)` / `deg2dms(x)` réécrivent un angle en texte :

```epher
deg2hms(90)
```

```text
6h 0m 0s
```

**Le ciel, quantifié.** Donnez à chaque fonction d'accès un numéro de
corps : Mercure 1, Vénus 2, Mars 4, Jupiter 5, Saturne 6, Uranus 7,
Neptune 8, Pluton 9, Soleil 10, Lune 11 (la Terre est 3, l'observatrice,
jamais une cible).

| Fonction | Signification |
|---|---|
| `ra(b, jd)`, `decl(b, jd)` | ascension droite et déclinaison géocentriques (degrés) |
| `dist(b, jd)` | distance en UA |
| `alt(b, jd, lat, lon)`, `az(b, jd, lat, lon)` | hauteur et azimut topocentriques (degrés, vrais) |
| `rise(b, jd, lat, lon)`, `set(...)`, `transit(...)` | événements du jour solaire local, en dates juliennes |
| `mag(b, jd)` | magnitude apparente |
| `phase(b, jd)`, `illum(b, jd)` | angle de phase (degrés) et fraction éclairée |
| `diam(b, jd)` | diamètre angulaire (degrés) |

```epher
decl(10, jd(2000, 6, 21, 1.8))
```

```text
23.437882351
```

Latitudes et longitudes sont en degrés, est positif. Les positions sont
géocentriques sauf observateur donné. Pluton suit une orbite approchée,
honnête à environ une minute d'arc, bien en dessous de la précision des
autres corps ; les éclipses et les recherches de conjonctions ne sont pas
incluses.

**Optique et lumière.** `kepler(M, e)` résout l'équation de Kepler,
`airmass(alt)` est la masse d'air sec(z), `dawes(d)` le pouvoir séparateur
d'une ouverture de d millimètres en secondes d'arc, et `dist_mod(mu)`
convertit un module de distance en parsecs.

**Saisons.** `march_equinox(year)`, `june_solstice(year)`,
`september_equinox(year)` et `december_solstice(year)` renvoient la date
julienne de chaque changement de saison :

```epher
march_equinox(2000)
```

```text
2451623.8159797275
```

**Le système solaire en 3D.** La commande `solar3d` dessine tout le
système : chaque orbite en courbe, chaque corps en point étiqueté, avec
une traînée montrant d'où il vient :

```epher
solar3d jd(2020, 7, 1)
```

Donnez le temps sous forme de constante et appuyez sur le bouton lecture
pour voir les planètes bouger : `const t = now(); solar3d t`. Glissez
ou utilisez les flèches pour pivoter, `clear` pour vider, et
`solar3d save file.svg` pour exporter.

L'éphéméride est calculée par le crate solar-ephemeris
(github.com/Protonmatter/sol), validé contre JPL Horizons ; merci à son
auteur. La précision est de l'ordre de la seconde d'arc pour le Soleil, la
Lune et les planètes sur environ 5000 ans autour du présent.

### 1.17 Nombres complexes

epher calcule automatiquement avec les nombres complexes. L'unité imaginaire est **i**, exactement comme `pi` :

```epher
i ^ 2
sqrt(-1)
```

```text
-1
i
```

Écrivez un nombre complexe avec le suffixe `i`, sans signe de multiplication : `3 + 4i` est un littéral, `2.5i` fonctionne, ainsi que les littéraux à base (`0xFFi`). L'arithmétique habituelle s'étend : addition, soustraction, multiplication, division et puissances fonctionnent, et `i` suit la précédence normale (`i ^ 2` se lie comme toute puissance).

Les fonctions réelles s'étendent aussi. Avec un argument complexe elles calculent dans le plan complexe ; avec un argument réel hors de leur domaine réel elles renvoient le résultat complexe principal au lieu d'une erreur :

```epher
ln(-1)
asin(2)
exp(i * pi)
```

```text
3.14159265359i
1.57079632679-1.31695789692i
-1+0.000000000000000122464679915i
```

(`exp(i * pi)` vaut exactement `-1` ; les derniers chiffres sont le bruit de `sin(pi)` dans l'arithmétique de la machine.)

Quatre fonctions lisent les parties d'un nombre complexe, et `abs()` en est le module :

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
3.14159265359
3+4i
5
```

Les fonctions à entiers (`fact`, `gcd`, `floor`, `isprime`, ...) rejettent les arguments complexes avec une erreur de type.

### 1.18 Résoudre des équations

**solve** trouve les racines d'une équation à une variable. L'équation utilise `==` :

```epher
solve x^2 == 5*x + 6
```

```text
x = -1, x = 6
```

Les équations polynomiales (construites avec `+ - * ^` et des constantes) donnent toutes les racines, réelles et complexes :

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

La variable recherchée est `x` lorsqu'elle apparaît, sinon l'unique autre variable. Les constantes et variables liées agissent comme des paramètres :

```epher
const k = 3
solve k*x == 12
```

```text
x = 4
```

Toute autre équation est balayée numériquement sur -100..100 : les racines sont encadrées par changements de signe, donc `solve sin(x) == 0.5` liste chaque racine de l'intervalle. Deux limites honnêtes : une racine où la fonction ne fait que toucher zéro (comme `x^2 == 0` par le chemin numérique) peut être manquée, et une équation à plusieurs variables non liées est une erreur.

### 1.19 Analyse : dérivée et intégrale

**derivative(expr, p)** est la dérivée numérique de `expr` en `p`. Le premier argument reste une expression, et sa variable libre est celle que l'on dérive :

```epher
derivative(x^2, 3)
derivative(sin(t), 0)
```

```text
6
1
```

Comme l'argument reste une expression, la dérivée se trace : `graph derivative(x^3 - x, x)` dessine la courbe des pentes.

**integral(expr, a, b)** est l'intégrale définie de `a` à `b`, calculée par quadrature de Simpson adaptative :

```epher
integral(x^2, 0, 3)
integral(sin(x), 0, pi)
```

```text
9
2
```

`integral(x^2, 3, 0)` vaut `-9` (l'intégrale signée), et une borne supérieure traçable fonctionne : `graph integral(x^2, 0, x)`.

Les deux sont numériques ; les expressions doivent être à valeurs réelles sur l'intervalle, et une expression à plusieurs variables est une erreur.

### 1.20 Données : listes, statistiques et régression

Une liste est une colonne de nombres entre accolades : `{1, 2, 3}`.
Les éléments sont des expressions, la liste vide `{}` est admise, et
une liste se lie à un nom comme n'importe quelle valeur :

```epher
d = {12, 15, 14, 16, 13, 15, 14, 17}
d[2]
len(d)
```

`list[i]` est le i-ème élément, indexé à partir de 1 comme sur une
calculatrice ; un index hors de la liste est une erreur. Le crochet
lie plus fort que `^`, donc `d[2]^2` vaut `(d[2])^2`.

L'arithmétique sur une liste est élément par élément, un simple
nombre s'appliquant à chaque élément :

```epher
{1, 2, 3} * 2
{1, 2, 3} + 10
```

Deux listes doivent avoir la même longueur pour `+ - * / ^`. `==` et
`!=` comparent des listes entières ; les comparaisons d'ordre les
refusent.

Les fonctions statistiques prennent une liste pour seul argument (la
forme à plusieurs arguments reste — `mean(1, 2, 3)` fonctionne
toujours) : `sum product mean median mode variance stdev min max
range`. Les nouvelles fonctions de forme sont `len(liste)`, `sort(liste)`
(copie croissante), `mode(liste)` (valeur la plus fréquente, la plus
petite en cas d'égalité), `range(liste)` (plus grande moins plus petite
valeur) et `quartile(liste, k)` pour k de 1 à 3 (quartiles à la TI,
médiane des moitiés) :

```epher
mean(d)
median(d)
quartile(d, 1)
```

**linreg(xs, ys)** ajuste la droite des moindres carrés à deux listes
de même longueur et la rapporte avec le coefficient de corrélation r :

```epher
linreg({1, 2, 3, 4}, {2.1, 4.2, 5.8, 8.1})
```

La droite ajustée est un affichage, comme les racines de solve ; le
dessin de l'ajustement vit sur le nuage de points (section 1.22).

### 1.21 Distributions et tests d'hypothèse

Les fonctions de probabilité couvrent la normale centrée réduite, la
loi de Student, le khi-deux, la loi binomiale et la loi de Poisson.
La famille normale prend un ou trois arguments — un seul argument est
la normale centrée réduite :

```epher
normcdf(1.96)
invnorm(0.975)
normcdf(12, 10, 2)
```

`normpdf(x[, mu, sigma])`, `normcdf(x[, mu, sigma])`, `invnorm(p[,
mu, sigma])` ; `tpdf(x, df)`, `tcdf(x, df)`, `invt(p, df)` ;
`chi2pdf(x, df)`, `chi2cdf(x, df)`, `invchi2(p, df)` ;
`binompdf(k, n, p)`, `binomcdf(k, n, p)` ; `poissonpdf(k, lambda)`,
`poissoncdf(k, lambda)`. Les fonctions `inv*` répondent à la question
inverse : `invt(0.975, 10)` est la valeur de t sous laquelle se trouve
97.5 % de la masse.

Les tests prennent une liste de données et rapportent la statistique
et la p-value bilatérale sous forme de texte ; les intervalles
rapportent `(bas, haut)` au niveau que vous nommez :

```epher
d = {12, 15, 14, 16, 13, 15, 14, 17}
ttest(d, 14)
tinterval(d, 0.95)
ztest(d, 14, 1.5)
chisq_gof({20, 30, 25, 25}, {25, 25, 25, 25})
```

`ttest(données, mu0)` et `tinterval(données, niveau)` utilisent
l'écart type d'échantillon (n−1) ; `ztest(données, mu0, sigma)` et
`zinterval(données, sigma, niveau)` ont besoin du sigma connu.
`chisq_gof(observés, attendus)` est le test d'adéquation à k−1 degrés
de liberté. Les résultats sont des textes d'affichage : lisibles et
copiables, mais l'arithmétique ne peut pas les toucher.

### 1.22 Graphiques de données

La famille graphique accepte aussi des listes : un nuage de points,
un histogramme et une boîte à moustaches. Un graphique de données
occupe le panneau seul, comme un système solaire — la dernière
commande gagne, et `graph clear` le vide.

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

**scatter(xs, ys)** trace les points et, à partir de deux points, la
droite des moindres carrés, légendée `y = a*x + b (r = …)`.
**histogram(données[, classes])** trace un histogramme de fréquences ;
le nombre de classes est facultatif (règle de Sturges par défaut) et
doit être un entier entre 1 et 50. **boxplot(données)** trace la boîte
à moustaches : minimum, Q1, médiane, Q3, maximum, moustaches jusqu'aux
extrêmes. La fenêtre s'ajuste toujours aux données — les mots-clés
`from a to b` ne s'appliquent pas — et l'image s'exporte et se
sauvegarde comme n'importe quel graphique.

### 1.23 Nombres aléatoires

`random()` tire un nombre uniforme dans `[0, 1)`, `random(a, b)` un dans
`[a, b)`, et `randint(a, b)` un entier de l'intervalle fermé `[a, b]` —
un lancer de dé :

```epher
randseed(7)
randint(1, 6)
```

```text
7
3
```

La séquence est reproductible : `randseed(n)` réinitialise le générateur
avec `n` et l'affiche, de sorte que la même graine rejoue les mêmes tirages
dans chaque session et chaque interface.

### 1.24 Unités et conversion

Un nombre suivi d'une unité devient une *grandeur* : la valeur en
unités SI plus ses dimensions. Le tableau des unités couvre les unités
de base et dérivées du SI (`m`, `s`, `kg`, `A`, `K`, `mol`, `cd`,
`Hz`, `N`, `Pa`, `J`, `W`, `C`, `V`, `F`, `ohm`, `S`, `Wb`, `T`, `H`,
`lm`, `lx`, `Bq`, `Gy`, `Sv`), les unités courantes (`min`, `hr`, `d`,
`yr`, `L`, `t`, `bar`, `atm`, `torr`, `psi`, `eV`, `mile`, `yd`, `ft`,
`inch`, `nmi`, `lb`, `oz`, `gal`, `qt`, `pt`, `mph`, `knot`) et les
suffixes d'astronomie de la section 1.16. Les unités composées
s'enchaînent : `60 mile/hr` et `5 m/s^2` sont des unités simples.

```epher
60 mile/hr
```

```text
60 mile/hr
```

Les préfixes SI les modifient toutes : `k M G T m µ n p` sont kilo,
méga, giga, téra, milli, micro, nano, pico — `5 km`, `3 MPa`, `1 GHz`
fonctionnent, et `2 kg` est le kilogramme lui-même.

Les dimensions sont vérifiées : additionner ou comparer des grandeurs
d'unités différentes donne une erreur au lieu de mélanger mètres et
secondes :

```epher
5 m + 3 s
```

```text
error: dimension error: cannot add 5 m and 3 s
```

L'arithmétique compose les dimensions : `5 m * 3 m` vaut `15 m^2`,
`(3 m)^2` vaut `9 m^2`, `sqrt(4 m^2)` vaut `2 m`, et une expression
entière dont les dimensions s'annulent redevient un nombre ordinaire
(`5 m / 5 m` vaut `1`). Les résultats préfèrent le nom dérivé exact
quand les dimensions correspondent — `5 kg * 3 m / 1 s^2` répond
`15 N`.

**Conversion.** `expr in unité` (ou `expr -> unité`) affiche une
grandeur dans l'unité nommée ; les dimensions doivent correspondre.
`in` a la liaison la plus faible des opérateurs, donc `5 m + 3 m in km`
convertit toute la somme :

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

Les échelles de température (Celsius, Fahrenheit) ne sont pas des
unités ici — les kelvins le sont, et `K` fonctionne comme n'importe
quelle autre.


### 1.25 Opérations binaires

Les littéraux de base de la section 1.13 sont faits pour ça : `0b101`,
`0o17`, `0xFF`. Les opérateurs binaires travaillent sur des entiers et
répondent avec des entiers exacts :

```epher
0xFF & 0x0F
```

```text
15
```

| Opérateur | Signification |
|---|---|
| `a & b` | et binaire |
| `a \| b` | ou binaire |
| `a xor b` | ou exclusif binaire |
| `~a` | non binaire (complément à deux) |
| `a << n` | décalage à gauche (multiplier par 2^n) |
| `a >> n` | décalage à droite, arithmétique (diviser par 2^n, arrondi vers le bas) |

Les résultats sont des entiers `big` exacts, donc `1 << 60` garde
chaque chiffre. La taille de mot est de 64 bits par défaut : les
résultats se lisent en complément à deux signé, donc `~0` vaut -1 et
`1 << 100` enveloppe à 0. `bits(n)` change la taille de mot à 8, 16,
32 ou 64, et `bits()` la rapporte :

```epher
bits(8)
~0
```

```text
8
-1
```

Un décalage négatif inverse la direction (`8 << -1` vaut `4`). Le
`and` et le `or` booléens gardent leurs sens ; `&` et `|` sont les
graphies binaires.


### 1.26 Relations implicites

Une équation à deux inconnues se trace comme une courbe : la famille graphique échantillonne la relation par marching squares et dessine son contour zéro. Le cercle, la parabole et la droite verticale, chacun une seule commande :

```epher
graph x^2 + y^2 == 1
```

```epher
graph y == x^2
```

```epher
graph x == 2
```

La relation est échantillonnée sur le carré de `from a to b` (ou la fenêtre par défaut), donc `graph x^2 + y^2 == 1 from -2 to 2` cadre la fenêtre du cercle. Tout ce qu'une courbe peut faire s'applique : la légende affiche l'équation, les curseurs animent ses constantes, et l'image se zoome, se déplace et s'exporte comme n'importe quelle autre. Les remplissages d'inégalité (`y < …`, `y > …`) restent des courbes ombrées ; une relation n'a pas de points d'intérêt.


### 1.27 Matrices

Une matrice est une grille de nombres, écrite comme des rangées de
listes : `[[1, 2], [3, 4]]` est la matrice 2×2. `+` et `-` sont terme à
terme (formes égales), `*` est le produit matriciel, un nombre met à
l'échelle terme à terme, et `^` est la puissance matricielle entière
(`A ^ 0` est l'identité, donc les puissances demandent des matrices
carrées). `M[2][1]` est l'élément de la ligne 2, colonne 1 — les
lignes s'indexent comme des listes, à partir de 1.

```epher
[[1, 2], [3, 4]] * [[5, 6], [7, 8]]
```

```text
[[19, 22], [43, 50]]
```

Les fonctions matricielles couvrent le minimum de la classe : `det(M)`
(carrées seulement), `inv(M)` (les singulières sont une erreur),
`transpose(M)`, `trace(M)` (carrées), `dim(M)` (la liste `{lignes,
colonnes}`), et `ref(M)` avec `rref(M)` pour la réduction. Les
systèmes linéaires se résolvent par rref sur la matrice augmentée :

```epher
rref([[2, 1, 5], [1, -1, 1]])
```

```text
[[1, 0, 2], [0, 1, 1]]
```

Les lignes se lisent `x = 2`, `y = 1` — la dernière colonne de la
matrice augmentée réduite. Les fractions exactes s'affichent dans les
matrices comme dans les listes, donc `inv([[1, 2], [3, 4]])` montre
`[[-2, 1], [3/2, -1/2]]`.


### 1.28 Finance

Le solveur de valeur temps de l'argent (convention de signes TI :
l'argent sortant est négatif, l'argent entrant positif) résout l'un
des cinq champs quand les quatre autres sont donnés. `i` est le taux
par période en fraction — 0,01 vaut 1 % — et le dernier argument
facultatif est le moment du paiement : 0 pour la fin de période (par
défaut), 1 pour le début (annuité due).

```epher
tvm_pmt(360, 0.08/12, -100000, 0)
```

```text
327259/446
```

Le prêt hypothécaire classique à 8 % : 360 mensualités de 733,76 pour
un prêt de 100 000 — `tvm_pmt` est la mensualité, `tvm_pv` le prêt,
`tvm_fv` le solde, `tvm_n` la durée et `tvm_i` le taux :

```epher
tvm_i(360, -100000, 733.76, 0)
```

```text
0.00666661199068
```

Le taux est ici un peu sous 8 %/12 parce que 733,76 est arrondi.
`npv(r, flows)` actualise une liste de flux et `irr(flows)` trouve le
taux où la valeur actuelle nette est nulle :

```epher
npv(0.1, {-100, 60, 60})
```

```text
500/121
```

`amort(p, r, n, k)` est le solde restant après k paiements d'un prêt à
n périodes, `simple_interest(p, r, t)` vaut `p*r*t`, et
`compound_interest(p, r, n)` vaut `p*(1+r)^n - p`.


## 2. L'application web (PWA)

### 2.1 L'ouvrir

L'application web se trouve à l'adresse :

```text
https://epher.org/pwa/
```

Aucune installation n'est nécessaire. Elle fonctionne dans tout navigateur
moderne, sur ordinateur, téléphone ou tablette.

Ce guide est également intégré à l'application : ouvrez **Help → User guide**
dans la barre de menus (touchez **☰** sur un téléphone) pour le lire dans
l'application, dans la langue actuelle de l'application. Touchez n'importe
quel exemple de ce guide pour le charger dans le champ de saisie. **Aide → Constantes** ouvre l'explorateur de constantes : toutes les constantes groupées (Mathématiques, Astronomie, Physique, Chimie), chacune avec sa valeur et une brève description ; touchez-en une pour insérer son nom dans le champ de saisie, et la zone de recherche filtre la liste.

### 2.2 Votre premier calcul

1. Cliquez sur le champ de texte (il est déjà focalisé au chargement).
2. Tapez une expression, par exemple `2 + 3 * 4`.
3. Appuyez sur **Entrée** ou cliquez sur le bouton **=**.

Le résultat apparaît en grand sous le champ. Tout le chapitre 1 fonctionne
ici, y compris variables, fonctions et scripts.

Pendant que vous tapez un nom, une liste de suggestions apparaît sous le champ : les flèches déplacent la sélection, **Entrée** ou **Tab** accepte, **Échap** referme, et un clic accepte sans quitter le clavier. Chaque suggestion porte une courte description de la fonction ou de la constante. **F1** affiche la même description pour le mot sous le curseur dans la barre d'aide au-dessus du clavier. Si le premier caractère saisi dans un champ vide est un opérateur (`+ - * / ^ % !`), epher insère `ans` pour vous : la ligne continue depuis le résultat précédent.

Le menu **Paramètres** (l'icône d'engrenage, ou **☰ → Paramètres** sur un téléphone) contient trois groupes. **Thème** et **Langue** font ce que leurs noms indiquent. **Résultats** façonne l'affichage des réponses : fractions exactes (activées par défaut, donc `1 / 3` s'affiche `1/3`), notation (Auto, scientifique ou ingénieur) et séparateurs de milliers. Ce ne sont que des réglages d'affichage ; les valeurs restent des nombres ordinaires.

### 2.3 Historique

Chaque calcul est ajouté à la liste d'historique sous le résultat, pour que
vous puissiez remonter et voir ce que vous avez fait. Les entrées les plus
récentes apparaissent en haut, et l'icône de corbeille à côté du
titre **Historique** la vide (dans le terminal, Ctrl+L ou un clic sur la
même icône). L'historique est conservé tant que la page est ouverte.

Chaque entrée est délimitée par de fines règles : une expression sur une ligne occupe une ligne, et un script multi-lignes est une entrée qui affiche toutes ses lignes. Cliquez sur une entrée pour la recharger dans le champ de saisie et la réexécuter.

### 2.4 Les graphiques

Tapez `graph` suivi d'une expression et appuyez sur **Entrée** :

```epher
graph x ^ 2
```

epher échantillonne la courbe y = f(x) de x = −10 à x = 10 et la dessine
sous le champ de saisie, sur une grille aux axes étiquetés. Vous pouvez
tracer n'importe quelle expression, y compris vos propres fonctions :

```epher
def f(x) = x ^ 3
graph f(x)
```

Chaque ligne `graph` ajoute une autre courbe au même tracé, chacune avec
sa propre couleur. Les courbes sont toutes pleines, et ce sont la
légende et les étiquettes qui les distinguent sans couleur. `graph clear` vide le tracé, et un bouton **Clear graph** en haut
du panneau graphique fait la même chose pour les courbes et les surfaces 3D
à la fois. La TUI conserve la commande dans son menu **Graph**.

En haut du panneau de graphique, à côté de **Clear graph** et
**Copy SVG**, la barre d'outils permet de masquer la liste des points
d'intérêt et les points mis en évidence sur le tracé lui-même. Le curseur
Juste au-dessus de chaque tracé se trouve une bande de curseurs
nommés par une icône, les mots étant dans leur info-bulle :
l'épaisseur du trait (0 à 4 par pas de 0.1 pour les courbes 2D, 0 à
0.2 par pas de 0.01 pour les surfaces 3D - seul le curseur du type
affiché est visible, et chaque type mémorise sa propre valeur), et sur
les vues 3D et solaires, la vitesse de rotation horizontale et verticale
ainsi que la vitesse de zoom. Chaque entrée de légende a une case à cocher, cochée par défaut :
la décocher masque la courbe du tracé, de ses points d'intérêt et de
l'export SVG.

```epher
graph x ^ 2
graph x ^ 3
```

Les points où l'expression n'a pas de valeur (une division par zéro, par
exemple) sont ignorés, laissant un vide dans la courbe. Un saut qui
est en réalité une asymptote verticale n'est jamais dessiné comme une
ligne de liaison.

#### 2.4.1 Ce que vous pouvez tracer

Un domaine de votre choix :

```epher
graph sin(x) from 0 to 2*pi
```

Courbes paramétriques (t va de 0 à 2π) :

```epher
graph param 2*cos(t), 3*sin(t)
```

Courbes polaires :

```epher
graph polar 1 + cos(theta)
```

Régions : `y <` ombrage la zone sous la courbe, `y >` ombrage celle du dessus :

```epher
graph y < x ^ 2
```
#### 2.4.2 Lire le tracé

**Suivi :** déplacez le pointeur sur le tracé, ou focalisez-le et
appuyez sur les touches fléchées. Le point le plus proche d'une
courbe est marqué, avec ses coordonnées affichées sous le tracé.

**Points d'intérêt :** après chaque commande graph, epher trouve les
racines et les extremums de chaque courbe et les intersections entre
courbes, les marque sur le tracé et les liste en dessous :

```text
root (-1, 0)   minimum (0, 0)   root (1, 0)
```

**Tableaux :** la commande `table` affiche un tableau de valeurs (les
lignes où l'expression n'a pas de valeur restent vides) :

Une clause facultative `derivative <expression>` ajoute une
troisième colonne, la dérivée numérique de cette expression en chaque
x :

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

Les cellules du tableau suivent les réglages des résultats :
avec les fractions exactes activées (par défaut), une valeur qui est
une fraction simple s'affiche comme telle — `table x / 3 from 0 to 1
points 4` liste `1/3` au lieu de `0.333`.
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

#### 2.4.3 Curseurs et exportation

Définissez une constante, utilisez-la dans un tracé, et un curseur
apparaît sous le tracé. Faites-le glisser (ou déplacez-le avec les
touches fléchées) et chaque courbe se redessine :

```epher
const a = 1
graph a * x ^ 2
```

**Copier le SVG** copie le tracé actuel comme une image SVG autonome à
coller dans des documents. Les couleurs sont intégrées, le rendu est
identique partout. **Enregistrer le PNG** enregistre la même image en bitmap au double de sa taille, pour des courbes bien nettes ; l'application de bureau demande où la placer, l'application web l'enregistre dans vos téléchargements (ou le demande, quand le navigateur le permet). Les rangées de curseurs et les constantes animées se
trouvent directement sous le tracé, au-dessus de la liste des points
d'intérêt.

#### 2.4.4 Surfaces 3D

`graph3d` trace une surface z = f(x, y) sur un domaine carré (de −5 à 5,
ou votre `from a to b`) :

```epher
graph3d x ^ 2 - y ^ 2
```

Les lignes de maillage les plus proches de vous sont dessinées plus
marquées, si bien que la forme se lit en profondeur. Plusieurs lignes
`graph3d` se superposent, comme les courbes, et `graph3d clear` vide le
tracé. Faites pivoter la vue en faisant glisser, ou focalisez le tracé et
utilisez les touches fléchées. L'interface de terminal dessine la même
surface sous forme de filaire ASCII, que les touches fléchées font
pivoter.

#### 2.4.5 Animation

Chaque curseur a un bouton de lecture. Il fait avancer sa constante sur
toute la plage du curseur, puis revient au début. C'est la façon standard dont
les calculatrices animent : vous animez un paramètre, et tout ce qui
l'utilise bouge.
Appuyez de nouveau sur le bouton pour mettre en pause.

Une variable "temps" n'est qu'une constante que vous animez :

```epher
const t = 0
graph sin(x - t)
```

Lancer la lecture du curseur de t fait voyager l'onde. Les surfaces 3D
s'animent de la même façon. Définissez d'abord une constante, puis lancez
la lecture de son curseur :

```epher
const a = 1
graph3d sin(a * (x ^ 2 + y ^ 2)) from -3 to 3
```

Dans l'interface de terminal, la barre
d'espace démarre et arrête l'animation.

### 2.5 L'installer et l'utiliser hors ligne

L'application web est une *progressive web app* : après une visite elle
fonctionne entièrement hors ligne, et vous pouvez l'installer comme une
application normale.

- **Chrome, Edge ou Android :** cliquez sur l'icône d'installation dans la
  barre d'adresse (ou *Installer l'application* dans le menu du navigateur),
  puis confirmez.
- **iPhone / iPad (Safari) :** touchez **Partager** → **Ajouter à l'écran
  d'accueil**.
- **Autres navigateurs :** cherchez *Installer* ou *Ajouter à l'écran
  d'accueil* dans le menu.

Une fois installée, lancez-la depuis votre écran d'accueil ou votre liste
d'applications. Elle s'ouvre instantanément, même sans connexion internet.

### 2.6 Ce que l'application web ne fait pas

L'application web conserve votre travail dans la session en cours : elle
évalue des expressions, les trace (section 2.4) et garde un historique.
Les commandes **save**, **save script** et **language** fonctionnent dans
les versions bureau, ligne de commande et terminal (chapitres 3, 4 et 5)
. Dans l'application web, elles répondent par une note indiquant que
l'enregistrement y est possible. L'historique n'est pas conservé entre
les visites.

## 3. L'application de bureau

L'application de bureau est une fenêtre normale autour de la même
application web. Tout le chapitre 2 s'applique ; seule l'installation et le
lancement diffèrent.

### 3.1 Installation

Téléchargez un installateur pour votre système depuis le site web de epher :

- **Windows :** lancez `epher-windows-x86_64.exe`. L'installateur met `epher`
  dans votre PATH. Ouvrez une nouvelle fenêtre CMD ou PowerShell et
  `epher "2 + 2"` fonctionne. Comme la compilation n'est pas signée,
  choisissez *Plus d'informations* → *Exécuter quand même* au premier
  lancement.
- **macOS :** ouvrez `epher-macos-aarch64.dmg` et glissez epher dans
  Applications. Comme la compilation n'est pas signée, le premier lancement
  nécessite un clic droit → **Ouvrir**.
- **Linux (Debian/Ubuntu) :** le paquet `.deb`

```sh
sudo apt install ./epher-linux-x86_64.deb
```

- **Linux (Fedora/RHEL) :** le paquet `.rpm`

```sh
sudo dnf install ./epher-linux-x86_64.rpm
```

- **Linux (toute distribution, Arch compris) :** l'AppImage. Rendez-la
  exécutable et lancez-la :

```sh
chmod +x epher-linux-x86_64.AppImage
./epher-linux-x86_64.AppImage
```

Chaque installateur contient *tout* epher : l'application de bureau, la ligne
de commande (chapitre 4) et l'interface de terminal (chapitre 5), sous la
forme de l'unique commande `epher`. Sur Linux, le paquet installe `epher`
dans `/usr/bin`.

### 3.2 Utilisation

Lancez epher comme n'importe quelle application. Vous obtenez une fenêtre
avec la même interface que l'application web : tapez une expression,
appuyez sur **Entrée** ou cliquez sur **=**, et lisez le résultat. Les
graphiques fonctionnent aussi ici. `graph x ^ 2` dessine dans la fenêtre
(chapitre 2.4). La fenêtre se redimensionne librement. La barre de menus
comprend **Help → User guide**, le même guide que cette page, avec des
exemples à charger d'un toucher.

Vous pouvez aussi l'ouvrir depuis un terminal : un `epher` sans argument (ou
`epher gui`) lance l'application de bureau. Sur macOS, utilisez le bouton
**Install the epher command** dans l'application pour mettre `epher` dans le
PATH de votre terminal.

### 3.3 Stockage : un seul magasin partagé avec la CLI et la TUI

L'application de bureau partage son stockage avec les versions ligne de
commande et terminal. Fonctions, constantes, scripts, historique et préférence de
langue vivent au même endroit, `~/.epher` sur votre ordinateur (ou
`EPHER_STORE_DIR`, chapitre 4.6), et tout ce qui est enregistré dans une
version est disponible dans les autres :

```text
def area(w, h) = w * h
save area
```

Définissez `area` dans l'application de bureau, `save`ez-la, fermez la
fenêtre. Puis ouvrez la CLI et `area(3, 4)` fonctionne. Ça marche aussi
dans l'autre sens : les fonctions et scripts enregistrés dans la CLI ou la
TUI sont déjà là à l'ouverture de la fenêtre, y compris les variables
définies par des scripts enregistrés. Les commandes `save`, `save script`
et `language` du chapitre 4 fonctionnent exactement pareil ici.

Les commandes que vous tapez dans la CLI, le REPL, la TUI ou
l'application de bureau rejoignent toutes le même historique, et la
session voyage aussi : les variables que vous affectez et la valeur
`ans` vous suivent d'une version à l'autre. Le stockage partagé est
vivant : lorsque deux versions sont ouvertes en même temps, un changement
dans l'une se reflète immédiatement dans l'autre (l'application de bureau
et la TUI observent le stockage et se rafraîchissent toutes seules).

> L'application web dans le navigateur est la seule version qui n'utilise
> pas ce stockage : chaque session vit isolée (chapitre 2.6).

## 4. La ligne de commande (CLI)

La CLI est le côté texte du même programme `epher` que l'application de
bureau. Elle a trois modes : l'évaluation à usage unique, les scripts en
pipeline, et une session interactive pour un travail plus long.

Pour obtenir de l'aide à tout moment, lancez `epher --help` (toutes les
commandes, avec des exemples) ou `epher help` (le manuel complet ; sur les
paquets Linux, c'est la page `man epher`).

### 4.1 Calculs à usage unique

Passez l'expression en argument :

```sh
epher "2 + 3 * 4"
```

```text
14
```

Vous pouvez faire tout ce qui est une seule expression dans le chapitre 1 :

```sh
epher "if 3 > 2 then 10 else 20"
```

```text
10
```

Une expression qui commence par un signe moins fonctionne directement :

```sh
epher "-2 + 5"
```

```text
3
```

Le mode à usage unique est fait pour les scripts, d'une simple expression
jusqu'à un programme complet. La valeur de chaque instruction s'affiche sur
sa propre ligne :

```sh
epher "x = 10; x + 5"
```

```text
10
15
```

Les instructions reliées par des retours à la ligne fonctionnent de la même
façon dans l'argument. Tout le chapitre 1 est disponible : variables,
fonctions, boucles, tout. Les lignes partagent une session, comme un
script en pipeline (section 4.2).

### 4.2 Scripts en pipeline

`epher -` lit des expressions depuis l'entrée standard, ligne par ligne,
comme on utilise les langages de script dans les pipelines :

```sh
printf "x = 3\nx * 10\n" | epher -
```

```text
= 3
= 30
```

Tout le chapitre 1 fonctionne, et les lignes partagent une session : une
fonction définie tôt est disponible plus tard, et `save` écrit dans le même
magasin que d'habitude. Les erreurs s'affichent et le script continue.
Une ligne peut relier plusieurs instructions avec `;`. Retours à la ligne
et `;` signifient la même chose partout dans epher.


Un fichier fonctionne pareil : `epher plots/sine.es` exécute chaque ligne du fichier dans l'ordre et affiche chaque résultat. L'argument est traité comme un fichier quand il désigne un fichier existant et contient un `.`, un `/` ou un `\` - `epher x` évalue donc toujours le nom `x`.
### 4.3 La session interactive (REPL)

Lancez-la avec `epher repl` :

```sh
epher repl
```

> Un `epher` sans argument ouvre l'application de bureau (chapitre 3).

epher affiche son invite et attend :

```text
epher>
```

Tapez maintenant n'importe quoi du chapitre 1, une ligne à la fois. Les
variables gardent leur valeur d'une ligne à l'autre :

```text
epher> x = 5
= 5
epher> x ^ 2
= 25
```

La commande `table` (section 2.4.2) affiche ici aussi un tableau de
valeurs :

```text
epher> table x ^ 2 from -2 to 2 points 5
         x           y
        -2           4
        -1           1
         0           0
         1           1
   Les lignes `graph` fonctionnent aussi ici : les courbes s'accumulent au
fil des lignes, et `graph save plot.svg` écrit la même image SVG que le
bouton **Copier le SVG** de l'application web. `graph3d
save fichier.svg` enregistre une surface 3D de la même façon. Ces mêmes
lignes fonctionnent en évaluation unique et en scripts injectés :
`epher "graph sin(x); graph save plot.svg"` est un tracé complet en une
seule commande.

      2           4
```

Chaque réponse s'affiche sous la forme `= résultat`. Pour quitter, tapez
`quit` (ou `exit`) :

```text
epher> quit
```

Votre historique est mémorisé : la prochaine fois que vous lancez
`epher repl`, les lignes de la session précédente sont toujours là.


La commande `load` exécute un script - un chemin de fichier ou le nom d'un script enregistré avec `save script` - ligne par ligne, exactement comme si vous veniez de le taper :

```text
epher> load plots/sine.es
epher> load my_setup
```
### 4.4 Enregistrer fonctions, constantes et scripts

Définissez une fonction, puis enregistrez-la :

```text
epher> def fib(n) = if n <= 1 then n else fib(n - 1) + fib(n - 2)
epher> save fib
saved fib
```

La commande `save fib` enregistre la fonction sur le disque. La prochaine
fois que vous lancez `epher`, `fib` est déjà définie :

```text
epher> fib(10)
= 55
```

Les constantes s'enregistrent pareil : `save` sur le nom de la constante :

```text
epher> const tax = 0.2
= 0.2
epher> save tax
saved tax
```

Pour enregistrer un script complet (la dernière ligne tapée) utilisez
`save script` :

```text
epher> x = 0; while x < 5 do x = x + 1; x
= 5
epher> save script count_to_five
saved script count_to_five
```

Les scripts enregistrés s'exécutent automatiquement au démarrage de epher,
donc tout ce qu'ils définissent est prêt pour vous.


Vous pouvez aussi charger un script enregistré à la demande avec `load count_to_five`, ou le garder en fichier simple et lancer `load count_to_five.es` ; `epher count_to_five.es` l'exécute directement en ligne de commande (section 4.2).
### 4.5 Changer la langue de l'interface

La langue de l'interface est choisie parmi les langues configurées sur votre
appareil. Pour la remplacer, tapez `language` suivi de l'un de : `en`,
`zh-CN`, `hi`, `es`, `fr`, `ar`, `de`, `pt` :

```text
epher> language fr
language set to fr
```

Le choix est mémorisé pour la prochaine fois. Notez : la langue que vous
*tapez*, le langage des expressions, est toujours la même, quelle que soit
la langue de l'interface.

### 4.6 Où vivent vos données

Les fonctions, scripts, l'historique et votre choix de langue sont stockés
dans un dossier de votre ordinateur :

```text
~/.epher
```

Supprimez ce dossier pour repartir de zéro. Pour utiliser un autre
emplacement, définissez la variable d'environnement `EPHER_STORE_DIR` avant
de lancer epher :

```sh
EPHER_STORE_DIR=/tmp/my-epher epher repl
```

## 5. L'interface de terminal (TUI)

La TUI est une version plein écran de la session interactive, dans votre
terminal. Elle fait partie du même programme `epher`. Lancez-la avec :

```sh
epher tui
```

### 5.1 L'écran

L'écran est divisé en panneaux :

- **Expression** : le champ de saisie (en haut). Shift+Entrée commence
  une nouvelle ligne ; les touches fléchées ou un clic de souris
  déplacent le curseur dans le texte.
- Le **résultat** courant juste en dessous.
- **Historique** : chaque ligne saisie, avec sa réponse.
- **Graphique** : le tracé de la commande `graph` (en bas).
- Une ligne d'aide affiche les raccourcis clavier.

### 5.2 Touches

| Touche | Action |
|---|---|
| Taper | ajouter à l'expression au curseur |
| **Entrée** | évaluer tout le script (une saisie multiligne s'exécute comme un seul élément d'historique) |
| **Shift+Entrée** | commencer une nouvelle ligne |
| **← → ↑ ↓** | déplacer le curseur (saisie vide : faire pivoter la vue 3D) |
| **Échap** | effacer la ligne de saisie |
| **F1** | décrire la fonction sous le curseur (dans la ligne de résultat) |
| **Ctrl+C** | quitter |
| **q** | quitter (quand la saisie est vide) |
| **Touches fléchées** | faire pivoter la vue 3D (quand la saisie est vide) |
| **Espace** | démarrer/arrêter l'animation (quand la saisie est vide) |
| **F10** | ouvrir les menus (Fichier, Édition, Graphique, Paramètres, Aide) |
| **Tab** | activer le clavier toujours visible (ou l'historique, depuis le clavier) ; changer de groupe (**Esc** revient à la saisie) |
| **Souris** | cliquez les menus et leurs entrées, les cellules et onglets du clavier, les lignes de l'historique (charge l'expression) ; faites glisser le panneau du graphique pour orbiter (3D) ou déplacer (2D), la molette zoome, un double-clic réinitialise la vue |
| **Ctrl+L** | effacer l'historique |

Le menu **Aide** ouvre le guide intégré, l'aide des touches du clavier et un explorateur de constantes : les constantes groupées, les flèches choisissent une ligne, **Entrée** insère son nom dans l'expression au curseur et **Échap** ferme.

Les groupes du clavier couvrent toutes les fonctions, constantes et
commandes du langage : **trig**, **fn**, **num**, **0x** et **var**
. Le groupe 0x contient les conversions exactes et de base (`frac`,
`dec`, `big`, `bin`, `oct`, `hex`) et la factorielle `!`. Les flèches
déplacent la sélection, **Entrée** insère le token et **Tab** change de
groupe. Un opérateur au début d'une ligne vide (ou inséré depuis le clavier) ajoute `ans` devant : la ligne continue depuis le résultat précédent.

Le menu **Paramètres** propose les mêmes choix d'affichage des résultats que l'application web (fractions exactes, notation, séparateurs de milliers), à côté des lignes thème et langue.

### 5.3 Les graphiques

Tapez `graph` suivi d'une expression, puis appuyez sur **Entrée** :

```epher
graph x ^ 2
```

epher échantillonne la courbe de x = −10 à x = 10 et la dessine sous
forme de graphique ASCII dans le panneau Graph ; la légende au-dessus du
tracé nomme ce qui est tracé.

`graph clear` vide le tracé, et le menu **Graph** fait de même ; le menu
**Help** ouvre ce guide dans la TUI (les touches fléchées font défiler,
**Esc** ferme). Le menu **Settings** peut masquer les points d'intérêt
listés sous le tracé.

Vous pouvez tracer n'importe quelle expression, y compris vos propres
fonctions. Définissez-en d'abord une, puis tracez-la :

```epher
def f(x) = x ^ 3
graph f(x)
```

Chaque ligne `graph` ajoute une courbe au tracé, dessinée avec son propre
symbole (`o`, `x`, `+`, `*`) ; `graph clear` vide le tracé. La même
grammaire que dans l'application web s'applique : un domaine
(`graph sin(x) from 0 to 2*pi`), des courbes paramétriques
(`graph param 2*cos(t), 3*sin(t)`), des courbes polaires
(`graph polar 1 + cos(theta)`) et des régions (`graph y < x ^ 2` ombrage
la zone sous la courbe).

Les points où l'expression n'a pas de valeur (par exemple la division par
zéro) sont simplement ignorés, laissant un vide dans le tracé. Après
chaque commande graph, la TUI liste les points d'intérêt (racines,
extremums et intersections) sous le tracé. La commande `table`
(section 2.4.2) fonctionne ici aussi.

`graph3d x ^ 2 - y ^ 2` trace une surface 3D sous forme de filaire ASCII.
Faites-la pivoter avec les touches fléchées tant que la saisie est vide,
et appuyez sur la barre d'espace pour animer une constante à curseur
(section 2.4.5). La ligne d'aide du bas n'affiche les indications flèches
et espace que lorsqu'une surface 3D ou une courbe animable est affichée.

`graph save plot.svg` écrit le tracé actuel comme la même image SVG que
le bouton **Copier le SVG** de l'application web ; `graph3d
save fichier.svg` enregistre le maillage 3D sous l'angle où vous le
regardez.

### 5.4 Enregistrement et persistance

La TUI partage son stockage avec la CLI : tout ce qui est enregistré dans
l'une est disponible dans l'autre. Les fonctions, scripts, historique et la
préférence de langue vivent dans `~/.epher` (chapitre 4.6), et les mêmes
commandes `save`, `save script` et `language` fonctionnent ici.

## 6. Vos données et la vie privée

- Le **programme epher installé** (application de bureau, CLI et TUI)
  stocke fonctions, scripts, historique et choix de langue localement dans
  `~/.epher` (ou `EPHER_STORE_DIR`). Rien ne quitte votre ordinateur.
- L'**application web** ne stocke rien sur le disque : l'historique ne dure
  que tant que la page est ouverte. L'application web peut fonctionner hors
  ligne parce que c'est votre navigateur qui stocke la page elle-même.

Les cinq versions exécutent le calcul entièrement sur votre appareil.
Rien n'est envoyé nulle part.
