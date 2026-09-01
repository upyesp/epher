# -*- coding: utf-8 -*-
# Round 4 guide localization (ADR-0045): the 6 CODATA rows, the lunar
# pair in the astronomy constants, two quick-reference rows, the new
# section 1.23, the web Help -> Constants sentence, and the TUI
# constants-browser paragraph. Verifies every epher fence stays
# byte-identical across all 8 locales.
import io, sys

PHI_ROWS = {
    "zh-CN": "| `phi_0` | 磁通量子 | 2.067833848e-15 |",
    "hi": "| `phi_0` | चुंबकीय फ्लक्स क्वांटम | 2.067833848e-15 |",
    "es": "| `phi_0` | cuanto de flujo magnético | 2.067833848e-15 |",
    "fr": "| `phi_0` | quantum de flux magnétique | 2.067833848e-15 |",
    "ar": "| `phi_0` | كمية الفيض المغناطيسي | 2.067833848e-15 |",
    "de": "| `phi_0` | Magnetisches Flussquantum | 2.067833848e-15 |",
    "pt": "| `phi_0` | quanto de fluxo magnético | 2.067833848e-15 |",
}
PHYS_ROWS = """| `m_P` | Planck-Masse | 2.176434e-8 |
| `l_P` | Planck-Länge | 1.616255e-35 |
| `t_P` | Planck-Zeit | 5.391247e-44 |
| `r_e` | klassischer Elektronenradius | 2.8179403205e-15 |
| `lambda_c` | Compton-Wellenlänge | 2.42631023867e-12 |
| `mu_n` | Kernmagneton | 5.050783699e-27 |"""

QR_ROWS = {
    "zh-CN": """| 随机数 | `random()`, `random(a, b)`, `randint(a, b)`, `randseed(n)` | `randint(1, 6)` |
| 常量浏览器 | 帮助 → 常量：全部内置常量，按组分类 | 帮助 → 常量 |""",
    "hi": """| यादृच्छिक संख्याएँ | `random()`, `random(a, b)`, `randint(a, b)`, `randseed(n)` | `randint(1, 6)` |
| स्थिरांक ब्राउज़र | सहायता → स्थिरांक: हर अंतर्निर्मित स्थिरांक, समूहों में | सहायता → स्थिरांक |""",
    "es": """| Números aleatorios | `random()`, `random(a, b)`, `randint(a, b)`, `randseed(n)` | `randint(1, 6)` |
| Explorador de constantes | Ayuda → Constantes: todas las constantes, agrupadas | Ayuda → Constantes |""",
    "fr": """| Nombres aléatoires | `random()`, `random(a, b)`, `randint(a, b)`, `randseed(n)` | `randint(1, 6)` |
| Explorateur de constantes | Aide → Constantes : toutes les constantes, groupées | Aide → Constantes |""",
    "ar": """| أعداد عشوائية | `random()`, `random(a, b)`, `randint(a, b)`, `randseed(n)` | `randint(1, 6)` |
| مستعرض الثوابت | المساعدة → الثوابت: كل ثابت مدمج، مجمّعًا | المساعدة → الثوابت |""",
    "de": """| Zufallszahlen | `random()`, `random(a, b)`, `randint(a, b)`, `randseed(n)` | `randint(1, 6)` |
| Konstanten-Browser | Hilfe → Konstanten: alle eingebauten Konstanten, nach Gruppe | Hilfe → Konstanten |""",
    "pt": """| Números aleatórios | `random()`, `random(a, b)`, `randint(a, b)`, `randseed(n)` | `randint(1, 6)` |
| Explorador de constantes | Ajuda → Constantes: todas as constantes, agrupadas | Ajuda → Constantes |""",
}

