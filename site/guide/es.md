# Guía de usuario de epher

¡Bienvenido! epher es una calculadora programable y con scripts. Puedes usarla
para un cálculo rápido o para construir tus propias funciones y pequeños
programas, y todo está disponible en ocho idiomas.

Esta guía es para principiantes absolutos. Empieza con el cálculo más simple
posible y llega hasta todo el poder del lenguaje. Cada ejemplo muestra lo que
escribes y lo que epher responde.

Hay cinco formas de usar epher. Elige la que más te convenga:

| Versión | Qué es | Cuándo conviene |
|---|---|---|
| **Línea de comandos** (CLI) | Comandos de texto en una terminal | Vives en la terminal y te gustan los scripts |
| **REPL** | Una sesión interactiva de `epher` en el indicador `epher>` | Quieres ida y vuelta rápida sin salir de la terminal |
| **Interfaz de terminal** (TUI) | Un programa a pantalla completa dentro de la terminal | Quieres una app de terminal con gráficos e historial |
| **Aplicación de escritorio** | Un programa normal con su propia ventana | Quieres una aplicación de escritorio |
| **Aplicación web** (PWA) | Se ejecuta en tu navegador, se puede instalar y funciona sin conexión | Quieres empezar rápido; sin instalación |

La aplicación de escritorio, la línea de comandos, el REPL y la interfaz de
terminal son un solo programa: una única descarga instala el comando
`epher`, que hace las cuatro cosas. La aplicación web es la excepción: no
necesita descarga.

Las cinco versiones entienden exactamente el mismo lenguaje. Apréndelo una
vez, úsalo en cualquier parte.

## 1. El lenguaje de epher

Este capítulo enseña el lenguaje compartido por todas las versiones de epher.
En la aplicación web o de escritorio, escribe una expresión y pulsa
**Intro** (o haz clic en el botón **=**). En la CLI, inicia la sesión con
`epher repl` y escribe después del prompt `epher>`. En la TUI (`epher tui`),
solo escribe y pulsa **Intro**. En la CLI también
puedes escribir `epher "expresión"` para evaluar una expresión directamente.

### 1.1 Tu primer cálculo

Escribe esto:

```epher
2 + 3 * 4
```

epher responde:

```text
14
```

La multiplicación se hace antes que la suma, exactamente como en
matemáticas. Esa regla se llama *precedencia de operadores*.

### 1.2 Orden de las operaciones

El orden completo de precedencia, de más fuerte a más débil:

1. `!` factorial y `%` porcentaje (ambos posfijos)
2. `^` potencia
3. `*` y `/` multiplicación y división
4. `+` y `-` suma y resta

Usa paréntesis para cambiar el orden:

```epher
(2 + 3) * 4
```

```text
20
```

El operador `^` calcula potencias y funciona de derecha a izquierda:

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

(`2 ^ 3 ^ 2` significa `2 ^ (3 ^ 2)`, es decir `2 ^ 9` = 512.)

Las potencias pueden ser fraccionarias. `2 ^ 0.5` es la raíz cuadrada de 2:

```epher
2 ^ 0.5
```

```text
1.41421356237
```

La resta y la división funcionan de izquierda a derecha:

```epher
10 - 3 - 2
```

```text
5
```

El signo `%` es un operador posfijo que significa «dividido entre 100»: `5%` es 0.05. Nunca mira los operadores que tiene alrededor, así que `200 + 10%` es 200.1. Para aumentar 200 un 10%, escribe la multiplicación:

```epher
200 * (1 + 10%)
```

```text
220
```


### 1.3 Los números especiales pi, e, tau y phi

Las constantes famosas están integradas:

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

Dos más: `tau` es una vuelta completa (2 pi) y `phi` es el número áureo:

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

### 1.4 Comparar y lógica

Puedes comparar números. El resultado es `true` (verdadero) o `false`
(falso):

| Comparación | Significado |
|---|---|
| `a > b` | a es mayor que b |
| `a < b` | a es menor que b |
| `a >= b` | a es mayor o igual que b |
| `a <= b` | a es menor o igual que b |
| `a == b` | a es igual a b (nota el doble `=`) |
| `a != b` | a no es igual a b |

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

Combina comparaciones con `and`, `or` y `not`:

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

Dale un nombre a un valor con un solo `=`:

```epher
x = 5
```

```text
5
```

epher te repite el valor. Desde ahora, `x` se puede usar en cualquier parte:

```epher
x ^ 2
```

```text
25
```

Puedes cambiar una variable cuando quieras. Conserva su valor hasta que la
cambies:

```epher
x = x + 1
```

```text
6
```

> Los nombres pueden contener letras y guiones bajos, como `radius` o
> `my_total`. No pueden contener espacios ni empezar por un número.

La variable especial `ans` contiene siempre la respuesta anterior,
como la tecla `Ans` de una calculadora de bolsillo, útil para encadenar
cálculos:

```epher
2 + 3
ans * 2
```

```text
5
10
```

### 1.6 Constantes: nombres que nunca cambian

Una *constante* es un nombre cuyo valor nunca cambia, como el `pi`
integrado, pero elegido por ti. Define una con `const`:

```epher
const tax = 0.2
```

```text
0.2
```

Úsala donde pueda ir un número:

```epher
100 * (1 + tax)
```

```text
120
```

El valor es fijo: cambiarlo con `=` es un error,

```epher
tax = 0.25
```

```text
error: cannot assign to constant tax
```

y redefinirla con un valor distinto también:

```epher
const tax = 0.25
```

```text
error: constant already defined: tax
```

Las constantes se diferencian de las variables en una cosa más: como `pi`,
funcionan dentro de tus propias funciones.

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

Guarda una constante para sesiones futuras con `save tax`, igual que una
función (capítulo 4.4).

> Una variable y una constante no pueden compartir nombre: tras
> `const tax = 0.2`, `tax = ...` siempre es un error. Elige otro nombre o
> empieza una sesión nueva.

### 1.7 Decisiones con if

`if` elige entre dos valores:

```epher
if 3 > 2 then 10 else 20
```

```text
10
```

La forma es siempre `if condición then valor_si_verdadero
else valor_si_falso`. La parte `else` es obligatoria.

Un ejemplo más útil con una variable:

```epher
price = 100
if price > 50 then 2 else 1
```

```text
100
2
```

> epher no tiene valores de texto: ambas ramas de un `if` deben ser números
> (o resultados de comparaciones).

### 1.8 Bucles con while

`while` repite una instrucción mientras se cumpla una condición:

```epher
x = 0; while x < 5 do x = x + 1; x
```

```text
0
5
```

Lee ese script así: *empieza x en 0; mientras x sea menor que 5, suma 1 a x;
luego muestra x.* El resultado es 5 porque el bucle se ejecutó cinco veces.

> **Red de seguridad:** epher detiene cualquier bucle después de 100 000
> pasos y muestra `error: step limit exceeded`. Eso te protege de bucles que
> nunca terminarían. Si lo ves, tu condición probablemente nunca se volvió
> falsa.

### 1.9 Tus propias funciones con def

Una función es un cálculo con nombre y parámetros:

```epher
def f(x) = x ^ 2
```

Luego úsala:

```epher
f(7)
```

```text
49
```

Las funciones pueden tener varios parámetros:

```epher
def area(w, h) = w * h
area(3, 4)
```

```text
12
```

También puedes definir una función sin parámetros:

```epher
def answer() = 42
answer()
```

```text
42
```

### 1.10 Recursión: una función que se llama a sí misma

El ejemplo más famoso son los números de Fibonacci:

```epher
def fib(n) = if n <= 1 then n else fib(n - 1) + fib(n - 2)
```

```epher
fib(10)
```

```text
55
```

`fib(10)` es el décimo número de Fibonacci. La función se llama a sí misma
con argumentos más pequeños hasta llegar a `n <= 1`. Esto funciona porque la
forma `if ... then ... else ...` solo calcula la rama que necesita.

> El cuerpo de una función es una sola expresión, una línea. Combina
> varios cálculos con `;` en un script en su lugar (sección siguiente).

### 1.11 Scripts: varias instrucciones a la vez

Un *script* es varias instrucciones unidas con `;` o con saltos de
línea, que significan exactamente lo mismo, ejecutadas una tras otra:

```epher
x = 10; y = x + 5; x + y
```

```text
10
15
25
```

Los scripts son la forma de construir pequeños programas: prepara
variables, haz bucles y muestra un resultado final.

Los saltos de línea y `;` son el mismo separador, y puedes mezclarlos
libremente. El botón **Copiar** situado encima de cada ejemplo de varias
líneas copia el script completo, y puedes pegarlo directamente en epher: el
campo de entrada de la aplicación web y de la aplicación de escritorio, la
interfaz de terminal y `epher repl` ejecutan todas las líneas en orden,
exactamente como si las hubieras escrito una a una. Unir varias
instrucciones con `;` en una línea funciona en todas partes también,
incluida la línea de comandos de un solo uso (sección 4.1).


