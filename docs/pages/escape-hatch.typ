#import "./book.typ": *
#import callout: *

#show: book-page.with(title: "Escape Hatch")

#show: render-examples

Typstyle aims to format all code consistently, but occasionally you may need to override its formatting decisions. The escape hatch is a workaround for rare cases where typstyle's output isn't suitable or when working around limitations.

#warning[
  The escape hatch should be used sparingly. It breaks consistency and may indicate areas where typstyle could be improved. Consider reporting issues instead of relying on escape hatches.
]

= When You Might Need This

Escape hatches are intended as a last resort for specific situations:

+ *Temporary workarounds* for formatting bugs
+ *Legacy code compatibility* during migration
+ *Rare edge cases* where automatic formatting genuinely harms readability
+ *Unsatisfactory formatting results* that significantly impact code clarity

Most formatting preferences can be achieved by working with typstyle's style rather than against it.

= Basic Usage

Use ```typ // @typstyle off``` or ```typ /* @typstyle off */``` to disable the formatter on the next non-trivial syntax node:

```typst
// Normal formatting applies here
#let normal = func(arg1, arg2)

// @typstyle off
#let preserved   =   bad_formatting   ;   // This line keeps its formatting

// Normal formatting resumes
#let formatted = another_func(arg1, arg2)
```

= Automatic Fallback

Typstyle automatically preserves original formatting when it encounters issues:

- *Syntax errors*: Code with parsing errors is left unchanged
- *Complex edge cases*: Rare constructs that may downgrade formatting capabilities

In these cases, you don't need an escape hatch—typstyle handles it automatically.

= Scope and Behavior

== Parbreak Penetration

The escape hatch comment must be placed directly before the code you want to preserve. It doesn't work across paragraph breaks:

```typst
// @typstyle off

#(1+2)  // This will be formatted normally because of the blank line above
```

== Comment Penetration

The escape hatch does not penetrate through comments:

```typst
// @typstyle off
/* This comment prevents turning typstyle off for the following code */
  #let formatted=func(arg1,   arg2)  // This will be formatted normally
```

== Specific Syntax Nodes

When used before `Code` or `Math`, the escape hatch applies only to the first non-trivial child:

```typst
#{
  // @typstyle off
  let preserved=bad_formatting; // Only this first statement preserves formatting
  let formatted=normal_formatting(); // This will be formatted normally
}

$
  // @typstyle off
  sin( x ) // Only the first math element preserves formatting
  cos( y ) // This part will be formatted normally
$
```


= Use Cases

#tip[
  Before using escape hatches, consider if the formatting issue could be solved by restructuring your code to work better with typstyle's conventions.
]

As a last resort, when you need to work around current limitations:

```typst
// @typstyle off
#let matrix = (
  ( 1,  0,  0,  1),
  ( 0,  1,  0,  1),
  ( 0,  0,  1,  1),
  (-1, -1, -1,  0),
)
```

```typst
// @typstyle off
#let config = (
  width:  100pt,  // Total width
  height:  50pt,  // Total height
  margin:  10pt,  // Edge spacing
)
```
