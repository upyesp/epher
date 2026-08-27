# Guia de utilizador do epher

Bem-vindo! O epher é uma calculadora programável e com scripts. Pode usá-la
para um cálculo rápido ou para construir as suas próprias funções e pequenos
programas — e tudo está disponível em oito idiomas.

Este guia destina-se a principiantes absolutos. Começa com o cálculo mais
simples possível e avança até todo o poder da linguagem. Cada exemplo mostra
o que escreve e o que o epher responde.

Há cinco formas de usar o epher — escolha a que mais lhe convier:

| Versão | O que é | Ideal quando |
|---|---|---|
| **Linha de comandos** (CLI) | Comandos de texto num terminal | Vive no terminal e gosta de scripts |
| **REPL** | Uma sessão interativa do `epher` no prompt `epher>` | Quer ida e volta rápida sem sair do terminal |
| **Interface de terminal** (TUI) | Um programa de ecrã inteiro dentro do terminal | Quer uma aplicação de terminal com gráficos e histórico no ecrã |
| **Aplicação de ambiente de trabalho** | Um programa normal com a sua própria janela | Quer uma aplicação normal |
| **Aplicação web** (PWA) | Funciona no seu navegador, pode ser instalada, funciona offline | Quer o arranque mais rápido; sem instalação |

A aplicação de ambiente de trabalho, a linha de comandos, o REPL e a
interface de terminal são um só programa: um único download instala o
comando `epher`, que faz as quatro coisas. A aplicação web é a exceção —
não precisa de qualquer download.

As cinco versões compreendem exatamente a mesma linguagem. Aprenda-a uma
vez, use-a em qualquer lado.

## 1. A linguagem do epher

Este capítulo ensina a linguagem partilhada por todas as versões do epher.
Na aplicação web ou na aplicação de ambiente de trabalho, escreva uma
expressão e prima **Enter** (ou clique no botão **=**). Na CLI, inicie a
sessão com `epher repl` e escreva a seguir ao prompt `epher>`. Na TUI
(`epher tui`), basta escrever e premir **Enter**. Na CLI também pode
escrever `epher "expressão"` para avaliar uma expressão diretamente.

### 1.1 O seu primeiro cálculo

Escreva isto:

```epher
2 + 3 * 4
```

O epher responde:

```text
14
```

A multiplicação é feita antes da adição, exatamente como na matemática.
Essa regra chama-se *precedência de operadores*.

### 1.2 Ordem das operações

A ordem de precedência completa, da mais forte para a mais fraca:

1. `!` fatorial
2. `^` potência
3. `*` e `/` multiplicação e divisão
4. `+` e `-` adição e subtração

Use parênteses para alterar a ordem:

```epher
(2 + 3) * 4
```

```text
20
```

O operador `^` calcula potências e funciona da direita para a esquerda:

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

(`2 ^ 3 ^ 2` significa `2 ^ (3 ^ 2)`, ou seja `2 ^ 9` = 512.)

As potências podem ser fracionárias — `2 ^ 0.5` é a raiz quadrada de 2:

```epher
2 ^ 0.5
```

```text
1.4142135623730951
```

A subtração e a divisão funcionam da esquerda para a direita:

```epher
10 - 3 - 2
```

```text
5
```

### 1.3 Os números especiais pi, e, tau e phi

As constantes famosas já vêm integradas:

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

Mais duas: `tau` é uma volta completa (2 pi) e `phi` é o número de ouro:

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

### 1.4 Comparações e lógica

Pode comparar números. O resultado é `true` ou `false`:

| Comparação | Significado |
|---|---|
| `a > b` | a é maior do que b |
| `a < b` | a é menor do que b |
| `a >= b` | a é maior ou igual a b |
| `a <= b` | a é menor ou igual a b |
| `a == b` | a é igual a b (repare no `=` duplo) |
| `a != b` | a não é igual a b |

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

Combine comparações com `and`, `or` e `not`:

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

### 1.5 Variáveis

Dê um nome a um valor com um único `=`:

```epher
x = 5
```

```text
5
```

O epher devolve-lhe o valor. A partir de agora, `x` pode ser usado em
qualquer lado:

```epher
x ^ 2
```

```text
25
```

Pode alterar uma variável sempre que quiser — ela mantém o valor até o
alterar:

```epher
x = x + 1
```

```text
6
```

> Os nomes podem conter letras e sublinhados, como `radius` ou `my_total`.
> Não podem conter espaços nem começar com um número.

A variável especial `ans` sempre guarda a resposta anterior, como a
tecla `Ans` de uma calculadora de bolso — útil para encadear cálculos:

```epher
2 + 3
ans * 2
```

```text
5
10
```