Los scripts pueden llevar **comentarios**: notas para ti que epher se salta, escritos al estilo PHP. `//` o `#` comentan hasta el final de la línea; `/* ... */` comenta un bloque, entre líneas o en medio de los tokens:

```epher
// a small script with notes
r = 3 # radius in metres
area = /* pi r squared */ pi * r ^ 2
area
```
### 1.12 Resultados exactos: frac, dec y big

Normalmente epher calcula con números decimales como una
calculadora de bolsillo, y los resultados se redondean a doce cifras
significativas como los muestra una calculadora: `0.1 + 0.2` es
`0.3`, nunca `0.30000000000000004`. Las fracciones exactas están
activadas por defecto — un resultado con una buena fracción de
denominador pequeño cuyo decimal se repite se muestra como tal.
`1 / 3` se muestra como `1/3` sin pedirlo:

```epher
1 / 3
```

```text
1/3
```

Con **fracciones exactas desactivadas** en los ajustes de resultados
(capítulo 2.2), la misma división muestra `0.333333333333`.
**frac(n, d)** crea una fracción exacta que se mantiene exacta a través
de los cálculos:

```epher
frac(1, 3)
```

```text
1/3
```

Las fracciones se mantienen exactas a través de los cálculos:

```epher
frac(1, 3) * 3
```

```text
1
```

**dec(x)** crea un decimal exacto. `0.1 + 0.2` muestra `0.3` en ambos
casos — la diferencia es aritmética:

```epher
0.1 * 3 - 0.3
dec(0.1) * 3 - dec(0.3)
```

```text
0.0000000000000000555111512313
0.0
```

El resultado flotante lleva el pequeño error de redondeo que toda
computadora comete con los decimales; `dec()` mantiene la aritmética
exacta.

**big(x)** crea un número entero exacto, para valores demasiado grandes para
una calculadora de bolsillo:

```epher
big(10 ^ 20)
```

```text
100000000000000000000
```

**Bases numéricas** escriben los enteros como los escribe la comunidad
matemática: `0b` para binario, `0o` para octal, `0x` para hexadecimal
(el prefijo cambia la escritura, nunca el valor):

```epher
0b1010 + 0xFF
```

```text
265
```

Para volver, **bin(x)**, **oct(x)** y **hex(x)** devuelven la escritura
con prefijo de un número entero, lista para usarse de nuevo:

```epher
hex(255)
bin(10)
```

```text
0xff
```
0b1010
```

**exact(x)** reconstruye la fracción exacta detrás de un resultado decimal: cualquier valor con una buena fracción de denominador pequeño se muestra así. Es la misma reconstrucción que las aplicaciones usan por defecto, por eso `1 / 3` normalmente se muestra como `1/3`:

```epher
exact(0.3333333333333333)
exact(0.30000000000000004)
```

```text
1/3
3/10
```

Un valor irracional como `pi` no tiene una buena fracción, así que `exact()` lo deja igual.

Los verbos de formato escriben un número en otra notación. **scientific(x)** usa una cifra antes del exponente, **engineering(x)** exponentes en pasos de tres (la mantisa queda entre 1 y 1000), y **grouped(x)** inserta espacios finos como separadores de miles:

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
1 234 567.89
```

La aplicación web y la TUI también ofrecen estas opciones de visualización (capítulos 2.2 y 5.2): fracciones exactas activadas o desactivadas, notación Auto/científica/de ingeniería y separadores de miles. Los ajustes solo cambian cómo se muestran los resultados; los valores siguen siendo números decimales normales.

### ### 1.13 Funciones integradas

epher tiene las funciones de una calculadora científica, agrupadas por familia.

La trigonometría trabaja en radianes. Usa `deg` y `rad` para convertir:

| Función | Significado | Ejemplo | Resultado |
|---|---|---|---|
| `sin(x)`, `cos(x)`, `tan(x)` | funciones trigonométricas | `sin(pi / 2)` | `1` |
| `asin(x)`, `acos(x)`, `atan(x)` | trigonometría inversa | `atan(1)` | `0.7853981633974483` |
| `atan2(y, x)` | ángulo del punto (x, y) | `atan2(1, 1)` | `0.7853981633974483` |
| `deg(x)` | radianes → grados | `deg(pi)` | `180` |
| `rad(x)` | grados → radianes | `rad(180)` | `3.14159265359` |
| `sinh(x)`, `cosh(x)`, `tanh(x)` | funciones hiperbólicas | `sinh(1)` | `1.1752011936438014` |
| `asinh(x)`, `acosh(x)`, `atanh(x)` | hiperbólicas inversas | `acosh(1)` | `0` |

Potencias, raíces y logaritmos (en una calculadora `log` es base 10):

| Función | Significado | Ejemplo | Resultado |
|---|---|---|---|
| `sqrt(x)` | raíz cuadrada | `sqrt(16)` | `4` |
| `cbrt(x)` | raíz cúbica | `cbrt(-27)` | `-3` |
| `root(n, x)` | raíz n-ésima | `root(3, 8)` | `2` |
| `exp(x)` | e elevado a x | `exp(1)` | `2.71828182846` |
| `ln(x)` | logaritmo natural | `ln(e)` | `1` |
| `log(x)` | logaritmo base 10 | `log(100)` | `2` |
| `log2(x)` | logaritmo base 2 | `log2(8)` | `3` |
| `logb(b, x)` | logaritmo en base b | `logb(2, 8)` | `3` |
| `hypot(a, b)` | hipotenusa | `hypot(3, 4)` | `5` |
| `5!` (también `fact(n)`) | factorial | `5!` | `120` |

Redondeo, signos y números enteros:

| Función | Significado | Ejemplo | Resultado |
|---|---|---|---|
| `abs(x)` | valor absoluto | `abs(-3)` | `3` |
| `floor(x)` / `ceil(x)` | redondear abajo / arriba | `floor(2.7)` | `2` |
| `round(x)` | el más cercano, medio siempre lejos de cero | `round(2.5)` | `3` |
| `trunc(x)` | quitar la parte decimal | `trunc(-2.9)` | `-2` |
| `sign(x)` | -1, 0 o 1 | `sign(-5)` | `-1` |
| `ncr(n, r)` | combinaciones | `ncr(52, 5)` | `2598960` |
| `npr(n, r)` | permutaciones | `npr(5, 2)` | `20` |
| `gcd(a, b)` / `lcm(a, b)` | divisores y múltiplos comunes | `gcd(12, 18)` | `6` |
| `mod(a, b)` | resto | `mod(7, 3)` | `1` |

Los primos y los divisores trabajan con números enteros:

| Función | Significado | Ejemplo | Resultado |
|---|---|---|---|
| `isprime(n)` | verdadero cuando n es primo | `isprime(97)` | `true` |
| `nextprime(n)` / `prevprime(n)` | los primos más cercanos | `nextprime(10)` | `11` |
| `factors(n)` | factorización en primos | `factors(360)` |
| Literal de lista | `{…}` | `{1, 2, 3}` |
| Elemento de lista | `list[i]` (base 1) | `{5, 6}[2]` |
| Estadística de lista | `mean(lista)`, `median(lista)`, … | `stdev(d)` |
| Forma de lista | `len(s)`, `sort(s)`, `mode(s)`, `range(s)`, `quartile(s, k)` | `quartile(d, 1)` |
| Regresión lineal | `linreg(xs, ys)` | `linreg(x, y)` |
| Familia normal | `normpdf` `normcdf` `invnorm` | `invnorm(0.975)` |
| Familia t | `tpdf` `tcdf` `invt` | `invt(0.975, 10)` |
| Familia chi-cuadrado | `chi2pdf` `chi2cdf` `invchi2` | `chi2cdf(3.84, 1)` |
| Familias discretas | `binompdf` `binomcdf` `poissonpdf` `poissoncdf` | `binomcdf(2, 10, 0.5)` |
| Pruebas e intervalos | `ztest` `ttest` `zinterval` `tinterval` `chisq_gof` | `tinterval(d, 0.95)` |
| Gráficos de datos | `graph scatter(xs, ys)` `histogram(data)` `boxplot(data)` | `graph boxplot(d)` |
| Números aleatorios | `random()`, `random(a, b)`, `randint(a, b)`, `randseed(n)` | `randint(1, 6)` |
| Explorador de constantes | Ayuda → Constantes: todas las constantes, agrupadas | Ayuda → Constantes |
| Cantidad | `5 m`, `60 mile/hr`, `1 km` | `2 m^2` |
| Convertir | `expr in unidad` o `expr -> unidad` | `72 km/hr in m/s` |
| Prefijos | `k M G T m µ n p` escalan cualquier unidad | `5 km`, `3 MPa`, `1 GHz` |
| Y, O bit a bit | `a & b`, `a \| b` | `0xFF & 0x0F` |
| O exclusivo bit a bit | `a xor b` | `5 xor 3` |
| No bit a bit | `~a` | `~0` |
| Desplazamientos | `a << n`, `a >> n` | `1 << 8` |
| Tamaño de palabra | `bits(n)` — 8, 16, 32, 64 | `bits(8)` |
| Relación implícita | `graph lhs == rhs` | `graph x^2 + y^2 == 1` |
| Literal de matriz | `[[1, 2], [3, 4]]` | `[[1, 2], [3, 4]] * [[5, 6], [7, 8]]` |
| Funciones matriciales | `det` `inv` `transpose` `trace` `dim` `ref` `rref` | `rref([[2, 1, 5], [1, -1, 1]])` |
| Solucionador TVM | `tvm_n` `tvm_i` `tvm_pv` `tvm_pmt` `tvm_fv` | `tvm_pmt(360, 0.08/12, -100000, 0)` |
| VAN y TIR | `npv(rate, flows)` `irr(flows)` | `irr({-100, 60, 60})` |
| Amortización | `amort(p, r, n, k)` | `amort(1000, 0.01, 12, 6)` |
| Interés | `simple_interest` `compound_interest` | `compound_interest(1000, 0.05, 2)` | `2^3 * 3^2 * 5` |
| `totient(n)` | phi de Euler | `totient(12)` | `4` |
| `ndivisors(n)` | cuántos divisores tiene | `ndivisors(360)` | `24` |
| `modpow(b, e, m)` | b elevado a e, módulo m, exacto | `modpow(2, 10, 1000)` | `24` |


