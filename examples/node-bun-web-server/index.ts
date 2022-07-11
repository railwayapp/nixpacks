console.log("Hello from a Bun web server!");

export default {
    port: 3000,
    request() {
        return new Response("Hello from a Bun web server!")
    }
}