### 1.6 Constantes: nomes que nunca mudam

Uma *constante* é um nome para um valor que nunca muda — como o `pi`
integrado, mas escolhido por si. Defina uma com `const`:

```epher
const tax = 0.2
```

```text
0.2
```

Use-a em qualquer sítio onde possa estar um número:

```epher
100 * (1 + tax)
```

```text
120
```

O valor é fixo: alterá-lo com `=` é um erro,

```epher
tax = 0.25
```

```text
error: cannot assign to constant tax
```

e definir a mesma constante duas vezes também:

```epher
const tax = 0.25
```

```text
error: constant already defined: tax
```

As constantes diferem das variáveis noutro aspeto: tal como `pi`, funcionam
dentro das suas próprias funções.

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

Guarde uma constante para sessões futuras com `save tax`, exatamente como
uma função (capítulo 4.4).

> Uma variável e uma constante não podem partilhar o nome: depois de
> `const tax = 0.2`, `tax = ...` é sempre um erro. Escolha um nome novo ou
> inicie uma nova sessão.

### 1.7 Decisões com if

O `if` escolhe entre dois valores:

```epher
if 3 > 2 then 10 else 20
```

```text
10
```

A forma é sempre `if condição then valor_se_verdadeiro else
valor_se_falso`. A parte `else` é obrigatória.

Um exemplo mais útil, com uma variável:

```epher
price = 100
if price > 50 then 2 else 1
```

```text
2
```

> O epher não tem valores de texto — os dois ramos de um `if` têm de ser
> números (ou resultados de comparações).

### 1.8 Ciclos com while

O `while` repete uma instrução enquanto uma condição se mantiver:

```epher
x = 0; while x < 5 do x = x + 1; x
```

```text
5
```

Leia esse script assim: *comece com x em 0; enquanto x for menor do que 5,
some 1 a x; depois mostre x.* O resultado é 5 porque o ciclo correu cinco
vezes.

> **Rede de segurança:** o epher para qualquer ciclo ao fim de 100 000
> passos e mostra `error: step limit exceeded`. Isto protege-o de ciclos que
> nunca terminariam. Se vir essa mensagem, a sua condição provavelmente
> nunca se tornou falsa.

### 1.9 As suas próprias funções com def

Uma função é um cálculo com um nome e parâmetros:

```epher
def f(x) = x ^ 2
```

Depois use-a:

```epher
f(7)
```

```text
49
```

As funções podem receber vários parâmetros:

```epher
def area(w, h) = w * h
area(3, 4)
```

```text
12
```

Também pode definir uma função sem parâmetros:

```epher
def answer() = 42
answer()
```

```text
42
```

### 1.10 Recursão: uma função que se chama a si própria

O exemplo mais famoso — os números de Fibonacci:

```epher
def fib(n) = if n <= 1 then n else fib(n - 1) + fib(n - 2)
```

```epher
fib(10)
```

```text
55
```

`fib(10)` é o décimo número de Fibonacci. A função chama-se a si própria
com argumentos cada vez menores até chegar a `n <= 1`. Isto funciona porque
a forma `if ... then ... else ...` só calcula o ramo de que precisa.

> O corpo de uma função é uma única expressão — uma linha. Em vez disso,
> combine vários cálculos com `;` num script (secção seguinte).

### 1.11 Scripts: várias instruções de uma vez

Um *script* é um conjunto de instruções unidas por `;` — ou por quebras de
linha, que significam exatamente a mesma coisa — executadas uma após a outra:

```epher
x = 10; y = x + 5; x + y
```

```text
25
```

Os scripts são a forma de construir pequenos programas: defina variáveis,
faça ciclos e mostre um resultado final.

As quebras de linha e o `;` são o mesmo separador, e pode misturá-los
livremente. O botão **Copiar** por cima de um exemplo de várias linhas
copia o script inteiro, e pode colá-lo diretamente no epher: o campo de
entrada na aplicação web e na aplicação de ambiente de trabalho, a
interface de terminal e o `epher repl` executam cada linha por ordem,
exatamente como se as tivesse escrito uma a uma. Unir várias instruções
com `;` numa só linha também funciona em todo o lado — incluindo a linha
de comandos de avaliação única (secção 4.1).

### 1.12 Resultados exatos: frac, dec e big

Normalmente o epher calcula com números decimais, como uma calculadora de
bolso. Há números que ficam melhor em forma exata.

**frac(n, d)** cria uma fração exata:

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

As frações mantêm-se exatas ao longo dos cálculos:

```epher
frac(1, 3) * 3
```

```text
1
```

**dec(x)** cria um decimal exato. Compare estes dois:

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

O primeiro resultado é o pequeno erro de arredondamento que todos os
computadores cometem com números decimais. O `dec()` elimina-o.

