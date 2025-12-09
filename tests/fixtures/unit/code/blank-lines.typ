// Test preserving blank lines in code

// Arrays
#let arr = (

  1 + 2,

  // Second element
  (

    4,

    5,


    6,

  ),


  // Third element
  "hello" + " world",

)

// Dicts
#let dict = (

  number: 42,

  // Computed value
  sum: 1 + 2,


  // Nested dictionary
  nested: (

    inner1: "a",

    inner2: "b",

  ),


  // String key with special chars
  "key-with-dashes": "value",

  none_val: none,

)

// Function calls
#complex-func(

  "positional",

  // Computed expression
  1 + 2,


  // Named argument
  name: "value",

  // Nested call
  inner(

    x,

    y,

  ),


  // Spread operator
  ..args,

  key: "final",

)

// Imports
#import "module.typ": (

  item1,

  // Second import
  item2,


  // Third import with renaming
  old-name as new-name,

  item4,

)

// Let-bindings
#let (

  outer1,

  // Comment on nested pattern
  (

    inner1,

    inner2,

  ),


  // Rest pattern
  ..rest

) = (1, (2, 3), 4, 5, 6)

// Show rules
#show heading.where(

  level: 1,

  outlined: true,

): it => {}

// Set rule with blank lines in arguments
#set text(


  font: "New Computer Modern",

  size: 11pt,

  lang: "en",

)
