#import "@preview/cetz:0.4.0": canvas, draw
#import "@preview/cetz-plot:0.1.2": plot

#set page(margin: 0.8in)
#set heading(numbering: "1.1")

#let random(seed: 0) = 0

= Advanced Document with Mixed Content

== Research Paper with Dynamic Content

This document demonstrates the seamless integration of *markup content* with _embedded code expressions_, showcasing how Typstyle formats complex documents that mix narrative text with computational elements.

The current date is #datetime.today().display("[weekday repr:long], [month repr:long] [day], [year]"), and this document was compiled using #sys.inputs.keys().len() system inputs.

=== Dynamic Data Generation and Visualization

#let generate-sample-data(n: 50, seed: 42) = {
  // Set random seed for reproducibility
  let data = ()
  for i in range(n) {
    let x = i / (n - 1) * 10
    let y = calc.sin(x) + random(seed: seed + i) * 0.3 - 0.15
    data.push((x, y))
  }
  data
}

Our analysis begins with a dataset of *#generate-sample-data().len() data points* generated using the following mathematical model:

$ y = sin(x) + epsilon $ where $epsilon tilde cal(N)(0, 0.15^2)$

#figure(
  canvas({
    import draw: *

    let data = generate-sample-data()

    // Create plot
    plot.plot(
      size: (8, 6),
      x-tick-step: 2,
      y-tick-step: 0.5,
      x-min: 0, x-max: 10,
      y-min: -2, y-max: 2,
      {
        // Plot data points
        plot.add(
          data,
          style: (stroke: blue + 0.5pt),
          mark: "o",
          mark-size: 0.05
        )

        // Plot theoretical sine curve
        plot.add(
          domain: (0, 10),
          x => calc.sin(x),
          style: (stroke: red + 1pt),
          samples: 100
        )
      }
    )
  }),
  caption: [Sample data (blue points) compared to theoretical sine function (red line)]
)

=== Statistical Analysis Integration

The following code block demonstrates *inline statistical calculations* embedded within narrative text:

#{
  let data = generate-sample-data()
  let y-values = data.map(point => point.at(1))
  let mean = y-values.fold(0, (sum, val) => sum + val) / y-values.len()
  let variance = y-values.fold(0, (sum, val) => calc.pow(val - mean, 2)) / y-values.len()
  let std-dev = calc.sqrt(variance)

  [
    Our analysis reveals that the sample mean is approximately *#calc.round(mean, digits: 4)*
    with a standard deviation of *#calc.round(std-dev, digits: 4)*. This aligns well with
    the theoretical expectation that the mean should be close to *0* for a sine function
    over the interval $[0, 10]$.

    The coefficient of variation is #calc.round(std-dev / calc.abs(mean) * 100, digits: 2)%,
    indicating #if std-dev / calc.abs(mean) < 0.5 [relatively low] else [high] variability
    in our measurements.
  ]
}

=== Complex Document Structure

==== Multi-level Lists with Embedded Calculations

The research methodology consists of several phases:

1. *Data Collection Phase*
   - Sample size: #generate-sample-data().len() observations
   - Time period: #calc.ceil(generate-sample-data().len() / 10) weeks
   - Measurement frequency: Daily recordings

   + Data validation using the following criteria:
     - Outlier detection: $|z| > 2.576$ (99% confidence level)
     - Missing value tolerance: < 5% per variable
     - Consistency checks across #range(5).len() different sensors

2. *Analysis Phase*
   - Descriptive statistics for all #{
     let vars = ("temperature", "humidity", "pressure", "wind_speed")
     str(vars.len())
   } variables
   - Correlation analysis between environmental factors
   - Time series decomposition for trend identification

3. *Reporting Phase*
   - Generate #(range(20).map(i => i + 1).filter(x => calc.rem(x, 5) == 0)).len() summary charts
   - Prepare findings for publication in #{
     let journals = ("Nature", "Science", "PNAS", "Journal of Climate")
     str(journals.len())
   } target journals

==== Interactive Content Blocks

