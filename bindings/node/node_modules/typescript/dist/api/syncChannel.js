/**
 * Pure JS replacement for @typescript/libsyncrpc.
 *
 * Spawns a child process and communicates with it synchronously over
 * stdin/stdout pipes using the same MessagePack-based tuple protocol:
 *   [MessageType (u8), method (bin), payload (bin)]
 *
 * Synchronous I/O is achieved by calling fs.readSync / fs.writeSync
 * directly on the pipe file descriptors obtained from the spawned
 * ChildProcess.
 */
import { spawn, } from "node:child_process";
import { closeSync, openSync, readSync, writeSync, } from "node:fs";
import { binHeaderSize, MSGPACK_BIN16, MSGPACK_BIN32, MSGPACK_BIN8, MSGPACK_FIXARRAY3, MSGPACK_UINT8, writeBinHeader, } from "./node/msgpack.js";
// ── MessageType constants ────────────────────────────────────────────
// Sent by channel (parent → child)
const MSG_REQUEST = 1;
const MSG_CALL_RESPONSE = 2;
const MSG_CALL_ERROR = 3;
// Sent by child (child → parent)
const MSG_RESPONSE = 4;
const MSG_ERROR = 5;
const MSG_CALL = 6;
// Pre-allocated buffer used by Atomics.wait for tiny sleeps when a
// non-blocking fd returns EAGAIN.
const sleepBuf = new Int32Array(new SharedArrayBuffer(4));
// Shared empty buffer – avoids allocating Buffer.alloc(0) on every
// zero-length bin field.
const EMPTY_BUF = Buffer.alloc(0);
// ── Global cleanup tracking ─────────────────────────────────────────
// Track all live child processes so they can be killed on process exit.
// This mimics the auto-cleanup behavior of the native libsyncrpc module,
// whose Rust/C++ destructors would kill children automatically.
const liveChildren = new Set();
process.on("exit", () => {
    for (const child of liveChildren) {
        try {
            child.kill();
        }
        catch {
            // swallow – process may already be dead
        }
    }
    liveChildren.clear();
});
/**
 * SyncRpcChannel – drop-in replacement for the native libsyncrpc class.
 *
 * API surface intentionally matches the original:
 *   - constructor(exe, args)
 *   - requestSync(method, payload): string
 *   - requestBinarySync(method, payload): Uint8Array
 *   - registerCallback(name, cb)
 *   - close()
 *
 * The protocol is unversioned; both sides (this JS channel and the Go
 * child process) must be built from the same tree.
 *
 * This class is **not** thread-safe. All calls must originate from a
 * single thread — do not share an instance across worker threads.
 */
