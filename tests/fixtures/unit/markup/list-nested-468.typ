/// typstyle: wrap-text

// Issue #468: nested list items flattened with wrap_text enabled

// Minimal repro: text followed by list item with block comment
Testing
- /* comment */

// Nested list with comment after first sub-item
- Parent
  - Sub 1 // comment
  - Sub 2

// Nested list in content block
#[
  - Parent
    - Sub 1 // comment
    - Sub 2
]

// Enum items
+ Parent
  + Sub 1 // comment
  + Sub 2

// Term items
/ Term: Description
  - Sub 1 // comment
  - Sub 2

// Heading followed by list
= Heading
- Item 1
- Item 2
