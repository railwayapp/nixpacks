open Microsoft.AspNetCore.Builder
open System


type WeatherForecast = {
    Date: DateTime
    TemperatureC: int
    Summary: string option
}
with member x.TemperatureF = 32 + int (float x.TemperatureC / 0.5556)

let Summaries =  ["Freezing"; "Bracing"; "Chilly"; "Cool"; "Mild"; "Warm"; "Balmy"; "Hot"; "Sweltering"; "Scorching"]

let getWeatherForecast() = 
    [1..5] 
    |> List.map (fun index -> {
            Date =  DateTime.Now.AddDays(index)
            TemperatureC = Random.Shared.Next(-20, 55)
            Summary = Some Summaries.[Random.Shared.Next(Summaries.Length)]
        })    


let builder = WebApplication.CreateBuilder()
let app = builder.Build()

app.MapGet("/", Func<WeatherForecast list>(getWeatherForecast)) |> ignore
app.Run()