(defproject sample "0.1.0-SNAPSHOT"
  :description "FIXME: write description"
  :url "http://example.com/FIXME"
  :min-lein-version "2.0.0"
  :dependencies [[org.clojure/clojure "1.10.3"]
                 [clojurewerkz/scrypt "1.2.0"]
                 [compojure "1.6.2"]
                 [funcool/struct "1.4.0"]
                 [hiccup "1.0.5"]
                 [org.postgresql/postgresql "42.2.23"]
                 [org.clojure/java.jdbc "0.7.12"]
                 [clj-time "0.15.2"]
                 [migratus "1.3.5"]
                 [prone "2021-04-23"]
                 [ring/ring-anti-forgery "1.3.0"]
                 [ring/ring-defaults "0.3.3"]]
  :plugins [[lein-ring "0.12.5"]
            [migratus-lein "0.5.0"]]
  :migratus {:store :database
             :migration-dir "migrations"
             :db (or (System/getenv "DATABASE_URL") "postgresql://localhost:5432/sample")}
  :ring {:handler sample.handler/app
         :init sample.handler/init
         :port    ~(System/getenv "PORT")}
  :profiles
  {:dev {:dependencies [[javax.servlet/servlet-api "2.5"]
                        [kerodon "0.9.1"]
                        [ring/ring-mock "0.4.0"]]
         :plugins [[lein-kibit "0.1.5"]
                   [lein-ancient "0.6.15"]]
         :ring {:stacktrace-middleware prone.middleware/wrap-exceptions}}
   :test {:prep-tasks [["migratus", "migrate"]]}})
