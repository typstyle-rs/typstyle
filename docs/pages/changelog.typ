#import "./book.typ": *

#show: book-page.with(title: "Changelog")

#import "../deps.typ": cmarker

#cmarker.render(read("../../CHANGELOG.md"), h1-level: 0)
