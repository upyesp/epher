# Guía de usuario de epher

¡Bienvenido! epher es una calculadora programable y con scripts. Puedes usarla
para un cálculo rápido o para construir tus propias funciones y pequeños
programas, y todo está disponible en ocho idiomas.

Esta guía es para principiantes absolutos. Empieza con el cálculo más simple
posible y llega hasta todo el poder del lenguaje. Cada ejemplo muestra lo que
escribes y lo que epher responde.

Hay cuatro formas de usar epher — elige la que más te convenga:

| Versión | Qué es | Cuándo conviene |
|---|---|---|
| **Aplicación web** (PWA) | Se ejecuta en tu navegador, se puede instalar y funciona sin conexión | Quieres empezar rápido; sin instalación |
| **Aplicación de escritorio** | Un programa normal con su propia ventana | Quieres una aplicación de escritorio |
| **Línea de comandos** (CLI) | Comandos de texto en una terminal; también una sesión interactiva | Vives en la terminal y te gustan los scripts |
| **Interfaz de terminal** (TUI) | Un programa a pantalla completa dentro de la terminal | Quieres una app de terminal con gráficos e historial |

La aplicación de escritorio, la línea de comandos y la interfaz de terminal
son un solo programa: una única descarga instala el comando `epher`, que hace
las tres cosas. La aplicación web es la excepción — no necesita descarga.

Las cuatro versiones entienden exactamente el mismo lenguaje. Apréndelo una
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

1. `!` factorial
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

Las potencias pueden ser fraccionarias — `2 ^ 0.5` es la raíz cuadrada de 2:

```epher
2 ^ 0.5
```

```text
1.4142135623730951
```

La resta y la división funcionan de izquierda a derecha:

```epher
10 - 3 - 2
```

```text
5
```

### 1.3 Los números especiales pi, e, tau y phi

Las constantes famosas están integradas:

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

Dos más: `tau` es una vuelta completa (2 pi) y `phi` es el número áureo:

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

Puedes cambiar una variable cuando quieras — conserva su valor hasta que la
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
como la tecla `Ans` de una calculadora de bolsillo — útil para encadenar
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

Una *constante* es un nombre cuyo valor nunca cambia — como el `pi`
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

y definir la misma constante dos veces también:

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
784.8000000000001
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
2
```

> epher no tiene valores de texto — ambas ramas de un `if` deben ser números
> (o resultados de comparaciones).

### 1.8 Bucles con while

`while` repite una instrucción mientras se cumpla una condición:

```epher
x = 0; while x < 5 do x = x + 1; x
```

```text
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

El ejemplo más famoso — los números de Fibonacci:

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

> El cuerpo de una función es una sola expresión — una línea. Combina
> varios cálculos con `;` en un script en su lugar (sección siguiente).

### 1.11 Scripts: varias instrucciones a la vez

Un *script* es varias instrucciones unidas con `;` — o con saltos de
línea, que significan exactamente lo mismo — ejecutadas una tras otra:

```epher
x = 10; y = x + 5; x + y
```

