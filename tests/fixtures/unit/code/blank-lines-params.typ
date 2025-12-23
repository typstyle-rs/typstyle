// Test preserving blank lines between parameters with doc-comments

#let func(

  /// Description of one
  /// parameter.
    /// -> int
  hey,



  /// Description of another
    /// parameter.
  /// -> array | content
  ho,


  pos,

  /// Named parameter
  named: "default",


    /// Rest parameter
  .. rest


) = {}
