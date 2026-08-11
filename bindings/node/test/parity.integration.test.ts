/**
 * 계약 패리티 — 자기서술(`capabilities`)과 TypeScript API 를 대조한다.
 *
 * **M19 의 핵심 수용 기준이 이 파일이다.** 바인딩은 새 표면이 아니라 기존 계약의
 * 재포장이므로, rhwp 에 명령이 늘었는데 바인딩이 뒤처지면 그 사실이 어디에도 드러나지
 * 않는다. 사용자는 "이 도구엔 그 기능이 없다"고 결론 내리고, 그 결론은 틀렸다.
 *
 * 그래서 여기서는 목록을 손으로 관리하지 않는다. `capabilities` 가 단일 출처이고,
 * 대조는 기계가 한다. 새 명령이 추가되면 이 테스트가 먼저 실패한다.
 */

import { existsSync } from 'node:fs';

import { describe, expect, it } from 'vitest';

import * as commands from '../src/commands.js';
import {
  EXIT_OK,
  EXIT_RUNTIME,
  EXIT_USAGE,
  EXIT_VERIFY,
  EXIT_VERIFY_PAGES,
  Plan,
  exportStructure,
  exportText,
  fields,
  fillFields,
  info,
  toCamel,
  type Envelope,
} from '../src/index.js';
import {
  declaredCommands,
  declaredTools,
  fieldSample,
  fieldSampleReady,
  hasBinary,
  loadCapabilities,
  useTempDir,
} from './helpers/integration.js';

/**
 * 바인딩이 감싸지 않는 계열.
 *
 * - `diagnostic` — 엔진 개발자용 덤프다. 봉투 모양이 자주 바뀌고, 감싸면 그 변동이
 *   바인딩의 공개 API 변경으로 새어 나온다.
 * - `internal` — 픽스처 생성기다.
 * - `serve` — 프로세스를 띄우는 축이라 2층(`Session`)이 이미 담당한다.
 */
const NOT_WRAPPED = new Set(['diagnostic', 'internal', 'serve']);

/**
 * CLI 에서 서브커맨드로 갈라지는 명령 — 바인딩은 서브커맨드마다 함수를 둔다.
 *
 * `edit(path, 'fill-fields', ...)` 같은 문자열 디스패치는 오타를 런타임까지 미룬다.
 */
const SUBCOMMAND_WRAPPERS: Readonly<Record<string, readonly string[]>> = {
  edit: ['fillFields', 'replaceText', 'setCell'],
  // 판정(harnessStatus)은 여기 없다 — CLI 에서 최상위 `harness-status`
  // (category=diagnostic)로 갈라져 NOT_WRAPPED 로 건너뛰기 때문이다.
  harness: ['harnessInit', 'harnessWrap'],
  anchor: ['anchorAdd', 'anchorCheckpoint', 'anchorVerify'],
  bundle: ['bundleExport', 'bundleVerify'],
  disclose: ['discloseRedact', 'discloseVerify', 'discloseRestore'],
  settle: ['settlePropose', 'settleVerify', 'settleRecord'],
};

/**
 * 위층이 감싸는 명령 — 무상태 함수로 노출하지 않는다.
 *
 * `run` 을 1층에 두면 호출자가 계획서 JSON 을 손으로 조립하게 되고, 빌더의 문법 검사와
 * `check()` 미리보기를 통째로 우회한다.
 */
const HIGHER_LAYER: Readonly<Record<string, { readonly label: string; readonly value: unknown }>> =
  {
    run: { label: 'Plan (3층 계획 빌더)', value: Plan },
  };

/** 세션 전용 도구의 이름 접두어. 무상태 매니페스트에 섞이면 안 된다. */
const SESSION_TOOL_PREFIXES = ['hwp_open', 'hwp_close', 'hwp_doc_'] as const;