```text
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
instrucciones con `;` en una línea funciona en todas partes también —
incluida la línea de comandos de un solo uso (sección 4.1).

### 1.12 Resultados exactos: frac, dec y big

Normalmente epher calcula con números decimales como una calculadora de
bolsillo. Algunos números se ven mejor exactos.

**frac(n, d)** crea una fracción exacta:

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

Las fracciones se mantienen exactas a través de los cálculos:

```epher
frac(1, 3) * 3
```

```text
1
```

**dec(x)** crea un decimal exacto. Compara estos dos:

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

El primer resultado es el pequeño error de redondeo que toda computadora
comete con los decimales. `dec()` lo elimina.

**big(x)** crea un número entero exacto, para valores demasiado grandes para
una calculadora de bolsillo:

```epher
big(10 ^ 20)
```

```text
100000000000000000000
```

### 1.13 Funciones integradas

epher tiene las funciones de una calculadora científica, agrupadas por familia.

La trigonometría trabaja en radianes — usa `deg` y `rad` para convertir:

| Función | Significado | Ejemplo | Resultado |
|---|---|---|---|
| `sin(x)`, `cos(x)`, `tan(x)` | funciones trigonométricas | `sin(pi / 2)` | `1` |
| `asin(x)`, `acos(x)`, `atan(x)` | trigonometría inversa | `atan(1)` | `0.7853981633974483` |
| `atan2(y, x)` | ángulo del punto (x, y) | `atan2(1, 1)` | `0.7853981633974483` |
| `deg(x)` | radianes → grados | `deg(pi)` | `180` |
| `rad(x)` | grados → radianes | `rad(180)` | `3.141592653589793` |
| `sinh(x)`, `cosh(x)`, `tanh(x)` | funciones hiperbólicas | `sinh(1)` | `1.1752011936438014` |
| `asinh(x)`, `acosh(x)`, `atanh(x)` | hiperbólicas inversas | `acosh(1)` | `0` |

Potencias, raíces y logaritmos (en una calculadora `log` es base 10):

| Función | Significado | Ejemplo | Resultado |
|---|---|---|---|
| `sqrt(x)` | raíz cuadrada | `sqrt(16)` | `4` |
| `cbrt(x)` | raíz cúbica | `cbrt(-27)` | `-3` |
| `root(n, x)` | raíz n-ésima | `root(3, 8)` | `2` |
| `exp(x)` | e elevado a x | `exp(1)` | `2.718281828459045` |
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

Se combinan como todo lo demás:

```epher
min(sqrt(16), 5)
```

```text
4
```

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

El último ejemplo es importante: epher te dice exactamente qué nombre no
conoce, para que puedas arreglar tu expresión.

### 1.15 Referencia rápida

| Qué | Sintaxis | Ejemplo |
|---|---|---|
| Sumar, restar, multiplicar, dividir | `+ - * /` | `7 / 2` |
| Potencia | `^` (de derecha a izquierda) | `2 ^ 10` |
| Factorial | `!` (postfijo) | `5!` |
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

## 2. La aplicación web (PWA)

### 2.1 Cómo abrirla

La aplicación web está en:

```text
https://epher.org/pwa/
```

No necesita instalación — funciona en cualquier navegador moderno, en
ordenador, móvil o tableta.

Esta guía también está integrada en la aplicación: abre **Help → User
guide** en la barra de menús (toca **☰** en el móvil) para leerla dentro de
la app, en el idioma que tengas activo. Toca cualquier ejemplo de esa guía
para cargarlo en el campo de entrada.

### 2.2 Tu primer cálculo

1. Haz clic en el campo de texto (ya está enfocado cuando la página carga).
2. Escribe una expresión, por ejemplo `2 + 3 * 4`.
3. Pulsa **Intro** o haz clic en el botón **=**.

El resultado aparece en texto grande debajo del campo. Todo lo del capítulo
1 funciona aquí, incluidas variables, funciones y scripts.

### 2.3 Historial

Cada cálculo se añade a la lista de historial debajo del resultado, para que
puedas desplazarte hacia atrás y ver lo que hiciste. Las entradas más
recientes aparecen arriba, y el botón **Clear history** encima de la lista
la vacía. El historial se conserva mientras la página está abierta.

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
propio color y patrón de trazos, y una leyenda que las nombra.
`graph clear` vacía el gráfico — y un botón **Clear graph** en la parte
superior del panel de la gráfica hace lo mismo para curvas y superficies 3D
a la vez. La TUI mantiene el comando en su menú **Graph**.

Al final del panel de gráficas, la fila de opciones permite ocultar la
lista de puntos de interés, ocultar los puntos destacados dibujados en
la propia gráfica y ajustar el grosor de las líneas con el deslizador
**Grosor de línea**.

```epher
graph x ^ 2
graph x ^ 3
```

Los puntos donde la expresión no tiene valor (una división por cero, por
ejemplo) se omiten, dejando un hueco en la curva — y un salto que en
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

**Seguimiento:** mueve el puntero sobre la gráfica — o enfócala y pulsa
las teclas de flecha — y se marca el punto más cercano de una curva, con
sus coordenadas mostradas debajo de la gráfica.

**Puntos de interés:** tras cada comando graph, epher encuentra las
raíces y los máximos y mínimos de cada curva y las intersecciones entre
curvas, los marca en la gráfica y los lista debajo de ella:

```text
root (-1, 0)   minimum (0, 0)   root (1, 0)
```

**Tablas:** el comando `table` imprime una tabla de valores (las filas
donde la expresión no tiene valor quedan en blanco):

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
de la gráfica — arrástralo (o muévelo con las teclas de flecha) y cada
curva se redibuja:

```epher
const a = 1
graph a * x ^ 2
```

**Copiar SVG** copia la gráfica actual como imagen SVG autónoma para
pegarla en documentos: los colores van incluidos, así que se ve igual en
todas partes. El deslizador **Grosor de línea**, al final del panel,
ajusta el grosor de cada línea dibujada.

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
largo del rango del deslizador y al llegar al final vuelve a empezar — la
forma estándar en que animan las calculadoras: animas un parámetro y todo
lo que lo usa se mueve. Pulsa el
botón de nuevo para pausar.

Una variable de "tiempo" no es más que una constante que animas:

```epher
const t = 0
graph sin(x - t)
```

Al reproducir el deslizador de t, la onda se desplaza. Las superficies 3D
se animan igual — define una constante primero y luego reproduce su
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

Una vez instalada, ábrela desde tu pantalla de inicio o lista de apps — se
abre al instante, incluso sin conexión a internet.

### 2.6 Lo que la aplicación web no hace

La aplicación web conserva tu trabajo en la sesión actual: evalúa
expresiones, las grafica (sección 2.4) y mantiene un historial. Los
comandos **save**, **save script** y **language** funcionan en las
versiones de escritorio, línea de comandos y terminal (capítulos 3, 4 y
5) — en la aplicación web responden con una nota de que guardar funciona
allí. El historial no se guarda entre visitas.

## 3. La aplicación de escritorio

La aplicación de escritorio es una ventana normal alrededor de la misma
aplicación web. Todo lo del capítulo 2 se aplica; la diferencia está solo en
cómo la instalas y la abres.

### 3.1 Instalación

Descarga un instalador para tu sistema desde el sitio web de epher:

- **Windows:** ejecuta `epher-windows-x86_64.exe`. El instalador pone `epher`
  en tu PATH — abre una ventana nueva de CMD o PowerShell y `epher "2 + 2"`
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

- **Linux (cualquier distro, incluida Arch):** el AppImage — hazlo ejecutable
  y ejecútalo:

```sh
chmod +x epher-linux-x86_64.AppImage
./epher-linux-x86_64.AppImage
```

Cada instalador contiene *todo* epher — la aplicación de escritorio, la línea
de comandos (capítulo 4) y la interfaz de terminal (capítulo 5) — como el
único comando `epher`. En Linux, el paquete instala `epher` en `/usr/bin`.

### 3.2 Uso

Inicia epher como cualquier otra aplicación. Obtienes una ventana con la misma
interfaz que la aplicación web: escribe una expresión, pulsa **Intro** o
haz clic en **=**, y lee el resultado. Las gráficas también funcionan aquí
— `graph x ^ 2` dibuja en la ventana (capítulo 2.4). La ventana se puede
redimensionar libremente. La barra de menús incluye **Help → User guide** —
la misma guía que esta página, con ejemplos que se cargan al tocarlos.

También puedes abrirla desde una terminal: un `epher` sin argumentos (o
`epher gui`) inicia la aplicación de escritorio. En macOS, usa el botón
**Install the epher command** dentro de la app para poner `epher` en el PATH
de tu terminal.

### 3.3 Almacenamiento: un mismo almacén con la CLI y la TUI

La aplicación de escritorio comparte su almacenamiento con las versiones
de línea de comandos y terminal. Funciones, constantes, scripts, historial y la
preferencia de idioma viven en un solo lugar — `~/.epher` en tu equipo (o
`EPHER_STORE_DIR`, capítulo 4.6) — y todo lo guardado en una versión está
disponible en las demás:

```text
def area(w, h) = w * h
save area
```

Define `area` en la aplicación de escritorio, `save` la, cierra la
ventana — luego abre la CLI y `area(3, 4)` simplemente funciona. También
funciona al revés: las funciones y scripts guardados en la CLI o la TUI ya
están ahí cuando se abre la ventana de escritorio, incluidas las variables
definidas por scripts guardados. Los comandos `save`, `save script` y
`language` del capítulo 4 funcionan exactamente igual aquí.

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
argumento. Todo lo del capítulo 1 está disponible — variables, funciones,
bucles, todo — y las líneas comparten una sesión, como un script por
tubería (sección 4.2).

### 4.2 Scripts por tubería

`epher -` lee expresiones de la entrada estándar, línea a línea — como se
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
sigue. Una línea puede unir varias instrucciones con `;` — los saltos de
línea y `;` significan lo mismo en todas partes de epher.

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

### 4.5 Cambiar el idioma de la interfaz

El idioma de la interfaz se elige entre los idiomas configurados en tu
dispositivo. Para cambiarlo, escribe `language` seguido de uno de: `en`,
`zh-CN`, `hi`, `es`, `fr`, `ar`, `de`, `pt`:

```text
epher> language fr
language set to fr
```

La elección se recuerda para la próxima vez. Nota: el idioma que *escribes*
— el lenguaje de las expresiones — es siempre el mismo, en cualquier idioma
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
de tu terminal. Forma parte del mismo programa `epher` — iníciala con:

```sh
epher tui
```

### 5.1 La pantalla

La pantalla está dividida en paneles:

- **Expresión** — la línea de entrada (arriba).
- El **resultado** actual justo debajo.
- **Historial** — cada línea que escribiste, con su respuesta.
- **Gráfica** — la gráfica del comando `graph` (abajo).
- Una línea de pistas muestra los atajos de teclado.

### 5.2 Teclas

| Tecla | Acción |
|---|---|
| Escribir | añadir a la expresión |
| **Intro** | evaluar |
| **Esc** | borrar la línea de entrada |
| **Ctrl+C** | salir |
| **q** | salir (cuando la entrada está vacía) |
| **Teclas de flecha** | girar la vista 3D (cuando la entrada está vacía) |
| **Espacio** | iniciar/detener la animación (cuando la entrada está vacía) |
| **F10** | abrir los menús (File, Edit, Graph, Settings, Help) |
| **Tab** | abrir el teclado; cambiar de grupo (**Esc** cierra) |
| **Ctrl+L** | borrar el historial |

Los grupos del teclado contienen todas las funciones, constantes y
comandos que admite el lenguaje: **trig**, **fn**, **num** y **var**.
Las flechas mueven el resaltado, **Intro** inserta el token y **Tab**
cambia de grupo.

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

Puedes graficar cualquier expresión, incluidas tus propias funciones —
primero define una y luego grafícala:

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
comando graph, la TUI lista los puntos de interés — raíces, máximos y
mínimos e intersecciones — bajo la gráfica. El comando `table`
(sección 2.4.2) también funciona aquí.

`graph3d x ^ 2 - y ^ 2` dibuja una superficie 3D como una malla alámbrica
ASCII — gírala con las teclas de flecha y pulsa la barra espaciadora para
animar una constante con deslizador (sección 2.4.5).

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

- El **programa epher instalado** — aplicación de escritorio, CLI y TUI —
  guarda funciones, scripts, historial y la elección de idioma localmente en
  `~/.epher` (o `EPHER_STORE_DIR`). Nada sale de tu equipo.
- La **aplicación web** no guarda nada en disco: el historial dura solo
  mientras la página está abierta. La aplicación web puede funcionar sin
  conexión porque la propia página la guarda tu navegador.

Las cuatro versiones ejecutan el cálculo íntegramente en tu dispositivo —
nada se envía a ningún sitio.
