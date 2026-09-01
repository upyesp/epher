# -*- coding: utf-8 -*-
# Round 3 guide section translations (prose; {F} fence placeholders).
# Consumed by scripts/round3-guide-translate.py.

SECTIONS["de"] = """### 1.20 Daten: Listen, Statistik und Regression

Eine Liste ist eine Zahlenreihe in geschweiften Klammern: `{1, 2, 3}`.
Die Elemente sind Ausdrücke, die leere Liste `{}` ist erlaubt, und eine
Liste wird wie jeder Wert an einen Namen gebunden:

%%PROBE_LIST%%

`list[i]` ist das i-te Element, 1-basiert wie ein Taschenrechner es
erwartet; ein Index außerhalb der Liste ist ein Fehler. Die Klammer
bindet enger als `^`, also ist `d[2]^2` gleich `(d[2])^2`.

Die Arithmetik über einer Liste ist elementweise; eine einzelne Zahl
wird auf jedes Element angewendet:

%%ARITH%%

Zwei Listen müssen für `+ - * / ^` gleich lang sein. `==` und `!=`
vergleichen ganze Listen; Ordnungsvergleiche lehnen Listen ab.

Die Statistikfunktionen nehmen eine Liste als einziges Argument (die
Mehrfachargument-Form bleibt — `mean(1, 2, 3)` funktioniert weiter):
`sum product mean median mode variance stdev min max range`. Die
neuen Formfunktionen sind `len(liste)`, `sort(liste)` (aufsteigende
Kopie), `mode(liste)` (häufigster Wert, bei Gleichstand der kleinste),
`range(liste)` (größter minus kleinster Wert) und `quartile(liste, k)`
für k in 1..3 (Quartile nach TI-Art, Median der Hälften):

%%QUART%%

**linreg(xs, ys)** passt die Ausgleichsgerade durch zwei gleich lange
Listen an und berichtet sie mit dem Korrelationskoeffizienten r:

%%LINREG%%

Die angepasste Gerade ist eine Anzeige wie die Lösungen von solve; das
Bild der Anpassung zeigt das Streudiagramm (Abschnitt 1.22).

### 1.21 Verteilungen und Hypothesentests

Die Wahrscheinlichkeitsfunktionen decken die Standardnormal-, die
t-, die Chi-Quadrat-, die Binomial- und die Poisson-Verteilung ab. Die
Normal-Familie nimmt ein oder drei Argumente — ein Argument ist die
Standardnormalverteilung:

%%NORM%%

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

%%TESTS%%

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

%%SCATTER%%

%%HISTOGRAM%%

%%BOXPLOT%%

**scatter(xs, ys)** zeichnet die Punkte und, ab zwei Punkten, die
Ausgleichsgerade mit der Beschriftung `y = a*x + b (r = …)` in der
Legende. **histogram(daten[, bins])** zeichnet ein
Häufigkeitshistogramm; die Klassenzahl ist optional (standardmäßig
nach Sturgess Regel) und muss eine ganze Zahl zwischen 1 und 50 sein.
**boxplot(daten)** zeichnet das Kastendiagramm: Minimum, Q1, Median,
Q3, Maximum, mit Antennen bis zu den Extremen. Das Fenster passt sich
immer den Daten an — die `from a to b`-Schlüsselwörter gelten nicht —
und das Bild exportiert und speichert wie jeder andere Plot.
"""