**big(x)** cria um número inteiro exato, para valores demasiado grandes para
uma calculadora de bolso:

```epher
big(10 ^ 20)
```

```text
100000000000000000000
```

**Bases numéricas** escrevem inteiros como a comunidade matemática os
escreve: `0b` para binário, `0o` para octal, `0x` para hexadecimal (o
prefixo muda a grafia, nunca o valor):

```epher
0b1010 + 0xFF
```

```text
265
```

Converta de volta com **bin(x)**, **oct(x)** e **hex(x)** — a grafia
com prefixo de um número inteiro, pronta para ser usada de novo:

```epher
hex(255)
bin(10)
```

```text
0xff
0b1010
```

### 1.13 Funções integradas

O epher tem as funções de uma calculadora científica, agrupadas por família.

A trigonometria funciona em radianos — use `deg` e `rad` para converter:

| Função | Significado | Exemplo | Resultado |
|---|---|---|---|
| `sin(x)`, `cos(x)`, `tan(x)` | funções trigonométricas | `sin(pi / 2)` | `1` |
| `asin(x)`, `acos(x)`, `atan(x)` | trigonométricas inversas | `atan(1)` | `0.7853981633974483` |
| `atan2(y, x)` | ângulo do ponto (x, y) | `atan2(1, 1)` | `0.7853981633974483` |
| `deg(x)` | radianos → graus | `deg(pi)` | `180` |
| `rad(x)` | graus → radianos | `rad(180)` | `3.141592653589793` |
| `sinh(x)`, `cosh(x)`, `tanh(x)` | funções hiperbólicas | `sinh(1)` | `1.1752011936438014` |
| `asinh(x)`, `acosh(x)`, `atanh(x)` | hiperbólicas inversas | `acosh(1)` | `0` |

Potências, raízes e logaritmos (numa calculadora, `log` é de base 10):

| Função | Significado | Exemplo | Resultado |
|---|---|---|---|
| `sqrt(x)` | raiz quadrada | `sqrt(16)` | `4` |
| `cbrt(x)` | raiz cúbica | `cbrt(-27)` | `-3` |
| `root(n, x)` | raiz de ordem n | `root(3, 8)` | `2` |
| `exp(x)` | e elevado a x | `exp(1)` | `2.718281828459045` |
| `ln(x)` | logaritmo natural | `ln(e)` | `1` |
| `log(x)` | logaritmo de base 10 | `log(100)` | `2` |
| `log2(x)` | logaritmo de base 2 | `log2(8)` | `3` |
| `logb(b, x)` | logaritmo na base b | `logb(2, 8)` | `3` |
| `hypot(a, b)` | hipotenusa | `hypot(3, 4)` | `5` |
| `5!` (também `fact(n)`) | fatorial | `5!` | `120` |

Arredondamento, sinais e números inteiros:

| Função | Significado | Exemplo | Resultado |
|---|---|---|---|
| `abs(x)` | valor absoluto | `abs(-3)` | `3` |
| `floor(x)` / `ceil(x)` | arredondar para baixo / para cima | `floor(2.7)` | `2` |
| `round(x)` | mais próximo, metades para longe de zero | `round(2.5)` | `3` |
| `trunc(x)` | eliminar a parte fracionária | `trunc(-2.9)` | `-2` |
| `sign(x)` | -1, 0 ou 1 | `sign(-5)` | `-1` |
| `ncr(n, r)` | combinações | `ncr(52, 5)` | `2598960` |
| `npr(n, r)` | permutações | `npr(5, 2)` | `20` |
| `gcd(a, b)` / `lcm(a, b)` | divisores e múltiplos comuns | `gcd(12, 18)` | `6` |
| `mod(a, b)` | resto | `mod(7, 3)` | `1` |

As estatísticas aceitam qualquer número de argumentos:

| Função | Significado | Exemplo | Resultado |
|---|---|---|---|
| `sum(...)` / `product(...)` | totais | `sum(1, 2, 3)` | `6` |
| `mean(...)` | média | `mean(1, 2, 3)` | `2` |
| `median(...)` | valor central | `median(1, 2, 3, 4)` | `2.5` |
| `min(...)` / `max(...)` | menor / maior | `max(4, 1, 3)` | `4` |
| `variance(...)` / `stdev(...)` | dispersão dos valores | `stdev(2, 4)` | `1` |

As camadas exatas da secção 1.12 mantêm-se:

