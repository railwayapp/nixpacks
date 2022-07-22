(ns hello-ring.core  
  (:require [ring.adapter.jetty :as jetty]))

(defn handler [request]
  {:status 200
   :headers {"Content-Type" "text/plain"}
   :body "Hello Clojure, Hello Ring!"})

(defn -main []
  (jetty/run-jetty handler {:port 3000}))