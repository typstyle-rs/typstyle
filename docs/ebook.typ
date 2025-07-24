#import "deps.typ": shiroa
#import shiroa: *

#import "templates/ebook.typ"
#import "templates/version.typ": package

#show: ebook.project.with(title: [Typstyle Documentation (v#package.version)], spec: "book.typ")

#external-book(spec: include "/docs/book.typ")

// set a resolver for inclusion
#ebook.resolve-inclusion(it => include "pages/" + it)
