# -*- coding: utf-8 -*-
# Round 6 guide localization (ADR-0047): section 1.25 (bitwise
# operations) plus five quick-reference rows, per locale. All epher
# fences stay byte-identical.
import io, re, sys

F = {
    "and": "```epher\n0xFF & 0x0F\n```\n\n```text\n15\n```",
    "bits": "```epher\nbits(8)\n~0\n```\n\n```text\n8\n-1\n```",
}

SECTIONS = {
    "zh-CN": """### 1.25 位运算

第 1.13 节的基本字面量就是为此而生的：`0b101`、`0o17`、`0xFF`。位运算符
作用于整数，返回精确的整数：

{F_AND}

| 运算符 | 含义 |
|---|---|
| `a & b` | 按位与 |
| `a \\| b` | 按位或 |
| `a xor b` | 按位异或 |
| `~a` | 按位取反（二进制补码） |
| `a << n` | 左移（乘以 2^n） |
| `a >> n` | 右移，算术移位（除以 2^n，向下取整） |

结果都是精确的 `big` 整数，所以 `1 << 60` 保留每一位数字。默认字长为
64 位：结果按有符号二进制补码解读，因此 `~0` 是 -1，`1 << 100` 回绕为
0。`bits(n)` 把字长改为 8、16、32 或 64，`bits()` 报告当前值：

{F_BITS}

负数移位会反向（`8 << -1` 是 `4`）。布尔 `and` 和 `or` 保持原意；
`&` 和 `|` 是位运算写法。

""",
    "hi": """### 1.25 बिटवाइज़ संक्रियाएँ

खंड 1.13 के आधार शाब्दिक इसी के लिए बने हैं: `0b101`, `0o17`, `0xFF`।
बिटवाइज़ ऑपरेटर पूर्णांकों पर काम करते हैं और सटीक पूर्णांक उत्तर देते हैं:

{F_AND}

| ऑपरेटर | अर्थ |
|---|---|
| `a & b` | बिटवाइज़ और |
| `a \\| b` | बिटवाइज़ या |
| `a xor b` | बिटवाइज़ अनन्य या |
| `~a` | बिटवाइज़ नहीं (दो का पूरक) |
| `a << n` | बायाँ शिफ़्ट (2^n से गुणा) |
| `a >> n` | दायाँ शिफ़्ट, अंकगणितीय (2^n से भाग, नीचे पूर्णांक) |

परिणाम सटीक `big` पूर्णांक हैं, इसलिए `1 << 60` हर अंक रखता है। कार्यशील
शब्द आकार डिफ़ॉल्ट रूप से 64 बिट है: परिणाम साइन्ड टूज़-कॉम्प्लिमेंट के
रूप में पढ़े जाते हैं, इसलिए `~0` = -1 है और `1 << 100` 0 पर लपेटता है।
`bits(n)` शब्द आकार 8, 16, 32 या 64 करता है, और `bits()` उसे बताता है:

{F_BITS}

ऋणात्मक मात्रा से शिफ़्ट दिशा उलट देता है (`8 << -1` = `4`)। बूलियन
`and` और `or` अपने अर्थ रखते हैं; `&` और `|` बिटवाइज़ लेखन हैं।

""",
    "es": """### 1.25 Operaciones bit a bit

Los literales de base de la sección 1.13 están hechos para esto:
`0b101`, `0o17`, `0xFF`. Los operadores bit a bit trabajan con números
enteros y responden con enteros exactos:

{F_AND}

| Operador | Significado |
|---|---|
| `a & b` | y bit a bit |
| `a \\| b` | o bit a bit |
| `a xor b` | o exclusivo bit a bit |
| `~a` | no bit a bit (complemento a dos) |
| `a << n` | desplazar a la izquierda (multiplicar por 2^n) |
| `a >> n` | desplazar a la derecha, aritmético (dividir por 2^n, redondeando hacia abajo) |

Los resultados son enteros `big` exactos, así que `1 << 60` conserva
cada dígito. El tamaño de palabra es de 64 bits por defecto: los
resultados se leen como complemento a dos con signo, así que `~0` es
-1 y `1 << 100` envuelve a 0. `bits(n)` cambia el tamaño de palabra a
8, 16, 32 o 64, y `bits()` lo informa:

{F_BITS}

Un desplazamiento negativo invierte la dirección (`8 << -1` es `4`).
El `and` y `or` booleanos conservan sus significados; `&` y `|` son
las grafías bit a bit.

""",
    "fr": """### 1.25 Opérations binaires

Les littéraux de base de la section 1.13 sont faits pour ça : `0b101`,
`0o17`, `0xFF`. Les opérateurs binaires travaillent sur des entiers et
répondent avec des entiers exacts :

{F_AND}

| Opérateur | Signification |
|---|---|
| `a & b` | et binaire |
| `a \\| b` | ou binaire |
| `a xor b` | ou exclusif binaire |
| `~a` | non binaire (complément à deux) |
| `a << n` | décalage à gauche (multiplier par 2^n) |
| `a >> n` | décalage à droite, arithmétique (diviser par 2^n, arrondi vers le bas) |

Les résultats sont des entiers `big` exacts, donc `1 << 60` garde
chaque chiffre. La taille de mot est de 64 bits par défaut : les
résultats se lisent en complément à deux signé, donc `~0` vaut -1 et
`1 << 100` enveloppe à 0. `bits(n)` change la taille de mot à 8, 16,
32 ou 64, et `bits()` la rapporte :

{F_BITS}

Un décalage négatif inverse la direction (`8 << -1` vaut `4`). Le
`and` et le `or` booléens gardent leurs sens ; `&` et `|` sont les
graphies binaires.

""",
    "ar": """### 1.25 العمليات الثنائية

الرموز الأساسية من القسم 1.13 مصنوعة لهذا: `0b101` و`0o17` و`0xFF`. عوامل
البت تعمل على الأعداد الصحيحة وتجيب بأعداد صحيحة دقيقة:

{F_AND}

| العامل | المعنى |
|---|---|
| `a & b` | و على مستوى البت |
| `a \\| b` | أو على مستوى البت |
| `a xor b` | أو الحصري على مستوى البت |
| `~a` | نفي على مستوى البت (متمم الثنائي) |
| `a << n` | إزاحة يسار (ضرب في 2^n) |
| `a >> n` | إزاحة يمين حسابية (قسمة على 2^n، تقريب لأسفل) |

النتائج أعداد صحيحة `big` دقيقة، لذا `1 << 60` يحتفظ بكل رقم. حجم الكلمة
الافتراضي 64 بت: تُقرأ النتائج كمتمم ثنائي مع الإشارة، لذا `~0` هو -1
و`1 << 100` يلتف إلى 0. يغيّر `bits(n)` حجم الكلمة إلى 8 أو 16 أو 32 أو
64، و`bits()` يعرضه:

{F_BITS}

الإزاحة بمقدار سالب تعكس الاتجاه (`8 << -1` هو `4`). يحتفظ `and` و`or`
المنطقيان بمعناهما؛ `&` و`|` هما كتابتا البت.

""",
    "de": """### 1.25 Bitoperationen

Die Basisschreibweisen aus Abschnitt 1.13 sind dafür gemacht:
`0b101`, `0o17`, `0xFF`. Die Bitoperatoren arbeiten mit ganzen Zahlen
und antworten mit exakten ganzen Zahlen:

{F_AND}

| Operator | Bedeutung |
|---|---|
| `a & b` | bitweises Und |
| `a \\| b` | bitweises Oder |
| `a xor b` | bitweises exklusives Oder |
| `~a` | bitweises Nicht (Zweierkomplement) |
| `a << n` | links schieben (mal 2^n) |
| `a >> n` | rechts schieben, arithmetisch (durch 2^n, abrunden) |

Die Ergebnisse sind exakte `big`-Ganzzahlen, also behält `1 << 60`
jede Ziffer. Die Wortbreite ist standardmäßig 64 Bit: Ergebnisse
werden als vorzeichenbehaftetes Zweierkomplement gelesen, also ist
`~0` = -1 und `1 << 100` wickelt auf 0. `bits(n)` ändert die Wortbreite
auf 8, 16, 32 oder 64, und `bits()` meldet sie:

{F_BITS}

Eine negative Verschiebung kehrt die Richtung um (`8 << -1` ist `4`).
Das boolesche `and` und `or` behalten ihre Bedeutung; `&` und `|` sind
die Bit-Schreibweisen.

""",
    "pt": """### 1.25 Operações bit a bit

Os literais de base da secção 1.13 são feitos para isto: `0b101`,
`0o17`, `0xFF`. Os operadores bit a bit trabalham com números inteiros
e respondem com inteiros exatos:

{F_AND}

| Operador | Significado |
|---|---|
| `a & b` | e bit a bit |
| `a \\| b` | ou bit a bit |
| `a xor b` | ou exclusivo bit a bit |
| `~a` | não bit a bit (complemento para dois) |
| `a << n` | deslocar à esquerda (multiplicar por 2^n) |
| `a >> n` | deslocar à direita, aritmético (dividir por 2^n, arredondando para baixo) |

Os resultados são inteiros `big` exatos, por isso `1 << 60` conserva
cada dígito. O tamanho de palavra é de 64 bits por predefinição: os
resultados são lidos como complemento para dois com sinal, por isso
`~0` é -1 e `1 << 100` envolve em 0. `bits(n)` muda o tamanho de
palavra para 8, 16, 32 ou 64, e `bits()` informa-o:

{F_BITS}

Um deslocamento negativo inverte a direção (`8 << -1` é `4`). O `and`
e o `or` booleanos mantêm os seus significados; `&` e `|` são as
grafias bit a bit.

""",
}

