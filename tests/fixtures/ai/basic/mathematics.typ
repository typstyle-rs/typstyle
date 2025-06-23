/// typstyle: wrap_text

#set page(margin: 1in)
#set text(font: "New Computer Modern", size: 11pt)
#set math.equation(numbering: "(1)")

= Mathematical Typography Showcase

== Linear Algebra

=== Matrix Operations

Matrix multiplication follows the rule:
$ C_(i j) = sum_(k=1)^n A_(i k) B_(k j) $

For example, multiplying two 2×2 matrices:
$ mat(a, b; c, d) mat(e, f; g, h) = mat(a e + b g, a f + b h; c e + d g, c f + d h) $

The determinant of a 2×2 matrix is:
$ det(mat(a, b; c, d)) = a d - b c $

For a 3×3 matrix, the determinant expands to:
$ det(mat(a, b, c; d, e, f; g, h, i)) = a(e i - f h) - b(d i - f g) + c(d h - e g) $

=== Eigenvalues and Eigenvectors

The characteristic polynomial of a matrix $A$ is:
$ p(lambda) = det(A - lambda I) = 0 $

For eigenvalue $lambda$ and eigenvector $bold(v)$:
$ A bold(v) = lambda bold(v) $

Example with a specific matrix:
$ mat(2, 1; 1, 2) mat(x; y) = lambda mat(x; y) $

This gives us the system:
$ cases(
  (2 - lambda)x + y = 0,
  x + (2 - lambda)y = 0
) $

== Calculus

=== Derivatives and Chain Rule

The chain rule for composition of functions:
$ frac(d, d x) f(g(x)) = f'(g(x)) dot g'(x) $

For multivariable functions, the total derivative is:
$ (d f)/(d x) = (partial f)/(partial x) + (partial f)/(partial y) (d y)/(d x) $

=== Integration Techniques

Integration by parts:
$ integral u dif v = u v - integral v dif u $

Substitution method:
$ integral f(g(x)) g'(x) dif x = integral f(u) dif u $ where $ u = g(x) $

Partial fractions for rational functions:
$ integral frac(P(x), Q(x)) dif x = integral (A_1)/(x - r_1) + (A_2)/(x - r_2) + ... dif x $

=== Advanced Integration

The Beta function:
$ B(p, q) = integral_0^1 t^(p-1) (1-t)^(q-1) dif t = frac(Gamma(p) Gamma(q), Gamma(p + q)) $

The Gamma function:
$ Gamma(n) = integral_0^infinity t^(n-1) e^(-t) dif t $

For integer values: $ Gamma(n) = (n-1)! $

== Series and Sequences

=== Taylor and Maclaurin Series

General Taylor series expansion:
$ f(x) = sum_(n=0)^infinity frac(f^((n))(a), n!) (x - a)^n $

Common Maclaurin series ($ a = 0 $):

$ e^x = sum_(n=0)^infinity frac(x^n, n!) = 1 + x + frac(x^2, 2!) + frac(x^3, 3!) + ... $

$ sin(x) = sum_(n=0)^infinity frac((-1)^n x^(2n+1), (2n+1)!) = x - frac(x^3, 3!) + frac(x^5, 5!) - ... $

$ cos(x) = sum_(n=0)^infinity frac((-1)^n x^(2n), (2n)!) = 1 - frac(x^2, 2!) + frac(x^4, 4!) - ... $

$ ln(1 + x) = sum_(n=1)^infinity frac((-1)^(n+1) x^n, n) = x - frac(x^2, 2) + frac(x^3, 3) - ... $ for $ |x| < 1 $

=== Fourier Series

A periodic function can be expressed as:
$ f(x) = frac(a_0, 2) + sum_(n=1)^infinity (a_n cos(frac(n pi x, L)) + b_n sin(frac(n pi x, L))) $

Where:
$ a_n = frac(1, L) integral_(-L)^L f(x) cos(frac(n pi x, L)) dif x $

