# Guia de utilizador do epher

Bem-vindo! O epher é uma calculadora programável e com scripts. Pode usá-la
para um cálculo rápido ou para construir as suas próprias funções e pequenos
programas. Tudo está disponível em oito idiomas.

Este guia destina-se a principiantes absolutos. Começa com o cálculo mais
simples possível e avança até todo o poder da linguagem. Cada exemplo mostra
o que escreve e o que o epher responde.

Há cinco formas de usar o epher. Escolha a que mais lhe convier:

| Versão | O que é | Ideal quando |
|---|---|---|
| **Linha de comandos** (CLI) | Comandos de texto num terminal | Vive no terminal e gosta de scripts |
| **REPL** | Uma sessão interativa do `epher` no prompt `epher>` | Quer ida e volta rápida sem sair do terminal |
| **Interface de terminal** (TUI) | Um programa de ecrã inteiro dentro do terminal | Quer uma aplicação de terminal com gráficos e histórico no ecrã |
| **Aplicação de ambiente de trabalho** | Um programa normal com a sua própria janela | Quer uma aplicação normal |
| **Aplicação web** (PWA) | Funciona no seu navegador, pode ser instalada, funciona offline | Quer o arranque mais rápido; sem instalação |

A aplicação de ambiente de trabalho, a linha de comandos, o REPL e a
interface de terminal são um só programa: um único download instala o
comando `epher`, que faz as quatro coisas. A aplicação web é a exceção:
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

1. `!` fatorial e `%` porcentagem (ambos pós-fixos)
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

As potências podem ser fracionárias. `2 ^ 0.5` é a raiz quadrada de 2:

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

O sinal `%` é um operador pós-fixo que significa «dividido por 100»: `5%` é 0.05. Ele nunca olha os operadores à volta, por isso `200 + 10%` é 200.1. Para aumentar 200 em 10%, escreva a multiplicação:

```epher
200 * (1 + 10%)
```

```text
220
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

Pode alterar uma variável sempre que quiser. Ela mantém o valor até o
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
tecla `Ans` de uma calculadora de bolso, útil para encadear cálculos:

```epher
2 + 3
ans * 2
```

```text
5
10
```

### 1.6 Constantes: nomes que nunca mudam

Uma *constante* é um nome para um valor que nunca muda, como o `pi`
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

e redefini-la com um valor diferente também:

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

> O epher não tem valores de texto: os dois ramos de um `if` têm de ser
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

O exemplo mais famoso são os números de Fibonacci:

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

> O corpo de uma função é uma única expressão, uma linha. Em vez disso,
> combine vários cálculos com `;` num script (secção seguinte).

### 1.11 Scripts: várias instruções de uma vez

Um *script* é um conjunto de instruções unidas por `;` ou por quebras de
linha, que significam exatamente a mesma coisa, executadas uma após a outra:

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
com `;` numa só linha também funciona em todo o lado, incluindo a linha
de comandos de avaliação única (secção 4.1).


Scripts podem carregar **comentários** - anotações para você que o epher ignora, no estilo PHP. `//` ou `#` comenta até o fim da linha; `/* ... */` comenta um bloco, através de linhas ou no meio dos tokens:

```epher
// a small script with notes
r = 3 # radius in metres
area = /* pi r squared */ pi * r ^ 2
area
```
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

Converta de volta com **bin(x)**, **oct(x)** e **hex(x)**. Dão a grafia
com prefixo de um número inteiro, pronta para ser usada de novo:

```epher
hex(255)
bin(10)
```

