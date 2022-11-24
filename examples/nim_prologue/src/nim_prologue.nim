import prologue
import ./urls


let
  settings = newSettings(appName = "nim_prologue",
                         debug = true,
                         port = Port(8080),
    )

var app = newApp(settings = settings)

app.addRoute(urls.urlPatterns, "")
app.run()