$ b_n = frac(1, L) integral_(-L)^L f(x) sin(frac(n pi x, L)) dif x $

== Probability and Statistics

=== Probability Distributions

Normal distribution:
$ f(x) = frac(1, sigma sqrt(2 pi)) e^(-frac((x - mu)^2, 2 sigma^2)) $

Standard normal ($ mu = 0, sigma = 1 $):
$ phi(z) = frac(1, sqrt(2 pi)) e^(-frac(z^2, 2)) $

Binomial distribution:
$ P(X = k) = binom(n, k) p^k (1-p)^(n-k) $

Poisson distribution:
$ P(X = k) = frac(lambda^k e^(-lambda), k!) $

=== Central Limit Theorem

For large $n$, the sample mean $overline(X)$ approaches:
$ overline(X) tilde N(mu, frac(sigma^2, n)) $

Standardized form:
$ Z = frac(overline(X) - mu, sigma/sqrt(n)) tilde N(0, 1) $

== Differential Equations

=== First-Order ODEs

Separable equations:
$ frac(d y, d x) = g(x) h(y) $

Solution: $ integral frac(d y, h(y)) = integral g(x) dif x $

Linear first-order:
$ frac(d y, d x) + P(x)y = Q(x) $

Solution using integrating factor $ mu(x) = e^(integral P(x) dif x) $:
$ y = frac(1, mu(x)) (integral mu(x) Q(x) dif x + C) $

=== Second-Order ODEs

Homogeneous with constant coefficients:
$ a y'' + b y' + c y = 0 $

Characteristic equation: $ a r^2 + b r + c = 0 $

Solutions depend on discriminant $ Delta = b^2 - 4 a c $:
- $ Delta > 0 $: $ y = C_1 e^(r_1 x) + C_2 e^(r_2 x) $
- $ Delta = 0 $: $ y = (C_1 + C_2 x) e^(r x) $
- $ Delta < 0 $: $ y = e^(alpha x) (C_1 cos(beta x) + C_2 sin(beta x)) $

== Complex Analysis

=== Euler's Formula and Complex Exponentials

Euler's formula:
$ e^(i theta) = cos(theta) + i sin(theta) $

De Moivre's theorem:
$ (cos(theta) + i sin(theta))^n = cos(n theta) + i sin(n theta) $

Complex exponential form:
$ z = r e^(i theta) $ where $ r = |z| $ and $ theta = arg(z) $

=== Residue Theorem

For a function $f$ with isolated singularities:
$ integral_C f(z) dif z = 2 pi i sum "Res"(f, z_k) $

Where the sum is over all singularities $z_k$ inside contour $C$.

== Advanced Topics

=== Vector Calculus

Gradient: $ gradient f = nabla f = (frac(partial f, partial x), frac(partial f, partial y), frac(partial f, partial z)) $

Divergence: $ "div" bold(F) = nabla dot bold(F) = frac(partial F_x, partial x) + frac(partial F_y, partial y) + frac(partial F_z, partial z) $

Curl: $ "curl" bold(F) = nabla times bold(F) = mat(delim: "|", bold(i), bold(j), bold(k); frac(partial, partial x), frac(partial, partial y), frac(partial, partial z); F_x, F_y, F_z) $

=== Partial Differential Equations

Heat equation:
$ frac(partial u, partial t) = alpha frac(partial^2 u, partial x^2) $

Wave equation:
$ frac(partial^2 u, partial t^2) = c^2 frac(partial^2 u, partial x^2) $

Laplace equation:
$ nabla^2 u = frac(partial^2 u, partial x^2) + frac(partial^2 u, partial y^2) = 0 $

#let hbar = sym.planck.reduce
Schrödinger equation:
$ i hbar frac(partial psi, partial t) = hat(H) psi = (-frac(hbar^2, 2m) nabla^2 + V) psi $
