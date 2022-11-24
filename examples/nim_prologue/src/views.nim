import prologue

proc index*(ctx: Context) {.async.} =
  resp "<h1>Hello from Nim with Prologue built using Nixpacks!</h1>"
