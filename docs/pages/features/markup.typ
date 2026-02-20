#import "../book.typ": *

#show: book-page.with(title: "Markup Formatting")

#show: render-examples

We strive to ensure that the eval results remain unchanged as much as possible. However, there are still some tricky (rare) cases that we cannot handle well, listed below.

= Lists and Enumerations

== List Indentation

List indentation is automatically corrected and standardized:

```typst
 -   Fruit
      - Apple
  -     Banana
- Vegetable
      -    Carrot
      - Tomato
```

== Lists in Content Blocks

Lists within content blocks are properly formatted with surrounding linebreaks:

```typst
#{
  [- single]
  [- indented
  - less
  ]
  [- indented
   - same
  - then less
   - then same
  ]
  [- indented
    - more
   - then same
  - then less
  ]
}
```

#callout.caution[
  For now, we can only safely handle the case that there is at most one list item in the markup body if that item appears on the first line.
  In other cases, we will prepend a line break to prevent mis-alignment of items, which is unsafe, as it changes the representation (one space prepended).

  ```typst
    #[- first // note: leading spaces
        - second ] // this is safe
  ```
  ```typst
  #[- a
      - b
    - c
  ] // this is unsafe
  ```

  This is a known limitation.
]

= Text Wrapping

When text wrapping is enabled with `--wrap-text`, typstyle intelligently wraps long lines while preserving important formatting and semantic structure:

```typst
/// typstyle: wrap_text, max_width=30
Let's say you have a long text that needs to be wrapped in the markup. This is a very long sentence.
```
== Wrapping Rules

typstyle applies specific wrapping logic based on node types:

- *Cannot break before*: Markup markers (`=`, `+`, `-`, `/`) and labels to prevent misinterpretation
- *Force hard breaks*: Around block equations to keep them on exclusive lines
- *Preserve breaks after*: Block elements, line comments, and structural nodes (code blocks, conditionals, loops)
- *Exclusive lines*: Single non-text nodes or hash-prefixed expressions (e.g., `#figure()`)
- *Soft breaks*: Regular spaces become flexible break points unless restricted by above rules

````typst
/// typstyle: wrap_text, max_width=30
Some pieces should be exclusion except $"inline equation"$ and `raw`:
- This is a list
$ "block equation" $
#figure([Figures])
#figure([Labeled figures]) <label>
That's all! Wait, a linebreak \
should be \ kept at the end
\ of the line. \
But \ / never \ = break \ + before \ - markers
#[Never \ / Give \ = You \ + Up \
]
#([Man!\ ], [\ What\ can\ I\ say\ ?])

#let fig = (..) => []
#fig("image1")[Caption] <label1>
#fig("image2")[Caption] <label2>

This is a code block: #{
  // code block!
}
I don't want to eat the linebreak.
A blocky node should not stick #[

]
the content after it.
Not #if true { }
Not #while false { }
Not #for i in range(5) {}
Not #context 1
And block equations: $ eq $
And
also ``` block
  raw ```
but not ``` non-block raw ```
End
...
````


== Multilingual Text Support

typstyle measures Unicode width and will not break between words if no space exists in the original text.

```typst
/// typstyle: wrap_text, max_width=40
这是一个中文段落，包含链接 https://typst.app/ 和*强调文本*。
続いて`コード要素`と https://docs.typst.app/ を含む日本語の段落です。

Mixed CJK and Latin: Visit 访问 https://example.com/文档 for documentation.
한글과 URL: https://한글.kr/ contains Korean text mixed with $alpha + beta$.

Multiple scripts: أهلاً بك في *타이프스트* เอกสาร with `inline code`.
```