La estadística acepta cualquier número de argumentos:

| Función | Significado | Ejemplo | Resultado |
|---|---|---|---|
| `sum(...)` / `product(...)` | totales | `sum(1, 2, 3)` | `6` |
| `mean(...)` | promedio | `mean(1, 2, 3)` | `2` |
| `median(...)` | valor central | `median(1, 2, 3, 4)` | `2.5` |
| `min(...)` / `max(...)` | el menor / el mayor | `max(4, 1, 3)` | `4` |
| `variance(...)` / `stdev(...)` | dispersión de los valores | `stdev(2, 4)` | `1` |

Las capas exactas de la sección 1.12 se mantienen:

| Función | Significado | Ejemplo | Resultado |
|---|---|---|---|
| `frac(n, d)` | fracción exacta | `frac(1, 3)` | `1/3` |
| `dec(x)` | decimal exacto | `dec(0.1)` | `0.1` |
| `big(x)` | número entero exacto | `big(10 ^ 20)` | `100000000000000000000` |
| Binario, octal, hex | `0b…`, `0o…`, `0x…` | `0xFF + 0b1` |
| Escritura en base | `bin(x)`, `oct(x)`, `hex(x)` | `hex(255)` |
| Primos | `isprime(n)`, `factors(n)`, … | `factors(360)` |
| `bin(x)` / `oct(x)` / `hex(x)` | escritura con prefijo en base 2 / 8 / 16 | `hex(255)` | `0xff` |

Se combinan como todo lo demás:

```epher
min(sqrt(16), 5)
```

```text
4
```

Las constantes físicas usan unidades del SI, como las astronómicas de la sección 1.16:

| Nombre | Significado | Valor |
|---|---|---|
| `G` | constante gravitacional de Newton | 6.6743e-11 |
| `gamma` | constante de Euler-Mascheroni | 0.577215664902 |
| `q_e` | carga elemental | 1.602176634e-19 |
| `ev` | electronvoltio, en julios | 1.602176634e-19 |
| `eps_0` | permitividad del vacío | 8.8541878128e-12 |
| `mu_0` | permeabilidad del vacío | 1.25663706212e-6 |
| `z_0` | impedancia del vacío | 376.730313668 |
| `m_e` | masa del electrón | 9.1093837139e-31 |
| `m_p` | masa del protón | 1.67262192595e-27 |
| `m_n` | masa del neutrón | 1.67492750056e-27 |
| `m_u` | unidad de masa atómica | 1.66053906892e-27 |
| `a_0` | radio de Bohr | 5.29177210544e-11 |
| `alpha` | constante de estructura fina | 0.0072973525643 |
| `r_inf` | constante de Rydberg | 10973731.568160 |
| `mu_b` | magnetrón de Bohr | 9.2740100783e-24 |
| `n_a` | constante de Avogadro | 6.02214076e23 |
| `faraday` | constante de Faraday, C/mol | 96485.33212 |
| `r_gas` | constante molar de los gases | 8.31446261815 |
| `atm` | atmósfera estándar, en pascales | 101325 |
| `wien` | constante de longitud de onda de Wien | 0.002897771955 |
| `phi_0` | cuanto de flujo magnético | 2.067833848e-15 |
| `m_P` | Planck-Masse | 2.176434e-8 |
| `l_P` | Planck-Länge | 1.616255e-35 |
| `t_P` | Planck-Zeit | 5.391247e-44 |
| `r_e` | klassischer Elektronenradius | 2.8179403205e-15 |
| `lambda_c` | Compton-Wellenlänge | 2.42631023867e-12 |
| `mu_n` | Kernmagneton | 5.050783699e-27 |


### 1.14 Leer los errores

Cuando algo sale mal, epher te lo dice en lugar de adivinar:

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
2i
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

El último ejemplo es importante: epher te dice exactamente qué nombre no
conoce, para que puedas arreglar tu expresión.

### 1.15 Referencia rápida

| Qué | Sintaxis | Ejemplo |
|---|---|---|
| Sumar, restar, multiplicar, dividir | `+ - * /` | `7 / 2` |
| Potencia | `^` (de derecha a izquierda) | `2 ^ 10` |
| Factorial | `!` (postfijo) | `5!` |
| Porcentaje | `%` (posfijo) | `200 * (1 + 10%)` |
| Paréntesis | `( )` | `(2 + 3) * 4` |
| Constantes | `pi`, `e`, `tau`, `phi` | `2 * pi` |
| Notación científica | `2.5e-3` | `6.02e23` |
| Comparar | `> < >= <= == !=` | `3 >= 2` |
| Lógica | `and or not` | `a > 1 and a < 10` |
| Variable | `name = value` | `x = 5` |
| Constante | `const name = value` | `const tax = 0.2` |
| Decisión | `if c then a else b` | `if x > 0 then 1 else -1` |
| Bucle | `while c do statement` | `while x < 5 do x = x + 1` |
| Función | `def name(params) = expr` | `def f(x) = x ^ 2` |
| Script | instrucciones unidas con `;` o saltos de línea | `x = 1; x + 1` |
| Fracción exacta | `frac(n, d)` | `frac(1, 3)` |
| Decimal exacto | `dec(x)` | `dec(0.1) + dec(0.2)` |
| Número entero exacto | `big(x)` | `big(10 ^ 20)` |
| Reconstruir una fracción | `exact(x)` | `exact(0.3333333333333333)` |
| Científica, ingeniería, agrupada | `scientific(x)` `engineering(x)` `grouped(x)` | `engineering(12345)` |
| Unidad imaginaria | `i`, o un literal `4i` | `sqrt(-1)` |
| Partes de un complejo | `re(z)` `im(z)` `arg(z)` `conj(z)` `abs(z)` | `re(3 + 4i)` |
| Resolver una ecuación | `solve lhs == rhs` | `solve x^2 == 9` |
| Derivada numérica | `derivative(expr, x)` | `derivative(x^2, 3)` |
| Integral definida | `integral(expr, a, b)` | `integral(x^2, 0, 3)` |
| Binario, octal, hex | `0b…`, `0o…`, `0x…` | `0xFF + 0b1` |
| Escritura en base | `bin(x)`, `oct(x)`, `hex(x)` | `hex(255)` |

### 1.16 Astronomía y el sistema solar

epher habla astronomía: sufijos de unidad, constantes físicas, funciones de
calendario y tiempo, y una efeméride en vivo para el Sol, la Luna, los
planetas y Plutón. Todo funciona sin conexión.

**Unidades que hablan astronomía.** Escribe un número seguido de un sufijo
de unidad y epher lo convierte a unidades SI al instante:

| Sufijo | Unidad | Convierte a |
|---|---|---|
| `AU` o `au` | unidad astronómica | metros |
| `pc` | pársec | metros |
| `ly` | año luz | metros |
| `deg` | grado | radianes |
| `arcmin`, `arcsec` | minuto y segundo de arco | radianes |
| `min`, `hr`, `d`, `yr` | minuto, hora, día, año juliano | segundos |
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

Los sufijos son parte de la gramática: ninguna constante propia puede
cambiar lo que significa `3.2 AU`, y `h` sigue siendo la constante de
Planck; las horas se escriben `hr`. Las funciones devuelven conteos en
unidades naturales; un sufijo convierte un conteo a SI, así que
`mag2jy(20)` es un conteo en janskys y `mag2jy(20) Jy` es el mismo flujo
en vatios por metro cuadrado y hercio.