SECT23 = {
    "zh-CN": """### 1.23 随机数

`random()` 抽取 `[0, 1)` 中的均匀随机数，`random(a, b)` 抽取 `[a, b)` 中的一个，
`randint(a, b)` 抽取闭区间 `[a, b]` 中的一个整数——掷骰子：

```epher
randseed(7)
randint(1, 6)
```

```text
7
3
```

序列是可复现的：`randseed(n)` 用 `n` 重新设定随机种子并显示它，因此相同的种子
在每个会话、每个前端都会重放相同的抽取结果。
""",
    "hi": """### 1.23 यादृच्छिक संख्याएँ

`random()` एक समान यादृच्छिक संख्या निकालता है `[0, 1)` में, `random(a, b)` एक
`[a, b)` में, और `randint(a, b)` बंद परास `[a, b]` से एक पूर्णांक — पासा फेंकना:

```epher
randseed(7)
randint(1, 6)
```

```text
7
3
```

क्रम प्रतिलिपि योग्य है: `randseed(n)` जनरेटर को `n` से फिर से सीड करता है और
उसे दिखाता है, इसलिए वही सीड हर सत्र और हर फ्रंटएंड में वही निकाल दोहराती है।
""",
    "es": """### 1.23 Números aleatorios

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
""",
    "fr": """### 1.23 Nombres aléatoires

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
""",
    "ar": """### 1.23 أعداد عشوائية

`random()` يسحب عددًا منتظمًا في `[0, 1)`، و`random(a, b)` واحدًا في `[a, b)`،
و`randint(a, b)` عددًا صحيحًا من الفترة المغلقة `[a, b]` — رمية نرد:

```epher
randseed(7)
randint(1, 6)
```

```text
7
3
```

التسلسل قابل لإعادة الإنتاج: يعيد `randseed(n)` بذر المولّد بـ `n` ويعرضه،
لذلك نفس البذرة تعيد نفس السحوبات في كل جلسة وكل واجهة.
""",
    "de": """### 1.23 Zufallszahlen

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
""",
    "pt": """### 1.23 Números aleatórios

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
""",
}

# §2.1 insertions: appended after the translated "tap an example" sentence.
WEB_MENTION = {
    "zh-CN": " **帮助 → 常量**打开常量浏览器：全部内置常量按组显示（数学、天文、物理、化学），每个都带值和简短说明；点按一个即可把它的名称插入输入框，搜索框可以筛选列表。",
    "hi": " **सहायता → स्थिरांक** स्थिरांक ब्राउज़र खोलता है: हर अंतर्निर्मित स्थिरांक समूहों में (गणित, खगोल विज्ञान, भौतिकी, रसायन विज्ञान), हर एक अपने मान और छोटे विवरण के साथ; किसी पर टैप करें और उसका नाम इनपुट फ़ील्ड में डाला जाता है, और खोज बॉक्स सूची को छांटता है।",
    "es": " **Ayuda → Constantes** abre el explorador de constantes: todas las constantes agrupadas (Matemáticas, Astronomía, Física, Química), cada una con su valor y una breve descripción; toca una para insertar su nombre en el campo de entrada, y la caja de búsqueda filtra la lista.",
    "fr": " **Aide → Constantes** ouvre l'explorateur de constantes : toutes les constantes groupées (Mathématiques, Astronomie, Physique, Chimie), chacune avec sa valeur et une brève description ; touchez-en une pour insérer son nom dans le champ de saisie, et la zone de recherche filtre la liste.",
    "ar": " يفتح **المساعدة → الثوابت** مستعرض الثوابت: كل ثابت مدمج في مجموعات (رياضيات، فلك، فيزياء، كيمياء)، كل واحد بقيمته ووصف مختصر؛ المس واحدًا لإدراج اسمه في حقل الإدخال، ويصفّي صندوق البحث القائمة.",
    "de": " **Hilfe → Konstanten** öffnet den Konstanten-Browser: alle eingebauten Konstanten in Gruppen (Mathematik, Astronomie, Physik, Chemie), jede mit ihrem Wert und einer kurzen Beschreibung; tippe eine an, um ihren Namen ins Eingabefeld einzufügen, und das Suchfeld filtert die Liste.",
    "pt": " **Ajuda → Constantes** abre o explorador de constantes: todas as constantes agrupadas (Matemática, Astronomia, Física, Química), cada uma com o seu valor e uma breve descrição; toque numa para inserir o nome no campo de entrada, e a caixa de pesquisa filtra a lista.",
}