SECTIONS["fr"] = """### 1.20 Données : listes, statistiques et régression

Une liste est une colonne de nombres entre accolades : `{1, 2, 3}`.
Les éléments sont des expressions, la liste vide `{}` est admise, et
une liste se lie à un nom comme n'importe quelle valeur :

%%PROBE_LIST%%

`list[i]` est le i-ème élément, indexé à partir de 1 comme sur une
calculatrice ; un index hors de la liste est une erreur. Le crochet
lie plus fort que `^`, donc `d[2]^2` vaut `(d[2])^2`.

L'arithmétique sur une liste est élément par élément, un simple
nombre s'appliquant à chaque élément :

%%ARITH%%

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

%%QUART%%

**linreg(xs, ys)** ajuste la droite des moindres carrés à deux listes
de même longueur et la rapporte avec le coefficient de corrélation r :

%%LINREG%%

La droite ajustée est un affichage, comme les racines de solve ; le
dessin de l'ajustement vit sur le nuage de points (section 1.22).

### 1.21 Distributions et tests d'hypothèse

Les fonctions de probabilité couvrent la normale centrée réduite, la
loi de Student, le khi-deux, la loi binomiale et la loi de Poisson.
La famille normale prend un ou trois arguments — un seul argument est
la normale centrée réduite :

%%NORM%%

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

%%TESTS%%

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

%%SCATTER%%

%%HISTOGRAM%%

%%BOXPLOT%%

**scatter(xs, ys)** trace les points et, à partir de deux points, la
droite des moindres carrés, légendée `y = a*x + b (r = …)`.
**histogram(données[, classes])** trace un histogramme de fréquences ;
le nombre de classes est facultatif (règle de Sturges par défaut) et
doit être un entier entre 1 et 50. **boxplot(données)** trace la boîte
à moustaches : minimum, Q1, médiane, Q3, maximum, moustaches jusqu'aux
extrêmes. La fenêtre s'ajuste toujours aux données — les mots-clés
`from a to b` ne s'appliquent pas — et l'image s'exporte et se
sauvegarde comme n'importe quel graphique.
"""

SECTIONS["es"] = """### 1.20 Datos: listas, estadística y regresión

Una lista es una columna de números entre llaves: `{1, 2, 3}`. Los
elementos son expresiones, la lista vacía `{}` está permitida, y una
lista se asigna a un nombre como cualquier valor:

%%PROBE_LIST%%

`list[i]` es el elemento i-ésimo, con base 1 como en una calculadora;
un índice fuera de la lista es un error. El corchete une más fuerte
que `^`, así que `d[2]^2` es `(d[2])^2`.

La aritmética sobre una lista es elemento a elemento, y un número solo
se aplica a cada elemento:

%%ARITH%%

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

%%QUART%%

**linreg(xs, ys)** ajusta la recta de mínimos cuadrados a dos listas
de la misma longitud y la informa con el coeficiente de correlación r:

%%LINREG%%

La recta ajustada es una presentación, como las raíces de solve; la
imagen del ajuste vive en el gráfico de dispersión (sección 1.22).

### 1.21 Distribuciones y pruebas de hipótesis

Las funciones de probabilidad cubren la normal estándar, la t de
Student, la chi-cuadrado, la binomial y la de Poisson. La familia
normal admite uno o tres argumentos — un solo argumento es la normal
estándar:

%%NORM%%

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

%%TESTS%%

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

%%SCATTER%%

%%HISTOGRAM%%

%%BOXPLOT%%

**scatter(xs, ys)** dibuja los puntos y, con dos o más, la recta de
mínimos cuadrados, con la leyenda `y = a*x + b (r = …)`.
**histogram(datos[, clases])** dibuja un histograma de frecuencias; el
número de clases es opcional (regla de Sturges por defecto) y debe ser
un entero entre 1 y 50. **boxplot(datos)** dibuja el diagrama de caja:
mínimo, Q1, mediana, Q3, máximo, con bigotes hasta los extremos. La
ventana siempre se ajusta a los datos — las palabras clave `from a to
b` no se aplican — y la imagen se exporta y guarda como cualquier
gráfico.
"""

