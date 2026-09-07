# -*- coding: utf-8 -*-
# Round 5 guide localization (ADR-0046): the AU example becomes an
# explicit `in m` conversion, and section 1.24 (units and conversion)
# plus three quick-reference rows are translated per locale. All epher
# fences stay byte-identical.
import io, re, sys

# The section fences: byte-identical in every locale.
F = {
    "speed": "```epher\n60 mile/hr\n```\n\n```text\n60 mile/hr\n```",
    "dimerr": "```epher\n5 m + 3 s\n```\n\n```text\nerror: dimension error: cannot add 5 m and 3 s\n```",
    "conv": "```epher\n72 km/hr in m/s\n```\n\n```text\n20 m/s\n```",
    "area": "```epher\n2 m^2 in cm^2\n```\n\n```text\n20000 cm^2\n```",
}

SECTIONS = {
    "zh-CN": """### 1.24 单位与换算

数字后跟单位就成为一个*量*：SI 单位的值加上它的量纲。单位表涵盖 SI
基本单位和导出单位（`m`、`s`、`kg`、`A`、`K`、`mol`、`cd`、`Hz`、`N`、
`Pa`、`J`、`W`、`C`、`V`、`F`、`ohm`、`S`、`Wb`、`T`、`H`、`lm`、`lx`、
`Bq`、`Gy`、`Sv`）、日常单位（`min`、`hr`、`d`、`yr`、`L`、`t`、`bar`、
`atm`、`torr`、`psi`、`eV`、`mile`、`yd`、`ft`、`inch`、`nmi`、`lb`、
`oz`、`gal`、`qt`、`pt`、`mph`、`knot`）以及第 1.16 节的天文后缀。
复合单位可以串联：`60 mile/hr` 和 `5 m/s^2` 都是单个单位。

{F_SPEED}

SI 词头可以缩放任意单位：`k M G T m µ n p` 是千、兆、吉、太、毫、微、
纳、皮——`5 km`、`3 MPa`、`1 GHz` 都可以，`2 kg` 就是千克本身。

量纲会被检查：不同单位的量相加或比较会报错，而不是混在一起算：

{F_DIMERR}

算术会组合量纲：`5 m * 3 m` 是 `15 m^2`，`(3 m)^2` 是 `9 m^2`，
`sqrt(4 m^2)` 是 `2 m`，量纲完全相消的整个表达式又回到普通数字
（`5 m / 5 m` 是 `1`）。量纲恰好匹配时，结果优先使用导出名称——
`5 kg * 3 m / 1 s^2` 得出 `15 N`。

**换算。** `expr in 单位`（或 `expr -> 单位`）以指定单位显示一个量；
量纲必须匹配。`in` 的绑定最松，所以 `5 m + 3 m in km` 换算的是整个和：

{F_CONV}

{F_AREA}

温度标度（摄氏、华氏）在这里不是单位——开尔文是，`K` 和其他单位一样用。

""",
    "hi": """### 1.24 इकाइयाँ और रूपांतरण

संख्या के बाद की इकाई एक *राशि* बनाती है: SI इकाइयों में मान और उसके
आयाम। इकाई तालिका में SI मूल और व्युत्पन्न इकाइयाँ हैं (`m`, `s`, `kg`,
`A`, `K`, `mol`, `cd`, `Hz`, `N`, `Pa`, `J`, `W`, `C`, `V`, `F`,
`ohm`, `S`, `Wb`, `T`, `H`, `lm`, `lx`, `Bq`, `Gy`, `Sv`), रोज़मर्रा
की इकाइयाँ (`min`, `hr`, `d`, `yr`, `L`, `t`, `bar`, `atm`, `torr`,
`psi`, `eV`, `mile`, `yd`, `ft`, `inch`, `nmi`, `lb`, `oz`, `gal`,
`qt`, `pt`, `mph`, `knot`), और खंड 1.16 के खगोल प्रत्यय। मिश्रित इकाइयाँ
जुड़ती हैं: `60 mile/hr` और `5 m/s^2` एक ही इकाई हैं।

{F_SPEED}

SI उपसर्ग उनमें से किसी को भी स्केल करते हैं: `k M G T m µ n p` किलो,
मेगा, गीगा, टेरा, मिली, माइक्रो, नैनो, पिको हैं — `5 km`, `3 MPa`,
`1 GHz` सब चलते हैं, और `2 kg` खुद किलोग्राम है।

आयाम जाँचे जाते हैं: अलग इकाइयों वाली राशियों को जोड़ना या तुलना करना
गलती देता है, मीटर और सेकंड मिलाने के बजाय:

{F_DIMERR}

अंकगणित आयामों को जोड़ता है: `5 m * 3 m` = `15 m^2`, `(3 m)^2` =
`9 m^2`, `sqrt(4 m^2)` = `2 m`, और जिस व्यंजक के आयाम पूरी तरह कट जाते
हैं वह फिर एक साधारण संख्या है (`5 m / 5 m` = `1`)। जब आयाम किसी
व्युत्पन्न नाम से मेल खाते हैं तो परिणाम वही दिखाता है — `5 kg * 3 m / 1
s^2` का उत्तर `15 N` है।

**रूपांतरण।** `expr in इकाई` (या `expr -> इकाई`) राशि को नामित इकाई में
दिखाता है; आयाम मेल खाने चाहिए। `in` सबसे ढीला बंधता है, इसलिए
`5 m + 3 m in km` पूरे योग को बदलता है:

{F_CONV}

{F_AREA}

तापमान पैमाने (सेल्सियस, फ़ारेनहाइट) यहाँ इकाइयाँ नहीं हैं — केल्विन है,
और `K` बाकियों की तरह काम करता है।

""",
    "es": """### 1.24 Unidades y conversión

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

{F_SPEED}

Los prefijos del SI escalan cualquiera de ellas: `k M G T m µ n p` son
kilo, mega, giga, tera, mili, micro, nano, pico — `5 km`, `3 MPa`,
`1 GHz` funcionan, y `2 kg` es el propio kilogramo.

Las dimensiones se comprueban: sumar o comparar cantidades con
unidades distintas da error en lugar de mezclar metros y segundos:

{F_DIMERR}

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

{F_CONV}

{F_AREA}

Las escalas de temperatura (Celsius, Fahrenheit) no son unidades aquí
— los kelvins sí, y `K` funciona como cualquier otra.

""",
    "fr": """### 1.24 Unités et conversion

Un nombre suivi d'une unité devient une *grandeur* : la valeur en
unités SI plus ses dimensions. Le tableau des unités couvre les unités
de base et dérivées du SI (`m`, `s`, `kg`, `A`, `K`, `mol`, `cd`,
`Hz`, `N`, `Pa`, `J`, `W`, `C`, `V`, `F`, `ohm`, `S`, `Wb`, `T`, `H`,
`lm`, `lx`, `Bq`, `Gy`, `Sv`), les unités courantes (`min`, `hr`, `d`,
`yr`, `L`, `t`, `bar`, `atm`, `torr`, `psi`, `eV`, `mile`, `yd`, `ft`,
`inch`, `nmi`, `lb`, `oz`, `gal`, `qt`, `pt`, `mph`, `knot`) et les
suffixes d'astronomie de la section 1.16. Les unités composées
s'enchaînent : `60 mile/hr` et `5 m/s^2` sont des unités simples.

{F_SPEED}

Les préfixes SI les modifient toutes : `k M G T m µ n p` sont kilo,
méga, giga, téra, milli, micro, nano, pico — `5 km`, `3 MPa`, `1 GHz`
fonctionnent, et `2 kg` est le kilogramme lui-même.

Les dimensions sont vérifiées : additionner ou comparer des grandeurs
d'unités différentes donne une erreur au lieu de mélanger mètres et
secondes :

{F_DIMERR}

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

{F_CONV}

{F_AREA}

Les échelles de température (Celsius, Fahrenheit) ne sont pas des
unités ici — les kelvins le sont, et `K` fonctionne comme n'importe
quelle autre.

""",
    "ar": """### 1.24 الوحدات والتحويل

الرقم الذي يتبعه وحدة يصبح *كمية*: القيمة بوحدات SI بالإضافة إلى أبعادها.
يغطي جدول الوحدات وحدات SI الأساسية والمشتقة (`m`, `s`, `kg`, `A`, `K`,
`mol`, `cd`, `Hz`, `N`, `Pa`, `J`, `W`, `C`, `V`, `F`, `ohm`, `S`,
`Wb`, `T`, `H`, `lm`, `lx`, `Bq`, `Gy`, `Sv`)، والوحدات اليومية (`min`,
`hr`, `d`, `yr`, `L`, `t`, `bar`, `atm`, `torr`, `psi`, `eV`, `mile`,
`yd`, `ft`, `inch`, `nmi`, `lb`, `oz`, `gal`, `qt`, `pt`, `mph`,
`knot`)، ولواحق الفلك من القسم 1.16. الوحدات المركبة تتسلسل: `60
mile/hr` و`5 m/s^2` وحدتان مفردتان.

{F_SPEED}

سوابق SI تدرّج أيًا منها: `k M G T m µ n p` هي كيلو وميجا وجيجا وتيرا
وميلي وميكرو ونانو وبيكو — `5 km` و`3 MPa` و`1 GHz` تعمل جميعًا،
و`2 kg` هو الكيلوجرام نفسه.

تُفحص الأبعاد: جمع أو مقارنة كميات بوحدات مختلفة ينتج خطأً بدل خلط
الأمتار والثواني:

{F_DIMERR}

الحساب يركّب الأبعاد: `5 m * 3 m` هي `15 m^2`، و`(3 m)^2` هي `9 m^2`،
و`sqrt(4 m^2)` هي `2 m`، والتعبير الكامل الذي تتلاشى أبعاده يعود عددًا
عاديًا (`5 m / 5 m` هو `1`). تفضّل النتائج الاسم المشتق الدقيق عندما
تطابقه الأبعاد — `5 kg * 3 m / 1 s^2` يجيب `15 N`.

**التحويل.** `expr in وحدة` (أو `expr -> وحدة`) يعرض كمية بالوحدة
المذكورة؛ يجب أن تتطابق الأبعاد. `in` أضعف الروابط بين العوامل، لذا
`5 m + 3 m in km` يحوّل المجموع كاملًا:

{F_CONV}

{F_AREA}

مقاييس الحرارة (سلسيوس، فهرنهايت) ليست وحدات هنا — الكلفن وحدة، و`K`
تعمل مثل أي وحدة أخرى.

""",
    "de": """### 1.24 Einheiten und Umrechnung

Eine Zahl mit einer Einheit dahinter wird zu einer *Größe*: dem Wert in
SI-Einheiten plus seinen Dimensionen. Die Einheitentabelle umfasst die
SI-Basis- und -abgeleiteten Einheiten (`m`, `s`, `kg`, `A`, `K`, `mol`,
`cd`, `Hz`, `N`, `Pa`, `J`, `W`, `C`, `V`, `F`, `ohm`, `S`, `Wb`, `T`,
`H`, `lm`, `lx`, `Bq`, `Gy`, `Sv`), die Alltagseinheiten (`min`, `hr`,
`d`, `yr`, `L`, `t`, `bar`, `atm`, `torr`, `psi`, `eV`, `mile`, `yd`,
`ft`, `inch`, `nmi`, `lb`, `oz`, `gal`, `qt`, `pt`, `mph`, `knot`) und
die Astronomie-Suffixe aus Abschnitt 1.16. Zusammengesetzte Einheiten
verkettet: `60 mile/hr` und `5 m/s^2` sind einzelne Einheiten.

{F_SPEED}

Die SI-Vorsätze skalieren jede davon: `k M G T m µ n p` sind Kilo,
Mega, Giga, Tera, Milli, Mikro, Nano, Piko — `5 km`, `3 MPa`, `1 GHz`
funktionieren alle, und `2 kg` ist das Kilogramm selbst.

Die Dimensionen werden geprüft: Addition oder Vergleich von Größen mit
verschiedenen Einheiten meldet einen Fehler, statt Meter und Sekunden
zu mischen:

{F_DIMERR}

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

{F_CONV}

{F_AREA}

Temperaturskalen (Celsius, Fahrenheit) sind hier keine Einheiten —
Kelvin schon, und `K` funktioniert wie jede andere.

""",
    "pt": """### 1.24 Unidades e conversão

Um número seguido de uma unidade torna-se uma *grandeza*: o valor em
unidades SI mais as suas dimensões. A tabela de unidades cobre as
unidades SI de base e derivadas (`m`, `s`, `kg`, `A`, `K`, `mol`, `cd`,
`Hz`, `N`, `Pa`, `J`, `W`, `C`, `V`, `F`, `ohm`, `S`, `Wb`, `T`, `H`,
`lm`, `lx`, `Bq`, `Gy`, `Sv`), as unidades do dia a dia (`min`, `hr`,
`d`, `yr`, `L`, `t`, `bar`, `atm`, `torr`, `psi`, `eV`, `mile`, `yd`,
`ft`, `inch`, `nmi`, `lb`, `oz`, `gal`, `qt`, `pt`, `mph`, `knot`) e
os sufixos de astronomia da secção 1.16. Unidades compostas encadeiam:
`60 mile/hr` e `5 m/s^2` são unidades únicas.

{F_SPEED}

Os prefixos SI escalam qualquer uma delas: `k M G T m µ n p` são quilo,
mega, giga, tera, mili, micro, nano, pico — `5 km`, `3 MPa`, `1 GHz`
funcionam, e `2 kg` é o próprio quilograma.

As dimensões são verificadas: somar ou comparar grandezas com unidades
diferentes dá erro em vez de misturar metros e segundos:

{F_DIMERR}

A aritmética compõe as dimensões: `5 m * 3 m` é `15 m^2`, `(3 m)^2` é
`9 m^2`, `sqrt(4 m^2)` é `2 m`, e uma expressão inteira cujas dimensões
se cancelam volta a ser um número vulgar (`5 m / 5 m` é `1`). Os
resultados preferem o nome derivado exato quando as dimensões
coincidem com um — `5 kg * 3 m / 1 s^2` responde `15 N`.

**Conversão.** `expr in unidade` (ou `expr -> unidade`) mostra uma
grandeza na unidade nomeada; as dimensões têm de coincidir. `in` liga
com a menor precedência dos operadores, por isso `5 m + 3 m in km`
converte a soma inteira:

{F_CONV}

{F_AREA}

As escalas de temperatura (Celsius, Fahrenheit) não são unidades aqui
— os kelvins são, e `K` funciona como qualquer outra.

""",
}

