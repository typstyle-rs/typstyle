#table(columns: 2, [1], [2], [3], [4])

#grid(columns: 2, [1], [2], [3], [4])

#std.table(columns: 2, [1], [2], [3], [4])

#std.grid(columns: 2, [1], [2], [3], [4])

#std.table(columns: 2, [1], [2], [3], [4])[5][6]

#std.grid(columns: 2, [1], [2], [3], [4])[5][6]

#table(columns: 2, std.table.header([1], [2], [3], [4]), [3], [4])

#grid(columns: 2, [1], [2], std.grid.footer([1], [2], [3], [4])[5][6])

#{
  table(
    columns: 2,
    [1], [2], [3], [4]
  )

  std.table(
    columns: 2,
    [1], [2], [3], [4]
  )

  grid(
    columns: 2,
    [1], [2], [3], [4]
  )

  std.grid(
    columns: 2,
    [1], [2], [3], [4]
  )
}