SECTIONS["pt"] = """### 1.20 Dados: listas, estatística e regressão

Uma lista é uma coluna de números entre chavetas: `{1, 2, 3}`. Os
elementos são expressões, a lista vazia `{}` é permitida, e uma lista
liga-se a um nome como qualquer valor:

%%PROBE_LIST%%

`list[i]` é o i-ésimo elemento, com base 1 como numa calculadora; um
índice fora da lista é um erro. O parêntese reto liga mais forte que
`^`, por isso `d[2]^2` é `(d[2])^2`.

A aritmética sobre uma lista é elemento a elemento, com um número
simples aplicado a cada elemento:

%%ARITH%%

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

%%QUART%%

**linreg(xs, ys)** ajusta a reta dos mínimos quadrados a duas listas
do mesmo comprimento e informa-a com o coeficiente de correlação r:

%%LINREG%%

A reta ajustada é uma apresentação, como as raízes de solve; a imagem
do ajuste vive no gráfico de dispersão (secção 1.22).

### 1.21 Distribuições e testes de hipótese

As funções de probabilidade cobrem a normal padrão, a t de Student,
o qui-quadrado, a binomial e a de Poisson. A família normal aceita um
ou três argumentos — um só argumento é a normal padrão:

%%NORM%%

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

%%TESTS%%

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

%%SCATTER%%

%%HISTOGRAM%%

%%BOXPLOT%%

**scatter(xs, ys)** desenha os pontos e, com dois ou mais, a reta dos
mínimos quadrados, com a legenda `y = a*x + b (r = …)`.
**histogram(dados[, classes])** desenha um histograma de frequências;
o número de classes é opcional (regra de Sturges por predefinição) e
tem de ser um inteiro entre 1 e 50. **boxplot(dados)** desenha o
gráfico de caixa: mínimo, Q1, mediana, Q3, máximo, com bigodes até
aos extremos. A janela ajusta-se sempre aos dados — as palavras-chave
`from a to b` não se aplicam — e a imagem exporta-se e guarda-se como
qualquer gráfico.
"""

SECTIONS["zh-CN"] = """### 1.20 数据：列表、统计与回归

列表是大括号中的一列数字：`{1, 2, 3}`。元素是表达式，允许空列表
`{}`，列表可以像任何值一样绑定到名称：

%%PROBE_LIST%%

`list[i]` 是第 i 个元素，从 1 开始计数，与计算器一致；索引越界是
错误。方括号比 `^` 绑定更紧，所以 `d[2]^2` 是 `(d[2])^2`。

对列表的运算是逐元素进行的，单个数字会广播到每个元素：

%%ARITH%%

两个列表做 `+ - * / ^` 运算时长度必须相同。`==` 和 `!=` 比较整个
列表；排序比较不接受列表。

统计函数接受一个列表作为唯一参数（多参数形式保留——`mean(1, 2,
3)` 仍然可用）：`sum product mean median mode variance stdev min max
range`。新的形态函数有 `len(列表)`、`sort(列表)`（升序副本）、
`mode(列表)`（出现最多的值，并列时取最小）、`range(列表)`（最大
减最小）和 `quartile(列表, k)`（k 为 1..3，TI 式四分位，取两半的
中位数）：

%%QUART%%

**linreg(xs, ys)** 对两个等长列表拟合最小二乘直线，并连同相关系数
r 一起报告：

%%LINREG%%

拟合直线是一种显示结果，就像 solve 的根；拟合的图形在散点图上
（第 1.22 节）。

### 1.21 分布与假设检验

概率函数覆盖标准正态、t、卡方、二项和泊松分布族。正态族接受一个
或三个参数——一个参数即标准正态：

%%NORM%%

`normpdf(x[, mu, sigma])`、`normcdf(x[, mu, sigma])`、`invnorm(p[,
mu, sigma])`；`tpdf(x, df)`、`tcdf(x, df)`、`invt(p, df)`；
`chi2pdf(x, df)`、`chi2cdf(x, df)`、`invchi2(p, df)`；
`binompdf(k, n, p)`、`binomcdf(k, n, p)`；`poissonpdf(k, lambda)`、
`poissoncdf(k, lambda)`。`inv*` 函数回答相反的问题：`invt(0.975,
10)` 是下方有 97.5% 质量的 t 值。

检验函数接受一个数据列表，以显示文本报告统计量和双侧 p 值；区间
函数在您指定的水平上报告 `(下, 上)`：

%%TESTS%%

`ttest(数据, mu0)` 和 `tinterval(数据, 水平)` 使用样本标准差
(n−1)；`ztest(数据, mu0, sigma)` 和 `zinterval(数据, sigma, 水平)`
需要已知的 sigma。`chisq_gof(观测值, 期望值)` 是自由度为 k−1 的拟
合优度检验。结果是显示文本：可读、可复制，但算术不能使用它们。

### 1.22 数据图

绘图家族也接受列表：散点图、直方图和箱线图。数据图独占面板，就像
太阳系一样——最新命令胜出，`graph clear` 清空它。

%%SCATTER%%

%%HISTOGRAM%%

%%BOXPLOT%%

**scatter(xs, ys)** 绘制点，当点数不少于两个时绘制最小二乘拟合
直线，图例标注为 `y = a*x + b (r = …)`。**histogram(数据[,
组数])** 绘制频率直方图；组数可选（默认使用斯特吉斯规则），必须是
1 到 50 之间的整数。**boxplot(数据)** 绘制箱线图：最小值、Q1、
中位数、Q3、最大值，须线延伸到两端。窗口始终适应数据——`from a
to b` 域关键字不适用——图像像任何其他图一样导出和保存。
"""