| Função | Significado | Exemplo | Resultado |
|---|---|---|---|
| `frac(n, d)` | fração exata | `frac(1, 3)` | `1/3` |
| `dec(x)` | decimal exato | `dec(0.1)` | `0.1` |
| `big(x)` | número inteiro exato | `big(10 ^ 20)` | `100000000000000000000` |
| Binário, octal, hex | `0b…`, `0o…`, `0x…` | `0xFF + 0b1` |
| Grafia em base | `bin(x)`, `oct(x)`, `hex(x)` | `hex(255)` |
| `bin(x)` / `oct(x)` / `hex(x)` | grafia com prefixo na base 2 / 8 / 16 | `hex(255)` | `0xff` |

Combinam-se como tudo o resto:

```epher
min(sqrt(16), 5)
```

```text
4
```

### 1.14 Ler erros

Quando algo corre mal, o epher diz-lhe em vez de adivinhar:

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

O último exemplo é importante: o epher diz-lhe exatamente que nome não
conhece, para poder corrigir a sua expressão.

### 1.15 Referência rápida

| O quê | Sintaxe | Exemplo |
|---|---|---|
| Somar, subtrair, multiplicar, dividir | `+ - * /` | `7 / 2` |
| Potência | `^` (da direita para a esquerda) | `2 ^ 10` |
| Fatorial | `!` (pós-fixo) | `5!` |
| Parênteses | `( )` | `(2 + 3) * 4` |
| Constantes | `pi`, `e`, `tau`, `phi` | `2 * pi` |
| Notação científica | `2.5e-3` | `6.02e23` |
| Comparar | `> < >= <= == !=` | `3 >= 2` |
| Lógica | `and or not` | `a > 1 and a < 10` |
| Variável | `name = value` | `x = 5` |
| Constante | `const name = value` | `const tax = 0.2` |
| Decisão | `if c then a else b` | `if x > 0 then 1 else -1` |
| Ciclo | `while c do statement` | `while x < 5 do x = x + 1` |
| Função | `def name(params) = expr` | `def f(x) = x ^ 2` |
| Script | instruções unidas por `;` ou quebras de linha | `x = 1; x + 1` |
| Fração exata | `frac(n, d)` | `frac(1, 3)` |
| Decimal exato | `dec(x)` | `dec(0.1) + dec(0.2)` |
| Número inteiro exato | `big(x)` | `big(10 ^ 20)` |
| Binário, octal, hex | `0b…`, `0o…`, `0x…` | `0xFF + 0b1` |
| Grafia em base | `bin(x)`, `oct(x)`, `hex(x)` | `hex(255)` |

## 2. A aplicação web (PWA)

### 2.1 Abri-la

A aplicação web está em:

```text
https://epher.org/pwa/
```

Não é necessária qualquer instalação — funciona em qualquer navegador
moderno, num computador, telemóvel ou tablet.

Este guia também está integrado na aplicação: abra **Help → User guide**
na barra de menu (toque em **☰** num telemóvel) para o ler dentro da
aplicação, no idioma atual da aplicação. Toque num exemplo desse guia
para o carregar no campo de entrada.

### 2.2 O seu primeiro cálculo

1. Clique no campo de texto (já vem focado quando a página carrega).
2. Escreva uma expressão, por exemplo `2 + 3 * 4`.
3. Prima **Enter** ou clique no botão **=**.

O resultado aparece em texto grande por baixo do campo. Tudo o que está no
capítulo 1 funciona aqui, incluindo variáveis, funções e scripts.

### 2.3 Histórico

Cada cálculo é adicionado à lista de histórico por baixo do resultado, para
poder recuar e ver o que fez. As entradas mais recentes aparecem no topo,
e o botão **Clear history** por cima da lista esvazia-a. O histórico é
mantido enquanto a página estiver aberta.

Cada entrada fica entre linhas finas: uma expressão de uma linha é uma fila, e um script de várias linhas é uma entrada que mostra todas as suas linhas. Clique numa entrada para a voltar a carregar no campo de entrada e executá-la de novo.

### 2.4 Gráficos

Escreva `graph` seguido de uma expressão e prima **Enter**:

```epher
graph x ^ 2
```

O epher desenha a curva y = f(x) de x = −10 a x = 10 por baixo da
entrada, numa grelha com eixos legendados. Pode desenhar o gráfico de
qualquer expressão, incluindo as suas próprias funções:

```epher
def f(x) = x ^ 3
graph f(x)
```

Cada linha `graph` acrescenta outra curva ao mesmo gráfico, cada uma com
a sua cor — as curvas são todas sólidas, e a legenda e as legendas
no gráfico são o que as distingue sem cor.
`graph clear` esvazia o gráfico — e um botão **Clear graph** no topo
do painel do gráfico faz o mesmo para curvas e superfícies 3D em
conjunto. A TUI mantém o comando no seu menu **Graph**.

