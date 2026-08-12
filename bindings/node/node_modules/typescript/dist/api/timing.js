/**
 * Client-side collection of per-request timing and transfer measurements.
 *
 * When enabled, each request records its round-trip latency and the number of
 * payload bytes sent and received, accumulated into running totals and a
 * fixed-size ring buffer of the most recent requests.
 *
 * The server measures its own per-request processing time independently. When a
 * timing snapshot is requested, the client fetches the server's collection via
 * a `getServerTiming` request and folds it into the returned {@link TimingInfo},
 * yielding per-request and total server processing time and an estimated
 * transport overhead (round-trip minus server processing time). Normal response
 * messages are left unchanged.
 */
/** Number of most-recent requests retained in the ring buffer. */
export const RECENT_REQUEST_CAPACITY = 5;
function emptyAccumulators() {
    return {
        requestCount: 0,
        roundTripMs: 0,
        bytesSent: 0,
        bytesReceived: 0,
        serverTimeMs: 0,
        transportOverheadMs: 0,
        nodesMaterialized: 0,
        sourceFilesFetched: 0,
        nodesFetched: 0,
    };
}
/** Returns a snapshot representing a disabled (never-collecting) timing state. */
export function disabledTimingInfo() {
    return {
        enabled: false,
        totals: emptyAccumulators(),
        recentRequests: [],
    };
}
/** Returns a snapshot representing disabled server-side timing collection. */
export function disabledServerTimingInfo() {
    return {
        enabled: false,
        totals: { requestCount: 0, totalProcessingTimeMs: 0 },
        recentRequests: [],
    };
}
/**
 * Folds a server-side timing snapshot into a client-side snapshot, producing a
 * combined {@link TimingInfo} with per-request and total server processing time
 * plus estimated transport overhead.
 *
 * Recent requests are paired newest-to-newest and only matched when the method
 * names agree, so that requests recorded by only one side (e.g. the meta
 * requests used to fetch timing) do not misalign the two ring buffers.
 */
export function combineTimingInfo(client, server) {
    if (!client.enabled) {
        return client;
    }
    const serverTimeMs = server.totals.totalProcessingTimeMs;
    const totals = {
        ...client.totals,
        serverTimeMs,
        transportOverheadMs: Math.max(0, client.totals.roundTripMs - serverTimeMs),
    };
    const recentRequests = client.recentRequests.map(r => ({ ...r }));
    const serverRecent = server.recentRequests;
    const pairs = Math.min(recentRequests.length, serverRecent.length);
    for (let i = 1; i <= pairs; i++) {
        const c = recentRequests[recentRequests.length - i];
        const s = serverRecent[serverRecent.length - i];
        if (c.method === s.method) {
            c.serverTimeMs = s.processingTimeMs;
            c.transportOverheadMs = Math.max(0, c.roundTripMs - s.processingTimeMs);
        }
    }
    return {
        enabled: true,
        totals,
        recentRequests,
    };
}
/**
 * Accumulates request timing samples into running totals and a fixed-size ring
 * buffer of the most recent requests.
 */
export class TimingCollector {
    totals = emptyAccumulators();
    // Ring buffer of the most recent requests. `ring` grows to at most
    // RECENT_REQUEST_CAPACITY; once full, `head` marks the oldest entry.
    ring = [];
    head = 0;
    /** Records a single request's measurements. */
    record(sample) {
        this.totals.requestCount++;
        this.totals.roundTripMs += sample.roundTripMs;
        this.totals.bytesSent += sample.bytesSent;
        this.totals.bytesReceived += sample.bytesReceived;
        const entry = {
            method: sample.method,
            roundTripMs: sample.roundTripMs,
            bytesSent: sample.bytesSent,
            bytesReceived: sample.bytesReceived,
            timestamp: Date.now(),
        };
        if (this.ring.length < RECENT_REQUEST_CAPACITY) {
            this.ring.push(entry);
        }
        else {
            this.ring[this.head] = entry;
            this.head = (this.head + 1) % RECENT_REQUEST_CAPACITY;
        }
    }
    /**
     * Records a single AST node materialization. Called on demand as the consumer
     * walks a binary source-file response's tree, so it is not tied to any one
     * request.
     */
    recordMaterialization() {
        this.totals.nodesMaterialized++;
    }
    /**
     * Records a fetched source file: increments the fetched-file counter and adds
     * the file's materializable node count to the fetched-node total, which serves
     * as the denominator for the share of fetched nodes that end up materialized.
     */
    recordSourceFileFetched(materializableNodeCount) {
        this.totals.sourceFilesFetched++;
        this.totals.nodesFetched += materializableNodeCount;
    }
    /** Returns a snapshot of the collected timing information. */
    getInfo() {
        const recentRequests = [];
        for (let i = 0; i < this.ring.length; i++) {
            recentRequests.push(this.ring[(this.head + i) % this.ring.length]);
        }
        return {
            enabled: true,
            totals: { ...this.totals },
            recentRequests,
        };
    }
    /** Clears all accumulated totals and recent-request history. */
    reset() {
        this.totals = emptyAccumulators();
        this.ring = [];
        this.head = 0;
    }
}
//# sourceMappingURL=timing.js.map