import { createMessageConnection, RequestType, SocketMessageReader, SocketMessageWriter, StreamMessageReader, StreamMessageWriter, } from "#vscode-jsonrpc/node";
import { fsCallbackNames, } from "../fs.js";
import { isSpawnOptions, resolveExePath, } from "../options.js";
import { combineTimingInfo, disabledServerTimingInfo, disabledTimingInfo, TimingCollector, } from "../timing.js";
/**
 * Client handles communication with the TypeScript API server
 * over STDIO (spawned process) or a Unix domain socket using JSON-RPC.
 */
export class Client {
    socket;
    process;
    connection;
    options;
    connected = false;
    timing;
    constructor(options) {
        this.options = options;
        if (isSpawnOptions(options) && options.collectTiming) {
            this.timing = new TimingCollector();
        }
    }
    async connect() {
        if (this.connected)
            return;
        if (isSpawnOptions(this.options)) {
            await this.connectViaSpawn(this.options);
        }
        else {
            await this.connectViaSocket(this.options);
        }
    }
    async connectViaSpawn(options) {
        const { spawn } = await import("node:child_process");
        return new Promise((resolve, reject) => {
            const args = [
                "--api",
                "--async",
                "--cwd",
                options.cwd ?? process.cwd(),
            ];
            if (options.collectTiming) {
                args.push("--timing");
            }
            // Enable virtual FS callbacks for each provided FS function
            const enabledCallbacks = [];
            if (options.fs) {
                for (const name of fsCallbackNames) {
                    if (options.fs[name]) {
                        enabledCallbacks.push(name);
                    }
                }
            }
            if (enabledCallbacks.length > 0) {
                args.push(`--callbacks=${enabledCallbacks.join(",")}`);
            }
            this.process = spawn(resolveExePath(options), args, {
                stdio: ["pipe", "pipe", "inherit"],
            });
            this.process.once("error", error => {
                reject(new Error(`Failed to start tsgo process: ${error.message}`));
            });
            this.process.once("spawn", () => {
                this.connected = true;
                resolve();
            });
            const reader = new StreamMessageReader(this.process.stdout);
            const writer = new StreamMessageWriter(this.process.stdin);
            this.connection = createMessageConnection(reader, writer);
            this.registerFSCallbacks(this.connection, options.fs);
            this.connection.listen();
        });
    }
    async connectViaSocket(options) {
        const { createConnection } = await import("node:net");
        return new Promise((resolve, reject) => {
            this.socket = createConnection(options.pipe, () => {
                const reader = new SocketMessageReader(this.socket);
                const writer = new SocketMessageWriter(this.socket);
                this.connection = createMessageConnection(reader, writer);
                this.connection.listen();
                this.connected = true;
                resolve();
            });
            this.socket.once("error", error => {
                reject(new Error(`Socket error: ${error.message}`));
            });
        });
    }
    registerFSCallbacks(connection, fs) {
        if (!fs)
            return;
        for (const name of fsCallbackNames) {
            const callback = fs[name];
            if (callback) {
                const requestType = new RequestType(name);
                connection.onRequest(requestType, (arg) => {
                    const result = callback(arg);
                    if (name === "readFile") {
                        // readFile has 3 returns: string (content), null (not found), undefined (fall back).
                        // JSON-RPC can't distinguish null from undefined, so wrap in object.
                        if (result === undefined)
                            return null;
                        return { content: result };
                    }
                    return result ?? null;
                });
            }
        }
    }
    async apiRequest(method, params) {
        if (!this.connected) {
            await this.connect();
        }
        if (!this.connection) {
            throw new Error("Connection not established");
        }
        const requestType = new RequestType(method);
        if (!this.timing) {
            return this.connection.sendRequest(requestType, params);
        }
        // Round-trip latency is measured here; byte counts approximate the wire
        // payload via the serialized JSON. Server-side processing time is not
        // carried on the response; it is retrieved separately (via a
        // getServerTiming request) and folded in by getTimingInfo().
        const bytesSent = params === undefined ? 0 : Buffer.byteLength(JSON.stringify(params), "utf-8");
        const start = performance.now();
        const result = await this.connection.sendRequest(requestType, params);
        const roundTripMs = performance.now() - start;
        this.timing.record({
            method,
            roundTripMs,
            bytesSent,
            bytesReceived: result === undefined || result === null
                ? 0
                : Buffer.byteLength(JSON.stringify(result), "utf-8"),
        });
        return result;
    }
    async apiRequestBinary(method, params) {
        const response = await this.apiRequest(method, params);
        if (!response)
            return undefined;
        const buffer = Buffer.from(response.data, "base64");
        return new Uint8Array(buffer.buffer, buffer.byteOffset, buffer.byteLength);
    }
    /**
     * Returns the timing collector that per-node materialization is reported
     * into, or undefined when timing collection is disabled. The returned
     * collector is the same one folded into {@link getTimingInfo}, so
     * materialization totals surface alongside request timings.
     */
    getTimingCollector() {
        return this.timing;
    }
    /**
     * Returns a combined timing snapshot: client-measured round-trip and byte
     * counts folded together with the server's own per-request processing time
     * (fetched via a getServerTiming request) and estimated transport overhead.
     */
    async getTimingInfo() {
        if (!this.timing) {
            return disabledTimingInfo();
        }
        const local = this.timing.getInfo();
        // No requests have been sent yet: nothing to fetch from the server.
        if (!this.connected || !this.connection) {
            return local;
        }
        return combineTimingInfo(local, await this.fetchServerTiming());
    }
    async resetTimingInfo() {
        if (!this.timing)
            return;
        this.timing.reset();
        if (this.connected && this.connection) {
            // Keep the server's collection in sync so combined totals stay meaningful.
            const requestType = new RequestType("resetServerTiming");
            await this.connection.sendRequest(requestType, undefined);
        }
    }
    async fetchServerTiming() {
        if (!this.connection) {
            return disabledServerTimingInfo();
        }
        // Fetch the server's own timing collection via a dedicated request. This
        // bypasses the client-side collector so the query does not pollute it.
        const requestType = new RequestType("getServerTiming");
        return this.connection.sendRequest(requestType, undefined);
    }
    async close() {
        if (this.connection) {
            this.connection.dispose();
            this.connection = undefined;
        }
        if (this.socket) {
            this.socket.destroy();
            this.socket = undefined;
        }
        if (this.process) {
            // Close stdin to unblock the server's read loop, allowing it to exit cleanly.
            // The server is blocked on stdin.Read(), so just sending SIGTERM would deadlock:
            // - Node won't exit while child is alive
            // - Child can't process SIGTERM while blocked on read
            // - Read won't error until stdin is closed
            this.process.stdin?.end();
            this.process = undefined;
        }
        this.connected = false;
    }
}
//# sourceMappingURL=client.js.map