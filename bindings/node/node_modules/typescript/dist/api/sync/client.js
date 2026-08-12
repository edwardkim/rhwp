import { fsCallbackNames } from "../fs.js";
import { isSpawnOptions, resolveExePath, } from "../options.js";
import { SyncRpcChannel } from "../syncChannel.js";
import { combineTimingInfo, disabledTimingInfo, TimingCollector, } from "../timing.js";
export class Client {
    channel;
    encoder = new TextEncoder();
    timing;
    constructor(options) {
        if (!isSpawnOptions(options)) {
            throw new Error("Socket connections are not yet supported in the sync client");
        }
        const cwd = options.cwd ?? process.cwd();
        const args = [
            "--api",
            "--cwd",
            cwd,
        ];
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
        const collectTiming = options.collectTiming ?? false;
        if (collectTiming) {
            args.push("--timing");
            this.timing = new TimingCollector();
        }
        const channel = new SyncRpcChannel(resolveExePath(options), args, collectTiming);
        this.channel = channel;
        if (options.fs) {
            for (const name of enabledCallbacks) {
                const callback = options.fs[name];
                channel.registerCallback(name, (_, arg) => {
                    const result = callback(JSON.parse(arg));
                    if (name === "readFile") {
                        // readFile has 3 returns: string (content), null (not found), undefined (fall back).
                        // Wrap in object to preserve null vs undefined distinction.
                        if (result === undefined)
                            return "";
                        return JSON.stringify({ content: result });
                    }
                    return JSON.stringify(result) ?? "";
                });
            }
        }
    }
    apiRequest(method, params) {
        const encodedPayload = JSON.stringify(params);
        const start = performance.now();
        const result = this.channel.requestSync(method, encodedPayload);
        this.recordTiming(method, start);
        if (result.length) {
            return JSON.parse(result);
        }
        return undefined;
    }
    apiRequestBinary(method, params) {
        const start = performance.now();
        const result = this.channel.requestBinarySync(method, this.encoder.encode(JSON.stringify(params)));
        this.recordTiming(method, start);
        if (result.length === 0)
            return undefined;
        return result;
    }
    echo(payload) {
        return this.channel.requestSync("echo", payload);
    }
    echoBinary(payload) {
        return this.channel.requestBinarySync("echo", payload);
    }
    /**
     * Returns a combined timing snapshot: client-measured round-trip and byte
     * counts folded together with the server's own per-request processing time
     * (fetched via a getServerTiming request) and estimated transport overhead.
     */
    getTimingInfo() {
        if (!this.timing) {
            return disabledTimingInfo();
        }
        const local = this.timing.getInfo();
        // requestSync bypasses recordTiming, so this query does not pollute the
        // client-side collector.
        const result = this.channel.requestSync("getServerTiming", "");
        return combineTimingInfo(local, JSON.parse(result));
    }
    resetTimingInfo() {
        if (!this.timing)
            return;
        this.timing.reset();
        // Keep the server's collection in sync so combined totals stay meaningful.
        this.channel.requestSync("resetServerTiming", "");
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
    recordTiming(method, start) {
        if (!this.timing)
            return;
        this.timing.record({
            method,
            roundTripMs: performance.now() - start,
            bytesSent: this.channel.lastBytesSent,
            bytesReceived: this.channel.lastBytesReceived,
        });
    }
    close() {
        this.channel.close();
    }
}
//# sourceMappingURL=client.js.map