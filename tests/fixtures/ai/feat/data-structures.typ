
= Data Structure Formatting Examples

== Nested Arrays and Matrices

// Multi-dimensional arrays with various formatting patterns
#let matrix-data = (
  (1, 2, 3, 4, 5),
  (6, 7, 8, 9, 10),
  (11, 12, 13, 14, 15),
  (16, 17, 18, 19, 20)
)

// Complex nested array structures
#let hierarchical-data = (
  level1: (
    level2a: (
      items: ("alpha", "beta", "gamma"),
      metadata: (count: 3, type: "greek")
    ),
    level2b: (
      items: ("one", "two", "three", "four", "five"),
      metadata: (count: 5, type: "numeric")
    )
  ),
  config: (
    display: "hierarchical",
    sorting: "alphabetical",
    filters: ("active", "visible", "public")
  )
)

// Array of dictionaries with mixed content types
#let dataset = (
  (
    id: 1,
    name: "Alice Johnson",
    scores: (85, 92, 78, 95),
    metadata: (
      department: "Computer Science",
      year: 2024,
      active: true,
      projects: ("AI Research", "Web Development", "Data Analysis")
    )
  ),
  (
    id: 2,
    name: "Bob Smith",
    scores: (88, 76, 92, 89),
    metadata: (
      department: "Mathematics",
      year: 2023,
      active: false,
      projects: ("Statistics", "Probability Theory")
    )
  ),
  (
    id: 3,
    name: "Carol Williams",
    scores: (95, 88, 91, 87),
    metadata: (
      department: "Physics",
      year: 2024,
      active: true,
      projects: ("Quantum Computing", "Thermodynamics", "Optics", "Particle Physics")
    )
  )
)

== Dictionary Formatting with Complex Values

// Configuration dictionary with nested structures
#let app-config = (
  database: (
    host: "localhost",
    port: 5432,
    credentials: (
      username: "admin",
      password: "secure-password-123"
    ),
    pools: (
      read: (min: 5, max: 20, timeout: 30),
      write: (min: 2, max: 10, timeout: 60)
    )
  ),
  api: (
    endpoints: (
      users: "/api/v1/users",
      auth: "/api/v1/auth",
      data: "/api/v1/data"
    ),
    rate-limiting: (
      requests-per-minute: 100,
      burst-size: 20,
      whitelist: ("admin", "service-account")
    ),
    middleware: (
      auth: true,
      logging: true,
      compression: "gzip",
      cors: (
        origins: ("https://example.com", "https://app.example.com"),
        methods: ("GET", "POST", "PUT", "DELETE"),
        headers: ("Content-Type", "Authorization")
      )
    )
  ),
  features: (
    experimental: (
      "advanced-search": true,
      "real-time-updates": false,
      "ai-recommendations": true
    ),
    deprecated: (
      "legacy-api": false,
      "old-ui": false
    )
  )
)

== Function Processing Arrays

// Complex array manipulation functions
#let process-scores(data) = {
  data.map(student => {
    let avg = student.scores.fold(0, (sum, score) => sum + score) / student.scores.len()
    let grade = if avg >= 90 {
      "A"
    } else if avg >= 80 {
      "B"
    } else if avg >= 70 {
      "C"
    } else if avg >= 60 {
      "D"
    } else {
      "F"
    }

    (
      ..student,
      average: calc.round(avg, digits: 2),
      grade: grade,
      status: if student.metadata.active { "Active" } else { "Inactive" }
    )
  })
}

#let filter-and-group(data, criteria) = {
  let filtered = data.filter(item =>
    criteria.keys().all(key =>
      if key in item {
        item.at(key) == criteria.at(key)
      } else if key in item.metadata {
        item.metadata.at(key) == criteria.at(key)
      } else {
        false
      }
    )
  )

  // Group by department
  let groups = (:)
  for item in filtered {
    let dept = item.metadata.department
    if dept not in groups {
      groups.insert(dept, ())
    }
    groups.at(dept).push(item)
  }

  groups
}

== Table Generation from Complex Data

#let create-summary-table(processed-data) = {
  table(
    columns: (auto, 1fr, auto, auto, auto, auto),
    stroke: 0.5pt,
    fill: (col, row) => if row == 0 { rgb("#f0f0f0") },

    [*ID*], [*Name*], [*Avg*], [*Grade*], [*Department*], [*Status*],

    ..processed-data.map(student => (
      str(student.id),
      student.name,
      str(student.average),
      student.grade,
      student.metadata.department,
      student.status
    )).flatten()
  )
}

// Demonstrating array comprehensions and complex transformations
#let statistical-analysis = (
  summary: dataset.fold((total: 0, count: 0, by-dept: (:)), (acc, student) => {
    let dept = student.metadata.department
    let avg = student.scores.fold(0, (sum, score) => sum + score) / student.scores.len()

    // Update department statistics
    if dept not in acc.by-dept {
      acc.by-dept.insert(dept, (total: 0, count: 0, students: ()))
    }

    acc.by-dept.at(dept).total += avg
    acc.by-dept.at(dept).count += 1
    acc.by-dept.at(dept).students.push(student.name)

    // Update overall statistics
    (
      total: acc.total + avg,
      count: acc.count + 1,
      by-dept: acc.by-dept
    )
  }),

  rankings: dataset
    .map(s => (
      name: s.name,
      avg: s.scores.fold(0, (sum, score) => sum + score) / s.scores.len()
    ))
    .sorted(key: s => s.avg)
    .rev(),

  active-projects: dataset
    .filter(s => s.metadata.active)
    .map(s => s.metadata.projects)
    .flatten()
    .dedup()
    .sorted()
)

== Display Results

#let processed = process-scores(dataset)
#let active-students = filter-and-group(processed, (active: true))

=== Student Summary
#create-summary-table(processed)

=== Department Statistics
#for (dept, students) in active-students [
  ==== #dept Department
  - Active students: #students.len()
  - Average score: #calc.round(
    students.fold(0, (sum, s) => sum + s.average) / students.len(),
    digits: 2
  )
  - Students: #students.map(s => s.name).join(", ")
]

=== Project Overview
Active projects across all departments:
#list(..statistical-analysis.active-projects.map(project => [#project]))

=== Top Performers
#table(
  columns: (1fr, auto),
  [*Student*], [*Average*],
  ..statistical-analysis.rankings.slice(0, 3).map(r => (
    r.name,
    str(calc.round(r.avg, digits: 2))
  )).flatten()
)