No topo do painel de gráficos, ao lado de **Clear graph** e
**Copy SVG**, a linha de opções permite ocultar a lista de pontos de
interesse, ocultar os pontos destacados desenhados no próprio gráfico e
ajustar a espessura das linhas com o controle **Espessura da linha**.

```epher
graph x ^ 2
graph x ^ 3
```

Os pontos onde a expressão não tem valor (uma divisão por zero, por
exemplo) são ignorados, deixando um intervalo na curva — e um salto que é
na verdade uma assíntota vertical nunca é desenhado como linha de ligação.

#### 2.4.1 O que pode desenhar

Um domínio à sua escolha:

```epher
graph sin(x) from 0 to 2*pi
```

Curvas paramétricas (t vai de 0 a 2π):

```epher
graph param 2*cos(t), 3*sin(t)
```

Curvas polares:

```epher
graph polar 1 + cos(theta)
```

Regiões: `y <` sombreia a área por baixo da curva, `y >` sombreia por cima:

```epher
graph y < x ^ 2
```
#### 2.4.2 Ler o gráfico

**Rastrear:** mova o ponteiro sobre o gráfico — ou foque-o e prima as
teclas de seta — e o ponto mais próximo de uma curva é marcado, com as
suas coordenadas anunciadas por baixo do gráfico.

**Pontos de interesse:** após cada comando graph o epher encontra as
raízes e os extremos de cada curva e as interseções entre curvas,
marca-os no gráfico e lista-os por baixo:

```text
root (-1, 0)   minimum (0, 0)   root (1, 0)
```

**Tabelas:** o comando `table` imprime uma tabela de valores (as linhas
onde a expressão não tem valor ficam em branco):

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

#### 2.4.3 Deslizadores e exportação

Defina uma constante, use-a num gráfico, e aparece um deslizador por
baixo do gráfico — arraste-o (ou mova-o com as teclas de seta) e todas
as curvas são redesenhadas:

```epher
const a = 1
graph a * x ^ 2
```

**Copiar SVG** copia o gráfico atual como uma imagem SVG autossuficiente
para colar em documentos — as cores vão embutidas, então ele fica igual
em qualquer lugar. O controle **Espessura da linha**, na parte inferior
do painel, ajusta a espessura de cada linha desenhada.

#### 2.4.4 Superfícies 3D

`graph3d` desenha uma superfície z = f(x, y) sobre um domínio quadrado
(−5 a 5, ou o seu `from a to b`):

```epher
graph3d x ^ 2 - y ^ 2
```

As linhas da malha mais próximas de si são desenhadas com mais força, de
modo a que a forma ganhe profundidade. Várias linhas `graph3d`
sobrepõem-se, tal como as curvas, e `graph3d clear` esvazia o gráfico.
Rode a vista arrastando, ou foque o gráfico e use as teclas de seta. A
TUI desenha a mesma superfície como uma malha de arame em ASCII, que
roda com as teclas de seta.

#### 2.4.5 Animação

Cada deslizador tem um botão de reprodução. Ele faz a sua constante
percorrer o intervalo do deslizador e recomeçar do início — a forma
habitual de as calculadoras animarem: anima-se um parâmetro e tudo o que
o usa move-se. Prima o
botão novamente para fazer uma pausa.

Uma variável de "tempo" é apenas uma constante que anima:

```epher
const t = 0
graph sin(x - t)
```

Reproduzir o deslizador de t faz a onda viajar. As superfícies 3D animam
da mesma forma — defina primeiro uma constante e depois reproduza o seu
deslizador:

```epher
const a = 1
graph3d sin(a * (x ^ 2 + y ^ 2)) from -3 to 3
```

Na TUI, a barra de espaço inicia e para a animação.

### 2.5 Instalá-la e usá-la offline

A aplicação web é uma *aplicação web progressiva*: após uma visita,
funciona totalmente offline e pode instalá-la como uma aplicação normal.

- **Chrome, Edge ou Android:** clique no ícone de instalação na barra de
  endereço (ou em *Instalar aplicação* no menu do navegador) e confirme.
- **iPhone / iPad (Safari):** toque em **Partilhar** → **Adicionar ao ecrã
  principal**.
- **Outros navegadores:** procure *Instalar* ou *Adicionar ao ecrã
  principal* no menu.

Depois de instalada, abra-a a partir do ecrã principal ou da lista de
aplicações — abre instantaneamente, mesmo sem ligação à internet.

### 2.6 O que a aplicação web não faz

A aplicação web mantém o seu trabalho na sessão atual: avalia expressões,
desenha os seus gráficos (secção 2.4) e mantém um histórico. Os comandos
**save**, **save script** e **language** funcionam nas versões de
ambiente de trabalho, linha de comandos e terminal (capítulos 3, 4 e 5)
— na aplicação web respondem com uma nota a dizer que guardar funciona
nessas versões. O histórico não é guardado entre visitas.

