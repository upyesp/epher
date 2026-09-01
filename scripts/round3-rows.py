# -*- coding: utf-8 -*-
# Round 3 quick-reference rows and table paragraphs (per locale).
# Consumed by scripts/round3-guide-translate.py.

QR["de"] = [
  "| Listenliteral | `{…}` | `{1, 2, 3}` |",
  "| Listenelement | `list[i]` (ab 1) | `{5, 6}[2]` |",
  "| Listenstatistik | `mean(liste)`, `median(liste)`, … | `stdev(d)` |",
  "| Listenform | `len(s)`, `sort(s)`, `mode(s)`, `range(s)`, `quartile(s, k)` | `quartile(d, 1)` |",
  "| Lineare Regression | `linreg(xs, ys)` | `linreg(x, y)` |",
  "| Normalverteilung | `normpdf` `normcdf` `invnorm` | `invnorm(0.975)` |",
  "| t-Verteilung | `tpdf` `tcdf` `invt` | `invt(0.975, 10)` |",
  "| Chi-Quadrat | `chi2pdf` `chi2cdf` `invchi2` | `chi2cdf(3.84, 1)` |",
  "| Diskrete Verteilungen | `binompdf` `binomcdf` `poissonpdf` `poissoncdf` | `binomcdf(2, 10, 0.5)` |",
  "| Tests und Intervalle | `ztest` `ttest` `zinterval` `tinterval` `chisq_gof` | `tinterval(d, 0.95)` |",
  "| Datenplots | `graph scatter(xs, ys)` `histogram(data)` `boxplot(data)` | `graph boxplot(d)` |",
]
QR["fr"] = [
  "| Littéral de liste | `{…}` | `{1, 2, 3}` |",
  "| Élément de liste | `list[i]` (à partir de 1) | `{5, 6}[2]` |",
  "| Statistiques de liste | `mean(liste)`, `median(liste)`, … | `stdev(d)` |",
  "| Forme de liste | `len(s)`, `sort(s)`, `mode(s)`, `range(s)`, `quartile(s, k)` | `quartile(d, 1)` |",
  "| Régression linéaire | `linreg(xs, ys)` | `linreg(x, y)` |",
  "| Famille normale | `normpdf` `normcdf` `invnorm` | `invnorm(0.975)` |",
  "| Famille t | `tpdf` `tcdf` `invt` | `invt(0.975, 10)` |",
  "| Famille khi-deux | `chi2pdf` `chi2cdf` `invchi2` | `chi2cdf(3.84, 1)` |",
  "| Familles discrètes | `binompdf` `binomcdf` `poissonpdf` `poissoncdf` | `binomcdf(2, 10, 0.5)` |",
  "| Tests et intervalles | `ztest` `ttest` `zinterval` `tinterval` `chisq_gof` | `tinterval(d, 0.95)` |",
  "| Graphiques de données | `graph scatter(xs, ys)` `histogram(data)` `boxplot(data)` | `graph boxplot(d)` |",
]
QR["es"] = [
  "| Literal de lista | `{…}` | `{1, 2, 3}` |",
  "| Elemento de lista | `list[i]` (base 1) | `{5, 6}[2]` |",
  "| Estadística de lista | `mean(lista)`, `median(lista)`, … | `stdev(d)` |",
  "| Forma de lista | `len(s)`, `sort(s)`, `mode(s)`, `range(s)`, `quartile(s, k)` | `quartile(d, 1)` |",
  "| Regresión lineal | `linreg(xs, ys)` | `linreg(x, y)` |",
  "| Familia normal | `normpdf` `normcdf` `invnorm` | `invnorm(0.975)` |",
  "| Familia t | `tpdf` `tcdf` `invt` | `invt(0.975, 10)` |",
  "| Familia chi-cuadrado | `chi2pdf` `chi2cdf` `invchi2` | `chi2cdf(3.84, 1)` |",
  "| Familias discretas | `binompdf` `binomcdf` `poissonpdf` `poissoncdf` | `binomcdf(2, 10, 0.5)` |",
  "| Pruebas e intervalos | `ztest` `ttest` `zinterval` `tinterval` `chisq_gof` | `tinterval(d, 0.95)` |",
  "| Gráficos de datos | `graph scatter(xs, ys)` `histogram(data)` `boxplot(data)` | `graph boxplot(d)` |",
]
QR["pt"] = [
  "| Literal de lista | `{…}` | `{1, 2, 3}` |",
  "| Elemento de lista | `list[i]` (base 1) | `{5, 6}[2]` |",
  "| Estatística de lista | `mean(lista)`, `median(lista)`, … | `stdev(d)` |",
  "| Forma de lista | `len(s)`, `sort(s)`, `mode(s)`, `range(s)`, `quartile(s, k)` | `quartile(d, 1)` |",
  "| Regressão linear | `linreg(xs, ys)` | `linreg(x, y)` |",
  "| Família normal | `normpdf` `normcdf` `invnorm` | `invnorm(0.975)` |",
  "| Família t | `tpdf` `tcdf` `invt` | `invt(0.975, 10)` |",
  "| Família qui-quadrado | `chi2pdf` `chi2cdf` `invchi2` | `chi2cdf(3.84, 1)` |",
  "| Famílias discretas | `binompdf` `binomcdf` `poissonpdf` `poissoncdf` | `binomcdf(2, 10, 0.5)` |",
  "| Testes e intervalos | `ztest` `ttest` `zinterval` `tinterval` `chisq_gof` | `tinterval(d, 0.95)` |",
  "| Gráficos de dados | `graph scatter(xs, ys)` `histogram(data)` `boxplot(data)` | `graph boxplot(d)` |",
]
QR["zh-CN"] = [
  "| 列表字面量 | `{…}` | `{1, 2, 3}` |",
  "| 列表元素 | `list[i]`（从 1 起） | `{5, 6}[2]` |",
  "| 列表统计 | `mean(列表)`, `median(列表)`, … | `stdev(d)` |",
  "| 列表形态 | `len(s)`, `sort(s)`, `mode(s)`, `range(s)`, `quartile(s, k)` | `quartile(d, 1)` |",
  "| 线性回归 | `linreg(xs, ys)` | `linreg(x, y)` |",
  "| 正态族 | `normpdf` `normcdf` `invnorm` | `invnorm(0.975)` |",
  "| t 族 | `tpdf` `tcdf` `invt` | `invt(0.975, 10)` |",
  "| 卡方族 | `chi2pdf` `chi2cdf` `invchi2` | `chi2cdf(3.84, 1)` |",
  "| 离散族 | `binompdf` `binomcdf` `poissonpdf` `poissoncdf` | `binomcdf(2, 10, 0.5)` |",
  "| 检验与区间 | `ztest` `ttest` `zinterval` `tinterval` `chisq_gof` | `tinterval(d, 0.95)` |",
  "| 数据图 | `graph scatter(xs, ys)` `histogram(data)` `boxplot(data)` | `graph boxplot(d)` |",
]
QR["hi"] = [
  "| सूची शाब्दिक | `{…}` | `{1, 2, 3}` |",
  "| सूची तत्व | `list[i]` (1-आधारित) | `{5, 6}[2]` |",
  "| सूची सांख्यिकी | `mean(सूची)`, `median(सूची)`, … | `stdev(d)` |",
  "| सूची आकार | `len(s)`, `sort(s)`, `mode(s)`, `range(s)`, `quartile(s, k)` | `quartile(d, 1)` |",
  "| रेखीय प्रतिगमन | `linreg(xs, ys)` | `linreg(x, y)` |",
  "| सामान्य परिवार | `normpdf` `normcdf` `invnorm` | `invnorm(0.975)` |",
  "| t परिवार | `tpdf` `tcdf` `invt` | `invt(0.975, 10)` |",
  "| काई-वर्ग परिवार | `chi2pdf` `chi2cdf` `invchi2` | `chi2cdf(3.84, 1)` |",
  "| असतत परिवार | `binompdf` `binomcdf` `poissonpdf` `poissoncdf` | `binomcdf(2, 10, 0.5)` |",
  "| परीक्षण और अंतराल | `ztest` `ttest` `zinterval` `tinterval` `chisq_gof` | `tinterval(d, 0.95)` |",
  "| डेटा प्लॉट | `graph scatter(xs, ys)` `histogram(data)` `boxplot(data)` | `graph boxplot(d)` |",
]
QR["ar"] = [
  "| حرفية قائمة | `{…}` | `{1, 2, 3}` |",
  "| عنصر قائمة | `list[i]` (يبدأ من 1) | `{5, 6}[2]` |",
  "| إحصاء قائمة | `mean(قائمة)`, `median(قائمة)`, … | `stdev(d)` |",
  "| شكل قائمة | `len(s)`, `sort(s)`, `mode(s)`, `range(s)`, `quartile(s, k)` | `quartile(d, 1)` |",
  "| انحدار خطي | `linreg(xs, ys)` | `linreg(x, y)` |",
  "| عائلة طبيعية | `normpdf` `normcdf` `invnorm` | `invnorm(0.975)` |",
  "| عائلة t | `tpdf` `tcdf` `invt` | `invt(0.975, 10)` |",
  "| عائلة كاي تربيع | `chi2pdf` `chi2cdf` `invchi2` | `chi2cdf(3.84, 1)` |",
  "| عائلات متقطعة | `binompdf` `binomcdf` `poissonpdf` `poissoncdf` | `binomcdf(2, 10, 0.5)` |",
  "| اختبارات وفترات | `ztest` `ttest` `zinterval` `tinterval` `chisq_gof` | `tinterval(d, 0.95)` |",
  "| مخططات بيانات | `graph scatter(xs, ys)` `histogram(data)` `boxplot(data)` | `graph boxplot(d)` |",
]

