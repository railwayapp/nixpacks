<h1 class="title">Congratulations, your <a class="alert-link" href="http://luminusweb.net">Luminus</a> site is ready!</h1>

This page will help guide you through the first steps of building your site.

<p class="title is-5">Why are you seeing this page?</p>

The `home-routes` handler in the `clojure-luminus.routes.home` namespace
defines the route that invokes the `home-page` function whenever an HTTP
request is made to the `/` URI using the `GET` method.

```
(defn home-routes []
  [""
   {:middleware [middleware/wrap-csrf
                 middleware/wrap-formats]}
   ["/" {:get home-page}]
   ["/about" {:get about-page}]])
```

The `home-page` function will in turn call the `clojure-luminus.layout/render` function
to render the HTML content:

```
(defn home-page [request]
  (layout/render
    request 
    "home.html" {:docs (-> "docs/docs.md" io/resource slurp)}))
```

The `render` function will render the `home.html` template found in the `resources/html`
folder using a parameter map containing the `:docs` key. This key points to the
contents of the `resources/docs/docs.md` file containing these instructions.

The HTML templates are written using [Selmer](https://github.com/yogthos/Selmer) templating engine.

```
<div class="content">
  {{docs|markdown}}
</div>
```

<a class="level-item button" href="https://luminusweb.com/docs/html_templating.html">learn more about HTML templating »</a>



<p class="title is-5">Organizing the routes</p>

The routes are aggregated and wrapped with middleware in the `clojure-luminus.handler` namespace:

```
(mount/defstate app-routes
  :start
  (ring/ring-handler
    (ring/router
      [(home-routes)])
    (ring/routes
      (ring/create-resource-handler
        {:path "/"})
      (wrap-content-type
        (wrap-webjars (constantly nil)))
      (ring/create-default-handler
        {:not-found
         (constantly (error-page {:status 404, :title "404 - Page not found"}))
         :method-not-allowed
         (constantly (error-page {:status 405, :title "405 - Not allowed"}))
         :not-acceptable
         (constantly (error-page {:status 406, :title "406 - Not acceptable"}))}))))
```

The `app` definition groups all the routes in the application into a single handler.
A default route group is added to handle the `404` case.

<a class="level-item button" href="https://luminusweb.com/docs/routes.html">learn more about routing »</a>

The `home-routes` are wrapped with two middleware functions. The first enables CSRF protection.
The second takes care of serializing and deserializing various encoding formats, such as JSON.

<p class="title is-5">Managing your middleware</p>

Request middleware functions are located under the `clojure-luminus.middleware` namespace.

This namespace is reserved for any custom middleware for the application. Some default middleware is
already defined here. The middleware is assembled in the `wrap-base` function.

Middleware used for development is placed in the `clojure-luminus.dev-middleware` namespace found in
the `env/dev/clj/` source path.

<a class="level-item button" href="https://luminusweb.com/docs/middleware.html">learn more about middleware »</a>




<p class="title is-5">Need some help?</p>

Visit the [official documentation](https://luminusweb.com/docs/guestbook) for examples
on how to accomplish common tasks with Luminus. The `#luminus` channel on the [Clojurians Slack](http://clojurians.net/) and [Google Group](https://groups.google.com/forum/#!forum/luminusweb) are both great places to seek help and discuss projects with other users.