export class SyncRpcChannel {
    child;
    readFd;
    writeFd;
    pipeFd;
    callbacks = new Map();
    methodBufCache = new Map();
    // When true, the payload byte lengths of each request/response are recorded
    // and exposed via the `last*` fields below. These count only the JSON/binary
    // payload bytes, not the MessagePack tuple framing (message type, method
    // name, and length headers).
    collectTiming;
    // Per-request payload byte measurements for the most recently completed
    // request. Only meaningful when `collectTiming` is true. Callers read these
    // immediately after a request returns (the channel is strictly serial).
    lastBytesSent = 0;
    lastBytesReceived = 0;
    _msgType = 0;
    _msgName = EMPTY_BUF;
    _msgPayload = EMPTY_BUF;
    headerBuf = Buffer.allocUnsafe(4);
    // Read-ahead buffer – reduces readSync syscalls by buffering data from the pipe.
    readBuf = Buffer.allocUnsafe(65536);
    readBufPos = 0;
    readBufLen = 0;
    // Write buffer – assembles entire tuples for a single writeSync.
    writeBuf = Buffer.allocUnsafe(65536);
    constructor(exe, args, collectTiming = false) {
        this.collectTiming = collectTiming;
        const isWindows = process.platform === "win32";
        if (isWindows) {
            // On Windows, libuv pipe handles don't expose POSIX fds, so
            // readSync/writeSync can't be used on stdio pipes. Instead,
            // we create a Windows named pipe path, pass it to the child
            // via --pipe, and open it with fs.openSync which returns a
            // real C-runtime fd backed by a proper HANDLE.
            const pipePath = `\\\\.\\pipe\\tsgo-sync-${process.pid}-${Date.now()}`;
            this.child = spawn(exe, [...args, "--pipe", pipePath], {
                stdio: ["ignore", "ignore", "inherit"],
            });
            // Retry openSync until the child creates the named pipe.
            let fd;
            for (let i = 0; i < 500; i++) {
                try {
                    fd = openSync(pipePath, "r+");
                    break;
                }
                catch {
                    if (this.child.exitCode !== null) {
                        throw new Error(`Child process exited with code ${this.child.exitCode} before pipe was ready`);
                    }
                    Atomics.wait(sleepBuf, 0, 0, 10);
                }
            }
            if (fd === undefined) {
                this.child.kill();
                throw new Error("SyncRpcChannel: timed out connecting to named pipe");
            }
            this.readFd = fd;
            this.writeFd = fd;
            this.pipeFd = fd;
        }
        else {
            // POSIX: use stdio pipe file descriptors directly.
            this.child = spawn(exe, args, {
                stdio: ["pipe", "pipe", "inherit"],
            });
            const stdout = this.child.stdout;
            const stdin = this.child.stdin;
            this.readFd = stdout._handle.fd;
            this.writeFd = stdin._handle.fd;
            if (typeof this.readFd !== "number" || this.readFd < 0 || typeof this.writeFd !== "number" || this.writeFd < 0) {
                stdout.destroy();
                stdin.destroy();
                this.child.kill();
                throw new Error("SyncRpcChannel: could not obtain pipe file descriptors.");
            }
            // Set the pipe handles to blocking mode. Under node --test's
            // process isolation, pipes are created in non-blocking mode
            // (for the IPC channel). This causes readSync/writeSync to get
            // EAGAIN, requiring costly 1ms sleeps per retry. Setting
            // blocking mode ensures readSync blocks properly until data
            // arrives, matching the behavior of the native libsyncrpc.
            stdout._handle.setBlocking?.(true);
            stdin._handle.setBlocking?.(true);
            // Prevent Node's event-loop from reading stdout or keeping the
            // process alive – we will use fs.readSync exclusively.
            stdout.pause();
            stdout.unref();
            stdin.unref();
        }
        // Track for auto-cleanup on process exit.
        liveChildren.add(this.child);
        this.child.unref();
    }
    // ── Public API ──────────────────────────────────────────────────
    /**
     * Send a request and synchronously wait for the response (string).
     * Handles Call (callback) messages from the child inline.
     */
    requestSync(method, payload) {
        this.ensureOpen();
        const result = this.requestBytesSync(method, payload);
        return result.toString("utf-8");
    }
    /**
     * Send a request and synchronously wait for the response (binary).
     * Handles Call (callback) messages from the child inline.
     */
    requestBinarySync(method, payload) {
        this.ensureOpen();
        return this.requestBytesSync(method, payload);
    }
    /** Register a string→string callback that the child may invoke. */
    registerCallback(name, callback) {
        this.callbacks.set(name, callback);
    }
    /** Kill the child process and release resources. */
    close() {
        try {
            liveChildren.delete(this.child);
            if (this.pipeFd !== undefined) {
                closeSync(this.pipeFd);
                this.pipeFd = undefined;
            }
            // Destroy the stdio streams so that their pipe handles are closed
            // and no longer prevent the event loop from draining.
            this.child.stdout?.destroy();
            this.child.stdin?.destroy();
            this.child.kill();
            this.readFd = -1;
            this.writeFd = -1;
        }
        catch {
            // swallow – process may already be dead
        }
    }
    // ── Core request loop ───────────────────────────────────────────
    ensureOpen() {
        if (this.readFd < 0) {
            throw new Error("SyncRpcChannel is closed");
        }
    }
    getMethodBuf(method) {
        let buf = this.methodBufCache.get(method);
        if (buf === undefined) {
            buf = Buffer.from(method, "utf-8");
            this.methodBufCache.set(method, buf);
        }
        return buf;
    }
    requestBytesSync(method, payload) {
        const methodBuf = this.getMethodBuf(method);
        if (this.collectTiming) {
            this.lastBytesSent = typeof payload === "string"
                ? Buffer.byteLength(payload, "utf-8")
                : payload.length;
            this.lastBytesReceived = 0;
        }
        this.writeTuple(MSG_REQUEST, methodBuf, payload);
        for (;;) {
            this.readTuple();
            switch (this._msgType) {
                case MSG_RESPONSE: {
                    // Compare raw bytes instead of decoding to string.
                    if (!methodBuf.equals(this._msgName)) {
                        throw new Error(`name mismatch for response: expected \`${method}\`, got \`${this._msgName.toString("utf-8")}\``);
                    }
                    if (this.collectTiming) {
                        this.lastBytesReceived = this._msgPayload.length;
                    }
                    return this._msgPayload;
                }
                case MSG_ERROR: {
                    if (methodBuf.equals(this._msgName)) {
                        throw new Error(this._msgPayload.toString("utf-8"));
                    }
                    throw new Error(`name mismatch for response: expected \`${method}\`, got \`${this._msgName.toString("utf-8")}\``);
                }
                case MSG_CALL: {
                    this.handleCall(this._msgName.toString("utf-8"), this._msgPayload);
                    break;
                }
                default:
                    throw new Error(`Invalid message type from child: ${this._msgType}`);
            }
        }
    }
    // ── Callback handling ───────────────────────────────────────────
    /**
     * Handle an incoming MSG_CALL from the child process.
     *
     * After sending the error response back to the child, this method
     * intentionally re-throws to abort the caller's request loop.
     * A failed callback is treated as unrecoverable to match the
     * behavior of the native libsyncrpc addon.
     */
    handleCall(name, payload) {
        const cb = this.callbacks.get(name);
        if (!cb) {
            const errMsg = `unknown callback: \`${name}\`. Please make sure to register it on the JavaScript side before invoking it.`;
            this.writeTuple(MSG_CALL_ERROR, Buffer.from(name, "utf-8"), Buffer.from(errMsg, "utf-8"));
            throw new Error(`no callback named \`${name}\` found`);
        }
        try {
            const result = cb(name, payload.toString("utf-8"));
            this.writeTuple(MSG_CALL_RESPONSE, Buffer.from(name, "utf-8"), Buffer.from(result, "utf-8"));
        }
        catch (e) {
            const errMsg = String(e instanceof Error ? e.message : e).trim();
            this.writeTuple(MSG_CALL_ERROR, Buffer.from(name, "utf-8"), Buffer.from(errMsg, "utf-8"));
            throw new Error(`Error calling callback \`${name}\`: ${errMsg}`);
        }
    }
    // ── MessagePack tuple write ─────────────────────────────────────
    /**
     * Write a complete [type, name, payload] tuple in as few writeSync
     * calls as possible.  For messages that fit in the pre-allocated
     * write buffer (64 KB), everything is assembled and sent in a single
     * syscall.  Larger messages use two syscalls: one for the header
     * portion and one for the payload data.
     */
    writeTuple(type, name, payload) {
        const nameLen = name.length;
        const payloadIsString = typeof payload === "string";
        const payloadLen = payloadIsString ? Buffer.byteLength(payload, "utf-8") : payload.length;
        const nameHdrSize = binHeaderSize(nameLen);
        const payloadHdrSize = binHeaderSize(payloadLen);
        const headerSize = 2 + nameHdrSize + nameLen + payloadHdrSize;
        const totalSize = headerSize + payloadLen;
        if (totalSize <= this.writeBuf.length) {
            // Small message: assemble into write buffer, one syscall
            let off = 0;
            this.writeBuf[off++] = MSGPACK_FIXARRAY3;
            this.writeBuf[off++] = type;
            off = writeBinHeader(this.writeBuf, off, nameLen);
            name.copy(this.writeBuf, off);
            off += nameLen;
            off = writeBinHeader(this.writeBuf, off, payloadLen);
            if (payloadLen > 0) {
                if (payloadIsString) {
                    // Encode string directly into write buffer — avoids
                    // Buffer.from(string, 'utf-8') allocation entirely.
                    this.writeBuf.write(payload, off, payloadLen, "utf-8");
                }
                else if (payload instanceof Buffer) {
                    payload.copy(this.writeBuf, off);
                }
                else {
                    this.writeBuf.set(payload, off);
                }
            }
            this.writeAllBuf(this.writeBuf, totalSize);
        }
        else {
            // Large message: header + name in one call, payload in another
            let off = 0;
            this.writeBuf[off++] = MSGPACK_FIXARRAY3;
            this.writeBuf[off++] = type;
            off = writeBinHeader(this.writeBuf, off, nameLen);
            name.copy(this.writeBuf, off);
            off += nameLen;
            off = writeBinHeader(this.writeBuf, off, payloadLen);
            this.writeAllBuf(this.writeBuf, off);
            if (payloadLen > 0) {
                if (payloadIsString) {
                    // Large string: must allocate (can't stream-encode
                    // across multiple writeSync calls).
                    this.writeAllBuf(Buffer.from(payload, "utf-8"));
                }
                else {
                    this.writeAllBuf(payload);
                }
            }
        }
    }
    // ── MessagePack tuple read ──────────────────────────────────────
    /**
     * Read a [type, name, payload] tuple into instance fields
     * (_msgType, _msgName, _msgPayload) to avoid allocating a
     * short-lived 3-element array on every call.
     */
    readTuple() {
        // Fixed 3-element array marker
        const marker = this.readByte();
        if (marker !== MSGPACK_FIXARRAY3) {
            throw new Error(`Expected fixed 3-element array (0x93), received: 0x${marker.toString(16)}`);
        }
        // Message type – positive fixint or uint8
        const tb = this.readByte();
        if (tb <= 0x7f) {
            this._msgType = tb;
        }
        else if (tb === MSGPACK_UINT8) {
            this._msgType = this.readByte();
        }
        else {
            throw new Error(`Expected positive fixint or uint8 marker, received: 0x${tb.toString(16)}`);
        }
        this._msgName = this.readBin();
        this._msgPayload = this.readBin();
    }
    /**
     * Read a MessagePack bin field.
     */
    readBin() {
        const marker = this.readByte();
        let size;
        switch (marker) {
            case MSGPACK_BIN8:
                size = this.readByte();
                break;
            case MSGPACK_BIN16:
                this.readExactInto(this.headerBuf, 2);
                size = (this.headerBuf[0] << 8) | this.headerBuf[1];
                break;
            case MSGPACK_BIN32:
                this.readExactInto(this.headerBuf, 4);
                size = this.headerBuf.readUInt32BE(0);
                break;
            default:
                throw new Error(`Expected binary data (0xc4-0xc6), received: 0x${marker.toString(16)}`);
        }
        if (size === 0)
            return EMPTY_BUF;
        return this.readExact(size);
    }
    // ── Low-level synchronous I/O ───────────────────────────────────
    /** Build an EOF error with the child's exit code/signal if available. */
    eofError() {
        const code = this.child.exitCode;
        const signal = this.child.signalCode;
        const detail = signal ? `killed by signal ${signal}` : code !== null ? `exited with code ${code}` : "unknown reason";
        return new Error(`Unexpected EOF while reading from child process (${detail})`);
    }
    /** Read a single byte from the buffered read-ahead. */
    readByte() {
        if (this.readBufPos >= this.readBufLen) {
            this.fillReadBuffer();
        }
        return this.readBuf[this.readBufPos++];
    }
    readExact(length) {
        // Use allocUnsafeSlow (not allocUnsafe) so the buffer has its own
        // backing ArrayBuffer at byteOffset 0.  This is critical because
        // callers such as RemoteSourceFile create DataView/Uint8Array over
        // buffer.buffer with absolute offsets.  Buffer.allocUnsafe returns
        // slices of a shared pool whose byteOffset is non-zero, corrupting
        // those downstream views.
        //
        // allocUnsafeSlow (vs alloc) skips zero-fill, which matters for
        // large transfers — e.g. checker.ts at ~57 MB saves ~5 ms of
        // unnecessary memset.  The buffer is immediately filled by
        // readExactInto so uninitialized memory is never exposed.
        const buf = Buffer.allocUnsafeSlow(length);
        this.readExactInto(buf, length);
        return buf;
    }
    /**
     * Fill the internal read-ahead buffer from the pipe fd.
     * Retries on EAGAIN for non-blocking mode compatibility.
     */
    fillReadBuffer() {
        this.readBufPos = 0;
        this.readBufLen = 0;
        for (;;) {
            try {
                const n = readSync(this.readFd, this.readBuf, 0, this.readBuf.length, null);
                if (n === 0) {
                    throw this.eofError();
                }
                this.readBufLen = n;
                return;
            }
            catch (e) {
                if (e instanceof Error && ("code" in e) && (e.code === "EAGAIN" || e.code === "EWOULDBLOCK")) {
                    Atomics.wait(sleepBuf, 0, 0, 1);
                    continue;
                }
                throw e;
            }
        }
    }
    /**
     * Synchronously read exactly `length` bytes into `buffer`.
     * Serves from the internal read-ahead buffer first; for large reads
     * that exceed the buffer size, reads directly from the fd to avoid
     * an extra copy.
     */
    readExactInto(buffer, length) {
        let pos = 0;
        while (pos < length) {
            const avail = this.readBufLen - this.readBufPos;
            if (avail > 0) {
                // Serve from read-ahead buffer
                const toCopy = Math.min(avail, length - pos);
                this.readBuf.copy(buffer, pos, this.readBufPos, this.readBufPos + toCopy);
                this.readBufPos += toCopy;
                pos += toCopy;
            }
            else if (length - pos >= this.readBuf.length) {
                // Remaining data is larger than read buffer; read directly
                // into the target to avoid unnecessary copying.
                try {
                    const n = readSync(this.readFd, buffer, pos, length - pos, null);
                    if (n === 0) {
                        throw this.eofError();
                    }
                    pos += n;
                }
                catch (e) {
                    if (e instanceof Error && ("code" in e) && (e.code === "EAGAIN" || e.code === "EWOULDBLOCK")) {
                        Atomics.wait(sleepBuf, 0, 0, 1);
                        continue;
                    }
                    throw e;
                }
            }
            else {
                // Refill the read-ahead buffer
                this.fillReadBuffer();
            }
        }
    }
    /**
     * Synchronously write all bytes from `data` (up to `length`).
     * Retries on EAGAIN.
     */
    writeAllBuf(data, length) {
        const total = length ?? data.length;
        let pos = 0;
        while (pos < total) {
            try {
                const n = writeSync(this.writeFd, data, pos, total - pos);
                pos += n;
            }
            catch (e) {
                if (e instanceof Error && ("code" in e) && (e.code === "EAGAIN" || e.code === "EWOULDBLOCK")) {
                    Atomics.wait(sleepBuf, 0, 0, 1);
                    continue;
                }
                throw e;
            }
        }
    }
}
//# sourceMappingURL=syncChannel.js.map