TABLE_PARA["de"] = """Ein optionaler `derivative <ausdruck>`-Zusatz fügt eine dritte
Spalte hinzu, die numerische Ableitung dieses Ausdrucks an jeder
Stelle x:"""
TABLE_PARA["fr"] = """Une clause facultative `derivative <expression>` ajoute une
troisième colonne, la dérivée numérique de cette expression en chaque
x :"""
TABLE_PARA["es"] = """Una cláusula opcional `derivative <expresión>` añade una tercera
columna, la derivada numérica de esa expresión en cada x:"""
TABLE_PARA["pt"] = """Uma cláusula opcional `derivative <expressão>` acrescenta uma
terceira coluna, a derivada numérica dessa expressão em cada x:"""
TABLE_PARA["zh-CN"] = """可选的 `derivative <表达式>` 子句添加第三列，即该表达式在每个
x 处的数值导数："""
TABLE_PARA["hi"] = """एक वैकल्पिक `derivative <व्यंजक>` खंड तीसरा स्तंभ जोड़ता है, हर x
पर उस व्यंजक का संख्यात्मक अवकलज:"""
TABLE_PARA["ar"] = """إضافة اختيارية `derivative <تعبير>` تضيف عمودًا ثالثًا، وهو المشتقة
العددية لذلك التعبير عند كل x:"""