## 3. A aplicação de ambiente de trabalho

A aplicação de ambiente de trabalho é uma janela normal à volta da mesma
aplicação web. Tudo o que está no capítulo 2 se aplica; a única diferença é
a forma de a instalar e iniciar.

### 3.1 Instalar

Descarregue um instalador para o seu sistema a partir do site do epher:

- **Windows:** execute `epher-windows-x86_64.exe`. O instalador coloca o
  `epher` no seu PATH — abra uma nova janela de CMD ou PowerShell e
  `epher "2 + 2"` funciona. Como a compilação não está assinada, escolha
  *Mais informações* → *Executar mesmo assim* no primeiro arranque.
- **macOS:** abra `epher-macos-aarch64.dmg` e arraste o epher para
  Aplicações. Como a compilação não está assinada, o primeiro arranque
  requer um clique com o botão direito → **Abrir**.
- **Linux (Debian/Ubuntu):** o pacote `.deb`

```sh
sudo apt install ./epher-linux-x86_64.deb
```

- **Linux (Fedora/RHEL):** o pacote `.rpm`

```sh
sudo dnf install ./epher-linux-x86_64.rpm
```

- **Linux (qualquer distribuição, incluindo Arch):** o AppImage — torne-o
  executável e execute-o:

```sh
chmod +x epher-linux-x86_64.AppImage
./epher-linux-x86_64.AppImage
```

Cada instalador contém o epher *completo* — a aplicação de ambiente de
trabalho, a linha de comandos (capítulo 4) e a interface de terminal
(capítulo 5) — no único comando `epher`. No Linux, o pacote coloca o
`epher` em `/usr/bin`.

### 3.2 Usá-la

Inicie o epher como qualquer outra aplicação. Obtém uma janela com a mesma
interface da aplicação web: escreva uma expressão, prima **Enter** ou
clique em **=** e leia o resultado. Os gráficos também funcionam aqui —
`graph x ^ 2` desenha na janela (capítulo 2.4). A janela pode ser
redimensionada livremente. A barra de menu inclui
**Help → User guide** — o mesmo guia desta página, com exemplos que
se carregam com um toque.

Também pode abri-la a partir de um terminal: um `epher` simples (ou
`epher gui`) inicia a aplicação de ambiente de trabalho. No macOS, use o
botão **Install the epher command** dentro da aplicação para colocar o
`epher` no PATH do seu terminal.

### 3.3 Armazenamento: um só arquivo com a CLI e a TUI

A aplicação de ambiente de trabalho partilha o seu armazenamento com as
versões de linha de comandos e de terminal. Funções, constantes, scripts,
histórico e a preferência de idioma vivem num único lugar — `~/.epher` no
seu computador (ou `EPHER_STORE_DIR`, capítulo 4.6) — e tudo o que for
guardado numa versão fica disponível nas outras:

```text
def area(w, h) = w * h
save area
```

Defina `area` na aplicação de ambiente de trabalho, guarde-a com `save`,
feche a janela — depois abra a CLI e `area(3, 4)` funciona sem mais.
Também funciona ao contrário: as funções e os scripts que guardou na CLI
ou na TUI já lá estão quando a janela da aplicação de ambiente de trabalho
abre, incluindo variáveis definidas por scripts guardados. Os comandos
`save`, `save script` e `language` do capítulo 4 funcionam exatamente da
mesma forma aqui.

Os comandos que escreve na CLI, no REPL, na TUI ou na aplicação de
ambiente de trabalho juntam-se todos ao mesmo histórico, e a sessão
também viaja: as variáveis que define e o valor `ans` seguem-no de
uma versão para a seguinte.

> A aplicação web no navegador é a única versão que não usa este
> armazenamento — mantém cada sessão para si (capítulo 2.6).

## 4. A linha de comandos (CLI)

A CLI é o lado textual do mesmo programa `epher` da aplicação de ambiente
de trabalho. Tem três modos: avaliação única, scripts por pipe e uma sessão
interativa para trabalho mais longo.

Para obter ajuda a qualquer momento, execute `epher --help` (todos os
comandos, com exemplos) ou `epher help` (o manual completo; nos pacotes
Linux é a página `man epher`).

### 4.1 Cálculos únicos

Passe a expressão como argumento:

```sh
epher "2 + 3 * 4"
```

```text
14
```

Pode fazer tudo o que for uma única expressão do capítulo 1:

```sh
epher "if 3 > 2 then 10 else 20"
```

```text
10
```

Uma expressão que comece com um sinal de menos funciona diretamente:

```sh
epher "-2 + 5"
```