**Constantes astronómicas.** `au`, `pc`, `ly`, `c`, `g`, `h`, `h_bar`,
`k_b`, `sigma_sb`, `m_sun`, `r_sun`, `l_sun`, `m_earth`, `r_earth`, `m_moon`, `r_moon` funcionan
como `pi`, y puedes ensombrecerlas con tus propias constantes.

**Fechas y tiempo.** `jd(y, m, d [, hr])` y `mjd(...)` convierten una fecha
de calendario en fecha juliana; `now()` lee el instante actual:

```epher
jd(2000, 1, 1, 12)
```

```text
2451545
```

`delta_t(jd)` es la corrección TT - UT1, y `lst(jd, lon)` es el tiempo
sideral local en horas para una longitud este en grados.

**Horas, minutos y segundos.** `hms2deg(h, m, s)` convierte ascensión recta
a grados, `dms2deg(d, m, s)` un ángulo sexagesimal, y `deg2hms(x)` /
`deg2dms(x)` escriben un ángulo como texto:

```epher
deg2hms(90)
```

```text
6h 0m 0s
```

**El cielo, cuantificado.** Da a cada función un número de cuerpo:
Mercurio 1, Venus 2, Marte 4, Júpiter 5, Saturno 6, Urano 7, Neptuno 8,
Plutón 9, Sol 10, Luna 11 (la Tierra es 3, la observadora, nunca un
objetivo).

| Función | Significado |
|---|---|
| `ra(b, jd)`, `decl(b, jd)` | ascensión recta y declinación geocéntricas (grados) |
| `dist(b, jd)` | distancia en UA |
| `alt(b, jd, lat, lon)`, `az(b, jd, lat, lon)` | altura y acimut topocéntricos (grados, verdaderos) |
| `rise(b, jd, lat, lon)`, `set(...)`, `transit(...)` | eventos del día solar local, como fechas julianas |
| `mag(b, jd)` | magnitud aparente |
| `phase(b, jd)`, `illum(b, jd)` | ángulo de fase (grados) y fracción iluminada |
| `diam(b, jd)` | diámetro angular (grados) |

```epher
decl(10, jd(2000, 6, 21, 1.8))
```

```text
23.437882351
```

Latitudes y longitudes son grados, este positivo. Las posiciones son
geocéntricas salvo que se dé un observador. Plutón usa una órbita
aproximada honesta a un minuto de arco, muy por debajo de la precisión de
los demás cuerpos; los eclipses y las búsquedas de conjunciones no están
incluidos.

**Óptica y luz.** `kepler(M, e)` resuelve la ecuación de Kepler,
`airmass(alt)` es la masa de aire sec(z), `dawes(d)` es el poder de
resolución de una apertura de d milímetros en segundos de arco, y
`dist_mod(mu)` convierte un módulo de distancia en pársecs.

**Estaciones.** `march_equinox(year)`, `june_solstice(year)`,
`september_equinox(year)` y `december_solstice(year)` devuelven la fecha
juliana de cada cambio de estación:

```epher
march_equinox(2000)
```

```text
1012520636/413
```

**El sistema solar en 3D.** El comando `solar3d` dibuja todo el sistema:
cada órbita como una curva, cada cuerpo como un punto etiquetado, con una
estela que muestra dónde estaba:

```epher
solar3d jd(2020, 7, 1)
```

Da el tiempo como una constante y pulsa el botón de reproducir para ver
moverse los planetas: `const t = now(); solar3d t`. Arrastra o usa las
flechas para orbitar, `clear` para vaciar y `solar3d save file.svg` para
exportar.

La efeméride la calcula el crate solar-ephemeris
(github.com/Protonmatter/sol), validado contra JPL Horizons; gracias a su
autor. La precisión es de clase arcsecond para el Sol, la Luna y los
planetas a lo largo de unos 5000 años alrededor del presente.

### 1.17 Números complejos

epher calcula con números complejos automáticamente. La unidad imaginaria es **i**, igual que `pi`:

```epher
i ^ 2
sqrt(-1)
```

```text
-1
i
```

Escribe un número complejo con el sufijo `i`, sin signo de multiplicación: `3 + 4i` es un literal, `2.5i` funciona, y también los literales con base (`0xFFi`). La aritmética habitual se extiende: sumar, restar, multiplicar, dividir y potencias funcionan, e `i` sigue la precedencia normal (`i ^ 2` se agrupa como cualquier potencia).

Las funciones reales también se extienden. Con un argumento complejo calculan en el plano complejo; con un argumento real fuera de su dominio real devuelven el resultado complejo principal en lugar de un error:

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

(`exp(i * pi)` es exactamente `-1`; los últimos dígitos son el ruido de `sin(pi)` en la aritmética de la máquina.)

Cuatro funciones leen las partes de un número complejo, y `abs()` es su módulo:

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

Las funciones de enteros (`fact`, `gcd`, `floor`, `isprime`, ...) rechazan argumentos complejos con un error de tipo.

### 1.18 Resolver ecuaciones

**solve** encuentra las raíces de una ecuación con una variable. La ecuación usa `==`:

```epher
solve x^2 == 5*x + 6
```

```text
x = -1, x = 6
```

Las ecuaciones polinómicas (construidas con `+ - * ^` y constantes) dan todas las raíces, reales y complejas:

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

La variable a resolver es `x` si aparece, si no la única otra variable. Las constantes y variables ligadas actúan como parámetros:

```epher
const k = 3
solve k*x == 12
```

```text
3
x = 4
```

Cualquier otra ecuación se explora numéricamente en -100..100: las raíces se encierran por cambios de signo, así que `solve sin(x) == 0.5` lista cada raíz del intervalo. Dos límites honestos: una raíz donde la función solo toca cero (como `x^2 == 0` por el camino numérico) puede perderse, y las ecuaciones con varias variables sin ligar son un error.

### 1.19 Cálculo: derivada e integral

**derivative(expr, p)** es la derivada numérica de `expr` en `p`. El primer argumento sigue siendo una expresión, y su variable libre es la que se deriva:

```epher
derivative(x^2, 3)
derivative(sin(t), 0)
```

```text
6
1
```

Como el argumento sigue siendo una expresión, la derivada se puede graficar: `graph derivative(x^3 - x, x)` dibuja la curva de pendientes.

**integral(expr, a, b)** es la integral definida de `a` a `b`, calculada con cuadratura adaptativa de Simpson:

```epher
integral(x^2, 0, 3)
integral(sin(x), 0, pi)
```

```text
9
2
```

`integral(x^2, 3, 0)` es `-9` (la integral con signo), y un límite superior graficable funciona: `graph integral(x^2, 0, x)`.

Ambos son numéricos; las expresiones deben tomar valores reales en el intervalo, y una expresión con varias variables es un error.

### 1.20 Datos: listas, estadística y regresión

Una lista es una columna de números entre llaves: `{1, 2, 3}`. Los
elementos son expresiones, la lista vacía `{}` está permitida, y una
lista se asigna a un nombre como cualquier valor:

```epher
d = {12, 15, 14, 16, 13, 15, 14, 17}
d[2]
len(d)
```

`list[i]` es el elemento i-ésimo, con base 1 como en una calculadora;
un índice fuera de la lista es un error. El corchete une más fuerte
que `^`, así que `d[2]^2` es `(d[2])^2`.

La aritmética sobre una lista es elemento a elemento, y un número solo
se aplica a cada elemento:

```epher
{1, 2, 3} * 2
{1, 2, 3} + 10
```

Dos listas deben tener la misma longitud para `+ - * / ^`. `==` y `!=`
comparan listas completas; las comparaciones de orden las rechazan.

Las funciones estadísticas aceptan una lista como único argumento (la
forma con varios argumentos se mantiene — `mean(1, 2, 3)` sigue
funcionando): `sum product mean median mode variance stdev min max
range`. Las nuevas funciones de forma son `len(lista)`, `sort(lista)`
(copia ascendente), `mode(lista)` (valor más frecuente, el menor en
caso de empate), `range(lista)` (mayor menos menor valor) y
`quartile(lista, k)` para k en 1..3 (cuartiles al estilo TI, mediana
de las mitades):

```epher
mean(d)
median(d)
quartile(d, 1)
```

**linreg(xs, ys)** ajusta la recta de mínimos cuadrados a dos listas
de la misma longitud y la informa con el coeficiente de correlación r:

```epher
linreg({1, 2, 3, 4}, {2.1, 4.2, 5.8, 8.1})
```

La recta ajustada es una presentación, como las raíces de solve; la
imagen del ajuste vive en el gráfico de dispersión (sección 1.22).

### 1.21 Distribuciones y pruebas de hipótesis

Las funciones de probabilidad cubren la normal estándar, la t de
Student, la chi-cuadrado, la binomial y la de Poisson. La familia
normal admite uno o tres argumentos — un solo argumento es la normal
estándar:

```epher
normcdf(1.96)
invnorm(0.975)
normcdf(12, 10, 2)
```