TABLE_CELLS["de"] = """Die Tabellenzellen folgen den Ergebniseinstellungen: bei
eingeschalteten exakten Brüchen (Standard) zeigt sich ein Wert, der
ein einfacher Bruch ist, als solcher — `table x / 3 from 0 to 1
points 4` listet `1/3` statt `0.333`."""
TABLE_CELLS["fr"] = """Les cellules du tableau suivent les réglages des résultats :
avec les fractions exactes activées (par défaut), une valeur qui est
une fraction simple s'affiche comme telle — `table x / 3 from 0 to 1
points 4` liste `1/3` au lieu de `0.333`."""
TABLE_CELLS["es"] = """Las celdas de la tabla siguen los ajustes de resultados: con las
fracciones exactas activadas (por defecto), un valor que es una
fracción simple se muestra como tal — `table x / 3 from 0 to 1
points 4` lista `1/3` en lugar de `0.333`."""
TABLE_CELLS["pt"] = """As células da tabela seguem os ajustes de resultados: com as
frações exatas ativadas (predefinição), um valor que é uma fração
simples mostra-se como tal — `table x / 3 from 0 to 1 points 4`
lista `1/3` em vez de `0.333`."""
TABLE_CELLS["zh-CN"] = """表格单元格遵循结果设置：启用精确分数（默认）时，是简单分数的值
会以分数显示——`table x / 3 from 0 to 1 points 4` 列出 `1/3` 而不是
`0.333`。"""
TABLE_CELLS["hi"] = """तालिका कक्ष परिणाम सेटिंग का पालन करते हैं: सटीक भिन्न चालू होने
पर (डिफ़ॉल्ट), साधारण भिन्न वाला मान भिन्न के रूप में दिखता है —
`table x / 3 from 0 to 1 points 4` में `0.333` की जगह `1/3` आता है।"""
TABLE_CELLS["ar"] = """تتبع خلايا الجدول إعدادات النتائج: مع تشغيل الكسور الدقيقة
(الافتراضي)، تظهر القيمة التي هي كسر بسيط ككسر — يسرد `table x / 3
from 0 to 1 points 4` قيمة `1/3` بدلًا من `0.333`."""