SECTIONS["hi"] = """### 1.20 डेटा: सूचियाँ, सांख्यिकी और प्रतिगमन

सूची घुंघराले कोष्ठकों में संख्याओं का एक स्तंभ है: `{1, 2, 3}`।
तत्व व्यंजक हैं, खाली सूची `{}` मान्य है, और सूची किसी भी मान की
तरह एक नाम से बंधती है:

%%PROBE_LIST%%

`list[i]` i-वाँ तत्व है, कैलकुलेटर की तरह 1-आधारित; सूची से बाहर का
सूचकांक एक त्रुटि है। कोष्ठक `^` से अधिक कसकर बंधता है, इसलिए
`d[2]^2` का अर्थ `(d[2])^2` है।

सूची पर अंकगणित तत्व-वार होता है, और एक अकेली संख्या हर तत्व पर
लागू होती है:

%%ARITH%%

`+ - * / ^` के लिए दो सूचियों की लंबाई समान होनी चाहिए। `==` और
`!=` पूरी सूचियों की तुलना करते हैं; क्रम तुलना सूचियों को अस्वीकार
करती है।

सांख्यिकी फलन एक सूची को एकमात्र तर्क के रूप में लेते हैं (बहु-तर्क
रूप बना रहता है — `mean(1, 2, 3)` अब भी काम करता है): `sum product
mean median mode variance stdev min max range`। नए आकार फलन हैं
`len(सूची)`, `sort(सूची)` (आरोही प्रति), `mode(सूची)` (सबसे बार-बार
आने वाला मान, बराबरी पर सबसे छोटा), `range(सूची)` (सबसे बड़ा घटा
सबसे छोटा) और `quartile(सूची, k)` जहाँ k = 1..3 (TI-शैली चतुर्थक,
आधे-आधे का माध्यिका):

%%QUART%%

**linreg(xs, ys)** दो समान-लंबाई सूचियों में न्यूनतम वर्ग रेखा फिट
करता है और सहसंबंध गुणांक r के साथ बताता है:

%%LINREG%%

फिट की गई रेखा एक प्रदर्शन है, जैसे solve के मूल; फिट की तस्वीर
स्कैटर प्लॉट में दिखती है (अनुभाग 1.22)।

### 1.21 वितरण और परिकल्पना परीक्षण

प्रायिकता फलन मानक सामान्य, स्टूडेंट t, काई-वर्ग, द्विपद और पॉइसन
परिवारों को कवर करते हैं। सामान्य परिवार एक या तीन तर्क लेता है —
एक तर्क मानक सामान्य है:

%%NORM%%

`normpdf(x[, mu, sigma])`, `normcdf(x[, mu, sigma])`, `invnorm(p[,
mu, sigma])`; `tpdf(x, df)`, `tcdf(x, df)`, `invt(p, df)`;
`chi2pdf(x, df)`, `chi2cdf(x, df)`, `invchi2(p, df)`;
`binompdf(k, n, p)`, `binomcdf(k, n, p)`; `poissonpdf(k, lambda)`,
`poissoncdf(k, lambda)`। `inv*` फलन उल्टा प्रश्न पूछते हैं:
`invt(0.975, 10)` वह t मान है जिसके नीचे 97.5% द्रव्यमान है।

परीक्षण एक डेटा सूची लेते हैं और आँकड़ा और दो-पक्षीय p-मान एक
प्रदर्शन पाठ के रूप में बताते हैं; अंतराल आपके नामित स्तर पर
`(निचला, ऊपरी)` बताते हैं:

%%TESTS%%

`ttest(डेटा, mu0)` और `tinterval(डेटा, स्तर)` नमूना मानक विचलन
(n−1) उपयोग करते हैं; `ztest(डेटा, mu0, sigma)` और `zinterval(डेटा,
sigma, स्तर)` को ज्ञात sigma चाहिए। `chisq_gof(प्रेक्षित, अपेक्षित)`
k−1 स्वतंत्रता-कोटि वाला अनुकूलता परीक्षण है। परिणाम प्रदर्शन पाठ
हैं: पठनीय और कॉपी-योग्य, पर अंकगणित उन्हें छू नहीं सकता।

### 1.22 डेटा प्लॉट

ग्राफ़ परिवार सूचियाँ भी लेता है: स्कैटर, हिस्टोग्राम और बॉक्स
प्लॉट। एक डेटा प्लॉट पैन पर अकेला होता है जैसे सौर मंडल — सबसे
नया आदेश जीतता है, और `graph clear` उसे खाली करता है।

%%SCATTER%%

%%HISTOGRAM%%

%%BOXPLOT%%

**scatter(xs, ys)** बिंदु बनाता है और, दो या अधिक बिंदुओं पर,
न्यूनतम वर्ग रेखा, लेजेंड में `y = a*x + b (r = …)` शीर्षक के
साथ। **histogram(डेटा[, bins])** आवृत्ति हिस्टोग्राम बनाता है;
bin गिनती वैकल्पिक है (डिफ़ॉल्ट स्टर्जेस नियम) और 1 से 50 के बीच
पूर्ण संख्या होनी चाहिए। **boxplot(डेटा)** बॉक्स-एंड-व्हिस्कर
बनाता है: न्यूनतम, Q1, माध्यिका, Q3, अधिकतम, छोर तक व्हिस्कर।
खिड़की हमेशा डेटा में फिट होती है — `from a to b` डोमेन कीवर्ड
लागू नहीं होते — और चित्र किसी भी अन्य प्लॉट की तरह निर्यात और
सहेजता है।
"""