describe.skipIf(!hasBinary)('패리티 — 명령 표면', () => {
  it('선언된 json 명령마다 바인딩에 대응 함수가 있다', async () => {
    const declared = await declaredCommands();
    // 수기 목록을 두지 않는다. 모듈이 실제로 내보내는 이름이 곧 표면이다.
    const exported = new Set(Object.keys(commands));
    const missing: string[] = [];

    for (const [name, spec] of declared) {
      if (spec.json !== true) continue;
      if (NOT_WRAPPED.has(spec.category ?? '')) continue;

      const subcommands = SUBCOMMAND_WRAPPERS[name];
      if (subcommands !== undefined) {
        for (const wrapper of subcommands) {
          if (!exported.has(wrapper)) missing.push(`${name} → commands.${wrapper}`);
        }
        continue;
      }

      const higher = HIGHER_LAYER[name];
      if (higher !== undefined) {
        if (typeof higher.value !== 'function') missing.push(`${name} → ${higher.label}`);
        continue;
      }

      // `export-tables` → `exportTables`. 바인딩 자신의 변환 규칙을 쓴다 — 여기서
      // 손으로 다시 매핑하면 규칙이 두 벌이 되고 언젠가 어긋난다.
      const expected = toCamel(name.replace(/-/g, '_'));
      if (!exported.has(expected)) missing.push(`${name} → commands.${expected}`);
    }

    expect(
      missing,
      `바인딩이 빠뜨린 명령:\n  ${missing.join('\n  ')}\n` +
        'rhwp 에 명령이 늘었습니다 — src/commands.ts 에 래퍼를 추가하세요.',
    ).toHaveLength(0);
  });

  it('json 을 낸다고 선언한 명령은 어떤 필드가 나오는지도 선언한다', async () => {
    const declared = await declaredCommands();
    const silent: string[] = [];

    for (const [name, spec] of declared) {
      if (spec.json !== true) continue;
      if ((spec.recordFields?.length ?? 0) === 0) silent.push(name);
    }

    // `recordFields` 는 타입 생성기의 입력이다. 비어 있으면 그 명령의 봉투는 영원히
    // `Record<string, unknown>` 으로 남는다 — 타입이 계약을 강제하지 못한다.
    expect(silent, `recordFields 를 선언하지 않은 json 명령: ${silent.join(', ')}`).toHaveLength(0);
  });
});

describe.skipIf(!hasBinary)('패리티 — MCP 매니페스트', () => {
  it('모든 MCP 도구가 실존하는 CLI 명령을 가리킨다', async () => {
    const declared = await declaredCommands();
    const tools = await declaredTools();
    const dangling: string[] = [];

    for (const tool of tools) {
      const target = tool.cli?.command;
      if (target !== undefined && !declared.has(target)) {
        dangling.push(`${tool.name} → ${target}`);
      }
    }

    // 도구 정의와 명령 구현이 갈라지면, 매니페스트를 읽고 만든 에이전트 코드가
    // 실행 시점에 깨진다. 선언과 실행은 단일 출처여야 한다.
    expect(dangling, `실존하지 않는 명령을 가리키는 도구: ${dangling.join(', ')}`).toHaveLength(0);
  });

  it('세션 도구가 무상태 매니페스트에 섞이지 않는다', async () => {
    const tools = await declaredTools();

    const leaked = tools
      .map((tool) => tool.name)
      .filter((name) => SESSION_TOOL_PREFIXES.some((prefix) => name.startsWith(prefix)));

    // 무상태 도구는 경로를 받고 세션 도구는 핸들을 받는다. 한 목록에 섞이면 소비자는
    // 핸들 없이 세션 도구를 부르거나, 세션을 열어 놓고 무상태 도구를 부른다.
    expect(leaked, `세션 도구가 무상태 매니페스트에 섞였다: ${leaked.join(', ')}`).toHaveLength(0);
  });
});

