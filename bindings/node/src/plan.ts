/**
 * API 3층 — 계획 실행기.
 *
 * 도구를 체이닝하는 대신 계획서 하나를 만든다. rhwp 가 **정적 선검증(실행 0)** →
 * **원자 실행**(전 step 인메모리 적용) → **사후 단언 통과 시에만 단 한 번 저장**
 * 순으로 처리하므로, 중간 실패가 반쪽 편집 문서를 남기지 않는다.
 *
 * ```ts
 * const plan = new Plan('서식.hwp', '제출본.hwp')
 *   .fillFields({ 성명: '홍길동' })
 *   .setCheckbox(1)
 *   .verify();
 *
 * const preview = await plan.check();   // 디스크 무변경
 * if (preview.ok) await plan.run();
 * ```
 *
 * @packageDocumentation
 */

import { capabilities } from './commands.js';
import { Envelope, type RawEnvelope } from './envelope.js';
import { RhwpError, UsageError } from './errors.js';
import { runJson, type RunOptions } from './process.js';

/** 계획 step 하나 (직렬화된 형태). */
export type PlanStep = Readonly<Record<string, unknown>> & { readonly action: string };

/** 계획서 전체. */
export interface PlanDocument {
  readonly planVersion: '1.0';
  readonly input: string;
  readonly output: string;
  readonly steps: readonly PlanStep[];
  readonly assertions?: Readonly<Record<string, boolean>>;
  /** 참이면 선검증만 하고 디스크를 건드리지 않는다. */
  readonly dryRun?: boolean;
}

/** 계획 실행/검사 결과 저널. */
export class PlanResult extends Envelope {
  /** 위반 없이 통과했는가 (검사·실행 공통). */
  get ok(): boolean {
    return this.violations.length === 0;
  }

  /** 선검증 위반 목록. 통과했으면 빈 배열. */
  get violations(): Envelope[] {
    return this.children('invalid');
  }

  /** 검사 전용 실행이었는가 (디스크 무변경). */
  get isDryRun(): boolean {
    return this.raw['dryRun'] === true;
  }

  /** 검사 모드의 step 별 미리보기. 실행 모드면 빈 배열. */
  get preview(): Envelope[] {
    return this.children('preview');
  }

  /** 실행 모드의 step 별 결과. 검사 모드면 빈 배열. */
  get steps(): Envelope[] {
    return this.children('steps');
  }

  /**
   * 위반을 사람이 읽을 여러 줄로 — 로그·오류 메시지에 그대로 쓴다.
   */
  describeViolations(): string {
    const items = this.violations;
    if (items.length === 0) return '위반 없음';
    return items
      .map((v) => {
        const raw = v.raw;
        const step = raw['step'] ?? '?';
        const action = raw['action'] ?? '?';
        const reason = raw['reason'] ?? '(사유 없음)';
        return `  step ${String(step)} (${String(action)}): ${String(reason)}`;
      })
      .join('\n');
  }
}

/** {@link Plan.replaceText} 옵션. */
export interface PlanReplaceOptions {
  /** 이 순번 하나만 (0 기준). */
  readonly occurrence?: number | undefined;
  /** 대소문자를 구분할지. 기본 참. */
  readonly caseSensitive?: boolean | undefined;
}

/** {@link Plan.setCell} 옵션. */
export interface PlanSetCellOptions {
  /** 기존 글자 모양을 유지할지. */
  readonly keepStyle?: boolean | undefined;
}

/** 좌표가 0 이상의 유한 정수인지. `NaN` 을 통과시키면 직렬화에서 `null` 이 되어 사라진다. */
function assertIndex(label: string, value: number): void {
  if (!Number.isInteger(value) || value < 0) {
    throw new RangeError(
      `${label} 은 0 이상의 정수여야 합니다 (받음: ${String(value)}) — ` +
        'NaN·소수는 계획서 직렬화에서 사라져 rhwp 가 다른 좌표를 편집한다',
    );
  }
}