SECTIONS["ar"] = """### 1.20 البيانات: القوائم والإحصاء والانحدار

القائمة هي عمود من الأرقام بين قوسين معقوفين: `{1, 2, 3}`. العناصر هي
تعبيرات، والقائمة الفارغة `{}` مسموحة، والقائمة ترتبط باسم مثل أي قيمة:

%%PROBE_LIST%%

`list[i]` هو العنصر رقم i، يبدأ العد من 1 كما في الآلة الحاسبة؛
المؤشر خارج القائمة خطأ. القوس المربع يلتصق أقوى من `^`، لذا `d[2]^2`
هي `(d[2])^2`.

العمليات الحسابية على القائمة عنصرًا بعنصر، ويُطبَّق الرقم المفرد على
كل عنصر:

%%ARITH%%

يجب أن يكون طولا القائمتين متساويين في `+ - * / ^`. يقارن `==` و
`!=` القائمتين كاملتين؛ ومقارنات الترتيب ترفض القوائم.

تأخذ دوال الإحصاء قائمة واحدة كوسيط وحيد (الصيغة متعددة الوسائط باقية —
`mean(1, 2, 3)` ما زالت تعمل): `sum product mean median mode variance
stdev min max range`. دوال الشكل الجديدة هي `len(قائمة)` و`sort(قائمة)`
(نسخة تصاعدية) و`mode(قائمة)` (القيمة الأكثر تكرارًا، والأصغر عند
التعادل) و`range(قائمة)` (أكبر ناقص أصغر قيمة) و`quartile(قائمة, k)`
حيث k من 1 إلى 3 (ربيعيات بأسلوب TI، وسيط النصفين):

%%QUART%%

**linreg(xs, ys)** يلائم خط المربعات الصغرى لقائمتين متساويتي الطول
ويُبلغ عنه مع معامل الارتباط r:

%%LINREG%%

الخط الملائم عرضٌ، مثل جذور solve؛ صورة الملاءمة تعيش في مخطط
التشتت (القسم 1.22).

### 1.21 التوزيعات واختبارات الفرضيات

تغطي دوال الاحتمال التوزيع الطبيعي المعياري و t لستودنت وكاي تربيع
وذات الحدين وبوزون. تأخذ عائلة التوزيع الطبيعي وسيطًا واحدًا أو
ثلاثة — الوسيط الواحد هو الطبيعي المعياري:

%%NORM%%

`normpdf(x[, mu, sigma])` و`normcdf(x[, mu, sigma])` و`invnorm(p[,
mu, sigma])`؛ و`tpdf(x, df)` و`tcdf(x, df)` و`invt(p, df)`؛
و`chi2pdf(x, df)` و`chi2cdf(x, df)` و`invchi2(p, df)`؛
و`binompdf(k, n, p)` و`binomcdf(k, n, p)`؛ و`poissonpdf(k, lambda)`
و`poissoncdf(k, lambda)`. تجيب دوال `inv*` عن السؤال المعاكس:
`invt(0.975, 10)` هي قيمة t التي تحتها 97.5% من الكتلة.

تأخذ الاختبارات قائمة بيانات وتُبلغ عن الإحصاء وقيمة p ثنائية الجانب
كنص عرض؛ وتُبلغ الفترات عن `(أسفل, أعلى)` عند المستوى الذي تسميه:

%%TESTS%%

يستخدمان `ttest(بيانات, mu0)` و`tinterval(بيانات, مستوى)` الانحراف
المعياري للعينة (n−1)؛ ويحتاجان `ztest(بيانات, mu0, sigma)` و
`zinterval(بيانات, sigma, مستوى)` إلى sigma معلوم. و`chisq_gof(ملاحظ,
متوقع)` هو اختبار جودة المطابقة بدرجات حرية k−1. النتائج نصوص عرض:
قابلة للقراءة والنسخ، لكن الحساب لا يلمسها.

### 1.22 مخططات البيانات

تقبل عائلة الرسم قوائم أيضًا: مخطط التشتت والمدرج التكراري ومخطط
الصندوق. يحتكر مخطط البيانات اللوحة مثل النظام الشمسي — أحدث أمر
يفوز، و`graph clear` يفرغه.

%%SCATTER%%

%%HISTOGRAM%%

%%BOXPLOT%%

**scatter(xs, ys)** يرسم النقاط، وعند وجود نقطتين أو أكثر يرسم خط
المربعات الصغرى، بعنوان `y = a*x + b (r = …)` في المفتاح.
**histogram(بيانات[, خانات])** يرسم مدرجًا تكراريًا؛ عدد الخانات
اختياري (قاعدة ستيرجس افتراضيًا) ويجب أن يكون عددًا صحيحًا بين 1 و50.
**boxplot(بيانات)** يرسم مخطط الصندوق والشارب: الحد الأدنى وQ1
والوسيط وQ3 والحد الأقصى، بشاربين حتى الطرفين. تتكيف النافذة دائمًا
مع البيانات — كلمتا النطاق `from a to b` لا تنطبقان — وتُصدَّر
الصورة وتُحفظ مثل أي مخطط آخر.
"""
