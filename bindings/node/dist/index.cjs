'use strict';

var fs = require('fs');
var os = require('os');
var path = require('path');
var url = require('url');
var child_process = require('child_process');

var _documentCurrentScript = typeof document !== 'undefined' ? document.currentScript : null;
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
  if (exitCode === exports.EXIT_OK) return;
  const { throwOnVerdict = false, ...context } = options;
  const ctx = { ...context, exitCode };
  if (exitCode === exports.EXIT_USAGE) {
    throw new exports.UsageError("\uD638\uCD9C \uC778\uC790\uAC00 \uC62C\uBC14\uB974\uC9C0 \uC54A\uC2B5\uB2C8\uB2E4", ctx);
  }
  if (exitCode === exports.EXIT_RUNTIME) {
    throw new exports.RhwpRuntimeError("\uBB38\uC11C \uCC98\uB9AC\uC5D0 \uC2E4\uD328\uD588\uC2B5\uB2C8\uB2E4", ctx);
  }
  if (exitCode === exports.EXIT_VERIFY || exitCode === exports.EXIT_VERIFY_PAGES) {
    if (throwOnVerdict) {
      const label = exitCode === exports.EXIT_VERIFY_PAGES ? "\uD398\uC774\uC9C0 \uC218\uAC00 \uC77C\uCE58\uD558\uC9C0 \uC54A\uC2B5\uB2C8\uB2E4" : "\uAC80\uC99D \uB2E8\uC5B8\uC774 \uC2E4\uD328\uD588\uC2B5\uB2C8\uB2E4";
      throw new exports.VerdictFailed(label, ctx);
    }
    return;
  }
  throw new exports.RhwpRuntimeError(
    `\uC54C \uC218 \uC5C6\uB294 \uC885\uB8CC \uCF54\uB4DC\uC785\uB2C8\uB2E4 (${exitCode}) \u2014 rhwp \uC640 \uBC14\uC778\uB529 \uBC84\uC804\uC774 \uC5B4\uAE0B\uB0AC\uC744 \uC218 \uC788\uC2B5\uB2C8\uB2E4`,
    ctx
  );
}
function isKnownExitCode(code) {
  return code === exports.EXIT_OK || code === exports.EXIT_RUNTIME || code === exports.EXIT_USAGE || code === exports.EXIT_VERIFY || code === exports.EXIT_VERIFY_PAGES;
}
exports.EXIT_OK = void 0; exports.EXIT_RUNTIME = void 0; exports.EXIT_USAGE = void 0; exports.EXIT_VERIFY = void 0; exports.EXIT_VERIFY_PAGES = void 0; exports.RhwpError = void 0; exports.BinaryNotFoundError = void 0; exports.UsageError = void 0; exports.RhwpRuntimeError = void 0; exports.VerdictFailed = void 0; exports.ProtocolError = void 0; exports.SessionClosedError = void 0; exports.EnvelopeKeyError = void 0; exports.RhwpTimeoutError = void 0;
var init_errors = __esm({
  "src/errors.ts"() {
    exports.EXIT_OK = 0;
    exports.EXIT_RUNTIME = 1;
    exports.EXIT_USAGE = 2;
    exports.EXIT_VERIFY = 3;
    exports.EXIT_VERIFY_PAGES = 4;
    exports.RhwpError = class extends Error {
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
    exports.BinaryNotFoundError = class extends exports.RhwpError {
    };
    exports.UsageError = class extends exports.RhwpError {
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
    exports.RhwpRuntimeError = class extends exports.RhwpError {
    };
    exports.VerdictFailed = class extends exports.RhwpError {
      /** exit 4 (페이지 수 불일치)인지. */
      get isPageCountMismatch() {
        return this.exitCode === exports.EXIT_VERIFY_PAGES;
      }
    };
    exports.ProtocolError = class extends exports.RhwpError {
    };
    exports.SessionClosedError = class extends exports.RhwpError {
    };
    exports.EnvelopeKeyError = class extends exports.RhwpError {
    };
    exports.RhwpTimeoutError = class extends exports.RhwpError {
    };
  }
});
function binaryName() {
  return process.platform === "win32" ? "rhwp.exe" : "rhwp";
}
function clearBinaryCache() {
  cached = void 0;
}
function bundledDir() {
  const here = typeof __dirname === "string" ? __dirname : path.dirname(url.fileURLToPath((typeof document === 'undefined' ? require('u' + 'rl').pathToFileURL(__filename).href : (_documentCurrentScript && _documentCurrentScript.tagName.toUpperCase() === 'SCRIPT' && _documentCurrentScript.src || new URL('index.cjs', document.baseURI).href))));
  return path.join(here, "_bin");
}
function expandTilde(raw) {
  if (raw === "~") return os.homedir();
  if (raw.startsWith("~/") || process.platform === "win32" && raw.startsWith("~\\")) {
    return path.join(os.homedir(), raw.slice(2));
  }
  return raw;
}
function isExecutableFile(path) {
  let stat;
  try {
    stat = fs.statSync(path);
  } catch {
    return false;
  }
  if (!stat.isFile()) return false;
  if (process.platform === "win32") {
    return /\.(exe|bat|cmd)$/i.test(path);
  }
  try {
    fs.accessSync(path, fs.constants.X_OK);
    return true;
  } catch {
    return false;
  }
}
function fromEnv() {
  const raw = (process.env[exports.ENV_VAR] ?? "").trim();
  if (!raw) return void 0;
  let candidate = path.resolve(expandTilde(raw));
  try {
    if (fs.statSync(candidate).isDirectory()) {
      candidate = path.join(candidate, binaryName());
    }
  } catch {
  }
  if (isExecutableFile(candidate)) return candidate;
  throw new exports.BinaryNotFoundError(
    `${exports.ENV_VAR} \uAC00 \uAC00\uB9AC\uD0A4\uB294 \uC2E4\uD589 \uD30C\uC77C\uC744 \uC4F8 \uC218 \uC5C6\uC2B5\uB2C8\uB2E4: ${raw}
  (\uC874\uC7AC\uD558\uC9C0 \uC54A\uAC70\uB098, \uD30C\uC77C\uC774 \uC544\uB2C8\uAC70\uB098, \uC2E4\uD589 \uAD8C\uD55C\uC774 \uC5C6\uC2B5\uB2C8\uB2E4)`
  );
}
function fromBundle() {
  const candidate = path.join(bundledDir(), binaryName());
  return isExecutableFile(candidate) ? candidate : void 0;
}
function fromPath() {
  const name = binaryName();
  const entries = (process.env.PATH ?? "").split(path.delimiter).filter(Boolean);
  for (const entry of entries) {
    const candidate = path.join(entry, name);
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
  tried.push(`${exports.ENV_VAR} (\uBBF8\uC124\uC815)`);
  const bundled = fromBundle();
  if (bundled) {
    cached = bundled;
    return cached;
  }
  tried.push(`\uD328\uD0A4\uC9C0 \uB3D9\uBD09 (${path.join(bundledDir(), binaryName())})`);
  const onPath = fromPath();
  if (onPath) {
    cached = onPath;
    return cached;
  }
  tried.push(`PATH (${binaryName()} \uC5C6\uC74C)`);
  throw new exports.BinaryNotFoundError(
    "rhwp \uC2E4\uD589 \uD30C\uC77C\uC744 \uCC3E\uC9C0 \uBABB\uD588\uC2B5\uB2C8\uB2E4. \uB2E4\uC74C \uC21C\uC11C\uB85C \uD0D0\uC0C9\uD588\uC2B5\uB2C8\uB2E4:\n" + tried.map((t, i) => `  ${i + 1}. ${t}`).join("\n") + `

\uD574\uACB0: rhwp \uB97C \uC124\uCE58\uD574 PATH \uC5D0 \uB450\uAC70\uB098, ${exports.ENV_VAR} \uB85C \uACBD\uB85C\uB97C \uC9C0\uC815\uD558\uC138\uC694.`
  );
}
exports.ENV_VAR = void 0; var cached;
var init_binary = __esm({
  "src/binary.ts"() {
    init_errors();
    exports.ENV_VAR = "RHWP_BIN";
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
function snakeKeys(value) {
  return walk(value, toSnake);
}
function camelKeys(value) {
  return walk(value, toCamel);
}
function walk(value, transform) {
  if (Array.isArray(value)) {
    return value.map((item) => walk(item, transform));
  }
  if (value !== null && typeof value === "object") {
    const out = {};
    for (const [key, item] of Object.entries(value)) {
      out[transform(key)] = walk(item, transform);
    }
    return out;
  }
  return value;
}
function isSafeIdentifier(name) {
  return /^[A-Za-z_$][A-Za-z0-9_$]*$/.test(name) && !RESERVED.has(name);
}
function propertyKey(name) {
  return isSafeIdentifier(name) ? name : JSON.stringify(name);
}
var ACRONYM_BOUNDARY, WORD_BOUNDARY, RESERVED;
var init_naming = __esm({
  "src/naming.ts"() {
    ACRONYM_BOUNDARY = /([A-Z])([A-Z][a-z])/g;
    WORD_BOUNDARY = /([a-z0-9])([A-Z])/g;
    RESERVED = /* @__PURE__ */ new Set([
      "break",
      "case",
      "catch",
      "class",
      "const",
      "continue",
      "debugger",
      "default",
      "delete",
      "do",
      "else",
      "enum",
      "export",
      "extends",
      "false",
      "finally",
      "for",
      "function",
      "if",
      "import",
      "in",
      "instanceof",
      "new",
      "null",
      "return",
      "super",
      "switch",
      "this",
      "throw",
      "true",
      "try",
      "typeof",
      "var",
      "void",
      "while",
      "with"
    ]);
  }
});

// src/envelope.ts
function asEnvelope(value) {
  return value instanceof exports.Envelope ? value : new exports.Envelope(value);
}
exports.VerifyReport = void 0; exports.Envelope = void 0;
var init_envelope = __esm({
  "src/envelope.ts"() {
    init_errors();
    init_naming();
    exports.VerifyReport = class {
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
    exports.Envelope = class _Envelope {
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
        throw new exports.EnvelopeKeyError(
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
        return new exports.VerifyReport(value);
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
function stringify(value) {
  if (typeof value === "boolean") {
    throw new TypeError("\uBD88\uB9AC\uC5B8\uC740 \uC778\uC790 \uAC12\uC774 \uB420 \uC218 \uC5C6\uC2B5\uB2C8\uB2E4 \u2014 \uD50C\uB798\uADF8\uB85C \uD45C\uD604\uD558\uC138\uC694");
  }
  return String(value);
}
function spawnCollected(argv, options) {
  const timeoutMs = options.timeoutMs === null ? null : options.timeoutMs ?? exports.DEFAULT_TIMEOUT_MS;
  return new Promise((resolve2, reject) => {
    let child;
    try {
      child = child_process.spawn(argv[0], argv.slice(1), {
        cwd: options.cwd,
        // 실행 파일 경로는 우리가 탐색한 것이므로 셸을 태우지 않는다 —
        // 셸을 거치면 윈도우 인용 규칙 때문에 한글 경로가 깨진다.
        shell: false,
        windowsHide: true
      });
    } catch (cause) {
      reject(new exports.RhwpError(`rhwp \uC2E4\uD589\uC5D0 \uC2E4\uD328\uD588\uC2B5\uB2C8\uB2E4: ${String(cause)}`, { argv, cause }));
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
        () => reject(new exports.RhwpError(`rhwp \uC2E4\uD589\uC5D0 \uC2E4\uD328\uD588\uC2B5\uB2C8\uB2E4: ${cause.message}`, { argv, cause }))
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
            new exports.RhwpTimeoutError(`\uC81C\uD55C \uC2DC\uAC04 ${timeoutMs}ms \uB97C \uCD08\uACFC\uD588\uC2B5\uB2C8\uB2E4`, {
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
      throw new exports.ProtocolError(`stdout \uC774 \uC21C\uC218 JSON \uC774 \uC544\uB2D9\uB2C8\uB2E4: ${String(cause)}`, {
        argv: result.argv,
        exitCode: result.exitCode,
        stderr: result.stderr,
        cause
      });
    }
    if (parsed === null || typeof parsed !== "object" || Array.isArray(parsed)) {
      throw new exports.ProtocolError(
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
    throw new exports.ProtocolError("\uC131\uACF5\uD588\uB294\uB370 stdout \uC774 \uBE44\uC5B4 \uC788\uC2B5\uB2C8\uB2E4 \u2014 --json \uBD09\uD22C \uACC4\uC57D \uC704\uBC18\uC785\uB2C8\uB2E4", {
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
    throw new exports.ProtocolError(`NDJSON ${lineNo}\uBC88\uC9F8 \uC904\uC774 JSON \uC774 \uC544\uB2D9\uB2C8\uB2E4: ${String(cause)}`, {
      argv,
      exitCode,
      stderr,
      cause
    });
  }
  if (parsed === null || typeof parsed !== "object" || Array.isArray(parsed)) {
    throw new exports.ProtocolError(`NDJSON ${lineNo}\uBC88\uC9F8 \uC904\uC774 \uAC1D\uCCB4\uAC00 \uC544\uB2D9\uB2C8\uB2E4`, {
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
async function* iterNdjson(args, options = {}) {
  const binary = findBinary();
  const argv = [binary, ...args.map(stringify)];
  const child = child_process.spawn(argv[0], argv.slice(1), {
    cwd: options.cwd,
    shell: false,
    windowsHide: true
  });
  if (options.stdin !== void 0) {
    child.stdin.on("error", () => {
    });
    child.stdin.end(options.stdin, "utf8");
  } else {
    child.stdin.end();
  }
  let buffer = "";
  let lineNo = 0;
  try {
    child.stdout.setEncoding("utf8");
    for await (const chunk of child.stdout) {
      buffer += chunk;
      let index = buffer.indexOf("\n");
      while (index >= 0) {
        const line = buffer.slice(0, index);
        buffer = buffer.slice(index + 1);
        lineNo += 1;
        const record = parseLine(line, lineNo, argv, 0, "");
        if (record) yield record;
        index = buffer.indexOf("\n");
      }
    }
    if (buffer.trim()) {
      lineNo += 1;
      const record = parseLine(buffer, lineNo, argv, 0, "");
      if (record) yield record;
    }
  } finally {
    if (child.exitCode === null && child.signalCode === null) {
      child.kill("SIGKILL");
    }
  }
}
exports.DEFAULT_TIMEOUT_MS = void 0;
var init_process = __esm({
  "src/process.ts"() {
    init_binary();
    init_errors();
    exports.DEFAULT_TIMEOUT_MS = 3e5;
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
  return new exports.Envelope(await runJson(args, toRunOptions(options)));
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
  exportDoclang: () => exports.exportDoclang,
  exportHml: () => exports.exportHml,
  exportHwpx: () => exportHwpx,
  exportIrSchema: () => exportIrSchema,
  exportMarkdown: () => exports.exportMarkdown,
  exportOntology: () => exportOntology,
  exportPdf: () => exports.exportPdf,
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
  thumbnail: () => exports.thumbnail,
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
  return new exports.Envelope(await runJson(args, toRunOptions2(options)));
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
    throw new exports.UsageError("convert \uB294 \uC0B0\uCD9C \uACBD\uB85C\uAC00 \uD544\uC694\uD569\uB2C8\uB2E4 \u2014 options.out \uC744 \uC9C0\uC815\uD558\uC138\uC694", {
      argv: ["convert", String(path), "--json"],
      exitCode: exports.EXIT_USAGE
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
exports.exportPdf = void 0; exports.exportMarkdown = void 0; exports.exportHml = void 0; exports.exportDoclang = void 0; exports.thumbnail = void 0;
var init_commands = __esm({
  "src/commands.ts"() {
    init_envelope();
    init_errors();
    init_process();
    init_document_analysis();
    exports.exportPdf = outputCommand("export-pdf", (args, options) => {
      flag2(args, "-p", options.page);
      flag2(args, "--backend", options.backend);
      flag2(args, "--profile", options.profile);
      repeat(args, "--font-path", options.fontPath);
    });
    exports.exportMarkdown = outputCommand(
      "export-markdown",
      (args, options) => {
        flag2(args, "-p", options.page);
      }
    );
    exports.exportHml = outputCommand("export-hml");
    exports.exportDoclang = outputCommand(
      "export-doclang",
      (args, options) => {
        flag2(args, "--assets-dir", options.assetsDir);
      }
    );
    exports.thumbnail = outputCommand("thumbnail", (args, options) => {
      toggle2(args, "--base64", options.base64);
      toggle2(args, "--data-uri", options.dataUri);
    });
  }
});

// src/index.ts
init_binary();
init_errors();
init_envelope();
init_naming();
init_process();
init_commands();

// src/session.ts
init_binary();
init_envelope();
init_errors();
var DEFAULT_SESSION_TIMEOUT_MS = 3e5;
var Session = class {
  child;
  argv;
  nextId = 0;
  closed = false;
  /** 요청을 직렬화한다 — 응답 id 대조가 성립하려면 한 번에 하나만 보내야 한다. */
  queue = Promise.resolve();
  buffer = "";
  pending = /* @__PURE__ */ new Map();
  stderrText = "";
  timeoutMs;
  constructor(options = {}) {
    const binary = findBinary();
    const args = ["mcp-serve"];
    if (options.profile) args.push("--profile", options.profile);
    this.argv = [binary, ...args];
    this.timeoutMs = options.timeoutMs === null ? null : options.timeoutMs ?? DEFAULT_SESSION_TIMEOUT_MS;
    try {
      this.child = child_process.spawn(binary, args, {
        cwd: options.cwd,
        shell: false,
        windowsHide: true
      });
    } catch (cause) {
      throw new exports.RhwpError(`mcp-serve \uAE30\uB3D9\uC5D0 \uC2E4\uD328\uD588\uC2B5\uB2C8\uB2E4: ${String(cause)}`, {
        argv: this.argv,
        cause
      });
    }
    this.child.stdout.setEncoding("utf8");
    this.child.stdout.on("data", (chunk) => this.onStdout(chunk));
    this.child.stderr.setEncoding("utf8");
    this.child.stderr.on("data", (chunk) => {
      this.stderrText = (this.stderrText + chunk).slice(-8192);
    });
    this.child.on("close", () => this.failAllPending("mcp-serve \uAC00 \uC751\uB2F5 \uC5C6\uC774 \uC885\uB8CC\uD588\uC2B5\uB2C8\uB2E4"));
    this.child.on("error", (cause) => this.failAllPending(`mcp-serve \uC624\uB958: ${cause.message}`));
  }
  /** 줄 단위로 프레임을 잘라 대기 중인 요청에 넘긴다. */
  onStdout(chunk) {
    this.buffer += chunk;
    let index = this.buffer.indexOf("\n");
    while (index >= 0) {
      const line = this.buffer.slice(0, index).trim();
      this.buffer = this.buffer.slice(index + 1);
      if (line) this.dispatch(line);
      index = this.buffer.indexOf("\n");
    }
  }
  dispatch(line) {
    let message;
    try {
      message = JSON.parse(line);
    } catch (cause) {
      this.failAllPending(`JSON-RPC \uD504\uB808\uC784\uC774 \uC544\uB2D9\uB2C8\uB2E4: ${String(cause)}`);
      return;
    }
    if (message.id === void 0 || message.id === null) return;
    const id = typeof message.id === "number" ? message.id : Number(message.id);
    const waiter = this.pending.get(id);
    if (!waiter) return;
    this.pending.delete(id);
    if (waiter.timer) clearTimeout(waiter.timer);
    waiter.resolve(message);
  }
  failAllPending(reason) {
    const error = new exports.ProtocolError(reason, {
      argv: this.argv,
      exitCode: this.child.exitCode ?? void 0,
      stderr: this.stderrText
    });
    for (const waiter of this.pending.values()) {
      if (waiter.timer) clearTimeout(waiter.timer);
      waiter.reject(error);
    }
    this.pending.clear();
  }
  /**
   * 도구 하나를 호출하고 결과 봉투를 돌려준다.
   *
   * @throws {SessionClosedError} 이미 닫힌 세션.
   * @throws {UsageError} 도구가 `isError` 를 세운 경우. 서버가 `didYouMean`·
   *   `nextCall` 교정 단서를 실어 보내면 예외의 `envelope` 에 담긴다.
   * @throws {ProtocolError} 응답이 JSON-RPC 계약을 어긴 경우.
   */
  async call(name, args) {
    if (this.closed) {
      throw new exports.SessionClosedError(`\uC138\uC158\uC774 \uC774\uBBF8 \uB2EB\uD614\uC2B5\uB2C8\uB2E4 (\uB3C4\uAD6C: ${name})`, {
        argv: this.argv
      });
    }
    const task = this.queue.then(() => this.send(name, args));
    this.queue = task.catch(() => void 0);
    return task;
  }
  async send(name, args) {
    if (this.closed) {
      throw new exports.SessionClosedError(`\uC138\uC158\uC774 \uC774\uBBF8 \uB2EB\uD614\uC2B5\uB2C8\uB2E4 (\uB3C4\uAD6C: ${name})`, {
        argv: this.argv
      });
    }
    this.nextId += 1;
    const id = this.nextId;
    const request = {
      jsonrpc: "2.0",
      id,
      method: "tools/call",
      params: { name, arguments: { ...args } }
    };
    const response = await new Promise((resolve2, reject) => {
      const waiter = { resolve: resolve2, reject };
      this.pending.set(id, waiter);
      if (this.timeoutMs !== null) {
        waiter.timer = setTimeout(() => {
          if (!this.pending.delete(id)) return;
          this.closed = true;
          try {
            this.child.kill("SIGKILL");
          } catch {
          }
          reject(
            new exports.RhwpTimeoutError(
              `${name} \uD638\uCD9C\uC774 \uC81C\uD55C \uC2DC\uAC04 ${this.timeoutMs}ms \uB97C \uCD08\uACFC\uD588\uC2B5\uB2C8\uB2E4`,
              { argv: this.argv, stderr: this.stderrText }
            )
          );
        }, this.timeoutMs);
        waiter.timer.unref?.();
      }
      if (this.child.exitCode !== null || !this.child.stdin.writable) {
        this.pending.delete(id);
        if (waiter.timer) clearTimeout(waiter.timer);
        reject(
          new exports.ProtocolError("mcp-serve \uAC00 \uC774\uBBF8 \uC885\uB8CC\uB418\uC5B4 \uC694\uCCAD\uC744 \uBCF4\uB0BC \uC218 \uC5C6\uC2B5\uB2C8\uB2E4", {
            argv: this.argv,
            exitCode: this.child.exitCode ?? void 0,
            stderr: this.stderrText
          })
        );
        return;
      }
      this.child.stdin.write(`${JSON.stringify(request)}
`, "utf8", (err) => {
        if (err) {
          this.pending.delete(id);
          if (waiter.timer) clearTimeout(waiter.timer);
          reject(
            new exports.ProtocolError(`mcp-serve \uB85C \uC4F0\uAE30\uC5D0 \uC2E4\uD328\uD588\uC2B5\uB2C8\uB2E4: ${err.message}`, {
              argv: this.argv,
              stderr: this.stderrText,
              cause: err
            })
          );
        }
      });
    });
    return this.unwrap(name, response);
  }
  /** JSON-RPC 응답에서 도구 결과 봉투를 꺼낸다. */
  unwrap(name, response) {
    if (response.error !== void 0) {
      const err = response.error;
      const message = err !== null && typeof err === "object" && "message" in err ? String(err["message"]) : String(err);
      throw new exports.ProtocolError(`${name}: ${message}`, { argv: this.argv });
    }
    const result = response.result;
    if (result === null || typeof result !== "object") {
      throw new exports.ProtocolError(`${name}: result \uAC00 \uC5C6\uC2B5\uB2C8\uB2E4`, { argv: this.argv });
    }
    const record = result;
    let body;
    const structured = record["structuredContent"];
    if (structured !== null && typeof structured === "object" && !Array.isArray(structured)) {
      body = { ...structured };
    } else {
      const content = record["content"];
      if (Array.isArray(content) && content.length > 0) {
        const first = content[0];
        const text = first?.["text"];
        if (typeof text === "string") {
          try {
            const parsed = JSON.parse(text);
            if (parsed !== null && typeof parsed === "object" && !Array.isArray(parsed)) {
              body = parsed;
            }
          } catch {
            body = { text };
          }
        }
      }
    }
    if (record["isError"] === true) {
      throw new exports.UsageError(`${name} \uD638\uCD9C\uC774 \uAC70\uBD80\uB410\uC2B5\uB2C8\uB2E4`, {
        argv: this.argv,
        exitCode: 2,
        stderr: body ? JSON.stringify(body) : "",
        envelope: body
      });
    }
    if (body === void 0) {
      throw new exports.ProtocolError(`${name}: \uACB0\uACFC \uBCF8\uBB38\uC744 \uD574\uC11D\uD558\uC9C0 \uBABB\uD588\uC2B5\uB2C8\uB2E4`, { argv: this.argv });
    }
    return new exports.Envelope(body);
  }
  /** 서버를 정리한다. 여러 번 불러도 안전하다. */
  async close() {
    if (this.closed) return;
    this.closed = true;
    try {
      this.child.stdin.end();
    } catch {
    }
    await new Promise((resolve2) => {
      if (this.child.exitCode !== null || this.child.signalCode !== null) {
        resolve2();
        return;
      }
      const timer = setTimeout(() => {
        this.child.kill("SIGKILL");
        resolve2();
      }, 5e3);
      timer.unref?.();
      this.child.once("close", () => {
        clearTimeout(timer);
        resolve2();
      });
    });
    this.failAllPending("\uC138\uC158\uC774 \uB2EB\uD614\uC2B5\uB2C8\uB2E4");
  }
  /** `await using` 지원 (TS 5.2+ / Node 20+). */
  async [Symbol.asyncDispose]() {
    await this.close();
  }
};
var Document = class {
  constructor(session, docId, ownsSession) {
    this.session = session;
    this.docId = docId;
    this.ownsSession = ownsSession;
  }
  session;
  docId;
  ownsSession;
  closed = false;
  async callTool(tool, args = {}) {
    if (this.closed) {
      throw new exports.SessionClosedError(`\uB2EB\uD78C \uBB38\uC11C \uD578\uB4E4\uC785\uB2C8\uB2E4 (${this.docId})`);
    }
    return this.session.call(tool, { docId: this.docId, ...args });
  }
  // ── 조회 ────────────────────────────────────────────────────────────────
  /** 문서 요약 (재파싱 없음). */
  async info() {
    return this.callTool("hwp_doc_info");
  }
  /** 평문. `page` 를 주면 그 쪽만. */
  async text(options = {}) {
    return this.callTool("hwp_doc_text", options.page === void 0 ? {} : { page: options.page });
  }
  /** 누름틀 목록. */
  async fields() {
    return this.callTool("hwp_doc_fields");
  }
  /** 표 전량. */
  async tables() {
    return this.callTool("hwp_doc_tables");
  }
  /** 주소가 붙은 검색. */
  async search(query, options = {}) {
    return this.callTool("hwp_doc_search", {
      query,
      caseSensitive: options.caseSensitive ?? true
    });
  }
  /**
   * 한 쪽을 SVG 파일로 — 편집 직후 눈검증 루프를 닫는 도구.
   *
   * @param page - 0 기준 쪽 번호. 편집 봉투의 `changedPages` 를 그대로 넘기면
   *   바뀐 쪽만 상수 비용으로 확인할 수 있다.
   * @param output - SVG 를 쓸 경로. **도구 계약상 필수**다.
   */
  async renderPage(page, output) {
    return this.callTool("hwp_doc_render_page", { page, output });
  }
  // ── 편집 ────────────────────────────────────────────────────────────────
  /** 누름틀 채우기. */
  async fillFields(data) {
    return this.callTool("hwp_doc_fill_fields", { data: { ...data } });
  }
  /** 문자열 치환. */
  async replaceText(find, replace, options = {}) {
    return this.callTool("hwp_doc_replace_text", {
      find,
      replace,
      caseSensitive: options.caseSensitive ?? true
    });
  }
  /** 표 셀 기록. 좌표는 {@link Document.tables} 로 확인한다. */
  async setCell(table, row, col, text) {
    return this.callTool("hwp_doc_set_cell", { table, row, col, text });
  }
  // ── 저장·정리 ───────────────────────────────────────────────────────────
  /** 저장. `verify: true` 면 저장 직후 자기검증 보고가 봉투에 담긴다. */
  async save(output, options = {}) {
    return this.callTool("hwp_doc_save", { output, verify: options.verify ?? false });
  }
  /** 핸들을 닫는다 (세션을 소유하면 서버도 함께 정리). */
  async close() {
    if (this.closed) return;
    try {
      await this.session.call("hwp_close", { docId: this.docId });
    } catch (error) {
      if (!(error instanceof exports.RhwpError)) throw error;
    } finally {
      this.closed = true;
      if (this.ownsSession) await this.session.close();
    }
  }
  /** `await using` 지원. */
  async [Symbol.asyncDispose]() {
    await this.close();
  }
  toString() {
    return `Document(${this.docId}, ${this.closed ? "closed" : "open"})`;
  }
};
async function openDocument(path, options = {}) {
  const ownsSession = options.session === void 0;
  const session = options.session ?? new Session({ profile: options.profile, cwd: options.cwd });
  try {
    const args = { path };
    if (options.password !== void 0) args["password"] = options.password;
    const result = await session.call("hwp_open", args);
    const docId = result.raw["docId"];
    if (typeof docId !== "string" || !docId) {
      throw new exports.ProtocolError(
        `hwp_open \uC774 docId \uB97C \uB3CC\uB824\uC8FC\uC9C0 \uC54A\uC558\uC2B5\uB2C8\uB2E4: ${JSON.stringify(result.raw)}`
      );
    }
    return new Document(session, docId, ownsSession);
  } catch (error) {
    if (ownsSession) await session.close();
    throw error;
  }
}

// src/plan.ts
init_commands();
init_envelope();
init_errors();
init_process();
var PlanResult = class extends exports.Envelope {
  /** 위반 없이 통과했는가 (검사·실행 공통). */
  get ok() {
    return this.violations.length === 0;
  }
  /** 선검증 위반 목록. 통과했으면 빈 배열. */
  get violations() {
    return this.children("invalid");
  }
  /** 검사 전용 실행이었는가 (디스크 무변경). */
  get isDryRun() {
    return this.raw["dryRun"] === true;
  }
  /** 검사 모드의 step 별 미리보기. 실행 모드면 빈 배열. */
  get preview() {
    return this.children("preview");
  }
  /** 실행 모드의 step 별 결과. 검사 모드면 빈 배열. */
  get steps() {
    return this.children("steps");
  }
  /**
   * 위반을 사람이 읽을 여러 줄로 — 로그·오류 메시지에 그대로 쓴다.
   */
  describeViolations() {
    const items = this.violations;
    if (items.length === 0) return "\uC704\uBC18 \uC5C6\uC74C";
    return items.map((v) => {
      const raw = v.raw;
      const step = raw["step"] ?? "?";
      const action = raw["action"] ?? "?";
      const reason = raw["reason"] ?? "(\uC0AC\uC720 \uC5C6\uC74C)";
      return `  step ${String(step)} (${String(action)}): ${String(reason)}`;
    }).join("\n");
  }
};
function assertIndex(label, value) {
  if (!Number.isInteger(value) || value < 0) {
    throw new RangeError(
      `${label} \uC740 0 \uC774\uC0C1\uC758 \uC815\uC218\uC5EC\uC57C \uD569\uB2C8\uB2E4 (\uBC1B\uC74C: ${String(value)}) \u2014 NaN\xB7\uC18C\uC218\uB294 \uACC4\uD68D\uC11C \uC9C1\uB82C\uD654\uC5D0\uC11C \uC0AC\uB77C\uC838 rhwp \uAC00 \uB2E4\uB978 \uC88C\uD45C\uB97C \uD3B8\uC9D1\uD55C\uB2E4`
    );
  }
}
var Plan = class {
  constructor(input, output) {
    this.input = input;
    this.output = output;
    if (!input) throw new Error("input \uACBD\uB85C\uAC00 \uD544\uC694\uD569\uB2C8\uB2E4");
    if (!output) throw new Error("output \uACBD\uB85C\uAC00 \uD544\uC694\uD569\uB2C8\uB2E4");
  }
  input;
  output;
  steps = [];
  assertions = {};
  /** 누름틀 채우기. `{ "이름#1": "값" }` 으로 동명 순번 지정. */
  fillFields(data) {
    if (data === null || typeof data !== "object" || Object.keys(data).length === 0) {
      throw new Error("fillFields \uB294 \uBE44\uC5B4 \uC788\uC9C0 \uC54A\uC740 { \uD544\uB4DC: \uAC12 } \uAC1D\uCCB4\uAC00 \uD544\uC694\uD569\uB2C8\uB2E4");
    }
    this.steps.push({ action: "fill_fields", data: { ...data } });
    return this;
  }
  /** 문자열 치환. `occurrence` 를 주면 그 순번 하나만. */
  replaceText(find, replace, options = {}) {
    if (!find) throw new Error("replaceText \uC758 find \uB294 \uBE44\uC5B4 \uC788\uC744 \uC218 \uC5C6\uC2B5\uB2C8\uB2E4");
    if (typeof replace !== "string") throw new TypeError("replace \uB294 \uBB38\uC790\uC5F4\uC774\uC5B4\uC57C \uD569\uB2C8\uB2E4");
    const step = {
      action: "replace_text",
      find,
      replace,
      caseSensitive: options.caseSensitive ?? true
    };
    if (options.occurrence !== void 0) {
      assertIndex("occurrence", options.occurrence);
      step["occurrence"] = options.occurrence;
    }
    this.steps.push(step);
    return this;
  }
  /** 표 셀 기록. 좌표는 `exportTables` 로 확인한다. */
  setCell(table, row, col, text, options = {}) {
    assertIndex("table", table);
    assertIndex("row", row);
    assertIndex("col", col);
    if (typeof text !== "string") throw new TypeError("text \uB294 \uBB38\uC790\uC5F4\uC774\uC5B4\uC57C \uD569\uB2C8\uB2E4");
    if (/[\r\n\t]/.test(text)) {
      throw new Error("\uC140 \uAC12\uC5D0 \uC904\uBC14\uAFC8\xB7\uD0ED\uC740 \uB123\uC744 \uC218 \uC5C6\uC2B5\uB2C8\uB2E4 (\uD55C \uC904 \uAC12 \uAE30\uB85D)");
    }
    const step = { action: "set_cell", table, row, col, text };
    if (options.keepStyle) step["keepStyle"] = true;
    this.steps.push(step);
    return this;
  }
  /** 빈 체크박스(□) 중 `occurrence` 번째를 표시(☑)한다. */
  setCheckbox(occurrence) {
    assertIndex("occurrence", occurrence);
    this.steps.push({ action: "set_checkbox", occurrence });
    return this;
  }
  /** 저장 직후 자기검증을 요구한다 (실패 시 저장 없이 exit 3). */
  verify(enabled = true) {
    this.assertions["verify"] = enabled;
    return this;
  }
  /** 채우지 못한 필드가 하나도 없어야 한다고 단언한다. */
  requireAllFieldsFound(enabled = true) {
    this.assertions["notFoundEmpty"] = enabled;
    return this;
  }
  /** 계획서 JSON 구조를 돌려준다 (검토·저장·전송용). */
  toJSON(options = {}) {
    if (this.steps.length === 0) {
      throw new Error("step \uC774 \uD558\uB098\uB3C4 \uC5C6\uB294 \uACC4\uD68D\uC740 \uC2E4\uD589\uD560 \uC218 \uC5C6\uC2B5\uB2C8\uB2E4");
    }
    const document = {
      planVersion: "1.0",
      input: this.input,
      output: this.output,
      steps: [...this.steps]
    };
    if (Object.keys(this.assertions).length > 0) {
      document["assertions"] = { ...this.assertions };
    }
    if (options.dryRun) document["dryRun"] = true;
    return document;
  }
  /**
   * **실행하지 않고** 검사만 한다 — 디스크 무변경, step 별 미리보기 반환.
   *
   * 위반이 있으면 예외가 아니라 `result.violations` 로 돌려준다. 계획을 고쳐서
   * 다시 검사하는 것이 정상 흐름이기 때문이다.
   *
   * @throws {RhwpError} rhwp 가 계획 `--dry-run` 을 지원하지 않을 때.
   *   **조용히 실제 실행으로 내려가지 않는다** — "검사"인 줄 알고 불렀는데
   *   문서가 편집·저장되면 그보다 나쁜 배신은 없다.
   */
  async check(options = {}) {
    await assertDryRunSupported(options);
    return execute(this.toJSON({ dryRun: true }), options);
  }
  /** 실행한다. 단언이 실패하면 **저장 없이** 판정이 담긴 저널을 돌려준다. */
  async run(options = {}) {
    return execute(this.toJSON(), options);
  }
  toString() {
    const actions = this.steps.map((s) => s.action).join(", ");
    return `Plan(${this.input} \u2192 ${this.output}: [${actions}])`;
  }
};
var dryRunSupport;
async function assertDryRunSupported(options) {
  if (dryRunSupport === void 0) {
    const caps = await capabilities({ timeoutMs: options.timeoutMs, cwd: options.cwd });
    const commands = caps.raw["commands"];
    dryRunSupport = false;
    if (Array.isArray(commands)) {
      for (const command of commands) {
        if (command !== null && typeof command === "object" && command["name"] === "run") {
          const flags = command["flags"];
          dryRunSupport = Array.isArray(flags) && flags.includes("--dry-run");
          break;
        }
      }
    }
  }
  if (!dryRunSupport) {
    throw new exports.RhwpError(
      "\uC774 rhwp \uB294 \uACC4\uD68D --dry-run \uC744 \uC9C0\uC6D0\uD558\uC9C0 \uC54A\uC2B5\uB2C8\uB2E4 (#3759 \uC774\uC804 \uBC84\uC804).\n  check() \uB97C \uC2E4\uD589\uC73C\uB85C \uB300\uCCB4\uD558\uC9C0 \uC54A\uC2B5\uB2C8\uB2E4 \u2014 \uAC80\uC0AC\uC778 \uC904 \uC54C\uACE0 \uBB38\uC11C\uAC00 \uD3B8\uC9D1\uB418\uBA74 \uC548 \uB429\uB2C8\uB2E4.\n  rhwp \uB97C \uAC31\uC2E0\uD558\uAC70\uB098, \uC704\uD5D8\uC744 \uAC10\uC218\uD55C\uB2E4\uBA74 run() \uC744 \uBA85\uC2DC\uC801\uC73C\uB85C \uBD80\uB974\uC138\uC694."
    );
  }
}
function clearPlanCapabilityCache() {
  dryRunSupport = void 0;
}
async function execute(plan, options) {
  const args = ["run", "--plan-json", JSON.stringify(plan), "--json"];
  try {
    return new PlanResult(await runJson(args, options));
  } catch (error) {
    if (error instanceof exports.UsageError && error.envelope && "invalid" in error.envelope) {
      return new PlanResult(error.envelope);
    }
    throw error;
  }
}
async function runPlan(plan, options = {}) {
  return execute(plan, options);
}

// src/schema.ts
init_commands();
function refName(spec) {
  if (spec === null || typeof spec !== "object") return void 0;
  const ref = spec["$ref"];
  if (typeof ref === "string" && ref.startsWith("#/$defs/")) {
    return ref.slice("#/$defs/".length);
  }
  return void 0;
}
var PRIMITIVE_TS = {
  string: "string",
  integer: "number",
  number: "number",
  boolean: "boolean",
  object: "Record<string, unknown>"
};
function scalarHint(spec) {
  if (spec === null || typeof spec !== "object") return "unknown";
  const jsonType = spec["type"];
  if (typeof jsonType === "string" && jsonType in PRIMITIVE_TS) {
    return PRIMITIVE_TS[jsonType];
  }
  return "unknown";
}
var FieldDef = class {
  constructor(name, raw, required) {
    this.name = name;
    this.raw = raw;
    this.required = required;
  }
  name;
  raw;
  required;
  /** 설명 — 생성된 바인딩의 JSDoc 원천. */
  get description() {
    const value = this.raw["description"];
    return typeof value === "string" ? value : "";
  }
  /** JSON 타입 (`object`/`array`/`string`/`integer`/`boolean`). */
  get jsonType() {
    const value = this.raw["type"];
    return typeof value === "string" ? value : void 0;
  }
  /** 다른 정의를 가리키면 그 이름. */
  get ref() {
    return refName(this.raw);
  }
  /** 배열이면 항목이 가리키는 정의 이름. */
  get itemRef() {
    return refName(this.raw["items"]);
  }
  /** 열거형이면 허용 값 목록. */
  get enumValues() {
    const values = this.raw["enum"];
    return Array.isArray(values) ? values.map((v) => String(v)) : void 0;
  }
  /**
   * TypeScript 타입 표기 — 코드 생성기가 그대로 쓴다.
   *
   * 열거형은 리터럴 유니온으로 낸다. TS 에서는 이게 `string` 보다 훨씬 유용하다
   * (오타를 컴파일러가 잡는다).
   */
  get tsType() {
    const ref = this.ref;
    if (ref) return ref;
    const enumValues = this.enumValues;
    if (enumValues && enumValues.length > 0) {
      return enumValues.map((v) => JSON.stringify(v)).join(" | ");
    }
    const jsonType = this.jsonType;
    if (jsonType === "array") {
      const inner = this.itemRef ?? scalarHint(this.raw["items"]);
      return `readonly ${inner}[]`;
    }
    if (jsonType !== void 0 && jsonType in PRIMITIVE_TS) {
      return PRIMITIVE_TS[jsonType];
    }
    const oneOf = this.raw["oneOf"];
    if (Array.isArray(oneOf)) {
      const names = oneOf.map((o) => refName(o)).filter((n) => Boolean(n));
      const first = names[0];
      if (first !== void 0) return `${first} | null`;
    }
    return "unknown";
  }
  toString() {
    return `FieldDef(${this.name}${this.required ? "" : "?"}: ${this.tsType})`;
  }
};
var TypeDef = class {
  constructor(name, raw) {
    this.name = name;
    this.raw = raw;
  }
  name;
  raw;
  /** 설명. */
  get description() {
    const value = this.raw["description"];
    return typeof value === "string" ? value : "";
  }
  /** 객체 타입인지. */
  get isObject() {
    return this.raw["type"] === "object";
  }
  /** `oneOf` 태그 유니온인지 (예: `Control`). */
  get isUnion() {
    return Array.isArray(this.raw["oneOf"]);
  }
  /** 유니온이면 변형 정의 이름 목록. */
  get variants() {
    const oneOf = this.raw["oneOf"];
    if (!Array.isArray(oneOf)) return [];
    return oneOf.map((o) => refName(o)).filter((n) => Boolean(n));
  }
  /** 필드 목록 (필수가 앞, 그 안에서 이름순). */
  get fields() {
    const props = this.raw["properties"];
    if (props === null || typeof props !== "object") return [];
    const requiredRaw = this.raw["required"];
    const required = new Set(
      Array.isArray(requiredRaw) ? requiredRaw.map((r) => String(r)) : []
    );
    return Object.entries(props).map(([name, spec]) => new FieldDef(name, spec ?? {}, required.has(name))).sort((a, b) => {
      if (a.required !== b.required) return a.required ? -1 : 1;
      return a.name.localeCompare(b.name);
    });
  }
  /**
   * 이름으로 필드 하나.
   *
   * @throws {Error} 없으면. 있는 필드를 함께 알려준다.
   */
  field(name) {
    const found = this.fields.find((f) => f.name === name);
    if (!found) {
      throw new Error(
        `${this.name} \uC5D0 '${name}' \uD544\uB4DC\uAC00 \uC5C6\uC2B5\uB2C8\uB2E4. \uC788\uB294 \uD544\uB4DC: ${this.fields.map((f) => f.name).join(", ")}`
      );
    }
    return found;
  }
  toString() {
    return `TypeDef(${this.name}, ${this.fields.length} fields)`;
  }
};
var IrSchema = class {
  constructor(envelope) {
    this.envelope = envelope;
    const schema = envelope["schema"] ?? envelope;
    if (schema === null || typeof schema !== "object" || Array.isArray(schema)) {
      throw new TypeError("\uC2A4\uD0A4\uB9C8 \uBCF8\uBB38\uC774 \uAC1D\uCCB4\uAC00 \uC544\uB2D9\uB2C8\uB2E4");
    }
    this.body = schema;
    const defs = this.body["$defs"];
    this.defs = defs !== null && typeof defs === "object" && !Array.isArray(defs) ? defs : {};
  }
  envelope;
  body;
  defs;
  /** 스키마 버전 — 봉투 `schemaVersion` 과 별개다. */
  get version() {
    const fromEnvelope = this.envelope["irSchemaVersion"] ?? this.envelope["capabilitiesSchemaVersion"];
    const fromBody = this.body["irSchemaVersion"] ?? this.body["capabilitiesSchemaVersion"];
    const value = fromEnvelope ?? fromBody;
    return typeof value === "string" ? value : "unknown";
  }
  /** JSON Schema 방언 URI. */
  get dialect() {
    const value = this.envelope["dialect"] ?? this.body["$schema"];
    return typeof value === "string" ? value : "";
  }
  /** 루트 타입 (보통 `Document`). */
  get root() {
    return this.get(refName(this.body) ?? "Document");
  }
  /** 정의 이름 목록 (정렬). */
  names() {
    return Object.keys(this.defs).sort();
  }
  /** 정의가 있는지. */
  has(name) {
    return name in this.defs;
  }
  /**
   * 이름으로 정의 하나.
   *
   * @throws {Error} 없으면. 있는 정의를 함께 알려준다.
   */
  get(name) {
    if (!(name in this.defs)) {
      throw new Error(
        `\uC2A4\uD0A4\uB9C8\uC5D0 '${name}' \uC815\uC758\uAC00 \uC5C6\uC2B5\uB2C8\uB2E4. \uC788\uB294 \uC815\uC758: ${this.names().join(", ")}`
      );
    }
    return new TypeDef(name, this.defs[name] ?? {});
  }
  /** 정의 개수. */
  get size() {
    return Object.keys(this.defs).length;
  }
  [Symbol.iterator]() {
    const names = this.names();
    let index = 0;
    const self = this;
    return {
      next() {
        if (index >= names.length) return { done: true, value: void 0 };
        const name = names[index];
        index += 1;
        return { done: false, value: self.get(name) };
      }
    };
  }
  /**
   * 끊어진 `$ref` 를 `[참조한 곳, 없는 이름]` 으로 돌려준다.
   *
   * 코드 생성 전에 이걸 확인하면 생성기가 절반쯤 만들다 죽는 일을 막는다.
   */
  danglingReferences() {
    const broken = [];
    for (const typeDef of this) {
      for (const field of typeDef.fields) {
        for (const target of [field.ref, field.itemRef]) {
          if (target && !this.has(target)) {
            broken.push([`${typeDef.name}.${field.name}`, target]);
          }
        }
      }
      for (const variant of typeDef.variants) {
        if (!this.has(variant)) broken.push([typeDef.name, variant]);
      }
    }
    return broken;
  }
  /** 원문 스키마 본문. */
  get raw() {
    return { ...this.body };
  }
  toString() {
    return `IrSchema(v${this.version}, ${this.size} defs)`;
  }
};
async function irSchema(options = {}) {
  const envelope = await exportIrSchema(options);
  return new IrSchema(envelope.raw);
}
async function capabilitiesSchema(options = {}) {
  const envelope = await exportCapabilitiesSchema(options);
  return new IrSchema(envelope.raw);
}

// src/browser.ts
init_envelope();
init_errors();
function toBytes(source) {
  if (source instanceof Uint8Array) return source;
  if (source instanceof ArrayBuffer) return new Uint8Array(source);
  throw new exports.RhwpError(
    '\uBE0C\uB77C\uC6B0\uC800 \uD074\uB77C\uC774\uC5B8\uD2B8\uB294 \uD30C\uC77C \uACBD\uB85C\uB97C \uC5F4 \uC218 \uC5C6\uC2B5\uB2C8\uB2E4 \u2014 \uBB38\uC11C \uBC14\uC774\uD2B8(Uint8Array)\uB97C \uB118\uAE30\uC138\uC694.\n  (fetch(...).then(r => r.arrayBuffer()) \uB610\uB294 <input type="file"> \uC758 File.arrayBuffer())'
  );
}
function parseEnvelope(label, json) {
  if (json === void 0) {
    throw new exports.RhwpError(
      `\uC774 WASM \uBE4C\uB4DC\uB294 ${label} \uC744 \uC9C0\uC6D0\uD558\uC9C0 \uC54A\uC2B5\uB2C8\uB2E4 \u2014 @rhwp/editor \uBC84\uC804\uC744 \uD655\uC778\uD558\uC138\uC694`
    );
  }
  let parsed;
  try {
    parsed = JSON.parse(json);
  } catch (cause) {
    throw new exports.RhwpError(`${label} \uACB0\uACFC\uAC00 JSON \uC774 \uC544\uB2D9\uB2C8\uB2E4`, { cause });
  }
  if (parsed === null || typeof parsed !== "object" || Array.isArray(parsed)) {
    throw new exports.RhwpError(`${label} \uACB0\uACFC\uAC00 \uAC1D\uCCB4\uAC00 \uC544\uB2D9\uB2C8\uB2E4`);
  }
  return new exports.Envelope(parsed);
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
        (doc) => new exports.Envelope({
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
        return new exports.Envelope({ schemaVersion: "1.0", pageCount, pages });
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
          throw new exports.RhwpError(
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
    throw new exports.RhwpError(
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
      throw new exports.RhwpError(
        "export-svg \uBD09\uD22C\uC5D0 SVG \uBCF8\uBB38\uC774 \uC5C6\uC2B5\uB2C8\uB2E4 \u2014 \uD30C\uC77C\uB85C \uC0B0\uCD9C\uB410\uC744 \uC218 \uC788\uC2B5\uB2C8\uB2E4(-o \uD655\uC778)"
      );
    }
  };
}

// src/index.ts
var VERSION = "0.1.0";
var SUPPORTED_SCHEMA_VERSION = "1.0";

exports.DEFAULT_SESSION_TIMEOUT_MS = DEFAULT_SESSION_TIMEOUT_MS;
exports.Document = Document;
exports.FieldDef = FieldDef;
exports.IrSchema = IrSchema;
exports.Plan = Plan;
exports.PlanResult = PlanResult;
exports.SUPPORTED_SCHEMA_VERSION = SUPPORTED_SCHEMA_VERSION;
exports.Session = Session;
exports.TypeDef = TypeDef;
exports.VERSION = VERSION;
exports.asEnvelope = asEnvelope;
exports.batch = batch;
exports.binaryName = binaryName;
exports.buildFromIngest = buildFromIngest;
exports.bundledDir = bundledDir;
exports.camelKeys = camelKeys;
exports.capabilities = capabilities;
exports.capabilitiesSchema = capabilitiesSchema;
exports.clearBinaryCache = clearBinaryCache;
exports.clearPlanCapabilityCache = clearPlanCapabilityCache;
exports.convert = convert;
exports.createBrowserClient = createBrowserClient;
exports.createNodeClient = createNodeClient;
exports.csvToTable = csvToTable;
exports.digest = digest;
exports.explain = explain;
exports.exportAgentManifest = exportAgentManifest;
exports.exportCapabilitiesSchema = exportCapabilitiesSchema;
exports.exportHwpx = exportHwpx;
exports.exportIrSchema = exportIrSchema;
exports.exportOntology = exportOntology;
exports.exportPlanSchema = exportPlanSchema;
exports.exportProvenanceMap = exportProvenanceMap;
exports.exportStructure = exportStructure;
exports.exportSvg = exportSvg;
exports.exportTables = exportTables;
exports.exportText = exportText;
exports.extractData = extractData;
exports.extractPages = extractPages;
exports.fields = fields;
exports.fillFields = fillFields;
exports.findBinary = findBinary;
exports.info = info;
exports.inspect = inspect;
exports.irDiff = irDiff;
exports.irSchema = irSchema;
exports.isKnownExitCode = isKnownExitCode;
exports.isSafeIdentifier = isSafeIdentifier;
exports.iterNdjson = iterNdjson;
exports.openDocument = openDocument;
exports.propertyKey = propertyKey;
exports.raiseForExit = raiseForExit;
exports.renderDiff = renderDiff;
exports.replaceText = replaceText;
exports.runJson = runJson;
exports.runNdjson = runNdjson;
exports.runPlan = runPlan;
exports.runRaw = runRaw;
exports.scan = scan;
exports.search = search;
exports.setCell = setCell;
exports.snakeKeys = snakeKeys;
exports.tableToCsv = tableToCsv;
exports.toCamel = toCamel;
exports.toSnake = toSnake;
exports.verify = verify;
//# sourceMappingURL=index.cjs.map
//# sourceMappingURL=index.cjs.map