/**
 * 계획서 빌더 — 체이닝으로 step 을 쌓는다.
 *
 * 빌더는 **문법만** 검사한다(값 타입·필수 인자). 실제 실행 가능성은 rhwp 의
 * 선검증이 판정한다 — 판정자를 두 곳에 두면 반드시 어긋난다.
 */
export class Plan {
  private readonly steps: PlanStep[] = [];
  private readonly assertions: Record<string, boolean> = {};

  constructor(
    private readonly input: string,
    private readonly output: string,
  ) {
    if (!input) throw new Error('input 경로가 필요합니다');
    if (!output) throw new Error('output 경로가 필요합니다');
  }

  /** 누름틀 채우기. `{ "이름#1": "값" }` 으로 동명 순번 지정. */
  fillFields(data: Readonly<Record<string, unknown>>): this {
    if (data === null || typeof data !== 'object' || Object.keys(data).length === 0) {
      throw new Error('fillFields 는 비어 있지 않은 { 필드: 값 } 객체가 필요합니다');
    }
    this.steps.push({ action: 'fill_fields', data: { ...data } });
    return this;
  }

  /** 문자열 치환. `occurrence` 를 주면 그 순번 하나만. */
  replaceText(find: string, replace: string, options: PlanReplaceOptions = {}): this {
    if (!find) throw new Error('replaceText 의 find 는 비어 있을 수 없습니다');
    if (typeof replace !== 'string') throw new TypeError('replace 는 문자열이어야 합니다');

    const step: Record<string, unknown> = {
      action: 'replace_text',
      find,
      replace,
      caseSensitive: options.caseSensitive ?? true,
    };
    if (options.occurrence !== undefined) {
      assertIndex('occurrence', options.occurrence);
      step['occurrence'] = options.occurrence;
    }
    this.steps.push(step as PlanStep);
    return this;
  }

  /** 표 셀 기록. 좌표는 `exportTables` 로 확인한다. */
  setCell(
    table: number,
    row: number,
    col: number,
    text: string,
    options: PlanSetCellOptions = {},
  ): this {
    assertIndex('table', table);
    assertIndex('row', row);
    assertIndex('col', col);
    if (typeof text !== 'string') throw new TypeError('text 는 문자열이어야 합니다');
    if (/[\r\n\t]/.test(text)) {
      throw new Error('셀 값에 줄바꿈·탭은 넣을 수 없습니다 (한 줄 값 기록)');
    }
    const step: Record<string, unknown> = { action: 'set_cell', table, row, col, text };
    if (options.keepStyle) step['keepStyle'] = true;
    this.steps.push(step as PlanStep);
    return this;
  }

  /** 빈 체크박스(□) 중 `occurrence` 번째를 표시(☑)한다. */
  setCheckbox(occurrence: number): this {
    assertIndex('occurrence', occurrence);
    this.steps.push({ action: 'set_checkbox', occurrence });
    return this;
  }

  /** 저장 직후 자기검증을 요구한다 (실패 시 저장 없이 exit 3). */
  verify(enabled = true): this {
    this.assertions['verify'] = enabled;
    return this;
  }

  /** 채우지 못한 필드가 하나도 없어야 한다고 단언한다. */
  requireAllFieldsFound(enabled = true): this {
    this.assertions['notFoundEmpty'] = enabled;
    return this;
  }