`normpdf(x[, mu, sigma])`, `normcdf(x[, mu, sigma])`, `invnorm(p[,
mu, sigma])`; `tpdf(x, df)`, `tcdf(x, df)`, `invt(p, df)`;
`chi2pdf(x, df)`, `chi2cdf(x, df)`, `invchi2(p, df)`;
`binompdf(k, n, p)`, `binomcdf(k, n, p)`; `poissonpdf(k, lambda)`,
`poissoncdf(k, lambda)`. Las funciones `inv*` responden a la pregunta
inversa: `invt(0.975, 10)` es el valor de t bajo el cual está el
97.5 % de la masa.

Las pruebas toman una lista de datos e informan el estadístico y el
valor p bilateral como texto de presentación; los intervalos informan
`(bajo, alto)` al nivel que usted nombre:

```epher
d = {12, 15, 14, 16, 13, 15, 14, 17}
ttest(d, 14)
tinterval(d, 0.95)
ztest(d, 14, 1.5)
chisq_gof({20, 30, 25, 25}, {25, 25, 25, 25})
```

```text
{12, 15, 14, 16, 13, 15, 14, 17}
t = 0.8819, p = 0.4071
(13.1594, 15.8406)
z = 0.9428, p = 0.3458
chi2 = 2, p = 0.5724
```

`ttest(datos, mu0)` y `tinterval(datos, nivel)` usan la desviación
estándar muestral (n−1); `ztest(datos, mu0, sigma)` y
`zinterval(datos, sigma, nivel)` necesitan el sigma conocido.
`chisq_gof(observados, esperados)` es la prueba de bondad de ajuste
con k−1 grados de libertad. Los resultados son textos de presentación:
legibles y copiables, pero la aritmética no puede tocarlos.

### 1.22 Gráficos de datos

La familia de gráficos también acepta listas: un diagrama de
dispersión, un histograma y un diagrama de caja. Un gráfico de datos
ocupa el panel solo, como un sistema solar — el comando más reciente
gana, y `graph clear` lo vacía.

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

**scatter(xs, ys)** dibuja los puntos y, con dos o más, la recta de
mínimos cuadrados, con la leyenda `y = a*x + b (r = …)`.
**histogram(datos[, clases])** dibuja un histograma de frecuencias; el
número de clases es opcional (regla de Sturges por defecto) y debe ser
un entero entre 1 y 50. **boxplot(datos)** dibuja el diagrama de caja:
mínimo, Q1, mediana, Q3, máximo, con bigotes hasta los extremos. La
ventana siempre se ajusta a los datos — las palabras clave `from a to
b` no se aplican — y la imagen se exporta y guarda como cualquier
gráfico.

### 1.23 Números aleatorios

`random()` sortea un número uniforme en `[0, 1)`, `random(a, b)` uno en
`[a, b)`, y `randint(a, b)` un número entero del intervalo cerrado
`[a, b]` — una tirada de dados:

```epher
randseed(7)
randint(1, 6)
```

```text
7
3
```

La secuencia es reproducible: `randseed(n)` reinicia el generador con `n`
y lo muestra, así que la misma semilla repite los mismos sorteos en cada
sesión y en cada interfaz.

### 1.24 Unidades y conversión

Un número seguido de una unidad se convierte en una *cantidad*: el
valor en unidades del SI más sus dimensiones. La tabla de unidades
cubre las unidades base y derivadas del SI (`m`, `s`, `kg`, `A`, `K`,
`mol`, `cd`, `Hz`, `N`, `Pa`, `J`, `W`, `C`, `V`, `F`, `ohm`, `S`,
`Wb`, `T`, `H`, `lm`, `lx`, `Bq`, `Gy`, `Sv`), las unidades cotidianas
(`min`, `hr`, `d`, `yr`, `L`, `t`, `bar`, `atm`, `torr`, `psi`, `eV`,
`mile`, `yd`, `ft`, `inch`, `nmi`, `lb`, `oz`, `gal`, `qt`, `pt`,
`mph`, `knot`) y los sufijos de astronomía de la sección 1.16. Las
unidades compuestas encadenan: `60 mile/hr` y `5 m/s^2` son unidades
individuales.

```epher
60 mile/hr
```

```text
60 mile/hr
```

Los prefijos del SI escalan cualquiera de ellas: `k M G T m µ n p` son
kilo, mega, giga, tera, mili, micro, nano, pico — `5 km`, `3 MPa`,
`1 GHz` funcionan, y `2 kg` es el propio kilogramo.

Las dimensiones se comprueban: sumar o comparar cantidades con
unidades distintas da error en lugar de mezclar metros y segundos:

```epher
5 m + 3 s
```

```text
error: dimension error: cannot add 5 m and 3 s
```

La aritmética compone las dimensiones: `5 m * 3 m` es `15 m^2`,
`(3 m)^2` es `9 m^2`, `sqrt(4 m^2)` es `2 m`, y una expresión completa
cuyas dimensiones se cancelan vuelve a ser un número ordinario
(`5 m / 5 m` es `1`). Los resultados prefieren el nombre derivado
exacto cuando las dimensiones coinciden con uno — `5 kg * 3 m / 1 s^2`
responde `15 N`.

**Conversión.** `expr in unidad` (o `expr -> unidad`) muestra una
cantidad en la unidad nombrada; las dimensiones deben coincidir. `in`
liga con la menor prioridad de los operadores, así que `5 m + 3 m in
km` convierte la suma entera:

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

Las escalas de temperatura (Celsius, Fahrenheit) no son unidades aquí
— los kelvins sí, y `K` funciona como cualquier otra.


### 1.25 Operaciones bit a bit

Los literales de base de la sección 1.13 están hechos para esto:
`0b101`, `0o17`, `0xFF`. Los operadores bit a bit trabajan con números
enteros y responden con enteros exactos:

```epher
0xFF & 0x0F
```

```text
15
```

| Operador | Significado |
|---|---|
| `a & b` | y bit a bit |
| `a \| b` | o bit a bit |
| `a xor b` | o exclusivo bit a bit |
| `~a` | no bit a bit (complemento a dos) |
| `a << n` | desplazar a la izquierda (multiplicar por 2^n) |
| `a >> n` | desplazar a la derecha, aritmético (dividir por 2^n, redondeando hacia abajo) |

Los resultados son enteros `big` exactos, así que `1 << 60` conserva
cada dígito. El tamaño de palabra es de 64 bits por defecto: los
resultados se leen como complemento a dos con signo, así que `~0` es
-1 y `1 << 100` envuelve a 0. `bits(n)` cambia el tamaño de palabra a
8, 16, 32 o 64, y `bits()` lo informa:

```epher
bits(8)
~0
```

```text
8
-1
```

Un desplazamiento negativo invierte la dirección (`8 << -1` es `4`).
El `and` y `or` booleanos conservan sus significados; `&` y `|` son
las grafías bit a bit.


### 1.26 Relaciones implícitas

Una ecuación con dos incógnitas se dibuja como una curva: la familia de gráficos muestrea la relación con marching squares y traza su contorno cero. El círculo, la parábola y la recta vertical, cada uno con un solo comando:

```epher
graph x^2 + y^2 == 1
```

```epher
graph y == x^2
```

```epher
graph x == 2
```

La relación se muestrea sobre el cuadrado de `from a to b` (o la ventana por defecto), así que `graph x^2 + y^2 == 1 from -2 to 2` encaja la ventana del círculo. Todo lo que hace una curva aplica: la leyenda rotula la ecuación, los deslizadores animan sus constantes, y la imagen se amplía, desplaza y exporta como cualquier otra. Los rellenos de desigualdad (`y < …`, `y > …`) siguen siendo curvas sombreadas; una relación no tiene puntos de interés.


### 1.27 Matrices

Una matriz es una rejilla de números, escrita como filas de listas:
`[[1, 2], [3, 4]]` es la matriz 2×2. `+` y `-` son elemento a elemento
(con formas que coinciden), `*` es el producto matricial, un número
escala elemento a elemento, y `^` es la potencia matricial entera
(`A ^ 0` es la identidad, así que las potencias necesitan matrices
cuadradas). `M[2][1]` es el elemento de la fila 2, columna 1 — las
filas se indexan como listas, desde 1.

```epher
[[1, 2], [3, 4]] * [[5, 6], [7, 8]]
```

```text
[[19, 22], [43, 50]]
```

Las funciones matriciales cubren el mínimo del aula: `det(M)` (solo
cuadradas), `inv(M)` (las singulares dan error), `transpose(M)`,
`trace(M)` (cuadradas), `dim(M)` (la lista `{filas, columnas}`), y
`ref(M)` con `rref(M)` para la reducción por filas. Los sistemas
lineales se resuelven con rref sobre la matriz aumentada:

```epher
rref([[2, 1, 5], [1, -1, 1]])
```

```text
[[1, 0, 2], [0, 1, 1]]
```