QR = {
    "zh-CN": """| 量 | `5 m`, `60 mile/hr`, `1 km` | `2 m^2` |
| 换算 | `expr in 单位` 或 `expr -> 单位` | `72 km/hr in m/s` |
| 词头 | `k M G T m µ n p` 缩放任意单位 | `5 km`, `3 MPa`, `1 GHz` |""",
    "hi": """| राशि | `5 m`, `60 mile/hr`, `1 km` | `2 m^2` |
| बदलें | `expr in इकाई` या `expr -> इकाई` | `72 km/hr in m/s` |
| उपसर्ग | `k M G T m µ n p` किसी इकाई को स्केल करते हैं | `5 km`, `3 MPa`, `1 GHz` |""",
    "es": """| Cantidad | `5 m`, `60 mile/hr`, `1 km` | `2 m^2` |
| Convertir | `expr in unidad` o `expr -> unidad` | `72 km/hr in m/s` |
| Prefijos | `k M G T m µ n p` escalan cualquier unidad | `5 km`, `3 MPa`, `1 GHz` |""",
    "fr": """| Grandeur | `5 m`, `60 mile/hr`, `1 km` | `2 m^2` |
| Convertir | `expr in unité` ou `expr -> unité` | `72 km/hr in m/s` |
| Préfixes | `k M G T m µ n p` modifient toute unité | `5 km`, `3 MPa`, `1 GHz` |""",
    "ar": """| كمية | `5 m`, `60 mile/hr`, `1 km` | `2 m^2` |
| تحويل | `expr in وحدة` أو `expr -> وحدة` | `72 km/hr in m/s` |
| سوابق | `k M G T m µ n p` تدرّج أي وحدة | `5 km`, `3 MPa`, `1 GHz` |""",
    "de": """| Größe | `5 m`, `60 mile/hr`, `1 km` | `2 m^2` |
| Umrechnen | `expr in Einheit` oder `expr -> Einheit` | `72 km/hr in m/s` |
| Vorsätze | `k M G T m µ n p` skalieren jede Einheit | `5 km`, `3 MPa`, `1 GHz` |""",
    "pt": """| Grandeza | `5 m`, `60 mile/hr`, `1 km` | `2 m^2` |
| Converter | `expr in unidade` ou `expr -> unidade` | `72 km/hr in m/s` |
| Prefixos | `k M G T m µ n p` escalam qualquer unidade | `5 km`, `3 MPa`, `1 GHz` |""",
}