  /** 계획서 JSON 구조를 돌려준다 (검토·저장·전송용). */
  toJSON(options: { readonly dryRun?: boolean } = {}): PlanDocument {
    if (this.steps.length === 0) {
      throw new Error('step 이 하나도 없는 계획은 실행할 수 없습니다');
    }
    const document: Record<string, unknown> = {
      planVersion: '1.0',
      input: this.input,
      output: this.output,
      steps: [...this.steps],
    };
    if (Object.keys(this.assertions).length > 0) {
      document['assertions'] = { ...this.assertions };
    }
    if (options.dryRun) document['dryRun'] = true;
    return document as unknown as PlanDocument;
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
  async check(options: RunOptions = {}): Promise<PlanResult> {
    await assertDryRunSupported(options);
    return execute(this.toJSON({ dryRun: true }), options);
  }

  /** 실행한다. 단언이 실패하면 **저장 없이** 판정이 담긴 저널을 돌려준다. */
  async run(options: RunOptions = {}): Promise<PlanResult> {
    return execute(this.toJSON(), options);
  }

  toString(): string {
    const actions = this.steps.map((s) => s.action).join(', ');
    return `Plan(${this.input} → ${this.output}: [${actions}])`;
  }
}

/** dry-run 지원 여부 캐시 — 명령마다 capabilities 를 부를 이유가 없다. */
let dryRunSupport: boolean | undefined;

/**
 * rhwp 가 계획 `--dry-run` 을 지원하는지 자기서술로 확인한다.
 *
 * 지원하지 않는 버전에서 `check()` 를 그냥 실행으로 내려보내면, 호출자는 "검사만
 * 했다"고 믿는데 파일이 만들어진다. 그건 조용한 데이터 사고다.
 */
async function assertDryRunSupported(options: RunOptions): Promise<void> {
  if (dryRunSupport === undefined) {
    const caps = await capabilities({ timeoutMs: options.timeoutMs, cwd: options.cwd });
    const commands = caps.raw['commands'];
    dryRunSupport = false;
    if (Array.isArray(commands)) {
      for (const command of commands) {
        if (
          command !== null &&
          typeof command === 'object' &&
          (command as Record<string, unknown>)['name'] === 'run'
        ) {
          const flags = (command as Record<string, unknown>)['flags'];
          dryRunSupport = Array.isArray(flags) && flags.includes('--dry-run');
          break;
        }
      }
    }
  }
  if (!dryRunSupport) {
    throw new RhwpError(
      '이 rhwp 는 계획 --dry-run 을 지원하지 않습니다 (#3759 이전 버전).\n' +
        '  check() 를 실행으로 대체하지 않습니다 — 검사인 줄 알고 문서가 편집되면 안 됩니다.\n' +
        '  rhwp 를 갱신하거나, 위험을 감수한다면 run() 을 명시적으로 부르세요.',
    );
  }
}

/** 테스트에서 지원 여부 캐시를 비운다. */
export function clearPlanCapabilityCache(): void {
  dryRunSupport = undefined;
}

/**
 * 계획서를 인라인으로 넘겨 실행한다.
 *
 * 선검증 위반은 exit 2 라 기본 규약대로면 {@link UsageError} 가 된다. 하지만 계획
 * 실행에서 위반은 **정상적인 결과**다 — 계획을 고쳐 다시 검사하는 것이 설계된
 * 흐름이므로, `invalid[]` 를 담은 봉투는 예외 대신 값으로 돌려준다. `invalid` 가
 * 없는 exit 2 는 진짜 호출 조립 버그이므로 그대로 올린다.
 */
async function execute(plan: PlanDocument, options: RunOptions): Promise<PlanResult> {
  const args = ['run', '--plan-json', JSON.stringify(plan), '--json'];
  try {
    return new PlanResult(await runJson<RawEnvelope>(args, options));
  } catch (error) {
    if (error instanceof UsageError && error.envelope && 'invalid' in error.envelope) {
      return new PlanResult(error.envelope);
    }
    throw error;
  }
}

/**
 * 이미 만들어 둔 계획서(객체)를 그대로 실행한다.
 *
 * 빌더를 쓰지 않고 JSON 파일에서 읽어온 계획을 돌릴 때 쓴다. 선검증 위반은
 * {@link execute} 와 같은 규약으로 예외가 아니라 결과로 돌아온다.
 */
export async function runPlan(
  plan: PlanDocument | Readonly<Record<string, unknown>>,
  options: RunOptions = {},
): Promise<PlanResult> {
  return execute(plan as PlanDocument, options);
}