Las filas leen `x = 2`, `y = 1` — la última columna de la matriz
aumentada reducida. Las fracciones exactas se muestran dentro de las
matrices como en las listas, así que `inv([[1, 2], [3, 4]])` muestra
`[[-2, 1], [3/2, -1/2]]`.


### 1.28 Finanzas

El solucionador de valor del dinero en el tiempo (convención de signos
TI: el dinero que sale es negativo, el que entra positivo) resuelve
cualquiera de los cinco campos dados los otros cuatro. `i` es la tasa
por periodo como fracción — 0.01 es el 1% — y el último argumento
opcional es el momento del pago: 0 para fin de periodo (el
predeterminado), 1 para inicio (anualidad anticipada).

```epher
tvm_pmt(360, 0.08/12, -100000, 0)
```

```text
733.764573879
```

La hipoteca clásica del 8%: 360 pagos mensuales de 733.76 contra un
préstamo de 100,000 — `tvm_pmt` es el pago, `tvm_pv` el préstamo,
`tvm_fv` el saldo, `tvm_n` el plazo y `tvm_i` la tasa:

```epher
tvm_i(360, -100000, 733.76, 0)
```

```text
0.00666661199068
```

La tasa aquí es algo inferior al 8%/12 porque 733.76 está redondeado.
`npv(r, flows)` descuenta una lista de flujos y `irr(flows)` halla la
tasa donde el valor actual neto es cero:

```epher
npv(0.1, {-100, 60, 60})
```

```text
500/121
```

`amort(p, r, n, k)` es el saldo restante tras k pagos de un préstamo a
n periodos, `simple_interest(p, r, t)` es `p*r*t`, y
`compound_interest(p, r, n)` es `p*(1+r)^n - p`.


## 2. La aplicación web (PWA)

### 2.1 Cómo abrirla

La aplicación web está en:

```text
https://epher.org/pwa/
```

No necesita instalación. Funciona en cualquier navegador moderno, en
ordenador, móvil o tableta.

Esta guía también está integrada en la aplicación: abre **Help → User
guide** en la barra de menús (toca **☰** en el móvil) para leerla dentro de
la app, en el idioma que tengas activo. Toca cualquier ejemplo de esa guía
para cargarlo en el campo de entrada. **Ayuda → Constantes** abre el explorador de constantes: todas las constantes agrupadas (Matemáticas, Astronomía, Física, Química), cada una con su valor y una breve descripción; toca una para insertar su nombre en el campo de entrada, y la caja de búsqueda filtra la lista.

### 2.2 Tu primer cálculo

1. Haz clic en el campo de texto (ya está enfocado cuando la página carga).
2. Escribe una expresión, por ejemplo `2 + 3 * 4`.
3. Pulsa **Intro** o haz clic en el botón **=**.

El resultado aparece en texto grande debajo del campo. Todo lo del capítulo
1 funciona aquí, incluidas variables, funciones y scripts.

Mientras escribes un nombre, aparece una lista de sugerencias debajo del campo: las flechas mueven el resaltado, **Intro** o **Tab** acepta, **Esc** cierra, y un clic acepta sin salir del teclado. Cada sugerencia lleva una descripción breve de la función o constante. **F1** muestra la misma descripción de la palabra bajo el cursor en la barra de pistas sobre el teclado. Si lo primero que escribes en un campo vacío es un operador (`+ - * / ^ % !`), epher inserta `ans` por ti, y la línea continúa desde la respuesta anterior.

El menú **Ajustes** (el icono de engranaje, o **☰ → Ajustes** en el teléfono) tiene tres grupos. **Tema** y **Idioma** hacen lo que dicen sus nombres. **Resultados** define cómo se muestran las respuestas: fracciones exactas (activadas por defecto, así `1 / 3` se muestra como `1/3`), la notación (Auto, científica o de ingeniería) y los separadores de miles. Son solo ajustes de visualización; los valores siguen siendo números normales.

### 2.3 Historial

Cada cálculo se añade a la lista de historial debajo del resultado, para que
puedas desplazarte hacia atrás y ver lo que hiciste. Las entradas más
recientes aparecen arriba, y el icono de la papelera junto al título
**Historial** la vacía (en el terminal, Ctrl+L o una pulsación en el
mismo icono). El historial se conserva mientras la página está
abierta.

Cada entrada queda entre reglas finas: una expresión de una línea es una fila, y un script de varias líneas es una entrada que muestra todas sus líneas. Haz clic en una entrada para volver a cargarla en el campo de entrada y ejecutarla de nuevo.

### 2.4 Gráficas

Escribe `graph` seguido de una expresión y pulsa **Intro**:

```epher
graph x ^ 2
```

epher dibuja la curva y = f(x) desde x = −10 hasta x = 10 debajo del
campo de entrada, sobre una cuadrícula con ejes etiquetados. Puedes
graficar cualquier expresión, incluidas tus propias funciones:

```epher
def f(x) = x ^ 3
graph f(x)
```

Cada línea `graph` añade otra curva al mismo gráfico, cada una con su
propio color. Las curvas son todas sólidas, y la leyenda y las
etiquetas son las que las distinguen sin color.
`graph clear` vacía el gráfico, y un botón **Clear graph** en la parte
superior del panel de la gráfica hace lo mismo para curvas y superficies 3D
a la vez. La TUI mantiene el comando en su menú **Graph**.

Arriba del panel de gráficas, junto a **Clear graph** y **Copy SVG**,
la barra de herramientas permite ocultar la lista de puntos de interés y
los puntos destacados dibujados en la propia gráfica. Justo encima de
cada gráfica hay una franja de deslizadores con icono, las palabras en
su tooltip: grosor de línea (0 a 4 en pasos de 0.1 para curvas 2D, 0 a
0.2 en pasos de 0.01 para superficies 3D - solo se muestra el del tipo en
pantalla, y cada tipo recuerda su propio valor), y en 3D y el sistema
solar, la velocidad de giro horizontal y vertical y la velocidad de zoom.
Cada entrada de la leyenda tiene una casilla, marcada por defecto:
desmarcarla oculta la curva de la gráfica, de sus puntos de interés y de
la exportación SVG.

```epher
graph x ^ 2
graph x ^ 3
```

Los puntos donde la expresión no tiene valor (una división por cero, por
ejemplo) se omiten, dejando un hueco en la curva. Un salto que en
realidad es una asíntota vertical nunca se dibuja como línea conectora.

#### 2.4.1 Qué puedes representar

Un dominio a tu elección:

```epher
graph sin(x) from 0 to 2*pi
```

Curvas paramétricas (t va de 0 a 2π):

```epher
graph param 2*cos(t), 3*sin(t)
```

Curvas polares:

```epher
graph polar 1 + cos(theta)
```

Regiones: `y <` sombrea el área bajo la curva, `y >` sombrea la de encima:

```epher
graph y < x ^ 2
```
#### 2.4.2 Leer la gráfica

**Seguimiento:** mueve el puntero sobre la gráfica, o enfócala y pulsa
las teclas de flecha. Se marca el punto más cercano de una curva, con
sus coordenadas mostradas debajo de la gráfica.

**Puntos de interés:** tras cada comando graph, epher encuentra las
raíces y los máximos y mínimos de cada curva y las intersecciones entre
curvas, los marca en la gráfica y los lista debajo de ella:

```text
root (-1, 0)   minimum (0, 0)   root (1, 0)
```

**Tablas:** el comando `table` imprime una tabla de valores (las filas
donde la expresión no tiene valor quedan en blanco):

Una cláusula opcional `derivative <expresión>` añade una tercera
columna, la derivada numérica de esa expresión en cada x:

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

Las celdas de la tabla siguen los ajustes de resultados: con las
fracciones exactas activadas (por defecto), un valor que es una
fracción simple se muestra como tal — `table x / 3 from 0 to 1
points 4` lista `1/3` en lugar de `0.333`.
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

#### 2.4.3 Deslizadores y exportación

Define una constante, úsala en una gráfica y aparece un deslizador debajo
de la gráfica. Arrástralo (o muévelo con las teclas de flecha) y cada
curva se redibuja:

```epher
const a = 1
graph a * x ^ 2
```

**Copiar SVG** copia la gráfica actual como imagen SVG autónoma para
pegarla en documentos: los colores van incluidos, así que se ve igual en
todas partes. **Guardar PNG** guarda la misma imagen como mapa de bits al doble de su tamaño, así las curvas quedan nítidas; la aplicación de escritorio pregunta dónde guardarla y la del navegador la guarda en tus descargas (o pregunta, donde el navegador lo ofrece). Las filas de deslizadores y las constantes animadas están
directamente debajo de la gráfica, sobre la lista de puntos de interés.

#### 2.4.4 Superficies 3D

`graph3d` dibuja una superficie z = f(x, y) sobre un dominio cuadrado
(de −5 a 5, o tu `from a to b`):

```epher
graph3d x ^ 2 - y ^ 2
```

