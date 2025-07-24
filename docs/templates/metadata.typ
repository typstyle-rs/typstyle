#import "../deps.typ": shiroa
#import shiroa: get-page-width, is-html-target, is-pdf-target, is-web-target, shiroa-sys-target

// Metadata
#let page-width = get-page-width()
#let is-html-target = is-html-target()
#let is-pdf-target = is-pdf-target()
#let is-web-target = is-web-target()
#let sys-is-html-target = ("target" in dictionary(std))
