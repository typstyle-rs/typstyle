#import "../book.typ": *

#show: book-page.with(title: "Math Equation Formatting")

#show: render-examples

= Formatting Rules

typstyle applies specific formatting rules to math equations:

- Spaces are preserved around fractions when they exist
- No padding is added to the last cell in math alignments
- Backslashes are preserved
- Inline equations are never aligned or padded
- Spaces between variables and underscores are preserved: `$ #mysum _(i=0) $`

= Alignment

typstyle aligns `&` symbols in math equations, even with multiline cells. Non-block equations are never aligned:

```typst
$1/2x + y &= 3 \ y &= 3 - 1/2x$

$
F_n&=sum_(i=1)^n i^2&n > 0 \
a&<b+1&forall b < 1
$

$
a&=cases(
x + y, "if condition A",
z + w, "if condition B"
) \
b&=matrix(
1, 2;
3, 4
) \
c&=sum_(i=1)^n x_i
$
```

= Block vs Inline Equations

typstyle uses flavor detection for equations. Block equations with newlines are formatted with proper indentation:

```typst
$ F(x) = integral_0^x f(t) dif t
$

$
F(x) = integral_0^x f(t) dif t $
```

= Comments in Math

typstyle can format math equations containing comments while preserving their meaning and proper placement:

```typst
$frac(// numerator
x, /* denominator */ y)$

$mat(1, /* row 1 */ 2; 3, // row 2
4)$

$sum_(i=1 /* start */ )^(n // end
) x_i$
```