Las líneas de malla más cercanas a ti se dibujan más marcadas, para que
la forma se lea en profundidad. Varias líneas `graph3d` se superponen,
como las curvas, y `graph3d clear` vacía la gráfica. Gira la vista
arrastrando, o enfoca la gráfica y usa las teclas de flecha. La interfaz
de terminal dibuja la misma superficie como una malla alámbrica ASCII,
girándola con las teclas de flecha.

#### 2.4.5 Animación

Cada deslizador tiene un botón de reproducción. Avanza su constante a lo
largo del rango del deslizador y al llegar al final vuelve a empezar. Es la
forma estándar en que animan las calculadoras: animas un parámetro y todo
lo que lo usa se mueve. Pulsa el
botón de nuevo para pausar.

Una variable de "tiempo" no es más que una constante que animas:

```epher
const t = 0
graph sin(x - t)
```

Al reproducir el deslizador de t, la onda se desplaza. Las superficies 3D
se animan igual. Define una constante primero y luego reproduce su
deslizador:

```epher
const a = 1
graph3d sin(a * (x ^ 2 + y ^ 2)) from -3 to 3
```

En la interfaz de terminal, la barra espaciadora
inicia y detiene la animación.

### 2.5 Instalarla y usarla sin conexión

La aplicación web es una *progressive web app*: después de una visita
funciona completamente sin conexión y puedes instalarla como una app normal.

- **Chrome, Edge o Android:** haz clic en el icono de instalar de la barra
  de direcciones (o *Instalar aplicación* en el menú del navegador) y
  confirma.
- **iPhone / iPad (Safari):** toca **Compartir** → **Añadir a pantalla de
  inicio**.
- **Otros navegadores:** busca *Instalar* o *Añadir a pantalla de inicio*
  en el menú.

Una vez instalada, ábrela desde tu pantalla de inicio o lista de apps. Se
abre al instante, incluso sin conexión a internet.

### 2.6 Lo que la aplicación web no hace

La aplicación web conserva tu trabajo en la sesión actual: evalúa
expresiones, las grafica (sección 2.4) y mantiene un historial. Los
comandos **save**, **save script** y **language** funcionan en las
versiones de escritorio, línea de comandos y terminal (capítulos 3, 4 y
5). En la aplicación web responden con una nota de que guardar funciona
allí. El historial no se guarda entre visitas.

## 3. La aplicación de escritorio

La aplicación de escritorio es una ventana normal alrededor de la misma
aplicación web. Todo lo del capítulo 2 se aplica; la diferencia está solo en
cómo la instalas y la abres.

### 3.1 Instalación

Descarga un instalador para tu sistema desde el sitio web de epher:

- **Windows:** ejecuta `epher-windows-x86_64.exe`. El instalador pone `epher`
  en tu PATH. Abre una ventana nueva de CMD o PowerShell y `epher "2 + 2"`
  funciona. Como la compilación no está firmada, elige *Más información* →
  *Ejecutar de todas formas* en el primer arranque.
- **macOS:** abre `epher-macos-aarch64.dmg` y arrastra epher a Aplicaciones.
  Como la compilación no está firmada, el primer arranque necesita clic
  derecho → **Abrir**.
- **Linux (Debian/Ubuntu):** el paquete `.deb`

```sh
sudo apt install ./epher-linux-x86_64.deb
```

- **Linux (Fedora/RHEL):** el paquete `.rpm`

```sh
sudo dnf install ./epher-linux-x86_64.rpm
```

- **Linux (cualquier distro, incluida Arch):** el AppImage. Hazlo ejecutable
  y ejecútalo:

```sh
chmod +x epher-linux-x86_64.AppImage
./epher-linux-x86_64.AppImage
```

Cada instalador contiene *todo* epher: la aplicación de escritorio, la línea
de comandos (capítulo 4) y la interfaz de terminal (capítulo 5), como el
único comando `epher`. En Linux, el paquete instala `epher` en `/usr/bin`.

### 3.2 Uso

Inicia epher como cualquier otra aplicación. Obtienes una ventana con la misma
interfaz que la aplicación web: escribe una expresión, pulsa **Intro** o
haz clic en **=**, y lee el resultado. Las gráficas también funcionan aquí.
`graph x ^ 2` dibuja en la ventana (capítulo 2.4). La ventana se puede
redimensionar libremente. La barra de menús incluye **Help → User guide**, la
misma guía que esta página, con ejemplos que se cargan al tocarlos.

También puedes abrirla desde una terminal: un `epher` sin argumentos (o
`epher gui`) inicia la aplicación de escritorio. En macOS, usa el botón
**Install the epher command** dentro de la app para poner `epher` en el PATH
de tu terminal.

### 3.3 Almacenamiento: un mismo almacén con la CLI y la TUI

La aplicación de escritorio comparte su almacenamiento con las versiones
de línea de comandos y terminal. Funciones, constantes, scripts, historial y la
preferencia de idioma viven en un solo lugar, `~/.epher` en tu equipo (o
`EPHER_STORE_DIR`, capítulo 4.6), y todo lo guardado en una versión está
disponible en las demás:

```text
def area(w, h) = w * h
save area
```

Define `area` en la aplicación de escritorio, `save` la, cierra la
ventana. Luego abre la CLI y `area(3, 4)` simplemente funciona. También
funciona al revés: las funciones y scripts guardados en la CLI o la TUI ya
están ahí cuando se abre la ventana de escritorio, incluidas las variables
definidas por scripts guardados. Los comandos `save`, `save script` y
`language` del capítulo 4 funcionan exactamente igual aquí.

Los comandos que escribes en la CLI, el REPL, la TUI o la aplicación
de escritorio se guardan todos en el mismo historial, y la sesión
también viaja: las variables que asignas y el valor `ans` te siguen
de una versión a la siguiente. El almacenamiento compartido es en vivo:
con dos versiones abiertas a la vez, un cambio en una se refleja al
instante en la otra (la aplicación de escritorio y la TUI observan el
almacenamiento y se refrescan solas).

> La aplicación web en el navegador es la única versión que no usa este
> almacenamiento: cada sesión vive aparte (capítulo 2.6).

## 4. La línea de comandos (CLI)

La CLI es el lado de texto del mismo programa `epher` que la aplicación de
escritorio. Tiene tres modos: evaluación de un solo uso, scripts por tubería
y una sesión interactiva para trabajos más largos.

Para obtener ayuda en cualquier momento, ejecuta `epher --help` (todos
los comandos, con ejemplos) o `epher help` (el manual completo; en los
paquetes de Linux es la página `man epher`).

### 4.1 Cálculos de un solo uso

Pasa la expresión como argumento:

```sh
epher "2 + 3 * 4"
```

```text
14
```

Puedes hacer cualquier cosa del capítulo 1 que sea una sola expresión:

```sh
epher "if 3 > 2 then 10 else 20"
```

```text
10
```

Una expresión que empieza con un signo menos funciona directamente:

```sh
epher "-2 + 5"
```

```text
3
```

El modo de un solo uso es para scripts, desde una sola expresión hasta un
programa completo. El valor de cada instrucción se imprime en su propia
línea:

```sh
epher "x = 10; x + 5"
```

```text
10
15
```

Las instrucciones unidas con saltos de línea funcionan igual dentro del
argumento. Todo lo del capítulo 1 está disponible: variables, funciones,
bucles, todo. Las líneas comparten una sesión, como un script por
tubería (sección 4.2).

### 4.2 Scripts por tubería

`epher -` lee expresiones de la entrada estándar, línea a línea, como se
usan los lenguajes de script en las tuberías:

```sh
printf "x = 3\nx * 10\n" | epher -
```

```text
= 3
= 30
```

Todo lo del capítulo 1 funciona, y las líneas comparten una sesión: una
función definida en una línea temprana está disponible después, y `save`
escribe en el mismo almacén de siempre. Los errores se imprimen y el script
sigue. Una línea puede unir varias instrucciones con `;`. Los saltos de
línea y `;` significan lo mismo en todas partes de epher.


Un archivo funciona igual: `epher plots/sine.es` ejecuta cada línea del archivo en orden y muestra cada resultado. El argumento se trata como archivo cuando nombra un archivo existente y contiene un `.`, `/` o `\`, así que `epher x` sigue evaluando el nombre `x`.
### 4.3 La sesión interactiva (REPL)

Iníciala con `epher repl`:

```sh
epher repl
```

> Un `epher` sin argumentos abre la aplicación de escritorio (capítulo 3).

epher muestra su prompt y espera:

```text
epher>
```

Ahora escribe cualquier cosa del capítulo 1, una línea cada vez. Las
variables conservan sus valores entre líneas:

```text
epher> x = 5
= 5
epher> x ^ 2
= 25
```

El comando `table` (sección 2.4.2) también imprime aquí una tabla de
valores:

```text
epher> table x ^ 2 from -2 to 2 points 5
         x           y
        -2           4
        -1           1
         0           0
    Las líneas `graph` también funcionan aquí: las curvas se acumulan entre
