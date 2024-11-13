(use-modules (haunt asset)
             (haunt builder blog)
             (haunt builder assets)
             (haunt reader commonmark)
             (haunt site))

(site #:title "Built with Guile"
      #:domain "example.com"
      #:default-metadata
      '((author . "John Doe")
        (email  . "jdoe@hotmail.com"))
      #:readers (list)
      #:builders (list)
)