```text
3
```

O modo de avaliação única é para scripts, de uma única expressão até um
programa inteiro. O valor de cada instrução é impresso na sua própria linha:

```sh
epher "x = 10; x + 5"
```

```text
10
15
```

As instruções unidas por quebras de linha funcionam da mesma forma dentro
do argumento. Tudo o que está no capítulo 1 está disponível — variáveis,
funções, ciclos, tudo — e as linhas partilham uma única sessão, como um
script por pipe (secção 4.2).

### 4.2 Scripts por pipe

O `epher -` lê expressões da entrada padrão, uma linha de cada vez — a
forma como as linguagens de scripting são usadas em pipelines:

```sh
printf "x = 3\nx * 10\n" | epher -
```

```text
= 3
= 30
```

Tudo o que está no capítulo 1 funciona, e as linhas partilham uma única
sessão: uma função definida numa linha inicial fica disponível mais tarde,
e o `save` escreve no mesmo armazenamento de sempre. Os erros são impressos
e o script continua. Uma linha pode unir várias instruções com `;` — as
quebras de linha e o `;` significam a mesma coisa em todo o lado no epher.

### 4.3 A sessão interativa (REPL)

Inicie-a com `epher repl`:

```sh
epher repl
```

> Um `epher` simples, sem argumentos, abre a aplicação de ambiente de
> trabalho (capítulo 3).

O epher imprime o prompt e fica à espera:

```text
epher>
```

Agora escreva qualquer coisa do capítulo 1, uma linha de cada vez. As
variáveis mantêm os seus valores entre linhas:

```text
epher> x = 5
= 5
epher> x ^ 2
= 25
```

O comando `table` (secção 2.4.2) também imprime aqui uma tabela de
valores:

```text
epher> table x ^ 2 from -2 to 2 points 5
         x           y
        -2           4
        -1        As linhas `graph` também funcionam aqui: as curvas se acumulam entre
linhas, e `graph save plot.svg` grava a mesma imagem SVG que o botão
**Copiar SVG** da aplicação web produz. `graph3d save arquivo.svg`
salva uma superfície 3D da mesma forma. As mesmas linhas valem na
avaliação única e em scripts canalizados:
`epher "graph sin(x); graph save plot.svg"` é um gráfico completo em um
comando.

   1
         0           0
         1           1
         2           4
```

Cada resposta é mostrada como `= resultado`. Para sair, escreva `quit` (ou
`exit`):

```text
epher> quit
```

O seu histórico é lembrado: da próxima vez que executar `epher repl`, as
linhas da sessão anterior ainda lá estão.

### 4.4 Guardar funções, constantes e scripts

Defina uma função e depois guarde-a:

```text
epher> def fib(n) = if n <= 1 then n else fib(n - 1) + fib(n - 2)
epher> save fib
saved fib
```

O comando `save fib` guarda a função em disco. Da próxima vez que iniciar
a sessão, `fib` já está definida:

```text
epher> fib(10)
= 55
```

As constantes guardam-se da mesma forma — `save` no nome da constante:

```text
epher> const tax = 0.2
= 0.2
epher> save tax
saved tax
```

Para guardar um script inteiro (a última linha que escreveu), use
`save script`:

```text
epher> x = 0; while x < 5 do x = x + 1; x
= 5
epher> save script count_to_five
saved script count_to_five
```

Os scripts guardados são executados automaticamente quando o epher inicia,
para que tudo o que definem esteja pronto para si.

### 4.5 Mudar o idioma da interface

O idioma da interface é escolhido a partir dos idiomas definidos no seu
dispositivo. Para o substituir, escreva `language` seguido de um destes:
`en`, `zh-CN`, `hi`, `es`, `fr`, `ar`, `de`, `pt`:

```text
epher> language fr
language set to fr
```

A escolha fica memorizada para a próxima vez. Nota: o idioma em que
*escreve* — a linguagem de expressões — é sempre o mesmo, em qualquer
idioma de interface.

### 4.6 Onde vivem os seus dados

As funções, os scripts, o histórico e a sua escolha de idioma são guardados
numa única pasta no seu computador:

```text
~/.epher
```

Apague essa pasta para começar completamente do zero. Para usar uma
localização diferente, defina a variável de ambiente `EPHER_STORE_DIR`
antes de iniciar o epher:

```sh
EPHER_STORE_DIR=/tmp/my-epher epher repl
```

## 5. A interface de terminal (TUI)

A TUI é uma versão de ecrã inteiro da sessão interativa, dentro do seu
terminal. Faz parte do mesmo programa `epher` — inicie-a com:

```sh
epher tui
```

### 5.1 O ecrã

O ecrã está dividido em painéis:

- **Expressão** — a linha de entrada (em cima).
- O **resultado** atual logo por baixo.
- **Histórico** — todas as linhas que introduziu, com a respetiva resposta.
- **Gráfico** — o gráfico do comando `graph` (em baixo).
- Uma linha de sugestões mostra os atalhos de teclado.

### 5.2 Teclas

| Tecla | Ação |
|---|---|
| Escrever | adicionar à expressão |
| **Enter** | avaliar |
| **Esc** | limpar a linha de entrada |
| **Ctrl+C** | sair |
| **q** | sair (quando a entrada está vazia) |
| **Arrow keys** | rodar a vista 3D (quando a entrada está vazia) |
| **Space** | iniciar/parar a animação (quando a entrada está vazia) |
| **F10** | abrir os menus (Ficheiro, Editar, Gráfico, Definições, Ajuda) |
| **Tab** | focar o teclado sempre visível (ou o histórico, a partir do teclado); trocar de grupo (**Esc** volta à escrita) |
| **Rato** | clique menus e itens de menu, células e separadores do teclado, linhas do histórico (carrega a expressão); arraste o painel do gráfico para orbitar (3D) ou mover (2D), a roda faz zoom, um duplo clique repõe a vista |
| **Ctrl+L** | limpar o histórico |

Os grupos do teclado cobrem todas as funções, constantes e comandos que
a linguagem suporta: **trig**, **fn**, **num**, **0x** e **var** —
o grupo 0x contém as conversões exatas e de base (`frac`, `dec`,
`big`, `bin`, `oct`, `hex`) e o fatorial `!`. As setas movem
o realce, **Enter** insere o token e a **Tab** troca de grupo.

### 5.3 Gráficos

Escreva `graph` seguido de uma expressão e prima **Enter**:

```epher
graph x ^ 2
```

O epher amostra a curva de x = −10 a x = 10 e desenha-a como um gráfico
ASCII no painel Graph; a legenda por cima do gráfico dá nome ao que está
traçado.

`graph clear` esvazia o gráfico, e o menu **Graph** faz o mesmo; o
menu **Help** abre este guia dentro da TUI (as setas fazem rolar,
**Esc** fecha). O menu **Settings** pode ocultar os pontos de interesse
listados sob o gráfico.

Pode desenhar o gráfico de qualquer expressão, incluindo as suas próprias
funções — primeiro defina uma, depois desenhe o gráfico:

```epher
def f(x) = x ^ 3
graph f(x)
```

Cada linha `graph` acrescenta uma curva ao gráfico, desenhada com o seu
próprio símbolo (`o`, `x`, `+`, `*`); `graph clear` esvazia o gráfico.
Aplica-se a mesma gramática da aplicação web: um domínio
(`graph sin(x) from 0 to 2*pi`), curvas paramétricas
(`graph param 2*cos(t), 3*sin(t)`), curvas polares
(`graph polar 1 + cos(theta)`) e regiões (`graph y < x ^ 2` sombreia a
área por baixo da curva).

Os pontos onde a expressão não tem valor (por exemplo, divisão por zero)
são simplesmente ignorados, deixando um intervalo no gráfico. Após cada
comando graph, a TUI lista os pontos de interesse — raízes, extremos e
interseções — por baixo do gráfico. O comando `table` (secção 2.4.2)
também funciona aqui.

`graph3d x ^ 2 - y ^ 2` desenha uma superfície 3D como uma malha de
arame em ASCII — rode-a com as teclas de seta e prima a barra de espaço
para animar uma constante de deslizador (secção 2.4.5).

`graph save plot.svg` grava o gráfico atual como a mesma imagem SVG que
o botão **Copiar SVG** da aplicação web produz; `graph3d
save arquivo.svg` salva a malha 3D do ângulo em que você a está vendo.

### 5.4 Guardar e persistência

A TUI partilha o armazenamento com a CLI: tudo o que for guardado numa fica
disponível na outra. As funções, os scripts, o histórico e a preferência de
idioma vivem em `~/.epher` (capítulo 4.6), e os mesmos comandos `save`,
`save script` e `language` funcionam aqui.

## 6. Os seus dados e a privacidade

- O **programa epher instalado** — aplicação de ambiente de trabalho, CLI e
  TUI — guarda as funções, os scripts, o histórico e a escolha de idioma
  localmente em `~/.epher` (ou `EPHER_STORE_DIR`). Nada sai do seu
  computador.
- A **aplicação web** não guarda nada em disco: o histórico dura apenas
  enquanto a página estiver aberta. A aplicação web pode funcionar offline
  porque a própria página é guardada pelo seu navegador.

As cinco versões executam o cálculo inteiramente no seu dispositivo — nada
é enviado para lado nenhum.