líneas, y `graph save plot.svg` escribe la misma imagen SVG que produce
el botón **Copiar SVG** de la aplicación web. `graph3d
save archivo.svg` guarda una superficie 3D igualmente. Las mismas líneas
valen en la evaluación única y en scripts entubados:
`epher "graph sin(x); graph save plot.svg"` es una gráfica completa en
un solo comando.

     1           1
         2           4
```

Cada respuesta se muestra como `= resultado`. Para salir, escribe `quit` (o
`exit`):

```text
epher> quit
```

Tu historial se recuerda: la próxima vez que ejecutes `epher repl`, las
líneas de la sesión anterior siguen ahí.


La orden `load` ejecuta un script (una ruta de archivo o el nombre de un script guardado con `save script`) línea a línea, exactamente como si lo hubieras escrito tú:

```text
epher> load plots/sine.es
epher> load my_setup
```
### 4.4 Guardar funciones, constantes y scripts

Define una función y luego guárdala:

```text
epher> def fib(n) = if n <= 1 then n else fib(n - 1) + fib(n - 2)
epher> save fib
saved fib
```

El comando `save fib` guarda la función en el disco. La próxima vez que
inicies la sesión, `fib` ya está definida:

```text
epher> fib(10)
= 55
```

Las constantes se guardan igual: `save` con el nombre de la constante:

```text
epher> const tax = 0.2
= 0.2
epher> save tax
saved tax
```

Para guardar un script completo (la última línea que escribiste) usa
`save script`:

```text
epher> x = 0; while x < 5 do x = x + 1; x
= 5
epher> save script count_to_five
saved script count_to_five
```

Los scripts guardados se ejecutan automáticamente cuando epher arranca, así
que todo lo que definen está listo para ti.


También puedes cargar un script guardado cuando quieras con `load count_to_five`, o conservarlo como archivo plano y ejecutar `load count_to_five.es`; `epher count_to_five.es` lo ejecuta directamente desde la línea de órdenes (sección 4.2).
### 4.5 Cambiar el idioma de la interfaz

El idioma de la interfaz se elige entre los idiomas configurados en tu
dispositivo. Para cambiarlo, escribe `language` seguido de uno de: `en`,
`zh-CN`, `hi`, `es`, `fr`, `ar`, `de`, `pt`:

```text
epher> language fr
language set to fr
```

La elección se recuerda para la próxima vez. Nota: el idioma que *escribes*
, el lenguaje de las expresiones, es siempre el mismo, en cualquier idioma
de la interfaz.

### 4.6 Dónde viven tus datos

Las funciones, scripts, historial y tu elección de idioma se guardan en una
carpeta de tu ordenador:

```text
~/.epher
```

Borra esa carpeta para empezar completamente de cero. Para usar otra
ubicación, define la variable de entorno `EPHER_STORE_DIR` antes de iniciar
epher:

```sh
EPHER_STORE_DIR=/tmp/my-epher epher repl
```

## 5. La interfaz de terminal (TUI)

La TUI es una versión a pantalla completa de la sesión interactiva, dentro
de tu terminal. Forma parte del mismo programa `epher`. Iníciala con:

```sh
epher tui
```

### 5.1 La pantalla

La pantalla está dividida en paneles:

- **Expresión**: el campo de entrada (arriba). Shift+Enter empieza una
  nueva línea; las teclas de flecha o un clic del ratón mueven el cursor
  dentro del texto.
- El **resultado** actual justo debajo.
- **Historial**: cada línea que escribiste, con su respuesta.
- **Gráfica**: la gráfica del comando `graph` (abajo).
- Una línea de pistas muestra los atajos de teclado.

### 5.2 Teclas

| Tecla | Acción |
|---|---|
| Escribir | añadir a la expresión en el cursor |
| **Intro** | evaluar todo el script (una entrada multilínea se ejecuta como un elemento del historial) |
| **Shift+Enter** | empezar una nueva línea |
| **← → ↑ ↓** | mover el cursor (con la entrada vacía: girar la vista 3D) |
| **Esc** | borrar la línea de entrada |
| **F1** | describir la función bajo el cursor (en la línea de resultado) |
| **Ctrl+C** | salir |
| **q** | salir (cuando la entrada está vacía) |
| **Teclas de flecha** | girar la vista 3D (cuando la entrada está vacía) |
| **Espacio** | iniciar/detener la animación (cuando la entrada está vacía) |
| **F10** | abrir los menús (File, Edit, Graph, Settings, Help) |
| **Tab** | enfocar el teclado siempre visible (o el historial, desde el teclado); cambiar de grupo (**Esc** vuelve a escribir) |
| **Ratón** | pulse menús y elementos de menú, celdas y pestañas del teclado, líneas del historial (carga la expresión); arrastre el panel de gráficas para orbitar (3D) o desplazar (2D), la rueda hace zoom y un doble clic restablece la vista |
| **Ctrl+L** | borrar el historial |

El menú **Ayuda** abre la guía integrada, la ayuda de teclas del teclado y un explorador de constantes: las constantes agrupadas, las flechas eligen una fila, **Intro** inserta su nombre en la expresión en el cursor y **Esc** cierra.

Los grupos del teclado contienen todas las funciones, constantes y
comandos que admite el lenguaje: **trig**, **fn**, **num**, **0x**
y **var**. El grupo 0x contiene las conversiones exactas y de base
(`frac`, `dec`, `big`, `bin`, `oct`, `hex`) y el factorial `!`.
Las flechas mueven el resaltado, **Intro** inserta el token y **Tab**
cambia de grupo. Un operador al principio de una línea vacía (o insertado desde el teclado) añade `ans` antes, así la línea continúa desde la respuesta anterior.

El menú **Ajustes** ofrece las mismas opciones de visualización de resultados que la aplicación web (fracciones exactas, notación, separadores de miles), junto a las filas de tema e idioma.

### 5.3 Gráficas

Escribe `graph` seguido de una expresión y pulsa **Intro**:

```epher
graph x ^ 2
```

epher muestrea la curva de x = −10 a x = 10 y la dibuja como una gráfica
ASCII en el panel Graph; la leyenda sobre la gráfica nombra lo que se
representa.

`graph clear` vacía la gráfica, y el menú **Graph** hace lo mismo; el menú
**Help** abre esta guía dentro de la TUI (las teclas de flecha desplazan,
**Esc** cierra). El menú **Settings** puede ocultar los puntos de interés
que se listan bajo la gráfica.

Puedes graficar cualquier expresión, incluidas tus propias funciones.
Primero define una y luego grafícala:

```epher
def f(x) = x ^ 3
graph f(x)
```

Cada línea `graph` añade una curva a la gráfica, dibujada con su propio
símbolo (`o`, `x`, `+`, `*`); `graph clear` vacía la gráfica. Se aplica
la misma gramática que en la aplicación web: un dominio
(`graph sin(x) from 0 to 2*pi`), curvas paramétricas
(`graph param 2*cos(t), 3*sin(t)`), curvas polares
(`graph polar 1 + cos(theta)`) y regiones (`graph y < x ^ 2` sombrea el
área bajo la curva).

Los puntos donde la expresión no tiene valor (por ejemplo división entre
cero) simplemente se omiten, dejando un hueco en la gráfica. Tras cada
comando graph, la TUI lista los puntos de interés (raíces, máximos y
mínimos e intersecciones) bajo la gráfica. El comando `table`
(sección 2.4.2) también funciona aquí.

`graph3d x ^ 2 - y ^ 2` dibuja una superficie 3D como una malla alámbrica
ASCII. Gírala con las teclas de flecha mientras la entrada esté vacía, y
pulsa la barra espaciadora para animar una constante con deslizador
(sección 2.4.5). La línea de ayuda inferior muestra los avisos de flechas
y espacio solo mientras haya una superficie 3D o una curva animable.

`graph save plot.svg` escribe la gráfica actual como la misma imagen SVG
que produce el botón **Copiar SVG** de la aplicación web; `graph3d
save archivo.svg` guarda la malla 3D desde el ángulo en que la estás
viendo.

### 5.4 Guardar y persistencia

La TUI comparte su almacenamiento con la CLI: todo lo guardado en una está
disponible en la otra. Las funciones, scripts, historial y preferencia de
idioma viven en `~/.epher` (capítulo 4.6), y los mismos comandos `save`,
`save script` y `language` funcionan aquí.

## 6. Tus datos y privacidad

- El **programa epher instalado** (aplicación de escritorio, CLI y TUI)
  guarda funciones, scripts, historial y la elección de idioma localmente en
  `~/.epher` (o `EPHER_STORE_DIR`). Nada sale de tu equipo.
- La **aplicación web** no guarda nada en disco: el historial dura solo
  mientras la página está abierta. La aplicación web puede funcionar sin
  conexión porque la propia página la guarda tu navegador.

Las cinco versiones ejecutan el cálculo íntegramente en tu dispositivo.
Nada se envía a ningún sitio.