# §5 insertions: appended after the translated Ctrl+L row.
TUI_MENTION = {
    "zh-CN": "\n\n**帮助**菜单打开应用内指南、键盘按键帮助，以及常量浏览器：内置常量按组显示，方向键选择一行，**回车**把它的名称插入光标处的表达式，**Esc** 关闭。",
    "hi": "\n\n**सहायता** मेनू अंतर्निर्मित गाइड, कीपैड की-हेल्प, और एक स्थिरांक ब्राउज़र खोलता है: स्थिरांक समूहों में, तीर एक पंक्ति चुनते हैं, **Enter** उसका नाम कर्सर पर व्यंजक में डालता है, और **Esc** बंद करता है।",
    "es": "\n\nEl menú **Ayuda** abre la guía integrada, la ayuda de teclas del teclado y un explorador de constantes: las constantes agrupadas, las flechas eligen una fila, **Intro** inserta su nombre en la expresión en el cursor y **Esc** cierra.",
    "fr": "\n\nLe menu **Aide** ouvre le guide intégré, l'aide des touches du clavier et un explorateur de constantes : les constantes groupées, les flèches choisissent une ligne, **Entrée** insère son nom dans l'expression au curseur et **Échap** ferme.",
    "ar": "\n\nيفتح **المساعدة** الدليل المدمج، ومساعدة مفاتيح لوحة المفاتيح، ومستعرض ثوابت: الثوابت في مجموعات، والأسهم تختار صفًا، و**Enter** يُدرج اسمه في التعبير عند المؤشر، و**Esc** يغلق.",
    "de": "\n\nDas Menü **Hilfe** öffnet das eingebaute Handbuch, die Tastenfeld-Hilfe und einen Konstanten-Browser: die Konstanten in Gruppen, die Pfeile wählen eine Zeile, **Enter** fügt ihren Namen in den Ausdruck am Cursor ein, **Esc** schließt.",
    "pt": "\n\nO menu **Ajuda** abre o guia integrado, a ajuda de teclas do teclado e um explorador de constantes: as constantes agrupadas, as setas escolhem uma linha, **Enter** insere o seu nome na expressão no cursor e **Esc** fecha.",
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

ASTRONOMY_LINE = {
    "zh-CN": "`m_sun`、`r_sun`、`l_sun`、`m_earth`、`r_earth` 的用法与 `pi` 相同，你也可以",
    "hi": "`sigma_sb`, `m_sun`, `r_sun`, `l_sun`, `m_earth`, `r_earth` `pi` की तरह",
    "es": "`k_b`, `sigma_sb`, `m_sun`, `r_sun`, `l_sun`, `m_earth`, `r_earth` funcionan",
    "fr": "`k_b`, `sigma_sb`, `m_sun`, `r_sun`, `l_sun`, `m_earth`, `r_earth`",
    "ar": "و`sigma_sb` و`m_sun` و`r_sun` و`l_sun` و`m_earth` و`r_earth` مثل `pi`، ويمكنك",
    "de": "`k_b`, `sigma_sb`, `m_sun`, `r_sun`, `l_sun`, `m_earth`, `r_earth` wirken",
    "pt": "`k_b`, `sigma_sb`, `m_sun`, `r_sun`, `l_sun`, `m_earth`, `r_earth`",
}

ASTRONOMY_NEW = {
    "zh-CN": "`m_sun`、`r_sun`、`l_sun`、`m_earth`、`r_earth`、`m_moon`、`r_moon` 的用法与 `pi` 相同，你也可以",
    "hi": "`sigma_sb`, `m_sun`, `r_sun`, `l_sun`, `m_earth`, `r_earth`, `m_moon`, `r_moon` `pi` की तरह",
    "es": "`k_b`, `sigma_sb`, `m_sun`, `r_sun`, `l_sun`, `m_earth`, `r_earth`, `m_moon`, `r_moon` funcionan",
    "fr": "`k_b`, `sigma_sb`, `m_sun`, `r_sun`, `l_sun`, `m_earth`, `r_earth`, `m_moon`, `r_moon`",
    "ar": "و`sigma_sb` و`m_sun` و`r_sun` و`l_sun` و`m_earth` و`r_earth` و`m_moon` و`r_moon` مثل `pi`، ويمكنك",
    "de": "`k_b`, `sigma_sb`, `m_sun`, `r_sun`, `l_sun`, `m_earth`, `r_earth`, `m_moon`, `r_moon` wirken",
    "pt": "`k_b`, `sigma_sb`, `m_sun`, `r_sun`, `l_sun`, `m_earth`, `r_earth`, `m_moon`, `r_moon`",
}

DATA_PLOT_ROWS = {
    "zh-CN": "| 数据图 | `graph scatter(xs, ys)` `histogram(data)` `boxplot(data)` | `graph boxplot(d)` |",
    "hi": "| डेटा प्लॉट | `graph scatter(xs, ys)` `histogram(data)` `boxplot(data)` | `graph boxplot(d)` |",
    "es": "| Gráficos de datos | `graph scatter(xs, ys)` `histogram(data)` `boxplot(data)` | `graph boxplot(d)` |",
    "fr": "| Graphiques de données | `graph scatter(xs, ys)` `histogram(data)` `boxplot(data)` | `graph boxplot(d)` |",
    "ar": "| مخططات بيانات | `graph scatter(xs, ys)` `histogram(data)` `boxplot(data)` | `graph boxplot(d)` |",
    "de": "| Datenplots | `graph scatter(xs, ys)` `histogram(data)` `boxplot(data)` | `graph boxplot(d)` |",
    "pt": "| Gráficos de dados | `graph scatter(xs, ys)` `histogram(data)` `boxplot(data)` | `graph boxplot(d)` |",
}

WEB_SENTENCE = {
    "zh-CN": "点击该指南中的任何示例，即可将其载入输入框。",
    "hi": "उस गाइड में किसी भी उदाहरण पर टैप करें। वह एंट्री फ़ील्ड में लोड हो जाता है।",
    "es": "para cargarlo en el campo de entrada.",
    "fr": "quel exemple de ce guide pour le charger dans le champ de saisie.",
    "ar": "المس أي مثال في ذلك الدليل لتحميله في حقل الإدخال.",
    "de": "in diesem Handbuch an, um es ins Eingabefeld zu laden.",
    "pt": "para o carregar no campo de entrada.",
}

CTRL_L_ROWS = {
    "zh-CN": "| **Ctrl+L** | 清空历史 |",
    "hi": "| **Ctrl+L** | इतिहास साफ़ करें |",
    "es": "| **Ctrl+L** | borrar el historial |",
    "fr": "| **Ctrl+L** | effacer l'historique |",
    "ar": "| **Ctrl+L** | مسح السجل |",
    "de": "| **Ctrl+L** | den Verlauf leeren |",
    "pt": "| **Ctrl+L** | limpar o histórico |",
}

FAILED = []
for loc, anchor2 in ANCHORS2.items():
    path = "site/guide/%s.md" % loc
    md = io.open(path, encoding="utf-8").read()
    try:
        # 1) physics table rows after the phi_0 row
        old = PHI_ROWS[loc]
        assert old in md, (loc, "phi_0 row")
        md = md.replace(old, old + "\n" + PHYS_ROWS, 1)
        # 2) astronomy constants: the lunar pair
        old = ASTRONOMY_LINE[loc]
        assert old in md, (loc, "astronomy line")
        md = md.replace(old, ASTRONOMY_NEW[loc], 1)
        # 3) quick-reference rows after the data-plots row
        old = DATA_PLOT_ROWS[loc]
        assert old in md, (loc, "data-plot quick-ref row")
        md = md.replace(old, old + "\n" + QR_ROWS[loc], 1)
        # 4) section 1.23 before the §2 anchor
        idx = md.index(anchor2)
        md = md[:idx] + SECT23[loc] + "\n" + md[idx:]
        # 5) web mention after the tap-an-example sentence
        old = WEB_SENTENCE[loc]
        assert old in md, (loc, "web sentence")
        md = md.replace(old, old + WEB_MENTION[loc], 1)
        # 6) TUI paragraph after the Ctrl+L row
        old = CTRL_L_ROWS[loc]
        assert old in md, (loc, "ctrl+L row")
        md = md.replace(old, old + TUI_MENTION[loc], 1)
        io.open(path, "w", encoding="utf-8").write(md)
        print(loc, "done")
    except AssertionError as e:
        FAILED.append(e.args)
        print(loc, "FAILED", e.args)

if FAILED:
    print("FAILURES:", FAILED)
    sys.exit(1)

# verify fence parity
import re
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