ANCHORS2 = {
    "zh-CN": "## 2. 网页应用（PWA）",
    "hi": "## 2. वेब ऐप (PWA)",
    "es": "## 2. La aplicación web (PWA)",
    "fr": "## 2. L'application web (PWA)",
    "ar": "## 2. تطبيق الويب (PWA)",
    "de": "## 2. Die Web-App (PWA)",
    "pt": "## 2. A aplicação web (PWA)",
}

QR_ANCHOR = {
    "zh-CN": "| 常量浏览器 | 帮助 → 常量：全部内置常量，按组分类 | 帮助 → 常量 |",
    "hi": "| स्थिरांक ब्राउज़र | सहायता → स्थिरांक: हर अंतर्निर्मित स्थिरांक, समूहों में | सहायता → स्थिरांक |",
    "es": "| Explorador de constantes | Ayuda → Constantes: todas las constantes, agrupadas | Ayuda → Constantes |",
    "fr": "| Explorateur de constantes | Aide → Constantes : toutes les constantes, groupées | Aide → Constantes |",
    "ar": "| مستعرض الثوابت | المساعدة → الثوابت: كل ثابت مدمج، مجمّعًا | المساعدة → الثوابت |",
    "de": "| Konstanten-Browser | Hilfe → Konstanten: alle eingebauten Konstanten, nach Gruppe | Hilfe → Konstanten |",
    "pt": "| Explorador de constantes | Ajuda → Constantes: todas as constantes, agrupadas | Ajuda → Constantes |",
}

