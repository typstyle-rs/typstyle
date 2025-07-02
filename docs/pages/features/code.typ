#import "../book.typ": *

#show: book-page.with(title: "Code Formatting")

#show: render-examples

= Code Formatting

== Basic Layout

Generally, there are some Typst syntaxes classified as "list-like":
- Code block (```typc { a }```)
- Array (```typc (1, 2, 3)```)
- Dictionary (```typc (a: 1, b: 2)```)
- Parameters (```typc let f(a, b) = {}```)
- Arguments (```typc f(a, b)```)
- Destructuring (```typc let (a, b) = c```)

Typstyle provides the following basic layouts for them:
- *Always-folded*: items are always put on one line regardless of width limit
- *Never-folded*: items are spread across multiple lines
- *Fitted*: items are folded if fitted the line
- *Compact*: (currently only applied to arguments) initial items are placed on the first line while the last combinable item spans multiple lines without extra indentation

=== Flavor Detection

Typstyle uses *flavor detection* to automatically choose the most suitable formatting style for list-like syntaxes.
If the first space between items contains a newline, Typstyle treats the list as "multiline" and spreads items across multiple lines (never-folded):

This can help match the original intent and maintain readability.

```typst
#let a = (1, 2, 3)
#let a = (1 , 2, 3)
#let a = (
  1, 2, 3)
#let a = (1,
2, 3)
```

```typst
#let my-f(arg1,
 arg2,
  args: none) = {
}
#let my-f(arg1, arg2,
  args: none) = {
}
```

#callout.note[
  Code will never become less expanded due to flavor detection—if the original code is already multiline, Typstyle will not collapse it into a single line.
]

== Code Block Structure

=== Single-Statement Blocks

Single-statement blocks remain inline when they fit within the line width, unless they are too long or have multiline flavor:

```typst
/// typstyle: max_width=40
#let x = if true { 1 } else { 2 }
#let x = if true {
  1 } else { 2 }
#let x = if true {
  1 } else {
     2 }
#let x = if true { "111111111111" } else { "222222222222222222222222222222" }
```

=== Linebreak Management

Typstyle strips excessive newlines in code blocks:

```typst
#{


  let x = 1

  let y = 2


}
```

== Content Blocks

When content blocks have leading or trailing spaces, they break into multiple lines for better readability:

```typst
#{
  let res = if true [ The Result is definitely true. ]else[ false. ]
}
```


== Parentheses Removal

typstyle removes unnecessary parentheses around:
- Literals
- Arrays
- Dictionaries
- Destructuring patterns
- Blocks
- Patterns

```typst
#let  (( (( ((a)),))) ) =((( ( (1)),)) )
#let  (((( (( (a)),)) ))) =((((( (1)),)) ))

#let a = ((b:((c : ((3))))))
#let a = ({(true)})
#let a = (([()]))
```

Parentheses around identifiers are preserved for safety to avoid changing semantics.

```typst
#let name = "naming";
#let a = (name: 1)
#let b = ((name): 1)
#let c = (((name)): 1)
#let d = ("name": 1)
#let e = (("name"): 1)
```

== Function Calls and Arguments

=== Compact Layout and Combinable Arguments

Typstyle chooses layouts for arguments with the following rules:

*Rules:*
+ *Empty argument list* ⟶ *always folded*
+ *Single combinable argument*:
  - The only argument is a function call ⟶ *compact*
  - Otherwise ⟶ *always folded*
+ *Multiple arguments without multiline flavor* with following conditions ⟶ *compact*
  - The last argument is *combinable* (e.g., nested call, array, dictionary, or parenthesized group)
  - All preceding arguments are simple (non-blocky)
  - The last argument must not be an array (or dictionary) if any previous argument is also an array (or dictionary).
  - No comments or intentional line breaks in arguments
  - Initial arguments can be flattened on one line
+ *Otherwise* ⟶ the original fold style (follows its flavor)

*Combinable Expression Types*:
- *Blocky expressions*: code blocks, conditionals, loops, context expressions, closures
- *Structured data*: arrays, dictionaries
- *Nested calls*: function calls, parenthesized expressions
- *Content blocks*: markup content in square brackets

#callout.note[
  We also have fine-grained check to avoid making simple expressions combinable.
]

```typst
#f(   if true {    let x = 3  })
#f(if true {
    let x = 3
  })
#f(    1111,    22222,    if true {
      let x = 3
      let y = 4
    },
  )
#f(1111,if true {let x = 3},22222, )
  #f(1111,if true {let x = 3 ;  let y = 4},22222, )
  #f(
context {
      1
    }
  )
```

```typst
#f(
  (x: 1, y: 2), (a: 3, b: 4), (m: 5, n: 6),)
#f((x: 1, y: 2), (a: 3, b: 4), (m: 5, n: 6),)

#f(xx: 1, 2, 3, yyy: [
  Multiple line
  content in array
])

#f("string", aaa: (1, 2), bbb : (x: 1, y: 2), {
  x + y
})
```

```typst
#set page(
  margin: 0.5in,
 footer: context {
  if counter(page).display() == "2" {
    [test]
  } else {
    []
  }
})

#assert.eq(parse(("asd",)), (description: "asd", types: none))
```

#callout.warning[
  This feature is still experimental and may not always produce the desired results. Its behavior may improve in future versions.
]

== Chainable Expressions

=== Binary Chains

Binary expressions are formatted as operator chains with proper breaking and alignment:

```typst
/// typstyle: max_width=40
#let _is_block(e,fn)=fn==heading or (fn==math.equation and e.block) or (fn==raw and e.has("block") and e.block) or fn==figure or fn==block or fn==list.item or fn==enum.item or fn==table or fn==grid or fn==align or (fn==quote and e.has("block") and e.block)
```

=== Dot Chains

Dot chains are broken intelligently based on complexity and function calls:

```typst
// Simple chains stay inline
#node.pos.xyz

// Complex chains with multiple calls break
#{let hlines_below_header = first-row-group-long-long.row_group-long-long-long-long.hlines-long-long-long-long}

#{
  let (title, _) = query(heading.where(level: 1)).map(e => (e.body, e.location().page())).rev().find(((_, v)) => v <= page)
}

#{padding.pairs().map((k, x) => (k, x * 1.5)).to-dict()}
```

== Import Statements

=== Item Ordering

Import items are sorted alphabetically by default:

```typst
#import "module.typ": zebra,alpha,beta,gamma
```

=== Soft Wrapping

Import statements use soft wrapping for long item lists, keeping them compact yet readable:

```typst
#import "module.typ": very,long,list,of,imported,items,that,exceeds,line,width,and_,continues,wrapping

#import "@preview/fletcher:0.5.7" as fletcher:diagram,node,edge
```
