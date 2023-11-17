export default class Logger {
    /** @type string */
    #tag;

    /**
    * @param {string} tag
    */
    constructor(tag) {
        this.#tag = tag
    }

    #log(color, messageType, message, fn = console.log) {
        fn(`\x1b[${color}m[${this.#tag}:${messageType}]\x1b[0m ${message}`)
    }

    info(message) {
        this.#log(34, 'info', message)
    }

    warn(message) {
        this.#log(35, 'warn', message, console.warn)
    }

    error(message) {
        this.#log(31, 'error', message, console.error)
    }
}
