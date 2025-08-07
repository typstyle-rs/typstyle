/// typstyle: format_doc_comments wrap_doc_comments
// Test cases for doc comment formatting - markup parsing and grouping

///  This is a simple doc comment with valid markup
#let simple = 1

///  This has valid markup: *bold* and `code` and _emphasis_
#let valid_markup = 2

///  Doc comment with code expressions: #(1+2) and #let x=5
///  Also has badly formatted code: #(a+b*c/d) and #calc.pow(2,3)
#let with_code_expressions = 3

///  This has invalid markup: *unclosed strong
///  and broken `code span
///  Should fallback to plain text formatting
#let invalid_markup = 4

///  Code with function calls: #text(size:12pt)[Hello] and #rect(width:100pt,height:50pt)
///  Should be formatted properly
#let with_function_calls = 6

///  Very long comment that should be wrapped when wrap_doc_comments is enabled and exceeds doc_comment_width
#let long_comment = 7