QR = {
    "zh-CN": """| 按位与、或 | `a & b`, `a \\| b` | `0xFF & 0x0F` |
| 按位异或 | `a xor b` | `5 xor 3` |
| 按位取反 | `~a` | `~0` |
| 移位 | `a << n`, `a >> n` | `1 << 8` |
| 字长 | `bits(n)`，取 8、16、32、64 | `bits(8)` |""",
    "hi": """| बिटवाइज़ और, या | `a & b`, `a \\| b` | `0xFF & 0x0F` |
| बिटवाइज़ अनन्य या | `a xor b` | `5 xor 3` |
| बिटवाइज़ नहीं | `~a` | `~0` |
| शिफ़्ट | `a << n`, `a >> n` | `1 << 8` |
| शब्द आकार | `bits(n)` — 8, 16, 32, 64 | `bits(8)` |""",
    "es": """| Y, O bit a bit | `a & b`, `a \\| b` | `0xFF & 0x0F` |
| O exclusivo bit a bit | `a xor b` | `5 xor 3` |
| No bit a bit | `~a` | `~0` |
| Desplazamientos | `a << n`, `a >> n` | `1 << 8` |
| Tamaño de palabra | `bits(n)` — 8, 16, 32, 64 | `bits(8)` |""",
    "fr": """| Et, ou binaires | `a & b`, `a \\| b` | `0xFF & 0x0F` |
| Ou exclusif binaire | `a xor b` | `5 xor 3` |
| Non binaire | `~a` | `~0` |
| Décalages | `a << n`, `a >> n` | `1 << 8` |
| Taille de mot | `bits(n)` — 8, 16, 32, 64 | `bits(8)` |""",
    "ar": """| و، أو على مستوى البت | `a & b`, `a \\| b` | `0xFF & 0x0F` |
| أو الحصري على مستوى البت | `a xor b` | `5 xor 3` |
| نفي على مستوى البت | `~a` | `~0` |
| إزاحات | `a << n`, `a >> n` | `1 << 8` |
| حجم الكلمة | `bits(n)` — 8، 16، 32، 64 | `bits(8)` |""",
    "de": """| Bitweises Und, Oder | `a & b`, `a \\| b` | `0xFF & 0x0F` |
| Bitweises exklusives Oder | `a xor b` | `5 xor 3` |
| Bitweises Nicht | `~a` | `~0` |
| Verschiebungen | `a << n`, `a >> n` | `1 << 8` |
| Wortbreite | `bits(n)` — 8, 16, 32, 64 | `bits(8)` |""",
    "pt": """| E, OU bit a bit | `a & b`, `a \\| b` | `0xFF & 0x0F` |
| OU exclusivo bit a bit | `a xor b` | `5 xor 3` |
| NÃO bit a bit | `~a` | `~0` |
| Deslocamentos | `a << n`, `a >> n` | `1 << 8` |
| Tamanho de palavra | `bits(n)` — 8, 16, 32, 64 | `bits(8)` |""",
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
    "zh-CN": "| 词头 | `k M G T m µ n p` 缩放任意单位 | `5 km`, `3 MPa`, `1 GHz` |",
    "hi": "| उपसर्ग | `k M G T m µ n p` किसी इकाई को स्केल करते हैं | `5 km`, `3 MPa`, `1 GHz` |",
    "es": "| Prefijos | `k M G T m µ n p` escalan cualquier unidad | `5 km`, `3 MPa`, `1 GHz` |",
    "fr": "| Préfixes | `k M G T m µ n p` modifient toute unité | `5 km`, `3 MPa`, `1 GHz` |",
    "ar": "| سوابق | `k M G T m µ n p` تدرّج أي وحدة | `5 km`, `3 MPa`, `1 GHz` |",
    "de": "| Vorsätze | `k M G T m µ n p` skalieren jede Einheit | `5 km`, `3 MPa`, `1 GHz` |",
    "pt": "| Prefixos | `k M G T m µ n p` escalam qualquer unidade | `5 km`, `3 MPa`, `1 GHz` |",
}

FAILED = []
for loc in ANCHORS2:
    path = "site/guide/%s.md" % loc
    md = io.open(path, encoding="utf-8").read()
    try:
        section = SECTIONS[loc]
        for tok, fence in F.items():
            section = section.replace("{F_%s}" % tok.upper(), fence)
        idx = md.index(ANCHORS2[loc])
        md = md[:idx] + section + "\n" + md[idx:]
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