describe.skipIf(!hasBinary)('패리티 — 종료 코드 사전', () => {
  it('바인딩이 매핑하는 다섯 코드를 도구가 전부 설명한다', async () => {
    const capabilities = await loadCapabilities();
    const dictionary = capabilities.get<Record<string, string>>('exitCodes');

    for (const code of [EXIT_OK, EXIT_RUNTIME, EXIT_USAGE, EXIT_VERIFY, EXIT_VERIFY_PAGES]) {
      const description = dictionary[String(code)];
      // 설명 없는 코드를 바인딩이 예외로 옮기면, 그 예외 메시지가 도구 문서보다
      // 앞서 나가게 된다. 사전이 먼저다.
      expect(typeof description, `exit ${code} 설명이 사전에 없다`).toBe('string');
      expect((description ?? '').trim(), `exit ${code} 설명이 비어 있다`).not.toBe('');
    }
  });
});

describe.skipIf(!fieldSampleReady)('패리티 — 선언한 필드가 실제 봉투에 나온다', () => {
  it('대표 명령 네 개의 recordFields 가 봉투에 그대로 있다', async () => {
    const declared = await declaredCommands();
    const sample = fieldSample();

    // 전 명령을 돌지 않는 이유: 산출 계열은 파일을 만들고 변환 계열은 오래 걸린다.
    // 조회 네 개로 "선언 ↔ 봉투"의 어긋남은 충분히 드러난다.
    const checks: readonly (readonly [string, Envelope])[] = [
      ['info', await info(sample)],
      ['export-text', await exportText(sample)],
      ['fields', await fields(sample)],
      ['export-structure', await exportStructure(sample)],
    ];

    const problems: string[] = [];
    for (const [name, envelope] of checks) {
      const spec = declared.get(name);
      if (spec === undefined) {
        problems.push(`${name}: capabilities 에 없음`);
        continue;
      }
      const actual = new Set(envelope.keys());
      for (const field of spec.recordFields ?? []) {
        // 중첩 경로(`steps[].confusable` 같은)는 최상위 대조 대상이 아니다.
        if (field.includes('[') || field.includes('.')) continue;
        if (!actual.has(field)) {
          problems.push(`${name}: 선언한 '${field}' 가 봉투에 없음 (실제: ${[...actual].sort().join(', ')})`);
        }
      }
      // 봉투 계약의 최소 조건 — 버전 없이는 소비자가 진화를 따라갈 수 없다.
      if (envelope.schemaVersion === undefined) problems.push(`${name}: schemaVersion 이 없음`);
    }

    expect(problems, problems.join('\n')).toHaveLength(0);
  });
});

describe.skipIf(!fieldSampleReady)('패리티 — 편집 계층과 계획 계층의 계약 차이', () => {
  const tempPath = useTempDir();

  it('없는 누름틀을 편집은 notFound 로 보고하고, 계획은 선검증으로 막는다', async () => {
    const sample = fieldSample();
    const missingField = '절대로존재하지않는필드XYZ';

    // 편집 계층: **일부만 채우는 것이 유효한 사용**이다. 서식 하나에 여러 부서가
    // 나눠 채우는 흐름이 실제로 있으므로, 못 찾은 이름은 오류가 아니라 보고다.
    const edited = await fillFields(sample, { [missingField]: '값' }, {
      out: tempPath('부분채움.hwp'),
    });
    expect(edited.get<readonly string[]>('notFound')).toContain(missingField);

    // 계획 계층: **전부 아니면 전무**다. 계획은 한 번의 저장으로 원자적으로 끝나야
    // 하므로, 실행 가능성이 의심스러우면 실행 자체를 하지 않는다.
    const planOut = tempPath('계획은안나옴.hwp');
    const planned = await new Plan(sample, planOut).fillFields({ [missingField]: '값' }).run();

    expect(planned.ok).toBe(false);
    expect(planned.violations.length).toBeGreaterThanOrEqual(1);
    expect(existsSync(planOut)).toBe(false);

    // 두 계약을 하나로 통일하고 싶은 유혹이 늘 있다. 통일하면 둘 중 하나가 망가진다 —
    // 편집을 엄격하게 하면 부분 채움이 불가능해지고, 계획을 느슨하게 하면 반쪽 편집
    // 문서가 돌아온다. 바인딩은 두 계약을 그대로 전달한다.
    expect(edited.get<number>('filledCount')).toBe(0);
  });
});
