(ns clojure-luminus.env
  (:require [clojure.tools.logging :as log]))

(def defaults
  {:init
   (fn []
     (log/info "\n-=[clojure-luminus started successfully]=-"))
   :stop
   (fn []
     (log/info "\n-=[clojure-luminus has shut down successfully]=-"))
   :middleware identity})
