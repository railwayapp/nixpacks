import { serve } from "https://deno.land/std@0.114.0/http/server.ts";

const port = parseInt(Deno.env.get("PORT") ?? "8000");
serve(
  () => new Response("Choo Choo! Welcome to your Deno app\n"),
  { addr: `:${port}` },
).catch(err => console.log("Failed to serve", err));

console.log(`http://localhost:${port}/`);