#let create-comparison-table(datasets) = {
  table(
    columns: (auto, ..datasets.map(_ => auto)),
    stroke: 0.5pt,
    fill: (col, row) => if row == 0 { rgb("#e8f4fd") } else if calc.rem(row, 2) == 0 { rgb("#f8f9fa") },

    [*Metric*], ..datasets.map(d => [*Dataset #d.id*]),

    [Mean], ..datasets.map(d => [#calc.round(d.stats.mean, digits: 3)]),
    [Std Dev], ..datasets.map(d => [#calc.round(d.stats.std, digits: 3)]),
    [Min], ..datasets.map(d => [#calc.round(d.stats.min, digits: 3)]),
    [Max], ..datasets.map(d => [#calc.round(d.stats.max, digits: 3)]),
    [Range], ..datasets.map(d => [#calc.round(d.stats.max - d.stats.min, digits: 3)]),
  )
}

Here we compare multiple datasets generated under different conditions:

#{
  let datasets = range(4).map(i => {
    let data = generate-sample-data(n: 30, seed: 100 + i * 10)
    let values = data.map(p => p.at(1))
    (
      id: i + 1,
      stats: (
        mean: values.fold(0, (s, v) => s + v) / values.len(),
        std: {
          let mean = values.fold(0, (s, v) => s + v) / values.len()
          let variance = values.fold(0, (s, v) => s + calc.pow(v - mean, 2)) / values.len()
          calc.sqrt(variance)
        },
        min: values.fold(values.first(), calc.min),
        max: values.fold(values.first(), calc.max)
      )
    )
  })

  create-comparison-table(datasets)
}

=== Advanced Formatting Showcase

==== Code Integration with Conditional Content

#let process-experimental-data(condition: "control") = {
  let base-effects = (
    control: (effect: 0.0, variance: 0.1),
    treatment-a: (effect: 0.3, variance: 0.15),
    treatment-b: (effect: 0.5, variance: 0.2)
  )

  let params = base-effects.at(condition)
  let samples = range(20).map(i => params.effect + (random(seed: i) - 0.5) * params.variance)

  (
    condition: condition,
    samples: samples,
    mean: samples.fold(0, (s, v) => s + v) / samples.len(),
    effect-size: params.effect
  )
}

Our experimental design included three conditions. The results show:

#for condition in ("control", "treatment-a", "treatment-b") [
  - *#condition.replace("-", " ").split(" ").map(word => upper(word.first()) + word.slice(1)).join(" "):*
    #{
      let results = process-experimental-data(condition: condition)
      let significance = if calc.abs(results.mean) > 0.2 { "significant" } else { "non-significant" }
      [
        Mean effect = #calc.round(results.mean, digits: 3),
        which is #significance at α = 0.05
        (#if results.mean > 0.2 [✓] else if results.mean < -0.2 [✗] else [~])
      ]
    }
]

==== Mathematical Content Integration

The mathematical foundation of our analysis relies on several key theorems. For instance, the *Central Limit Theorem* states that for sufficiently large sample sizes:

$ overline(X) tilde cal(N)(mu, sigma^2/n) $

In our case, with $n = #{generate-sample-data().len()}$ samples, we can apply this theorem because $n > 30$. The sampling distribution of our mean has:

- Expected value: $E[overline(X)] = #calc.round(generate-sample-data().map(p => p.at(1)).fold(0, (s, v) => s + v) / generate-sample-data().len(), digits: 4)$
- Standard error: $"SE" = sigma/sqrt(n) ≈ #{
  let data = generate-sample-data()
  let values = data.map(p => p.at(1))
  let mean = values.fold(0, (s, v) => s + v) / values.len()
  let std = calc.sqrt(values.fold(0, (s, v) => s + calc.pow(v - mean, 2)) / values.len())
  calc.round(std / calc.sqrt(values.len()), digits: 4)
}$

This allows us to construct confidence intervals and perform hypothesis tests with known distributional properties.

=== Conclusion with Dynamic Summary

Our comprehensive analysis of #{generate-sample-data().len()} data points across #{("control", "treatment-a", "treatment-b").len()} experimental conditions has revealed several important findings:

1. The theoretical model explains #{calc.round(random(seed: 999) * 0.3 + 0.7, digits: 1) * 100}% of the observed variance
2. Statistical significance was achieved in #{("treatment-a", "treatment-b").filter(c => process-experimental-data(condition: c).mean > 0.2).len()} out of #{("treatment-a", "treatment-b").len()} treatment conditions
3. The effect sizes range from #{
  let effects = ("treatment-a", "treatment-b").map(c => process-experimental-data(condition: c).mean)
  calc.round(effects.fold(effects.first(), calc.min), digits: 2)
} to #{
  let effects = ("treatment-a", "treatment-b").map(c => process-experimental-data(condition: c).mean)
  calc.round(effects.fold(effects.first(), calc.max), digits: 2)
}

These results have important implications for future research directions and practical applications in the field. The automated generation of this summary ensures that all numerical values remain consistent throughout the document, demonstrating the power of *computational documents* created with Typst.

#align(center)[
  _This document contains #{
    // Count total words approximately
    let content-words = 850  // Approximate word count
    content-words
  } words and was automatically formatted using Typstyle._
]
