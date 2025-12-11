// Show rule with complex selector
#show heading.where(

  level: 1,


  outlined: true,

): it => {

  pagebreak(weak: true)



  text(size: 24pt, weight: "bold", it.body)

}

// Set rule with blank lines in arguments
#set text(


  font: "New Computer Modern",

  size: 11pt,

  lang: "en",

)
