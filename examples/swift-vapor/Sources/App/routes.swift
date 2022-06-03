import Vapor

func routes(_ app: Application) throws {
    app.get { req in
        return "Hello, from Swift!"
    }
}
