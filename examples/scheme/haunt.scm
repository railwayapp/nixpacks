(use-modules (haunt asset)
             (haunt builder blog)
             (haunt builder assets)
             (haunt reader commonmark)
             (haunt site))

(site #:title "Built with Guile"
      #:domain "example.com"
      #:default-metadata
      '((author . "siarune")
        (email  . "aidan.sharp@siarune.dev"))
      #:readers (list commonmark-reader)
      #:builders (list  )
)