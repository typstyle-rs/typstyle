#import "./book.typ": *

#show: book-page.with(title: "Limitations")

To ensure that source code remains valid, typstyle refrains from formatting in certain scenarios. The following cases outline situations where typstyle either does not apply formatting or applies only conservative changes.

= Expressions with Comments

Currently, we are capable of formatting everything with comments.
However, when expressions contain comments, typstyle may not be able to preserve a visually pleasing layout or may even refuse to format the code. Embedded comments can interrupt alignment and grouping, making it difficult to apply standard formatting rules without altering the intended structure. In such cases, typstyle falls back to conservative formatting or skips formatting entirely to avoid breaking the code’s semantics.

We guarantee that in all supported cases the source will be formatted correctly and no comments will be lost.
If you find that a comment is lost or the formatting result is unsatisfactory due to comments, please submit a PR to present the issue.

= Spaces in Math

Math mode is highly sensitive to spacing, and users may play on content magics. Therefore, typstyle avoids changing spaces within math mode to ensure the evaluation result unchanged.

Additionally, typstyle will not convert spaces into line breaks (or vice versa) in math, as such changes can adversely affect the appearance of equations. We respect the user's intent regarding spaces and linebreaks.

= Tables

Typstyle attempts to format tables into neat, rectangular layouts—only when the table is simple enough.

Since there is no runtime function-signature provider, we treat any call named `table` or `grid` as a table and apply table layout.

We fall back to a plain layout (structure preserved) in these cases:

- The table contains a block comment or has no positional arguments.
- It lacks a `columns` argument or uses spread arguments which possibly define columns.
- The `columns` argument is not a simple constant expression.

= Compact Layout


// TODO: compact
