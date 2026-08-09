/**
 * API 2층 — 세션(핸들) 클라이언트.
 *
 * `mcp-serve` 를 stdio JSON-RPC 로 띄우고 `hwp_doc_*` 도구를 그대로 노출한다.
 * 1층(무상태)이 호출마다 문서를 재파싱하는 반면, 2층은 한 번 열어 두고 여러 번
 * 만진다 — 대형 문서 반복 작업에서 차이가 크다.
 *
 * ```ts
 * const doc = await openDocument('서식.hwp');
 * try {
 *   await doc.fillFields({ 성명: '홍길동' });
 *   const saved = await doc.save('제출본.hwp', { verify: true });
 * } finally {
 *   await doc.close();   // 서버가 남으면 다음 작업이 파일을 못 연다
 * }
 * ```
 *
 * @packageDocumentation
 */

import { spawn, type ChildProcessWithoutNullStreams } from 'node:child_process';

import { findBinary } from './binary.js';
import { Envelope, type RawEnvelope } from './envelope.js';
import {
  ProtocolError,
  RhwpError,
  RhwpTimeoutError,
  SessionClosedError,
  UsageError,
} from './errors.js';

/** 세션 호출 기본 제한 시간(ms). `null` 을 주면 무제한. */
export const DEFAULT_SESSION_TIMEOUT_MS = 300_000;

/** JSON-RPC 응답 프레임. */
interface RpcResponse {
  readonly jsonrpc?: string;
  readonly id?: number | string | null;
  readonly result?: unknown;
  readonly error?: unknown;
}

/** {@link Session} 생성 옵션. */
export interface SessionOptions {
  /** 역할 프로필 — 도구 노출 범위를 제한한다. */
  readonly profile?: string | undefined;
  /** 작업 디렉터리. */
  readonly cwd?: string | undefined;
  /**
   * 호출 하나당 제한 시간(ms). 기본 {@link DEFAULT_SESSION_TIMEOUT_MS}, `null` 이면 무제한.
   *
   * 파이썬판(`Session(timeout=300.0)`)엔 있었지만 Node 엔 없어 응답이 영원히
   * 안 와도 끊지 못했다(D-14). stdio 가 이벤트 기반이라 파이썬처럼 블로킹
   * `readline` 을 건드릴 필요 없이 대기 중인 요청 하나만 타이머로 정리하면 된다.
   */
  readonly timeoutMs?: number | null | undefined;
}

/**
 * `mcp-serve` 자식 프로세스 하나를 감싼 JSON-RPC 클라이언트.
 *
 * 보통은 {@link openDocument} 가 만들어 주는 {@link Document} 를 쓰면 되고,
 * 여러 문서를 한 서버에서 열고 싶을 때만 직접 만든다.
 */
export class Session {
  private readonly child: ChildProcessWithoutNullStreams;
  private readonly argv: readonly string[];
  private nextId = 0;
  private closed = false;
  /** 요청을 직렬화한다 — 응답 id 대조가 성립하려면 한 번에 하나만 보내야 한다. */
  private queue: Promise<unknown> = Promise.resolve();
  private buffer = '';
  private readonly pending = new Map<
    number,
    {
      resolve: (value: RpcResponse) => void;
      reject: (reason: unknown) => void;
      timer?: NodeJS.Timeout;
    }
  >();
  private stderrText = '';
  private readonly timeoutMs: number | null;

  constructor(options: SessionOptions = {}) {
    const binary = findBinary();
    const args = ['mcp-serve'];
    if (options.profile) args.push('--profile', options.profile);
    this.argv = [binary, ...args];
    this.timeoutMs =
      options.timeoutMs === null ? null : (options.timeoutMs ?? DEFAULT_SESSION_TIMEOUT_MS);

    try {
      this.child = spawn(binary, args, {
        cwd: options.cwd,
        shell: false,
        windowsHide: true,
      }) as ChildProcessWithoutNullStreams;
    } catch (cause) {
      throw new RhwpError(`mcp-serve 기동에 실패했습니다: ${String(cause)}`, {
        argv: this.argv,
        cause,
      });
    }

    this.child.stdout.setEncoding('utf8');
    this.child.stdout.on('data', (chunk: string) => this.onStdout(chunk));
    this.child.stderr.setEncoding('utf8');
    this.child.stderr.on('data', (chunk: string) => {
      // 진단은 마지막 것이 가장 구체적이라 뒤에서 자른다.
      this.stderrText = (this.stderrText + chunk).slice(-8192);
    });
    this.child.on('close', () => this.failAllPending('mcp-serve 가 응답 없이 종료했습니다'));
    this.child.on('error', (cause) => this.failAllPending(`mcp-serve 오류: ${cause.message}`));
  }

