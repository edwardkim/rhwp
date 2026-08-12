import { statSync, accessSync, constants } from 'fs';
import { homedir } from 'os';
import { join, resolve, dirname, delimiter } from 'path';
import { fileURLToPath } from 'url';
import { spawn } from 'child_process';

var __defProp = Object.defineProperty;
var __getOwnPropNames = Object.getOwnPropertyNames;
var __esm = (fn, res) => function __init() {
  return fn && (res = (0, fn[__getOwnPropNames(fn)[0]])(fn = 0)), res;
};
var __export = (target, all) => {
  for (var name in all)
    __defProp(target, name, { get: all[name], enumerable: true });
};

// src/errors.ts
function quoteArgument(arg) {
  if (arg && !/[\s"'\\]/.test(arg)) return arg;
  const escaped = arg.replace(/\\/g, "\\\\").replace(/"/g, '\\"');
  return `"${escaped}"`;
}
function raiseForExit(exitCode, options = {}) {
  if (exitCode === EXIT_OK) return;
  const { throwOnVerdict = false, ...context } = options;
  const ctx = { ...context, exitCode };
  if (exitCode === EXIT_USAGE) {
    throw new UsageError("\uD638\uCD9C \uC778\uC790\uAC00 \uC62C\uBC14\uB974\uC9C0 \uC54A\uC2B5\uB2C8\uB2E4", ctx);
  }
  if (exitCode === EXIT_RUNTIME) {
    throw new RhwpRuntimeError("\uBB38\uC11C \uCC98\uB9AC\uC5D0 \uC2E4\uD328\uD588\uC2B5\uB2C8\uB2E4", ctx);
  }
  if (exitCode === EXIT_VERIFY || exitCode === EXIT_VERIFY_PAGES) {
    if (throwOnVerdict) {
      const label = exitCode === EXIT_VERIFY_PAGES ? "\uD398\uC774\uC9C0 \uC218\uAC00 \uC77C\uCE58\uD558\uC9C0 \uC54A\uC2B5\uB2C8\uB2E4" : "\uAC80\uC99D \uB2E8\uC5B8\uC774 \uC2E4\uD328\uD588\uC2B5\uB2C8\uB2E4";
      throw new VerdictFailed(label, ctx);
    }
    return;
  }
  throw new RhwpRuntimeError(
    `\uC54C \uC218 \uC5C6\uB294 \uC885\uB8CC \uCF54\uB4DC\uC785\uB2C8\uB2E4 (${exitCode}) \u2014 rhwp \uC640 \uBC14\uC778\uB529 \uBC84\uC804\uC774 \uC5B4\uAE0B\uB0AC\uC744 \uC218 \uC788\uC2B5\uB2C8\uB2E4`,
    ctx
  );
}
var EXIT_OK, EXIT_RUNTIME, EXIT_USAGE, EXIT_VERIFY, EXIT_VERIFY_PAGES, RhwpError, BinaryNotFoundError, UsageError, RhwpRuntimeError, VerdictFailed, ProtocolError, EnvelopeKeyError, RhwpTimeoutError;
var init_errors = __esm({
  "src/errors.ts"() {
    EXIT_OK = 0;
    EXIT_RUNTIME = 1;
    EXIT_USAGE = 2;
    EXIT_VERIFY = 3;
    EXIT_VERIFY_PAGES = 4;
    RhwpError = class extends Error {
      // exactOptionalPropertyTypes 가 켜져 있어 `| undefined` 를 명시한다 —
      // "필드가 없음"과 "undefined 를 담음"을 타입이 구분하기 때문이다.
      /** 실행한 명령줄. */
      argv;
      /** 종료 코드. */
      exitCode;
      /** 도구 진단 원문. */
      stderr;
      /** 판정 근거가 담긴 봉투. */
      envelope;
      constructor(message, context = {}) {
        super(message);
        this.name = new.target.name;
        this.argv = context.argv ? [...context.argv] : void 0;
        this.exitCode = context.exitCode;
        this.stderr = context.stderr ?? "";
        this.envelope = context.envelope ? { ...context.envelope } : void 0;
        if (context.cause !== void 0) {
          this.cause = context.cause;
        }
        Object.setPrototypeOf(this, new.target.prototype);
        if (Error.captureStackTrace) {
          Error.captureStackTrace(this, new.target);
        }
      }
      /**
       * 재현 가능한 명령 문자열. 버그 리포트에 그대로 붙일 수 있게 공백을 감싼다.
       */
      get command() {
        if (!this.argv?.length) return "";
        return this.argv.map(quoteArgument).join(" ");
      }
      /** 가장 구체적인 진단 (stderr 마지막 줄). */
      get lastDiagnostic() {
        const lines = this.stderr.split("\n").filter((l) => l.trim());
        return lines.at(-1)?.trim() ?? "";
      }
      toString() {
        const parts = [`${this.name}: ${this.message}`];
        if (this.exitCode !== void 0) parts.push(`(exit ${this.exitCode})`);
        const detail = this.lastDiagnostic;
        if (detail) parts.push(`\u2014 ${detail}`);
        return parts.join(" ");
      }
    };
    BinaryNotFoundError = class extends RhwpError {
    };
    UsageError = class extends RhwpError {
      /**
       * stderr 의 `힌트:` 줄에서 did-you-mean 교정 제안을 추출한다.
       *
       * @returns 제안 문구. 없으면 undefined.
       */
      get suggestion() {
        const lines = this.stderr.split("\n");
        for (let i = lines.length - 1; i >= 0; i -= 1) {
          const trimmed = lines[i]?.trim();
          if (trimmed?.startsWith("\uD78C\uD2B8:")) {
            return trimmed.slice("\uD78C\uD2B8:".length).trim();
          }
        }
        return void 0;
      }
      /**
       * 서버가 실어 보낸 교정 호출(`nextCall`). 기계가 그대로 따라할 수 있는 형태다.
       */
      get nextCall() {
        const next = this.envelope?.nextCall;
        if (next && typeof next === "object" && "name" in next) {
          return next;
        }
        return void 0;
      }
    };
    RhwpRuntimeError = class extends RhwpError {
    };
    VerdictFailed = class extends RhwpError {
      /** exit 4 (페이지 수 불일치)인지. */
      get isPageCountMismatch() {
        return this.exitCode === EXIT_VERIFY_PAGES;
      }
    };
    ProtocolError = class extends RhwpError {
    };
    EnvelopeKeyError = class extends RhwpError {
    };
    RhwpTimeoutError = class extends RhwpError {
    };
  }
});

// src/naming.ts
function toSnake(name) {
  if (!name) return name;
  return name.replace(ACRONYM_BOUNDARY, "$1_$2").replace(WORD_BOUNDARY, "$1_$2").toLowerCase();
}
function toCamel(name) {
  if (!name || !name.includes("_")) return name;
  const [head, ...rest] = name.split("_");
  return (head ?? "") + rest.filter((part) => part.length > 0).map((part) => part.charAt(0).toUpperCase() + part.slice(1)).join("");
}
var ACRONYM_BOUNDARY, WORD_BOUNDARY;
var init_naming = __esm({
  "src/naming.ts"() {
    ACRONYM_BOUNDARY = /([A-Z])([A-Z][a-z])/g;
    WORD_BOUNDARY = /([a-z0-9])([A-Z])/g;
  }
});

// src/envelope.ts
var VerifyReport, Envelope;
var init_envelope = __esm({
  "src/envelope.ts"() {
    init_errors();
    init_naming();
    VerifyReport = class {
      constructor(report) {
        this.report = report;
      }
      report;
      /** 저장본이 메모리 IR 과 동일한가. 이 값이 판정의 전부다. */
      get identical() {
        return this.report.identical === true;
      }
      /** 차이 개수. 재파싱 자체가 실패했으면 null. */
      get diffCount() {
        const value = this.report.diffCount;
        return typeof value === "number" ? value : null;
      }
      /** 저장본을 다시 읽지 못했을 때의 사유. 정상이면 undefined. */
      get reparseError() {
        return this.report.reparseError;
      }
      /** 원문. */
      get raw() {
        return { ...this.report };
      }
      toString() {
        if (this.reparseError) return `\uC7AC\uD30C\uC2F1 \uC2E4\uD328: ${this.reparseError}`;
        return this.identical ? "\uB3D9\uC77C" : `\uCC28\uC774 ${this.diffCount ?? "?"}\uAC74`;
      }
    };
    Envelope = class _Envelope {
      constructor(source) {
        this.source = source;
        if (source === null || typeof source !== "object" || Array.isArray(source)) {
          throw new TypeError(
            `\uBD09\uD22C\uB294 \uAC1D\uCCB4\uC5EC\uC57C \uD569\uB2C8\uB2E4 (\uBC1B\uC74C: ${Array.isArray(source) ? "array" : typeof source})`
          );
        }
        const index = /* @__PURE__ */ new Map();
        for (const key of Object.keys(source)) {
          const snake = toSnake(key);
          if (!index.has(snake)) index.set(snake, key);
        }
        this.snakeIndex = index;
      }
      source;
      /** snake_case 로 물었을 때 원문 키를 찾기 위한 색인. */
      snakeIndex;
      /**
       * 원문 봉투 (**사본**).
       *
       * 생성 타입을 주면 여기서 정적으로 좁혀진다 — `env.raw.pageCount` 가 `number`.
       */
      get raw() {
        return { ...this.source };
      }
      /** 봉투에 있는 키 목록. */
      keys() {
        return Object.keys(this.source);
      }
      /** 필드가 있는지. */
      has(key) {
        return key in this.source || this.snakeIndex.has(key) || toCamel(key) in this.source;
      }
      /**
       * 필드 하나를 꺼낸다. 원문 키·snake_case·camelCase 를 모두 받는다.
       *
       * @throws {Error} 없는 필드일 때. **조용한 undefined 를 돌려주지 않는다** —
       *   오타가 "값 없음"으로 둔갑하면 그게 가장 찾기 어려운 버그가 된다. 메시지에
       *   있는 필드를 함께 담아 즉시 고칠 수 있게 한다.
       */
      get(key) {
        const record = this.source;
        if (key in record) return record[key];
        const original = this.snakeIndex.get(key);
        if (original !== void 0) return record[original];
        const camel = toCamel(key);
        if (camel in record) return record[camel];
        throw new EnvelopeKeyError(
          `\uBD09\uD22C\uC5D0 '${key}' \uD544\uB4DC\uAC00 \uC5C6\uC2B5\uB2C8\uB2E4. \uC788\uB294 \uD544\uB4DC: ${this.keys().sort().join(", ")}`
        );
      }
      /**
       * 없으면 기본값을 돌려준다 — "없어도 되는" 선택 필드를 읽을 때만 쓴다.
       *
       * 필수 필드에 이걸 쓰면 {@link get} 의 보호를 스스로 버리는 것이다.
       */
      getOr(key, fallback) {
        return this.has(key) ? this.get(key) : fallback;
      }
      /**
       * `"verify.identical"` 처럼 점 경로로 꺼낸다. 없으면 `undefined`.
       */
      getPath(dotted) {
        let cursor = this.source;
        for (const part of dotted.split(".")) {
          if (cursor === null || typeof cursor !== "object") return void 0;
          const record = cursor;
          if (part in record) {
            cursor = record[part];
            continue;
          }
          const camel = toCamel(part);
          if (camel in record) {
            cursor = record[camel];
            continue;
          }
          return void 0;
        }
        return cursor;
      }
      /** 봉투 스키마 버전. */
      get schemaVersion() {
        const value = this.source.schemaVersion;
        return typeof value === "string" ? value : void 0;
      }
      /**
       * `--verify` 보고가 있으면 {@link VerifyReport}, **미요청이면 `null`**.
       *
       * `null` 은 "검증 안 함"이지 "검증 실패"가 아니다.
       */
      get verify() {
        const value = this.source.verify;
        if (value === null || value === void 0) return null;
        if (typeof value !== "object") return null;
        return new VerifyReport(value);
      }
      /**
       * 편집이 바꾼 쪽 목록(0 기준). **확정 불가·무산출이면 `null`.**
       *
       * `null`(모른다)과 `[]`(바뀐 쪽이 없다)는 다른 결론이다. 둘을 falsy 로
       * 뭉뚱그리면 "확인할 게 없다"고 잘못 판단한다.
       */
      get changedPages() {
        const value = this.source.changedPages;
        if (!Array.isArray(value)) return null;
        return value.filter((n) => typeof n === "number");
      }
      /** 하위 객체를 봉투로 감싸 돌려준다. */
      child(key) {
        const value = this.has(key) ? this.get(key) : void 0;
        if (value === null || typeof value !== "object" || Array.isArray(value)) return null;
        return new _Envelope(value);
      }
      /**
       * 배열 필드를 봉투 배열로.
       *
       * 배열 항목이 다시 배열인 경우(`[[1,2]]`)를 걸러낸다 — `Envelope` 생성자가
       * 배열을 거부하므로, 안 거르면 조회 한 번이 `TypeError` 로 터진다.
       * 객체가 아닌 항목은 봉투가 될 수 없으니 조용히 제외하는 것이 맞다.
       */
      children(key) {
        const value = this.has(key) ? this.get(key) : void 0;
        if (!Array.isArray(value)) return [];
        return value.filter((item) => item !== null && typeof item === "object" && !Array.isArray(item)).map((item) => new _Envelope(item));
      }
      /** JSON 직렬화 시 원문을 그대로 내보낸다. */
      toJSON() {
        return this.raw;
      }
      toString() {
        const keys = this.keys().sort().slice(0, 6).join(", ");
        const more = this.keys().length > 6 ? "\u2026" : "";
        return `Envelope(${keys}${more})`;
      }
    };
  }
});
function binaryName() {
  return process.platform === "win32" ? "rhwp.exe" : "rhwp";
}
function bundledDir() {
  const here = typeof __dirname === "string" ? __dirname : dirname(fileURLToPath(import.meta.url));
  return join(here, "_bin");
}
function expandTilde(raw) {
  if (raw === "~") return homedir();
  if (raw.startsWith("~/") || process.platform === "win32" && raw.startsWith("~\\")) {
    return join(homedir(), raw.slice(2));
  }
  return raw;
}
function isExecutableFile(path) {
  let stat;
  try {
    stat = statSync(path);
  } catch {
    return false;
  }
  if (!stat.isFile()) return false;
  if (process.platform === "win32") {
    return /\.(exe|bat|cmd)$/i.test(path);
  }
  try {
    accessSync(path, constants.X_OK);
    return true;
  } catch {
    return false;
  }
}
function fromEnv() {
  const raw = (process.env[ENV_VAR] ?? "").trim();
  if (!raw) return void 0;
  let candidate = resolve(expandTilde(raw));
  try {
    if (statSync(candidate).isDirectory()) {
      candidate = join(candidate, binaryName());
    }
  } catch {
  }
  if (isExecutableFile(candidate)) return candidate;
  throw new BinaryNotFoundError(
    `${ENV_VAR} \uAC00 \uAC00\uB9AC\uD0A4\uB294 \uC2E4\uD589 \uD30C\uC77C\uC744 \uC4F8 \uC218 \uC5C6\uC2B5\uB2C8\uB2E4: ${raw}
  (\uC874\uC7AC\uD558\uC9C0 \uC54A\uAC70\uB098, \uD30C\uC77C\uC774 \uC544\uB2C8\uAC70\uB098, \uC2E4\uD589 \uAD8C\uD55C\uC774 \uC5C6\uC2B5\uB2C8\uB2E4)`
  );
}
function fromBundle() {
  const candidate = join(bundledDir(), binaryName());
  return isExecutableFile(candidate) ? candidate : void 0;
}
function fromPath() {
  const name = binaryName();
  const entries = (process.env.PATH ?? "").split(delimiter).filter(Boolean);
  for (const entry of entries) {
    const candidate = join(entry, name);
    if (isExecutableFile(candidate)) return candidate;
  }
  return void 0;
}
function findBinary(options = {}) {
  if (cached !== void 0 && !options.refresh) return cached;
  const tried = [];
  const fromEnvironment = fromEnv();
  if (fromEnvironment) {
    cached = fromEnvironment;
    return cached;
  }
  tried.push(`${ENV_VAR} (\uBBF8\uC124\uC815)`);
  const bundled = fromBundle();
  if (bundled) {
    cached = bundled;
    return cached;
  }
  tried.push(`\uD328\uD0A4\uC9C0 \uB3D9\uBD09 (${join(bundledDir(), binaryName())})`);
  const onPath = fromPath();
  if (onPath) {
    cached = onPath;
    return cached;
  }
  tried.push(`PATH (${binaryName()} \uC5C6\uC74C)`);
  throw new BinaryNotFoundError(
    "rhwp \uC2E4\uD589 \uD30C\uC77C\uC744 \uCC3E\uC9C0 \uBABB\uD588\uC2B5\uB2C8\uB2E4. \uB2E4\uC74C \uC21C\uC11C\uB85C \uD0D0\uC0C9\uD588\uC2B5\uB2C8\uB2E4:\n" + tried.map((t, i) => `  ${i + 1}. ${t}`).join("\n") + `

\uD574\uACB0: rhwp \uB97C \uC124\uCE58\uD574 PATH \uC5D0 \uB450\uAC70\uB098, ${ENV_VAR} \uB85C \uACBD\uB85C\uB97C \uC9C0\uC815\uD558\uC138\uC694.`
  );
}
var ENV_VAR, cached;
var init_binary = __esm({
  "src/binary.ts"() {
    init_errors();
    ENV_VAR = "RHWP_BIN";
  }
});
function stringify(value) {
  if (typeof value === "boolean") {
    throw new TypeError("\uBD88\uB9AC\uC5B8\uC740 \uC778\uC790 \uAC12\uC774 \uB420 \uC218 \uC5C6\uC2B5\uB2C8\uB2E4 \u2014 \uD50C\uB798\uADF8\uB85C \uD45C\uD604\uD558\uC138\uC694");
  }
  return String(value);
}
function spawnCollected(argv, options) {
  const timeoutMs = options.timeoutMs === null ? null : options.timeoutMs ?? DEFAULT_TIMEOUT_MS;
  return new Promise((resolve2, reject) => {
    let child;
    try {
      child = spawn(argv[0], argv.slice(1), {
        cwd: options.cwd,
        // 실행 파일 경로는 우리가 탐색한 것이므로 셸을 태우지 않는다 —
        // 셸을 거치면 윈도우 인용 규칙 때문에 한글 경로가 깨진다.
        shell: false,
        windowsHide: true
      });
    } catch (cause) {
      reject(new RhwpError(`rhwp \uC2E4\uD589\uC5D0 \uC2E4\uD328\uD588\uC2B5\uB2C8\uB2E4: ${String(cause)}`, { argv, cause }));
      return;
    }
    const stdoutChunks = [];
    const stderrChunks = [];
    let settled = false;
    let timer;
    const finish = (fn) => {
      if (settled) return;
      settled = true;
      if (timer) clearTimeout(timer);
      fn();
    };
    child.stdout.on("data", (chunk) => stdoutChunks.push(chunk));
    child.stderr.on("data", (chunk) => stderrChunks.push(chunk));
    child.on("error", (cause) => {
      finish(
        () => reject(new RhwpError(`rhwp \uC2E4\uD589\uC5D0 \uC2E4\uD328\uD588\uC2B5\uB2C8\uB2E4: ${cause.message}`, { argv, cause }))
      );
    });
    child.on("close", (code) => {
      finish(
        () => resolve2({
          argv,
          exitCode: code ?? 1,
          // 봉투는 UTF-8 이 계약이다. 잘못된 바이트가 섞여도 죽지 않고 치환하되,
          // 그 경우 JSON 파싱이 실패해 ProtocolError 로 드러난다.
          stdout: Buffer.concat(stdoutChunks).toString("utf8"),
          stderr: Buffer.concat(stderrChunks).toString("utf8")
        })
      );
    });
    if (timeoutMs !== null) {
      timer = setTimeout(() => {
        child.kill("SIGKILL");
        finish(
          () => reject(
            new RhwpTimeoutError(`\uC81C\uD55C \uC2DC\uAC04 ${timeoutMs}ms \uB97C \uCD08\uACFC\uD588\uC2B5\uB2C8\uB2E4`, {
              argv,
              stderr: Buffer.concat(stderrChunks).toString("utf8")
            })
          )
        );
      }, timeoutMs);
      timer.unref?.();
    }
    if (options.stdin !== void 0) {
      child.stdin.on("error", () => {
      });
      child.stdin.end(options.stdin, "utf8");
    } else {
      child.stdin.end();
    }
  });
}
async function runRaw(args, options = {}) {
  const binary = findBinary();
  const argv = [binary, ...args.map(stringify)];
  const result = await spawnCollected(argv, options);
  if (options.check !== false) {
    raiseForExit(result.exitCode, {
      argv: result.argv,
      stderr: result.stderr,
      envelope: options.envelopeHint,
      throwOnVerdict: options.throwOnVerdict ?? false
    });
  }
  return result;
}
async function runJson(args, options = {}) {
  const result = await runRaw(args, { ...options, check: false });
  let envelope;
  const text = result.stdout.trim();
  if (text) {
    let parsed;
    try {
      parsed = JSON.parse(text);
    } catch (cause) {
      throw new ProtocolError(`stdout \uC774 \uC21C\uC218 JSON \uC774 \uC544\uB2D9\uB2C8\uB2E4: ${String(cause)}`, {
        argv: result.argv,
        exitCode: result.exitCode,
        stderr: result.stderr,
        cause
      });
    }
    if (parsed === null || typeof parsed !== "object" || Array.isArray(parsed)) {
      throw new ProtocolError(
        `\uBD09\uD22C\uB294 JSON \uAC1D\uCCB4\uC5EC\uC57C \uD569\uB2C8\uB2E4 (\uBC1B\uC74C: ${Array.isArray(parsed) ? "array" : typeof parsed})`,
        { argv: result.argv, exitCode: result.exitCode, stderr: result.stderr }
      );
    }
    envelope = parsed;
  }
  raiseForExit(result.exitCode, {
    argv: result.argv,
    stderr: result.stderr,
    envelope,
    throwOnVerdict: options.throwOnVerdict ?? false
  });
  if (envelope === void 0) {
    throw new ProtocolError("\uC131\uACF5\uD588\uB294\uB370 stdout \uC774 \uBE44\uC5B4 \uC788\uC2B5\uB2C8\uB2E4 \u2014 --json \uBD09\uD22C \uACC4\uC57D \uC704\uBC18\uC785\uB2C8\uB2E4", {
      argv: result.argv,
      exitCode: result.exitCode,
      stderr: result.stderr
    });
  }
  return envelope;
}
function parseLine(line, lineNo, argv, exitCode, stderr) {
  const trimmed = line.trim();
  if (!trimmed) return void 0;
  let parsed;
  try {
    parsed = JSON.parse(trimmed);
  } catch (cause) {
    throw new ProtocolError(`NDJSON ${lineNo}\uBC88\uC9F8 \uC904\uC774 JSON \uC774 \uC544\uB2D9\uB2C8\uB2E4: ${String(cause)}`, {
      argv,
      exitCode,
      stderr,
      cause
    });
  }
  if (parsed === null || typeof parsed !== "object" || Array.isArray(parsed)) {
    throw new ProtocolError(`NDJSON ${lineNo}\uBC88\uC9F8 \uC904\uC774 \uAC1D\uCCB4\uAC00 \uC544\uB2D9\uB2C8\uB2E4`, {
      argv,
      exitCode,
      stderr
    });
  }
  return parsed;
}
async function runNdjson(args, options = {}) {
  const result = await runRaw(args, { ...options, check: false });
  const records = [];
  const lines = result.stdout.split("\n");
  for (let i = 0; i < lines.length; i += 1) {
    const record = parseLine(
      lines[i],
      i + 1,
      result.argv,
      result.exitCode,
      result.stderr
    );
    if (record) records.push(record);
  }
  if (result.exitCode === 2) {
    raiseForExit(2, { argv: result.argv, stderr: result.stderr });
  }
  return records;
}
var DEFAULT_TIMEOUT_MS;
var init_process = __esm({
  "src/process.ts"() {
    init_binary();
    init_errors();
    DEFAULT_TIMEOUT_MS = 3e5;
  }
});

// src/document-analysis.ts
function toRunOptions(options) {
  return {
    timeoutMs: options.timeoutMs,
    cwd: options.cwd,
    throwOnVerdict: options.throwOnVerdict
  };
}
function flag(args, name, value) {
  if (value !== void 0) args.push(name, value);
}
function toggle(args, name, enabled) {
  if (enabled) args.push(name);
}
function editFlags(args, options) {
  flag(args, "-o", options.out);
  toggle(args, "--dry-run", options.dryRun);
  toggle(args, "--verify", options.verify);
}
async function call(args, options = {}) {
  return new Envelope(await runJson(args, toRunOptions(options)));
}
async function exportProvenanceMap(options = {}) {
  return call(["export-provenance-map", "--json"], options);
}
async function tableToCsv(path, options = {}) {
  const args = ["table-to-csv", path];
  flag(args, "--table", options.table);
  flag(args, "-o", options.out);
  toggle(args, "--bom", options.bom);
  args.push("--json");
  return call(args, options);
}
async function csvToTable(path, options) {
  const args = [
    "csv-to-table",
    path,
    "--csv",
    options.csv,
    "--table",
    options.table
  ];
  editFlags(args, options);
  args.push("--json");
  return call(args, options);
}
async function extractData(path, options = {}) {
  const args = ["extract-data", path];
  flag(args, "--kind", options.kind);
  flag(args, "--limit", options.limit);
  args.push("--json");
  return call(args, options);
}
async function inspect(target, path, options = {}) {
  const args = ["inspect", target, path];
  switch (target) {
    case "hidden-text": {
      const hidden = options;
      flag(args, "--threshold-pt", hidden.thresholdPt);
      toggle(args, "--include-offpage", hidden.includeOffpage);
      break;
    }
    case "injection": {
      const injection = options;
      flag(args, "--min-confidence", injection.minConfidence);
      toggle(args, "--include-fields", injection.includeFields);
      break;
    }
    case "unicode": {
      const unicode = options;
      flag(args, "--kind", unicode.kind);
      break;
    }
  }
  args.push("--json");
  return call(args, options);
}
var init_document_analysis = __esm({
  "src/document-analysis.ts"() {
    init_envelope();
    init_process();
  }
});

// src/commands.ts
var commands_exports = {};
__export(commands_exports, {
  batch: () => batch,
  buildFromIngest: () => buildFromIngest,
  capabilities: () => capabilities,
  convert: () => convert,
  csvToTable: () => csvToTable,
  digest: () => digest,
  explain: () => explain,
  exportAgentManifest: () => exportAgentManifest,
  exportCapabilitiesSchema: () => exportCapabilitiesSchema,
  exportDoclang: () => exportDoclang,
  exportHml: () => exportHml,
  exportHwpx: () => exportHwpx,
  exportIrSchema: () => exportIrSchema,
  exportMarkdown: () => exportMarkdown,
  exportOntology: () => exportOntology,
  exportPdf: () => exportPdf,
  exportPlanSchema: () => exportPlanSchema,
  exportProvenanceMap: () => exportProvenanceMap,
  exportStructure: () => exportStructure,
  exportSvg: () => exportSvg,
  exportTables: () => exportTables,
  exportText: () => exportText,
  extractData: () => extractData,
  extractPages: () => extractPages,
  fields: () => fields,
  fillFields: () => fillFields,
  info: () => info,
  inspect: () => inspect,
  irDiff: () => irDiff,
  renderDiff: () => renderDiff,
  replaceText: () => replaceText,
  scan: () => scan,
  search: () => search,
  setCell: () => setCell,
  tableToCsv: () => tableToCsv,
  thumbnail: () => thumbnail,
  verify: () => verify
});
function toRunOptions2(options) {
  return {
    timeoutMs: options.timeoutMs,
    cwd: options.cwd,
    throwOnVerdict: options.throwOnVerdict
  };
}
function flag2(args, name, value) {
  if (value !== void 0) args.push(name, value);
}
function toggle2(args, name, enabled) {
  if (enabled) args.push(name);
}
function repeat(args, name, values) {
  if (values === void 0) return;
  for (const value of typeof values === "string" ? [values] : values) {
    args.push(name, value);
  }
}
function editFlags2(args, options) {
  flag2(args, "-o", options.out);
  toggle2(args, "--dry-run", options.dryRun);
  toggle2(args, "--verify", options.verify);
}
async function call2(args, options = {}) {
  return new Envelope(await runJson(args, toRunOptions2(options)));
}
async function info(path, options = {}) {
  return call2(["info", path, "--json"], options);
}
async function exportText(path, options = {}) {
  const args = ["export-text", path];
  flag2(args, "-p", options.page);
  args.push("--json");
  return call2(args, options);
}
async function exportStructure(path, options = {}) {
  const args = ["export-structure", path];
  flag2(args, "--mode", options.mode);
  args.push("--json");
  return call2(args, options);
}
async function exportTables(path, options = {}) {
  return call2(["export-tables", path, "--json"], options);
}
async function fields(path, options = {}) {
  return call2(["fields", path, "--json"], options);
}
async function search(path, query, options = {}) {
  const args = ["search", path];
  flag2(args, "--limit", options.limit);
  if (options.caseSensitive === false) args.push("--ignore-case");
  args.push("--json", "--", query);
  return call2(args, options);
}
async function digest(path, options = {}) {
  const args = ["digest", path];
  toggle2(args, "--sections", options.sections);
  flag2(args, "--pages", options.pages);
  flag2(args, "--max-chars", options.maxChars);
  args.push("--json");
  return call2(args, options);
}
async function explain(path, options = {}) {
  return call2(["explain", path, "--json"], options);
}
async function capabilities(options = {}) {
  const args = ["capabilities"];
  toggle2(args, "--mcp", options.mcp);
  return call2(args, options);
}
async function exportIrSchema(options = {}) {
  const args = ["export-ir-schema"];
  toggle2(args, "--bare", options.bare);
  args.push("--json");
  return call2(args, options);
}
async function exportPlanSchema(options = {}) {
  const args = ["export-plan-schema"];
  toggle2(args, "--bare", options.bare);
  flag2(args, "-o", options.out);
  args.push("--json");
  return call2(args, options);
}
async function exportAgentManifest(options = {}) {
  const args = ["export-agent-manifest"];
  toggle2(args, "--bare", options.bare);
  args.push("--json");
  return call2(args, options);
}
async function exportCapabilitiesSchema(options = {}) {
  const args = ["export-capabilities-schema"];
  toggle2(args, "--bare", options.bare);
  flag2(args, "-o", options.out);
  args.push("--json");
  return call2(args, options);
}
async function exportOntology(options = {}) {
  const args = ["export-ontology"];
  toggle2(args, "--bare", options.bare);
  flag2(args, "-o", options.out);
  args.push("--json");
  return call2(args, options);
}
async function exportSvg(path, options = {}) {
  const args = ["export-svg", path];
  flag2(args, "-o", options.out);
  flag2(args, "-p", options.page);
  args.push("--json");
  return call2(args, options);
}
function outputCommand(command, extra) {
  return async (path, options) => {
    const args = [command, path];
    flag2(args, "-o", options?.out);
    if (extra !== void 0 && options !== void 0) extra(args, options);
    args.push("--json");
    return call2(args, options ?? {});
  };
}
async function extractPages(path, from, to, options = {}) {
  const args = ["extract-pages", path, "--from", from, "--to", to];
  flag2(args, "-o", options.out);
  args.push("--json");
  return call2(args, options);
}
async function buildFromIngest(spec, options = {}) {
  const args = ["build-from-ingest", spec];
  flag2(args, "--media-dir", options.mediaDir);
  flag2(args, "-o", options.out);
  args.push("--json");
  return call2(args, options);
}
async function exportHwpx(path, options = {}) {
  const args = ["export-hwpx", path];
  if (options.out !== void 0) args.push(options.out);
  toggle2(args, "--verify", options.verify);
  toggle2(args, "--verify-pages", options.verifyPages);
  args.push("--json");
  return call2(args, options);
}
async function convert(path, options = {}) {
  if (options.out === void 0) {
    throw new UsageError("convert \uB294 \uC0B0\uCD9C \uACBD\uB85C\uAC00 \uD544\uC694\uD569\uB2C8\uB2E4 \u2014 options.out \uC744 \uC9C0\uC815\uD558\uC138\uC694", {
      argv: ["convert", String(path), "--json"],
      exitCode: EXIT_USAGE
    });
  }
  const args = ["convert", path, options.out];
  toggle2(args, "--verify", options.verify);
  toggle2(args, "--verify-pages", options.verifyPages);
  args.push("--json");
  return call2(args, options);
}
async function irDiff(a, b, options = {}) {
  const args = ["ir-diff", a, b];
  flag2(args, "-s", options.section);
  flag2(args, "-p", options.paragraph);
  args.push("--json");
  return call2(args, options);
}
async function renderDiff(path, pathB, options = {}) {
  const args = ["render-diff", path];
  if (pathB !== void 0) args.push(pathB);
  flag2(args, "--via", options.via);
  flag2(args, "-p", options.page);
  flag2(args, "--max-disp", options.maxDisp);
  args.push("--json");
  return call2(args, options);
}
async function verify(path, options = {}) {
  const args = ["verify", path];
  flag2(args, "--expect-pages", options.expectPages);
  flag2(args, "--expect-min-pages", options.expectMinPages);
  flag2(args, "--expect-max-pages", options.expectMaxPages);
  flag2(args, "--expect-min-chars", options.expectMinChars);
  flag2(args, "--expect-min-tables", options.expectMinTables);
  flag2(args, "--expect-table-count", options.expectTableCount);
  repeat(args, "--expect-contains", options.expectContains);
  repeat(args, "--expect-not-contains", options.expectNotContains);
  repeat(args, "--expect-field", options.expectField);
  flag2(args, "--expect-format", options.expectFormat);
  args.push("--json");
  return call2(args, options);
}
async function fillFields(path, data, options = {}) {
  const args = ["edit", "fill-fields", path, "--data", JSON.stringify(data)];
  editFlags2(args, options);
  args.push("--json");
  return call2(args, options);
}
async function replaceText(path, find, replace, options = {}) {
  const args = [
    "edit",
    "replace-text",
    path,
    "--find",
    find,
    "--replace",
    replace
  ];
  flag2(args, "--occurrence", options.occurrence);
  toggle2(args, "--ignore-case", options.ignoreCase);
  editFlags2(args, options);
  args.push("--json");
  return call2(args, options);
}
async function setCell(path, table, row, col, text, options = {}) {
  const args = [
    "edit",
    "set-cell",
    path,
    "--table",
    table,
    "--row",
    row,
    "--col",
    col,
    "--text",
    text
  ];
  toggle2(args, "--keep-style", options.keepStyle);
  editFlags2(args, options);
  args.push("--json");
  return call2(args, options);
}
async function scan(paths, options = {}) {
  const roots = typeof paths === "string" ? [paths] : paths;
  if (roots.length === 0) {
    throw new Error("\uAC80\uC0C9\uD560 \uACBD\uB85C\uAC00 \uC5C6\uC2B5\uB2C8\uB2E4 \u2014 scan \uC740 \uCD5C\uC18C 1\uAC1C\uAC00 \uD544\uC694\uD569\uB2C8\uB2E4");
  }
  const args = ["scan", ...roots];
  toggle2(args, "--probe", options.probe);
  flag2(args, "--max-depth", options.maxDepth);
  flag2(args, "--limit", options.limit);
  args.push("--json");
  return call2(args, options);
}
async function batch(subcommand, paths, options = {}) {
  if (paths.length === 0) {
    throw new Error("\uCC98\uB9AC\uD560 \uD30C\uC77C\uC774 \uC5C6\uC2B5\uB2C8\uB2E4 \u2014 batch \uB294 \uCD5C\uC18C 1\uAC1C\uAC00 \uD544\uC694\uD569\uB2C8\uB2E4");
  }
  const args = ["batch", subcommand];
  flag2(args, "--threads", options.threads);
  flag2(args, "--mode", options.mode);
  flag2(args, "--query", options.query);
  flag2(args, "--out-dir", options.outDir);
  toggle2(args, "--verify", options.verify);
  toggle2(args, "--verify-pages", options.verifyPages);
  args.push(...options.extraArgs ?? [], "--json");
  return runNdjson(args, {
    stdin: `${paths.join("\n")}
`,
    timeoutMs: options.timeoutMs ?? null,
    cwd: options.cwd
  });
}
var exportPdf, exportMarkdown, exportHml, exportDoclang, thumbnail;
var init_commands = __esm({
  "src/commands.ts"() {
    init_envelope();
    init_errors();
    init_process();
    init_document_analysis();
    exportPdf = outputCommand("export-pdf", (args, options) => {
      flag2(args, "-p", options.page);
      flag2(args, "--backend", options.backend);
      flag2(args, "--profile", options.profile);
      repeat(args, "--font-path", options.fontPath);
    });
    exportMarkdown = outputCommand(
      "export-markdown",
      (args, options) => {
        flag2(args, "-p", options.page);
      }
    );
    exportHml = outputCommand("export-hml");
    exportDoclang = outputCommand(
      "export-doclang",
      (args, options) => {
        flag2(args, "--assets-dir", options.assetsDir);
      }
    );
    thumbnail = outputCommand("thumbnail", (args, options) => {
      toggle2(args, "--base64", options.base64);
      toggle2(args, "--data-uri", options.dataUri);
    });
  }
});

// src/browser.ts
init_envelope();
init_errors();
function toBytes(source) {
  if (source instanceof Uint8Array) return source;
  if (source instanceof ArrayBuffer) return new Uint8Array(source);
  throw new RhwpError(
    '\uBE0C\uB77C\uC6B0\uC800 \uD074\uB77C\uC774\uC5B8\uD2B8\uB294 \uD30C\uC77C \uACBD\uB85C\uB97C \uC5F4 \uC218 \uC5C6\uC2B5\uB2C8\uB2E4 \u2014 \uBB38\uC11C \uBC14\uC774\uD2B8(Uint8Array)\uB97C \uB118\uAE30\uC138\uC694.\n  (fetch(...).then(r => r.arrayBuffer()) \uB610\uB294 <input type="file"> \uC758 File.arrayBuffer())'
  );
}
function parseEnvelope(label, json) {
  if (json === void 0) {
    throw new RhwpError(
      `\uC774 WASM \uBE4C\uB4DC\uB294 ${label} \uC744 \uC9C0\uC6D0\uD558\uC9C0 \uC54A\uC2B5\uB2C8\uB2E4 \u2014 @rhwp/editor \uBC84\uC804\uC744 \uD655\uC778\uD558\uC138\uC694`
    );
  }
  let parsed;
  try {
    parsed = JSON.parse(json);
  } catch (cause) {
    throw new RhwpError(`${label} \uACB0\uACFC\uAC00 JSON \uC774 \uC544\uB2D9\uB2C8\uB2E4`, { cause });
  }
  if (parsed === null || typeof parsed !== "object" || Array.isArray(parsed)) {
    throw new RhwpError(`${label} \uACB0\uACFC\uAC00 \uAC1D\uCCB4\uAC00 \uC544\uB2D9\uB2C8\uB2E4`);
  }
  return new Envelope(parsed);
}
function createBrowserClient(wasm) {
  async function withDocument(source, fn) {
    const doc = wasm.fromBytes(toBytes(source));
    try {
      return fn(doc);
    } finally {
      doc.free?.();
    }
  }
  return {
    async info(source) {
      return withDocument(
        source,
        (doc) => new Envelope({
          schemaVersion: "1.0",
          source: "(bytes)",
          pageCount: doc.pageCount()
        })
      );
    },
    async exportText(source) {
      return withDocument(source, (doc) => {
        const pageCount = doc.pageCount();
        const pages = [];
        for (let page = 0; page < pageCount; page += 1) {
          const text = doc.extractPageText?.(page) ?? (page === 0 ? doc.extractText() : "");
          pages.push({ page, text });
        }
        return new Envelope({ schemaVersion: "1.0", pageCount, pages });
      });
    },
    async exportStructure(source) {
      return withDocument(source, (doc) => parseEnvelope("\uAD6C\uC870 \uCD94\uCD9C", doc.structureJson?.()));
    },
    async exportTables(source) {
      return withDocument(source, (doc) => parseEnvelope("\uD45C \uCD94\uCD9C", doc.tablesJson?.()));
    },
    async fields(source) {
      return withDocument(source, (doc) => parseEnvelope("\uB204\uB984\uD2C0 \uC870\uD68C", doc.fieldsJson?.()));
    },
    async search(source, query, options = {}) {
      return withDocument(
        source,
        (doc) => parseEnvelope("\uAC80\uC0C9", doc.searchJson?.(query, options.caseSensitive ?? true))
      );
    },
    async renderPage(source, page) {
      return withDocument(source, (doc) => {
        const pageCount = doc.pageCount();
        if (!Number.isInteger(page) || page < 0 || page >= pageCount) {
          throw new RhwpError(
            `\uCABD ${page} \uC774(\uAC00) \uBC94\uC704\uB97C \uBC97\uC5B4\uB0AC\uC2B5\uB2C8\uB2E4 (0..${pageCount - 1})`
          );
        }
        return doc.renderPageSvg(page);
      });
    }
  };
}
function createNodeClient() {
  const load = async () => Promise.resolve().then(() => (init_commands(), commands_exports));
  const asPath = (source) => {
    if (typeof source === "string") return source;
    throw new RhwpError(
      "Node \uD074\uB77C\uC774\uC5B8\uD2B8\uB294 \uBB38\uC11C \uBC14\uC774\uD2B8\uB97C \uC9C1\uC811 \uBC1B\uC9C0 \uC54A\uC2B5\uB2C8\uB2E4 \u2014 \uD30C\uC77C \uACBD\uB85C\uB97C \uB118\uAE30\uC138\uC694.\n  (\uBC14\uC774\uD2B8\uB97C \uB2E4\uB904\uC57C \uD558\uBA74 \uC784\uC2DC \uD30C\uC77C\uB85C \uC4F4 \uB4A4 \uACBD\uB85C\uB97C \uB118\uAE30\uC138\uC694)"
    );
  };
  return {
    async info(source) {
      return (await load()).info(asPath(source));
    },
    async exportText(source) {
      return (await load()).exportText(asPath(source));
    },
    async exportStructure(source) {
      return (await load()).exportStructure(asPath(source));
    },
    async exportTables(source) {
      return (await load()).exportTables(asPath(source));
    },
    async fields(source) {
      return (await load()).fields(asPath(source));
    },
    async search(source, query, options = {}) {
      const commands = await load();
      return options.caseSensitive === void 0 ? commands.search(asPath(source), query) : commands.search(asPath(source), query, { caseSensitive: options.caseSensitive });
    },
    async renderPage(source, page) {
      const commands = await load();
      const result = await commands.exportSvg(asPath(source), { page });
      const svg = result.raw["svg"];
      if (typeof svg === "string") return svg;
      throw new RhwpError(
        "export-svg \uBD09\uD22C\uC5D0 SVG \uBCF8\uBB38\uC774 \uC5C6\uC2B5\uB2C8\uB2E4 \u2014 \uD30C\uC77C\uB85C \uC0B0\uCD9C\uB410\uC744 \uC218 \uC788\uC2B5\uB2C8\uB2E4(-o \uD655\uC778)"
      );
    }
  };
}

export { createBrowserClient, createNodeClient };
//# sourceMappingURL=browser.js.map
//# sourceMappingURL=browser.js.map