#{
  ()
      // .rev()
      .rev()

  x
      // comment before method
      .method()

  a.b
    // comment before chain continuation
    .c.d

  obj
      // first comment
      .method1()
        /* second comment */
        .method2()
}

#{
  (
1
      // first comment
      + 2
      /* second comment */   -3+   4 // attached
      - 5
  )
}
