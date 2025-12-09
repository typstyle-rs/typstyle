// Test preserving blank lines between parameters with doc-comments

/// Function with doc-commented parameters
#let func1(

  /// Description of one 
  /// parameter. 
  /// -> int
  hey,

  /// Description of another
  /// parameter. 
  /// -> array | content
  ho,

) = {}

// Parameters without doc-comments should also preserve blank lines
#let func2(

  a,

  b,

  c,

) = {}

// Mixed: some with doc-comments, some without
#let func3(

  /// Documented parameter
  x,

  y,

  /// Another documented parameter
  z,

) = {}

// Named parameters with doc-comments
#let func4(

  /// Positional parameter
  pos,

  /// Named parameter with default
  named: "default",

  /// Rest parameter
  ..rest

) = {}

// Multiple blank lines should be capped at configured limit (default: 2)
#let func5(

  /// First param
  a,




  /// Second param (should have max 2 blank lines above)
  b,

) = {}
