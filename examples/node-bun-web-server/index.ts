const server = Bun.serve({
  port: 5005,
  fetch(req) {
    return new Response(`Bun!`);
  },
});

console.log(`Hello from a Bun web server!`);