  /** 줄 단위로 프레임을 잘라 대기 중인 요청에 넘긴다. */
  private onStdout(chunk: string): void {
    this.buffer += chunk;
    let index = this.buffer.indexOf('\n');
    while (index >= 0) {
      const line = this.buffer.slice(0, index).trim();
      this.buffer = this.buffer.slice(index + 1);
      if (line) this.dispatch(line);
      index = this.buffer.indexOf('\n');
    }
  }

  private dispatch(line: string): void {
    let message: RpcResponse;
    try {
      message = JSON.parse(line) as RpcResponse;
    } catch (cause) {
      this.failAllPending(`JSON-RPC 프레임이 아닙니다: ${String(cause)}`);
      return;
    }
    // 서버가 보낼 수 있는 알림은 id 가 없다 — 우리 응답이 아니므로 흘린다.
    if (message.id === undefined || message.id === null) return;

    const id = typeof message.id === 'number' ? message.id : Number(message.id);
    const waiter = this.pending.get(id);
    if (!waiter) return; // 우리가 기다리지 않는 id — 무시가 안전하다.
    this.pending.delete(id);
    if (waiter.timer) clearTimeout(waiter.timer);
    waiter.resolve(message);
  }

  private failAllPending(reason: string): void {
    const error = new ProtocolError(reason, {
      argv: this.argv,
      exitCode: this.child.exitCode ?? undefined,
      stderr: this.stderrText,
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
  async call<T extends RawEnvelope = RawEnvelope>(
    name: string,
    args: Readonly<Record<string, unknown>>,
  ): Promise<Envelope<T>> {
    if (this.closed) {
      throw new SessionClosedError(`세션이 이미 닫혔습니다 (도구: ${name})`, {
        argv: this.argv,
      });
    }
    // 요청을 큐로 직렬화한다 — 병렬로 보내면 응답 순서를 보장할 수 없다.
    const task = this.queue.then(() => this.send<T>(name, args));
    this.queue = task.catch(() => undefined);
    return task;
  }

  private async send<T extends RawEnvelope>(
    name: string,
    args: Readonly<Record<string, unknown>>,
  ): Promise<Envelope<T>> {
    this.nextId += 1;
    const id = this.nextId;
    const request = {
      jsonrpc: '2.0',
      id,
      method: 'tools/call',
      params: { name, arguments: { ...args } },
    };

    const response = await new Promise<RpcResponse>((resolve, reject) => {
      const waiter: {
        resolve: (value: RpcResponse) => void;
        reject: (reason: unknown) => void;
        timer?: NodeJS.Timeout;
      } = { resolve, reject };
      this.pending.set(id, waiter);

      if (this.timeoutMs !== null) {
        waiter.timer = setTimeout(() => {
          this.pending.delete(id);
          reject(
            new RhwpTimeoutError(
              `${name} 호출이 제한 시간 ${this.timeoutMs}ms 를 초과했습니다`,
              { argv: this.argv, stderr: this.stderrText },
            ),
          );
        }, this.timeoutMs);
        waiter.timer.unref?.();
      }

      if (this.child.exitCode !== null || !this.child.stdin.writable) {
        this.pending.delete(id);
        if (waiter.timer) clearTimeout(waiter.timer);
        reject(
          new ProtocolError('mcp-serve 가 이미 종료되어 요청을 보낼 수 없습니다', {
            argv: this.argv,
            exitCode: this.child.exitCode ?? undefined,
            stderr: this.stderrText,
          }),
        );
        return;
      }
      this.child.stdin.write(`${JSON.stringify(request)}\n`, 'utf8', (err) => {
        if (err) {
          this.pending.delete(id);
          if (waiter.timer) clearTimeout(waiter.timer);
          reject(
            new ProtocolError(`mcp-serve 로 쓰기에 실패했습니다: ${err.message}`, {
              argv: this.argv,
              stderr: this.stderrText,
              cause: err,
            }),
          );
        }
      });
    });

    return this.unwrap<T>(name, response);
  }

  /** JSON-RPC 응답에서 도구 결과 봉투를 꺼낸다. */
  private unwrap<T extends RawEnvelope>(name: string, response: RpcResponse): Envelope<T> {
    if (response.error !== undefined) {
      const err = response.error;
      const message =
        err !== null && typeof err === 'object' && 'message' in err
          ? String((err as Record<string, unknown>)['message'])
          : String(err);
      throw new ProtocolError(`${name}: ${message}`, { argv: this.argv });
    }

    const result = response.result;
    if (result === null || typeof result !== 'object') {
      throw new ProtocolError(`${name}: result 가 없습니다`, { argv: this.argv });
    }
    const record = result as Record<string, unknown>;

    // 구조화 결과가 있으면 그걸 쓴다 (텍스트 재파싱보다 정확).
    let body: Record<string, unknown> | undefined;
    const structured = record['structuredContent'];
    if (structured !== null && typeof structured === 'object' && !Array.isArray(structured)) {
      body = { ...(structured as Record<string, unknown>) };
    } else {
      const content = record['content'];
      if (Array.isArray(content) && content.length > 0) {
        const first = content[0] as Record<string, unknown> | undefined;
        const text = first?.['text'];
        if (typeof text === 'string') {
          try {
            const parsed: unknown = JSON.parse(text);
            if (parsed !== null && typeof parsed === 'object' && !Array.isArray(parsed)) {
              body = parsed as Record<string, unknown>;
            }
          } catch {
            // 텍스트가 JSON 이 아닌 도구도 있다 — 그대로 담아 돌려준다.
            body = { text };
          }
        }
      }
    }

    if (record['isError'] === true) {
      throw new UsageError(`${name} 호출이 거부됐습니다`, {
        argv: this.argv,
        exitCode: 2,
        stderr: body ? JSON.stringify(body) : '',
        envelope: body,
      });
    }

    if (body === undefined) {
      throw new ProtocolError(`${name}: 결과 본문을 해석하지 못했습니다`, { argv: this.argv });
    }
    return new Envelope(body as T);
  }

  /** 서버를 정리한다. 여러 번 불러도 안전하다. */
  async close(): Promise<void> {
    if (this.closed) return;
    this.closed = true;

    try {
      this.child.stdin.end();
    } catch {
      /* 이미 닫혔으면 무시 */
    }

    await new Promise<void>((resolve) => {
      if (this.child.exitCode !== null || this.child.signalCode !== null) {
        resolve();
        return;
      }
      const timer = setTimeout(() => {
        // stdin 을 닫아도 안 죽으면 강제 종료 — 서버가 남아 파일을 잡고 있으면
        // 다음 작업이 막힌다.
        this.child.kill('SIGKILL');
        resolve();
      }, 5000);
      timer.unref?.();
      this.child.once('close', () => {
        clearTimeout(timer);
        resolve();
      });
    });

    this.failAllPending('세션이 닫혔습니다');
  }

  /** `await using` 지원 (TS 5.2+ / Node 20+). */
  async [Symbol.asyncDispose](): Promise<void> {
    await this.close();
  }
}

/** 열린 문서 핸들 — 세션 위의 얇은 편의 계층. */
export class Document {
  private closed = false;

  constructor(
    private readonly session: Session,
    /** 서버가 발급한 핸들 식별자. */
    readonly docId: string,
    private readonly ownsSession: boolean,
  ) {}

  private async callTool<T extends RawEnvelope = RawEnvelope>(
    tool: string,
    args: Readonly<Record<string, unknown>> = {},
  ): Promise<Envelope<T>> {
    if (this.closed) {
      throw new SessionClosedError(`닫힌 문서 핸들입니다 (${this.docId})`);
    }
    return this.session.call<T>(tool, { docId: this.docId, ...args });
  }

  // ── 조회 ────────────────────────────────────────────────────────────────

  /** 문서 요약 (재파싱 없음). */
  async info(): Promise<Envelope> {
    return this.callTool('hwp_doc_info');
  }

  /** 평문. `page` 를 주면 그 쪽만. */
  async text(options: { readonly page?: number | undefined } = {}): Promise<Envelope> {
    return this.callTool('hwp_doc_text', options.page === undefined ? {} : { page: options.page });
  }

  /** 누름틀 목록. */
  async fields(): Promise<Envelope> {
    return this.callTool('hwp_doc_fields');
  }

  /** 표 전량. */
  async tables(): Promise<Envelope> {
    return this.callTool('hwp_doc_tables');
  }

  /** 주소가 붙은 검색. */
  async search(
    query: string,
    options: { readonly caseSensitive?: boolean | undefined } = {},
  ): Promise<Envelope> {
    return this.callTool('hwp_doc_search', {
      query,
      caseSensitive: options.caseSensitive ?? true,
    });
  }

  /**
   * 한 쪽을 SVG 파일로 — 편집 직후 눈검증 루프를 닫는 도구.
   *
   * @param page - 0 기준 쪽 번호. 편집 봉투의 `changedPages` 를 그대로 넘기면
   *   바뀐 쪽만 상수 비용으로 확인할 수 있다.
   * @param output - SVG 를 쓸 경로. **도구 계약상 필수**다.
   */
  async renderPage(page: number, output: string): Promise<Envelope> {
    return this.callTool('hwp_doc_render_page', { page, output });
  }

  // ── 편집 ────────────────────────────────────────────────────────────────

  /** 누름틀 채우기. */
  async fillFields(data: Readonly<Record<string, unknown>>): Promise<Envelope> {
    return this.callTool('hwp_doc_fill_fields', { data: { ...data } });
  }

  /** 문자열 치환. */
  async replaceText(
    find: string,
    replace: string,
    options: { readonly caseSensitive?: boolean | undefined } = {},
  ): Promise<Envelope> {
    return this.callTool('hwp_doc_replace_text', {
      find,
      replace,
      caseSensitive: options.caseSensitive ?? true,
    });
  }

  /** 표 셀 기록. 좌표는 {@link Document.tables} 로 확인한다. */
  async setCell(table: number, row: number, col: number, text: string): Promise<Envelope> {
    return this.callTool('hwp_doc_set_cell', { table, row, col, text });
  }

  // ── 저장·정리 ───────────────────────────────────────────────────────────

  /** 저장. `verify: true` 면 저장 직후 자기검증 보고가 봉투에 담긴다. */
  async save(
    output: string,
    options: { readonly verify?: boolean | undefined } = {},
  ): Promise<Envelope> {
    return this.callTool('hwp_doc_save', { output, verify: options.verify ?? false });
  }

  /** 핸들을 닫는다 (세션을 소유하면 서버도 함께 정리). */
  async close(): Promise<void> {
    if (this.closed) return;
    try {
      await this.session.call('hwp_close', { docId: this.docId });
    } catch (error) {
      // 이미 서버가 죽었거나 핸들이 만료된 경우 — 정리 경로에서 새 예외를
      // 만들지 않는다. 원인 예외를 가리면 진단이 어려워진다.
      if (!(error instanceof RhwpError)) throw error;
    } finally {
      this.closed = true;
      if (this.ownsSession) await this.session.close();
    }
  }

  /** `await using` 지원. */
  async [Symbol.asyncDispose](): Promise<void> {
    await this.close();
  }

  toString(): string {
    return `Document(${this.docId}, ${this.closed ? 'closed' : 'open'})`;
  }
}

/** {@link openDocument} 옵션. */
export interface OpenOptions {
  /** 보호 문서 암호. */
  readonly password?: string | undefined;
  /** 이미 만든 세션에 얹는다. 주면 문서를 닫아도 세션은 남는다. */
  readonly session?: Session | undefined;
  /** 새 세션을 만들 때의 역할 프로필. */
  readonly profile?: string | undefined;
  /** 작업 디렉터리. */
  readonly cwd?: string | undefined;
}

/**
 * 문서를 열어 핸들을 돌려준다.
 *
 * `session` 을 주지 않으면 전용 서버를 띄우고, 문서를 닫을 때 함께 정리한다.
 */
export async function openDocument(path: string, options: OpenOptions = {}): Promise<Document> {
  const ownsSession = options.session === undefined;
  const session =
    options.session ?? new Session({ profile: options.profile, cwd: options.cwd });

  try {
    const args: Record<string, unknown> = { path };
    if (options.password !== undefined) args['password'] = options.password;
    const result = await session.call('hwp_open', args);

    const docId = result.raw['docId'];
    if (typeof docId !== 'string' || !docId) {
      throw new ProtocolError(
        `hwp_open 이 docId 를 돌려주지 않았습니다: ${JSON.stringify(result.raw)}`,
      );
    }
    return new Document(session, docId, ownsSession);
  } catch (error) {
    if (ownsSession) await session.close();
    throw error;
  }
}