FAILED = []
for loc in ANCHORS2:
    path = "site/guide/%s.md" % loc
    md = io.open(path, encoding="utf-8").read()
    try:
        # 1) the AU example becomes an explicit conversion (fence code is
        # identical across locales)
        old = "```epher\n3.2 AU\n```\n\n```text\n478713186240\n```"
        assert old in md, (loc, "AU fence")
        md = md.replace(old, "```epher\n3.2 AU in m\n```\n\n```text\n478713186240 m\n```", 1)
        # 2) section 1.24 before the §2 anchor
        section = SECTIONS[loc]
        for tok, fence in F.items():
            section = section.replace("{F_%s}" % tok.upper(), fence)
        idx = md.index(ANCHORS2[loc])
        md = md[:idx] + section + "\n" + md[idx:]
        # 3) quick-reference rows
        old = QR_ANCHOR[loc]
        assert old in md, (loc, "qr anchor")
        md = md.replace(old, old + "\n" + QR[loc], 1)
        io.open(path, "w", encoding="utf-8").write(md)
        print(loc, "done")
    except AssertionError as e:
        FAILED.append(e.args)
        print(loc, "FAILED", e.args)

if FAILED:
    print("FAILURES:", FAILED)
    sys.exit(1)

counts = {}
for loc in ["en"] + list(ANCHORS2):
    md = io.open("site/guide/%s.md" % loc, encoding="utf-8").read()
    counts[loc] = len(re.findall(r"^```epher", md, re.M))
base = counts["en"]
ok = True
for loc, n in counts.items():
    if n != base:
        print("COUNT MISMATCH", loc, n, "vs", base)
        ok = False
print("en epher fences:", base)
print("ALL IDENTICAL" if ok else "FAILED")
sys.exit(0 if ok else 1)