```text
0xff
```
0b1010
```

**exact(x)** reconstrói a fração exata por trás de um resultado decimal: qualquer valor com uma boa fração de denominador pequeno é mostrado assim. É a mesma reconstrução que os aplicativos usam por padrão, por isso `1 / 3` normalmente aparece como `1/3`:

```epher
exact(0.3333333333333333)
exact(0.30000000000000004)
```

```text
1/3
3/10
```

Um valor irracional como `pi` não tem boa fração, então `exact()` o deixa como está.

Os verbos de formatação escrevem um número em outra notação. **scientific(x)** usa um dígito antes do expoente, **engineering(x)** expoentes em passos de três (a mantissa fica entre 1 e 1000), e **grouped(x)** insere espaços finos como separadores de milhares:

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

O aplicativo web e o TUI também oferecem essas opções de exibição (capítulos 2.2 e 5.2): frações exatas ligadas ou desligadas, notação Auto/científica/engenharia e separadores de milhares. Os ajustes só mudam a exibição; os valores continuam números decimais comuns.

### ### 1.13 Funções integradas

O epher tem as funções de uma calculadora científica, agrupadas por família.

A trigonometria funciona em radianos. Use `deg` e `rad` para converter:

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

Primos e divisores trabalham com números inteiros:

| Função | Significado | Exemplo | Resultado |
|---|---|---|---|
| `isprime(n)` | verdadeiro quando n é primo | `isprime(97)` | `true` |
| `nextprime(n)` / `prevprime(n)` | os primos mais próximos | `nextprime(10)` | `11` |
| `factors(n)` | fatoração em primos | `factors(360)` |
| Literal de lista | `{…}` | `{1, 2, 3}` |
| Elemento de lista | `list[i]` (base 1) | `{5, 6}[2]` |
| Estatística de lista | `mean(lista)`, `median(lista)`, … | `stdev(d)` |
| Forma de lista | `len(s)`, `sort(s)`, `mode(s)`, `range(s)`, `quartile(s, k)` | `quartile(d, 1)` |
| Regressão linear | `linreg(xs, ys)` | `linreg(x, y)` |
| Família normal | `normpdf` `normcdf` `invnorm` | `invnorm(0.975)` |
| Família t | `tpdf` `tcdf` `invt` | `invt(0.975, 10)` |
| Família qui-quadrado | `chi2pdf` `chi2cdf` `invchi2` | `chi2cdf(3.84, 1)` |
| Famílias discretas | `binompdf` `binomcdf` `poissonpdf` `poissoncdf` | `binomcdf(2, 10, 0.5)` |
| Testes e intervalos | `ztest` `ttest` `zinterval` `tinterval` `chisq_gof` | `tinterval(d, 0.95)` |
| Gráficos de dados | `graph scatter(xs, ys)` `histogram(data)` `boxplot(data)` | `graph boxplot(d)` |
| Números aleatórios | `random()`, `random(a, b)`, `randint(a, b)`, `randseed(n)` | `randint(1, 6)` |
| Explorador de constantes | Ajuda → Constantes: todas as constantes, agrupadas | Ajuda → Constantes |
| Grandeza | `5 m`, `60 mile/hr`, `1 km` | `2 m^2` |
| Converter | `expr in unidade` ou `expr -> unidade` | `72 km/hr in m/s` |
| Prefixos | `k M G T m µ n p` escalam qualquer unidade | `5 km`, `3 MPa`, `1 GHz` |
| E, OU bit a bit | `a & b`, `a \| b` | `0xFF & 0x0F` |
| OU exclusivo bit a bit | `a xor b` | `5 xor 3` |
| NÃO bit a bit | `~a` | `~0` |
| Deslocamentos | `a << n`, `a >> n` | `1 << 8` |
| Tamanho de palavra | `bits(n)` — 8, 16, 32, 64 | `bits(8)` | `2^3 * 3^2 * 5` |
| `totient(n)` | totiente de Euler | `totient(12)` | `4` |
| `ndivisors(n)` | quantos divisores n tem | `ndivisors(360)` | `24` |
| `modpow(b, e, m)` | b elevado a e, módulo m, exato | `modpow(2, 10, 1000)` | `24` |


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
| Primos | `isprime(n)`, `factors(n)`, … | `factors(360)` |
| `bin(x)` / `oct(x)` / `hex(x)` | grafia com prefixo na base 2 / 8 / 16 | `hex(255)` | `0xff` |

Combinam-se como tudo o resto:

```epher
min(sqrt(16), 5)
```

```text
4
```

As constantes físicas usam unidades SI, como as astronómicas da secção 1.16:

| Nome | Significado | Valor |
|---|---|---|
| `G` | constante gravitacional de Newton | 6.6743e-11 |
| `gamma` | constante de Euler-Mascheroni | 0.5772156649015329 |
| `q_e` | carga elementar | 1.602176634e-19 |
| `ev` | elétron-volt, em joules | 1.602176634e-19 |
| `eps_0` | permissividade do vácuo | 8.8541878128e-12 |
| `mu_0` | permeabilidade do vácuo | 1.25663706212e-6 |
| `z_0` | impedância do vácuo | 376.730313668 |
| `m_e` | massa do elétron | 9.1093837139e-31 |
| `m_p` | massa do protão | 1.67262192595e-27 |
| `m_n` | massa do neutrão | 1.67492750056e-27 |
| `m_u` | unidade de massa atómica | 1.66053906892e-27 |
| `a_0` | raio de Bohr | 5.29177210544e-11 |
| `alpha` | constante de estrutura fina | 0.0072973525643 |
| `r_inf` | constante de Rydberg | 10973731.568160 |
| `mu_b` | magnéton de Bohr | 9.2740100783e-24 |
| `n_a` | constante de Avogadro | 6.02214076e23 |
| `faraday` | constante de Faraday, C/mol | 96485.33212 |
| `r_gas` | constante molar dos gases | 8.31446261815324 |
| `atm` | atmosfera padrão, em pascais | 101325 |
| `wien` | constante de comprimento de onda de Wien | 0.002897771955 |
| `phi_0` | quanto de fluxo magnético | 2.067833848e-15 |
| `m_P` | Planck-Masse | 2.176434e-8 |
| `l_P` | Planck-Länge | 1.616255e-35 |
| `t_P` | Planck-Zeit | 5.391247e-44 |
| `r_e` | klassischer Elektronenradius | 2.8179403205e-15 |
| `lambda_c` | Compton-Wellenlänge | 2.42631023867e-12 |
| `mu_n` | Kernmagneton | 5.050783699e-27 |


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
| Porcentagem | `%` (pós-fixo) | `200 * (1 + 10%)` |
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
| Reconstruir uma fração | `exact(x)` | `exact(0.3333333333333333)` |
| Científica, engenharia, agrupada | `scientific(x)` `engineering(x)` `grouped(x)` | `engineering(12345)` |
| Unidade imaginária | `i`, ou um literal `4i` | `sqrt(-1)` |
| Partes de um complexo | `re(z)` `im(z)` `arg(z)` `conj(z)` `abs(z)` | `re(3 + 4i)` |
| Resolver uma equação | `solve lhs == rhs` | `solve x^2 == 9` |
| Derivada numérica | `derivative(expr, x)` | `derivative(x^2, 3)` |
| Integral definida | `integral(expr, a, b)` | `integral(x^2, 0, 3)` |
| Binário, octal, hex | `0b…`, `0o…`, `0x…` | `0xFF + 0b1` |
| Grafia em base | `bin(x)`, `oct(x)`, `hex(x)` | `hex(255)` |

### 1.16 Astronomia e o sistema solar

O epher fala astronomia: sufixos de unidade, constantes físicas, funções
de calendário e tempo, e uma efeméride em direto para o Sol, a Lua, os
planetas e Plutão. Tudo funciona offline.

**Unidades que falam astronomia.** Escreva um número seguido de um sufixo
de unidade e o epher converte para unidades SI de imediato:

| Sufixo | Unidade | Converte para |
|---|---|---|
| `AU` ou `au` | unidade astronómica | metros |
| `pc` | parsec | metros |
| `ly` | ano-luz | metros |
| `deg` | grau | radianos |
| `arcmin`, `arcsec` | minuto e segundo de arco | radianos |
| `min`, `hr`, `d`, `yr` | minuto, hora, dia, ano juliano | segundos |
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

Os sufixos fazem parte da gramática: nenhuma constante própria pode mudar
o que `3.2 AU` significa, e o `h` continua a ser a constante de Planck; as
horas escrevem-se `hr`. As funções devolvem contagens em unidades
naturais; um sufixo converte uma contagem para SI, por isso `mag2jy(20)`
é uma contagem em janskys e `mag2jy(20) Jy` é o mesmo fluxo em watts por
metro quadrado hertz.

**Constantes astronómicas.** `au`, `pc`, `ly`, `c`, `g`, `h`, `h_bar`,
`k_b`, `sigma_sb`, `m_sun`, `r_sun`, `l_sun`, `m_earth`, `r_earth`, `m_moon`, `r_moon`
funcionam como `pi`, e pode sombreá-las com as suas próprias constantes.

**Datas e tempo.** `jd(y, m, d [, hr])` e `mjd(...)` convertem uma data do
calendário numa data juliana; `now()` lê o instante atual:

```epher
jd(2000, 1, 1, 12)
```

```text
2451545
```

`delta_t(jd)` é a correção TT - UT1, e `lst(jd, lon)` é o tempo sidéreo
local em horas para uma longitude leste em graus.

**Horas, minutos e segundos.** `hms2deg(h, m, s)` converte ascensão reta
em graus, `dms2deg(d, m, s)` um ângulo sexagesimal, e `deg2hms(x)` /
`deg2dms(x)` escrevem um ângulo de volta como texto:

```epher
deg2hms(90)
```

```text
6h 0m 0s
```

**O céu, quantificado.** Dê a cada função um número de corpo: Mercúrio 1,
Vénus 2, Marte 4, Júpiter 5, Saturno 6, Urano 7, Neptuno 8, Plutão 9,
Sol 10, Lua 11 (a Terra é 3, a observadora, nunca um alvo).

| Função | Significado |
|---|---|
| `ra(b, jd)`, `decl(b, jd)` | ascensão reta e declinação geocêntricas (graus) |
| `dist(b, jd)` | distância em UA |
| `alt(b, jd, lat, lon)`, `az(b, jd, lat, lon)` | altitude e azimute topocêntricos (graus, verdadeiros) |
| `rise(b, jd, lat, lon)`, `set(...)`, `transit(...)` | eventos do dia solar local, como datas julianas |
| `mag(b, jd)` | magnitude aparente |
| `phase(b, jd)`, `illum(b, jd)` | ângulo de fase (graus) e fração iluminada |
| `diam(b, jd)` | diâmetro angular (graus) |

```epher
decl(10, jd(2000, 6, 21, 1.8))
```

```text
23.437882351
```

Latitudes e longitudes são graus, leste positivo. As posições são
geocêntricas salvo observador dado. Plutão usa uma órbita aproximada,
honesta a cerca de um minuto de arco, muito abaixo da precisão dos outros
corpos; eclipses e pesquisas de conjunções não estão incluídos.

**Ótica e luz.** `kepler(M, e)` resolve a equação de Kepler, `airmass(alt)`
é a massa de ar sec(z), `dawes(d)` é o poder de resolução de uma abertura
de d milímetros em segundos de arco, e `dist_mod(mu)` converte um módulo
de distância em parsecs.

**Estações.** `march_equinox(year)`, `june_solstice(year)`,
`september_equinox(year)` e `december_solstice(year)` devolvem a data
juliana de cada mudança de estação:

```epher
march_equinox(2000)
```

```text
2451623.8159797275
```

**O sistema solar em 3D.** O comando `solar3d` desenha todo o sistema:
cada órbita como uma curva, cada corpo como um ponto etiquetado, com um
rasto a mostrar onde ele estava:

```epher
solar3d jd(2020, 7, 1)
```

Dê o tempo como uma constante e prima o botão de reprodução para ver os
planetas mover-se: `const t = now(); solar3d t`. Arraste ou use as
setas para rodar, `clear` para esvaziar e `solar3d save file.svg` para
exportar.

A efeméride é calculada pelo crate solar-ephemeris
(github.com/Protonmatter/sol), validado contra o JPL Horizons; obrigado ao
seu autor. A precisão é de classe arcsecond para o Sol, a Lua e os
planetas ao longo de cerca de 5000 anos em torno do presente.

### 1.17 Números complexos

epher calcula com números complexos automaticamente. A unidade imaginária é **i**, exatamente como `pi`:

```epher
i ^ 2
sqrt(-1)
```

```text
-1
i
```

Escreva um número complexo com o sufixo `i`, sem sinal de multiplicação: `3 + 4i` é um literal, `2.5i` funciona, e os literais com base também (`0xFFi`). A aritmética usual se estende: somar, subtrair, multiplicar, dividir e potências funcionam, e `i` segue a precedência normal (`i ^ 2` se liga como qualquer potência).

As funções reais também se estendem. Com um argumento complexo elas calculam no plano complexo; com um argumento real fora do domínio real devolvem o resultado complexo principal em vez de um erro:

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

(`exp(i * pi)` é exatamente `-1`; os últimos dígitos são o ruído de `sin(pi)` na aritmética do computador.)

Quatro funções leem as partes de um número complexo, e `abs()` é o seu módulo:

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

As funções só de inteiros (`fact`, `gcd`, `floor`, `isprime`, ...) rejeitam argumentos complexos com um erro de tipo.

### 1.18 Resolver equações

**solve** encontra as raízes de uma equação em uma variável. A equação usa `==`:

```epher
solve x^2 == 5*x + 6
```

```text
x = -1, x = 6
```

Equações polinomiais (construídas com `+ - * ^` e constantes) dão todas as raízes, reais e complexas:

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

A variável a resolver é `x` quando aparece, senão a única outra variável. Constantes e variáveis ligadas agem como parâmetros:

```epher
const k = 3
solve k*x == 12
```

```text
x = 4
```

Qualquer outra equação é varrida numericamente em -100..100: as raízes são enquadradas por mudanças de sinal, então `solve sin(x) == 0.5` lista cada raiz do intervalo. Dois limites honestos: uma raiz onde a função apenas toca o zero (como `x^2 == 0` pelo caminho numérico) pode ser perdida, e equações com várias variáveis não ligadas são um erro.

### 1.19 Cálculo: derivada e integral

**derivative(expr, p)** é a derivada numérica de `expr` em `p`. O primeiro argumento continua uma expressão, e a sua variável livre é a que se deriva:

```epher
derivative(x^2, 3)
derivative(sin(t), 0)
```

```text
6
1
```

Como o argumento continua uma expressão, a derivada pode ser grafada: `graph derivative(x^3 - x, x)` desenha a curva das inclinações.

**integral(expr, a, b)** é a integral definida de `a` a `b`, calculada por quadratura adaptativa de Simpson:

```epher
integral(x^2, 0, 3)
integral(sin(x), 0, pi)
```

```text
9
2
```

`integral(x^2, 3, 0)` é `-9` (a integral com sinal), e um limite superior gravável funciona: `graph integral(x^2, 0, x)`.

Ambos são numéricos; as expressões precisam ter valores reais no intervalo, e uma expressão com várias variáveis é um erro.

### 1.20 Dados: listas, estatística e regressão

Uma lista é uma coluna de números entre chavetas: `{1, 2, 3}`. Os
elementos são expressões, a lista vazia `{}` é permitida, e uma lista
liga-se a um nome como qualquer valor:

```epher
d = {12, 15, 14, 16, 13, 15, 14, 17}
d[2]
len(d)
```

`list[i]` é o i-ésimo elemento, com base 1 como numa calculadora; um
índice fora da lista é um erro. O parêntese reto liga mais forte que
`^`, por isso `d[2]^2` é `(d[2])^2`.

A aritmética sobre uma lista é elemento a elemento, com um número
simples aplicado a cada elemento:

```epher
{1, 2, 3} * 2
{1, 2, 3} + 10
```

Duas listas têm de ter o mesmo comprimento para `+ - * / ^`. `==` e
`!=` comparam listas inteiras; as comparações de ordem rejeitam-nas.

As funções estatísticas aceitam uma lista como único argumento (a
forma com vários argumentos mantém-se — `mean(1, 2, 3)` continua a
funcionar): `sum product mean median mode variance stdev min max
range`. As novas funções de forma são `len(lista)`, `sort(lista)`
(cópia crescente), `mode(lista)` (valor mais frequente, o menor em
caso de empate), `range(lista)` (maior menos menor valor) e
`quartile(lista, k)` para k em 1..3 (quartis à moda TI, mediana das
metades):

```epher
mean(d)
median(d)
quartile(d, 1)
```

**linreg(xs, ys)** ajusta a reta dos mínimos quadrados a duas listas
do mesmo comprimento e informa-a com o coeficiente de correlação r:

```epher
linreg({1, 2, 3, 4}, {2.1, 4.2, 5.8, 8.1})
```

A reta ajustada é uma apresentação, como as raízes de solve; a imagem
do ajuste vive no gráfico de dispersão (secção 1.22).

### 1.21 Distribuições e testes de hipótese

As funções de probabilidade cobrem a normal padrão, a t de Student,
o qui-quadrado, a binomial e a de Poisson. A família normal aceita um
ou três argumentos — um só argumento é a normal padrão:

```epher
normcdf(1.96)
invnorm(0.975)
normcdf(12, 10, 2)
```

`normpdf(x[, mu, sigma])`, `normcdf(x[, mu, sigma])`, `invnorm(p[,
mu, sigma])`; `tpdf(x, df)`, `tcdf(x, df)`, `invt(p, df)`;
`chi2pdf(x, df)`, `chi2cdf(x, df)`, `invchi2(p, df)`;
`binompdf(k, n, p)`, `binomcdf(k, n, p)`; `poissonpdf(k, lambda)`,
`poissoncdf(k, lambda)`. As funções `inv*` respondem à pergunta
inversa: `invt(0.975, 10)` é o valor de t abaixo do qual está 97.5 %
da massa.

Os testes tomam uma lista de dados e informam o estatístico e o valor
p bilateral como texto de apresentação; os intervalos informam
`(baixo, alto)` no nível que nomear:

```epher
d = {12, 15, 14, 16, 13, 15, 14, 17}
ttest(d, 14)
tinterval(d, 0.95)
ztest(d, 14, 1.5)
chisq_gof({20, 30, 25, 25}, {25, 25, 25, 25})
```

`ttest(dados, mu0)` e `tinterval(dados, nível)` usam o desvio padrão
amostral (n−1); `ztest(dados, mu0, sigma)` e `zinterval(dados, sigma,
nível)` precisam do sigma conhecido. `chisq_gof(observados,
esperados)` é o teste de ajuste com k−1 graus de liberdade. Os
resultados são textos de apresentação: legíveis e copiáveis, mas a
aritmética não pode tocá-los.

### 1.22 Gráficos de dados

A família de gráficos também aceita listas: um gráfico de dispersão,
um histograma e um gráfico de caixa. Um gráfico de dados ocupa o
painel sozinho, como um sistema solar — o comando mais recente ganha,
e `graph clear` esvazia-o.

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

**scatter(xs, ys)** desenha os pontos e, com dois ou mais, a reta dos
mínimos quadrados, com a legenda `y = a*x + b (r = …)`.
**histogram(dados[, classes])** desenha um histograma de frequências;
o número de classes é opcional (regra de Sturges por predefinição) e
tem de ser um inteiro entre 1 e 50. **boxplot(dados)** desenha o
gráfico de caixa: mínimo, Q1, mediana, Q3, máximo, com bigodes até
aos extremos. A janela ajusta-se sempre aos dados — as palavras-chave
`from a to b` não se aplicam — e a imagem exporta-se e guarda-se como
qualquer gráfico.

### 1.23 Números aleatórios

`random()` sorteia um número uniforme em `[0, 1)`, `random(a, b)` um em
`[a, b)`, e `randint(a, b)` um inteiro do intervalo fechado `[a, b]` —
um lançamento de dados:

```epher
randseed(7)
randint(1, 6)
```

```text
7
3
```

A sequência é reproduzível: `randseed(n)` reinicia o gerador com `n` e
mostra-o, por isso a mesma semente repete os mesmos sorteios em cada
sessão e em cada interface.

### 1.24 Unidades e conversão

Um número seguido de uma unidade torna-se uma *grandeza*: o valor em
unidades SI mais as suas dimensões. A tabela de unidades cobre as
unidades SI de base e derivadas (`m`, `s`, `kg`, `A`, `K`, `mol`, `cd`,
`Hz`, `N`, `Pa`, `J`, `W`, `C`, `V`, `F`, `ohm`, `S`, `Wb`, `T`, `H`,
`lm`, `lx`, `Bq`, `Gy`, `Sv`), as unidades do dia a dia (`min`, `hr`,
`d`, `yr`, `L`, `t`, `bar`, `atm`, `torr`, `psi`, `eV`, `mile`, `yd`,
`ft`, `inch`, `nmi`, `lb`, `oz`, `gal`, `qt`, `pt`, `mph`, `knot`) e
os sufixos de astronomia da secção 1.16. Unidades compostas encadeiam:
`60 mile/hr` e `5 m/s^2` são unidades únicas.

```epher
60 mile/hr
```

```text
60 mile/hr
```

Os prefixos SI escalam qualquer uma delas: `k M G T m µ n p` são quilo,
mega, giga, tera, mili, micro, nano, pico — `5 km`, `3 MPa`, `1 GHz`
funcionam, e `2 kg` é o próprio quilograma.

As dimensões são verificadas: somar ou comparar grandezas com unidades
diferentes dá erro em vez de misturar metros e segundos:

```epher
5 m + 3 s
```

```text
error: dimension error: cannot add 5 m and 3 s
```

A aritmética compõe as dimensões: `5 m * 3 m` é `15 m^2`, `(3 m)^2` é
`9 m^2`, `sqrt(4 m^2)` é `2 m`, e uma expressão inteira cujas dimensões
se cancelam volta a ser um número vulgar (`5 m / 5 m` é `1`). Os
resultados preferem o nome derivado exato quando as dimensões
coincidem com um — `5 kg * 3 m / 1 s^2` responde `15 N`.

**Conversão.** `expr in unidade` (ou `expr -> unidade`) mostra uma
grandeza na unidade nomeada; as dimensões têm de coincidir. `in` liga
com a menor precedência dos operadores, por isso `5 m + 3 m in km`
converte a soma inteira:

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

As escalas de temperatura (Celsius, Fahrenheit) não são unidades aqui
— os kelvins são, e `K` funciona como qualquer outra.


### 1.25 Operações bit a bit

Os literais de base da secção 1.13 são feitos para isto: `0b101`,
`0o17`, `0xFF`. Os operadores bit a bit trabalham com números inteiros
e respondem com inteiros exatos:

```epher
0xFF & 0x0F
```

```text
15
```

| Operador | Significado |
|---|---|
| `a & b` | e bit a bit |
| `a \| b` | ou bit a bit |
| `a xor b` | ou exclusivo bit a bit |
| `~a` | não bit a bit (complemento para dois) |
| `a << n` | deslocar à esquerda (multiplicar por 2^n) |
| `a >> n` | deslocar à direita, aritmético (dividir por 2^n, arredondando para baixo) |

Os resultados são inteiros `big` exatos, por isso `1 << 60` conserva
cada dígito. O tamanho de palavra é de 64 bits por predefinição: os
resultados são lidos como complemento para dois com sinal, por isso
`~0` é -1 e `1 << 100` envolve em 0. `bits(n)` muda o tamanho de
palavra para 8, 16, 32 ou 64, e `bits()` informa-o:

```epher
bits(8)
~0
```

```text
8
-1
```

Um deslocamento negativo inverte a direção (`8 << -1` é `4`). O `and`
e o `or` booleanos mantêm os seus significados; `&` e `|` são as
grafias bit a bit.


## 2. A aplicação web (PWA)

### 2.1 Abri-la

A aplicação web está em:

```text
https://epher.org/pwa/
```

Não é necessária qualquer instalação. Funciona em qualquer navegador
moderno, num computador, telemóvel ou tablet.

Este guia também está integrado na aplicação: abra **Help → User guide**
na barra de menu (toque em **☰** num telemóvel) para o ler dentro da
aplicação, no idioma atual da aplicação. Toque num exemplo desse guia
para o carregar no campo de entrada. **Ajuda → Constantes** abre o explorador de constantes: todas as constantes agrupadas (Matemática, Astronomia, Física, Química), cada uma com o seu valor e uma breve descrição; toque numa para inserir o nome no campo de entrada, e a caixa de pesquisa filtra a lista.

### 2.2 O seu primeiro cálculo

1. Clique no campo de texto (já vem focado quando a página carrega).
2. Escreva uma expressão, por exemplo `2 + 3 * 4`.
3. Prima **Enter** ou clique no botão **=**.

O resultado aparece em texto grande por baixo do campo. Tudo o que está no
capítulo 1 funciona aqui, incluindo variáveis, funções e scripts.

Enquanto escreve um nome, aparece uma lista de sugestões por baixo do campo: as setas movem o realce, **Enter** ou **Tab** aceita, **Esc** fecha, e um clique aceita sem sair do teclado. Cada sugestão traz uma breve descrição da função ou constante. **F1** mostra a mesma descrição da palavra sob o cursor, na barra de dicas acima do teclado. Se o primeiro que escreve num campo vazio for um operador (`+ - * / ^ % !`), o epher insere `ans` por si, e a linha continua a partir da resposta anterior.

O menu **Ajustes** (o ícone de engrenagem, ou **☰ → Ajustes** no celular) tem três grupos. **Tema** e **Idioma** fazem o que os nomes dizem. **Resultados** define como as respostas aparecem: frações exatas (ligadas por padrão, então `1 / 3` aparece como `1/3`), a notação (Auto, científica ou engenharia) e separadores de milhares. São só ajustes de exibição; os valores continuam números comuns.

### 2.3 Histórico

Cada cálculo é adicionado à lista de histórico por baixo do resultado, para
poder recuar e ver o que fez. As entradas mais recentes aparecem no topo,
e o ícone do cesto de papel junto ao título **Histórico** esvazia-a
(no terminal, Ctrl+L ou um clique no mesmo ícone). O histórico é
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
a sua cor. As curvas são todas sólidas, e a legenda e as legendas
no gráfico são o que as distingue sem cor.
`graph clear` esvazia o gráfico, e um botão **Clear graph** no topo
do painel do gráfico faz o mesmo para curvas e superfícies 3D em
conjunto. A TUI mantém o comando no seu menu **Graph**.

No topo do painel de gráficos, ao lado de **Clear graph** e
**Copy SVG**, a barra de ferramentas permite ocultar a lista de pontos de
interesse e os pontos destacados desenhados no próprio gráfico. O controle
Mesmo por cima de cada gráfico há uma faixa de controles nomeados por
um ícone, com as palavras na respetiva dica: espessura da linha (0 a 4
em passos de 0.1 para curvas 2D, 0 a 0.2 em passos de 0.01 para
superfícies 3D - só é mostrado o do tipo em vista, e cada tipo lembra
o seu próprio valor), e nas vistas 3D e solar, a velocidade de rotação
horizontal e vertical e a velocidade de zoom. Cada
entrada da legenda tem uma caixa de verificação, marcada por omissão:
desmarcá-la esconde a curva do gráfico, dos seus pontos de interesse e da
exportação SVG.

```epher
graph x ^ 2
graph x ^ 3
```

Os pontos onde a expressão não tem valor (uma divisão por zero, por
exemplo) são ignorados, deixando um intervalo na curva. Um salto que é
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

**Rastrear:** mova o ponteiro sobre o gráfico, ou foque-o e prima as
teclas de seta. O ponto mais próximo de uma curva é marcado, com as
suas coordenadas anunciadas por baixo do gráfico.

**Pontos de interesse:** após cada comando graph o epher encontra as
raízes e os extremos de cada curva e as interseções entre curvas,
marca-os no gráfico e lista-os por baixo:

```text
root (-1, 0)   minimum (0, 0)   root (1, 0)
```

**Tabelas:** o comando `table` imprime uma tabela de valores (as linhas
onde a expressão não tem valor ficam em branco):

Uma cláusula opcional `derivative <expressão>` acrescenta uma
terceira coluna, a derivada numérica dessa expressão em cada x:

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

As células da tabela seguem os ajustes de resultados: com as
frações exatas ativadas (predefinição), um valor que é uma fração
simples mostra-se como tal — `table x / 3 from 0 to 1 points 4`
lista `1/3` em vez de `0.333`.
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
baixo do gráfico. Arraste-o (ou mova-o com as teclas de seta) e todas
as curvas são redesenhadas:

```epher
const a = 1
graph a * x ^ 2
```

**Copiar SVG** copia o gráfico atual como uma imagem SVG autossuficiente
para colar em documentos. As cores vão embutidas, então ele fica igual
em qualquer lugar. **Guardar PNG** guarda a mesma imagem como bitmap com o dobro do tamanho, para as curvas ficarem nítidas; a aplicação de Secretária pergunta onde guardá-la, e a do navegador guarda-a nas transferências (ou pergunta, onde o navegador o permite). As linhas de controlos e as constantes animadas ficam
diretamente por baixo do gráfico, acima da lista de pontos de interesse.

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
percorrer o intervalo do deslizador e recomeçar do início. É a forma
habitual de as calculadoras animarem: anima-se um parâmetro e tudo o que
o usa move-se. Prima o
botão novamente para fazer uma pausa.

Uma variável de "tempo" é apenas uma constante que anima:

```epher
const t = 0
graph sin(x - t)
```

Reproduzir o deslizador de t faz a onda viajar. As superfícies 3D animam
da mesma forma. Defina primeiro uma constante e depois reproduza o seu
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
aplicações. Abre instantaneamente, mesmo sem ligação à internet.

### 2.6 O que a aplicação web não faz

A aplicação web mantém o seu trabalho na sessão atual: avalia expressões,
desenha os seus gráficos (secção 2.4) e mantém um histórico. Os comandos
**save**, **save script** e **language** funcionam nas versões de
ambiente de trabalho, linha de comandos e terminal (capítulos 3, 4 e 5)
. Na aplicação web respondem com uma nota a dizer que guardar funciona
nessas versões. O histórico não é guardado entre visitas.

## 3. A aplicação de ambiente de trabalho

A aplicação de ambiente de trabalho é uma janela normal à volta da mesma
aplicação web. Tudo o que está no capítulo 2 se aplica; a única diferença é
a forma de a instalar e iniciar.

### 3.1 Instalar

Descarregue um instalador para o seu sistema a partir do site do epher:

- **Windows:** execute `epher-windows-x86_64.exe`. O instalador coloca o
  `epher` no seu PATH. Abra uma nova janela de CMD ou PowerShell e
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

- **Linux (qualquer distribuição, incluindo Arch):** o AppImage. Torne-o
  executável e execute-o:

```sh
chmod +x epher-linux-x86_64.AppImage
./epher-linux-x86_64.AppImage
```

Cada instalador contém o epher *completo*: a aplicação de ambiente de
trabalho, a linha de comandos (capítulo 4) e a interface de terminal
(capítulo 5), no único comando `epher`. No Linux, o pacote coloca o
`epher` em `/usr/bin`.

### 3.2 Usá-la

Inicie o epher como qualquer outra aplicação. Obtém uma janela com a mesma
interface da aplicação web: escreva uma expressão, prima **Enter** ou
clique em **=** e leia o resultado. Os gráficos também funcionam aqui.
`graph x ^ 2` desenha na janela (capítulo 2.4). A janela pode ser
redimensionada livremente. A barra de menu inclui
**Help → User guide**, o mesmo guia desta página, com exemplos que
se carregam com um toque.

Também pode abri-la a partir de um terminal: um `epher` simples (ou
`epher gui`) inicia a aplicação de ambiente de trabalho. No macOS, use o
botão **Install the epher command** dentro da aplicação para colocar o
`epher` no PATH do seu terminal.

### 3.3 Armazenamento: um só arquivo com a CLI e a TUI

A aplicação de ambiente de trabalho partilha o seu armazenamento com as
versões de linha de comandos e de terminal. Funções, constantes, scripts,
histórico e a preferência de idioma vivem num único lugar, `~/.epher` no
seu computador (ou `EPHER_STORE_DIR`, capítulo 4.6), e tudo o que for
guardado numa versão fica disponível nas outras:

```text
def area(w, h) = w * h
save area
```

Defina `area` na aplicação de ambiente de trabalho, guarde-a com `save`,
feche a janela. Depois abra a CLI e `area(3, 4)` funciona sem mais.
Também funciona ao contrário: as funções e os scripts que guardou na CLI
ou na TUI já lá estão quando a janela da aplicação de ambiente de trabalho
abre, incluindo variáveis definidas por scripts guardados. Os comandos
`save`, `save script` e `language` do capítulo 4 funcionam exatamente da
mesma forma aqui.

Os comandos que escreve na CLI, no REPL, na TUI ou na aplicação de
ambiente de trabalho juntam-se todos ao mesmo histórico, e a sessão
também viaja: as variáveis que define e o valor `ans` seguem-no de
uma versão para a seguinte. O armazenamento partilhado é vivo: com duas
versões abertas ao mesmo tempo, uma alteração numa reflete-se de imediato
na outra (a aplicação de ambiente de trabalho e a TUI observam o
armazenamento e atualizam-se sozinhas).

> A aplicação web no navegador é a única versão que não usa este
> armazenamento. Mantém cada sessão para si (capítulo 2.6).

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
do argumento. Tudo o que está no capítulo 1 está disponível: variáveis,
funções, ciclos, tudo. As linhas partilham uma única sessão, como um
script por pipe (secção 4.2).

### 4.2 Scripts por pipe

O `epher -` lê expressões da entrada padrão, uma linha de cada vez, a
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
e o script continua. Uma linha pode unir várias instruções com `;`. As
quebras de linha e o `;` significam a mesma coisa em todo o lado no epher.


Um arquivo funciona do mesmo jeito: `epher plots/sine.es` executa cada linha do arquivo em ordem e mostra cada resultado. O argumento é tratado como arquivo quando nomeia um arquivo existente e contém um `.`, `/` ou `\` - `epher x` continua avaliando o nome `x`.
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


O comando `load` executa um script - um caminho de arquivo ou o nome de um script salvo com `save script` - linha por linha, exatamente como se você o tivesse digitado:

```text
epher> load plots/sine.es
epher> load my_setup
```
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

As constantes guardam-se da mesma forma. `save` no nome da constante:

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


Você também pode carregar um script salvo quando quiser com `load count_to_five`, ou mantê-lo como arquivo simples e rodar `load count_to_five.es`; `epher count_to_five.es` o executa direto da linha de comando (secção 4.2).
### 4.5 Mudar o idioma da interface

O idioma da interface é escolhido a partir dos idiomas definidos no seu
dispositivo. Para o substituir, escreva `language` seguido de um destes:
`en`, `zh-CN`, `hi`, `es`, `fr`, `ar`, `de`, `pt`:

```text
epher> language fr
language set to fr
```

A escolha fica memorizada para a próxima vez. Nota: o idioma em que
*escreve*, a linguagem de expressões, é sempre o mesmo, em qualquer
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
terminal. Faz parte do mesmo programa `epher`. Inicie-a com:

```sh
epher tui
```

### 5.1 O ecrã

O ecrã está dividido em painéis:

- **Expressão**: a caixa de entrada (em cima). Shift+Enter começa uma
  nova linha; as teclas de seta ou um clique do rato movem o cursor
  dentro do texto.
- O **resultado** atual logo por baixo.
- **Histórico**: todas as linhas que introduziu, com a respetiva resposta.
- **Gráfico**: o gráfico do comando `graph` (em baixo).
- Uma linha de sugestões mostra os atalhos de teclado.

### 5.2 Teclas

| Tecla | Ação |
|---|---|
| Escrever | adicionar à expressão no cursor |
| **Enter** | avaliar todo o script (uma entrada multilinha corre como um item do histórico) |
| **Shift+Enter** | começar uma nova linha |
| **← → ↑ ↓** | mover o cursor (com a entrada vazia: rodar a vista 3D) |
| **Esc** | limpar a linha de entrada |
| **F1** | descrever a função sob o cursor (na linha de resultado) |
| **Ctrl+C** | sair |
| **q** | sair (quando a entrada está vazia) |
| **Arrow keys** | rodar a vista 3D (quando a entrada está vazia) |
| **Space** | iniciar/parar a animação (quando a entrada está vazia) |
| **F10** | abrir os menus (Ficheiro, Editar, Gráfico, Definições, Ajuda) |
| **Tab** | focar o teclado sempre visível (ou o histórico, a partir do teclado); trocar de grupo (**Esc** volta à escrita) |
| **Rato** | clique menus e itens de menu, células e separadores do teclado, linhas do histórico (carrega a expressão); arraste o painel do gráfico para orbitar (3D) ou mover (2D), a roda faz zoom, um duplo clique repõe a vista |
| **Ctrl+L** | limpar o histórico |

O menu **Ajuda** abre o guia integrado, a ajuda de teclas do teclado e um explorador de constantes: as constantes agrupadas, as setas escolhem uma linha, **Enter** insere o seu nome na expressão no cursor e **Esc** fecha.

Os grupos do teclado cobrem todas as funções, constantes e comandos que
a linguagem suporta: **trig**, **fn**, **num**, **0x** e **var**. O
grupo 0x contém as conversões exatas e de base (`frac`, `dec`,
`big`, `bin`, `oct`, `hex`) e o fatorial `!`. As setas movem
o realce, **Enter** insere o token e a **Tab** troca de grupo. Um operador no início de uma linha vazia (ou inserido pelo teclado) acrescenta `ans` antes, e a linha continua a partir da resposta anterior.

O menu **Ajustes** oferece as mesmas opções de exibição de resultados que o aplicativo web (frações exatas, notação, separadores de milhares), junto às linhas de tema e idioma.

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
funções. Primeiro defina uma, depois desenhe o gráfico:

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
comando graph, a TUI lista os pontos de interesse (raízes, extremos e
interseções) por baixo do gráfico. O comando `table` (secção 2.4.2)
também funciona aqui.

`graph3d x ^ 2 - y ^ 2` desenha uma superfície 3D como uma malha de
arame em ASCII. Rode-a com as teclas de seta enquanto a entrada estiver
vazia, e prima a barra de espaço para animar uma constante de deslizador
(secção 2.4.5). A linha de ajudas no fundo mostra as indicações de setas
e espaço apenas enquanto estiver visível uma superfície 3D ou uma curva
animável.

`graph save plot.svg` grava o gráfico atual como a mesma imagem SVG que
o botão **Copiar SVG** da aplicação web produz; `graph3d
save arquivo.svg` salva a malha 3D do ângulo em que você a está vendo.

### 5.4 Guardar e persistência

A TUI partilha o armazenamento com a CLI: tudo o que for guardado numa fica
disponível na outra. As funções, os scripts, o histórico e a preferência de
idioma vivem em `~/.epher` (capítulo 4.6), e os mesmos comandos `save`,
`save script` e `language` funcionam aqui.

## 6. Os seus dados e a privacidade

- O **programa epher instalado** (aplicação de ambiente de trabalho, CLI e
  TUI) guarda as funções, os scripts, o histórico e a escolha de idioma
  localmente em `~/.epher` (ou `EPHER_STORE_DIR`). Nada sai do seu
  computador.
- A **aplicação web** não guarda nada em disco: o histórico dura apenas
  enquanto a página estiver aberta. A aplicação web pode funcionar offline
  porque a própria página é guardada pelo seu navegador.

As cinco versões executam o cálculo inteiramente no seu dispositivo. Nada
é enviado para lado nenhum.
