import tsParser from '@typescript-eslint/parser';
import tsPlugin from '@typescript-eslint/eslint-plugin';

/**
 * ESLint flat config.
 *
 * `.eslintrc.cjs` 가 아니라 flat config 를 쓰는 이유: ESLint 9 부터 flat 이 기본이고
 * `.eslintrc` 계열은 10 에서 제거됐다. 새 패키지를 이미 사라진 형식으로 시작하면
 * 첫 메이저 업그레이드에서 통째로 다시 써야 한다.
 *
 * 타입 인지(type-aware) 규칙은 켜지 않는다. 린트는 "바이너리 없이" 도는 단위 잡에서
 * 매 푸시마다 실행되는데, 프로그램 전체 타입 정보를 요구하면 실행 시간이 몇 배가 되고
 * 정작 타입 문제는 `npm run typecheck` 가 이미 더 엄격하게 잡는다. 린트는 타입이
 * 아니라 **습관**을 본다.
 */
export default [
  {
    // 배포 산출물과 생성물은 대상이 아니다. 생성 파일(`src/ir.ts`,`src/envelopes.ts`)은
    // 사람이 고치지 않으며, 최신 여부는 린트가 아니라 `gen:check` 가 보증한다.
    ignores: ['dist/**', 'node_modules/**', 'coverage/**', 'src/ir.ts', 'src/envelopes.ts'],
  },
  {
    files: ['**/*.ts'],
    languageOptions: {
      parser: tsParser,
      ecmaVersion: 2022,
      sourceType: 'module',
      parserOptions: {
        // 프로젝트를 물리지 않는다 — 위에 적은 대로 타입 인지 규칙을 쓰지 않기 때문.
        project: false,
      },
    },
    plugins: {
      '@typescript-eslint': tsPlugin,
    },
    rules: {
      // --- 조용한 실패를 막는 규칙 ---
      // 이 바인딩의 최대 위험은 "판정을 못 읽고 성공으로 넘어가는 것"이다.
      // 빈 catch 와 버려진 반환값이 그 통로다.
      'no-empty': ['error', { allowEmptyCatch: false }],
      'no-fallthrough': 'error',
      eqeqeq: ['error', 'always', { null: 'ignore' }],
      // `null`(검증 안 함)과 `undefined`(모름)를 구분하는 계약이라 `== null` 만 예외로 둔다.

      // --- 타입스크립트 습관 ---
      '@typescript-eslint/no-unused-vars': [
        'error',
        { argsIgnorePattern: '^_', varsIgnorePattern: '^_', caughtErrorsIgnorePattern: '^_' },
      ],
      // `any` 는 봉투 필드의 판정 의미를 지운다. 모르는 값은 `unknown` 으로 받고
      // 좁혀서 쓴다.
      '@typescript-eslint/no-explicit-any': 'error',
      // 타입 전용 import 를 값 import 와 섞으면 `verbatimModuleSyntax` 아래에서
      // 런타임에 빈 모듈을 부르는 코드가 남는다.
      '@typescript-eslint/consistent-type-imports': [
        'error',
        { prefer: 'type-imports', fixStyle: 'inline-type-imports' },
      ],
      '@typescript-eslint/no-non-null-assertion': 'error',
      '@typescript-eslint/no-empty-object-type': 'off',
      // 예외 계층이 빈 서브클래스(`class BinaryNotFoundError extends RhwpError {}`)를
      // 쓴다 — 이름 자체가 정보이므로 "빈 클래스"로 지적받으면 안 된다.
      'no-useless-constructor': 'off',

      // --- 프로세스를 다루는 코드의 안전장치 ---
      // 자식 프로세스를 셸에 태우면 인용 규칙이 플랫폼마다 달라진다. 윈도우에서
      // 공백·따옴표가 든 문서 경로가 조용히 다른 인자로 쪼개지는 사고가 여기서 난다.
      // 이 바인딩은 항상 argv 배열(`spawn`/`execFile`)로 실행한다.
      'no-restricted-imports': [
        'error',
        {
          paths: [
            {
              name: 'node:child_process',
              importNames: ['exec', 'execSync'],
              message: 'spawn/execFile 로 argv 배열을 넘기세요 — 셸 인용은 플랫폼마다 다릅니다.',
            },
            {
              name: 'child_process',
              message: "'node:' 접두사를 붙이세요 — 번들러가 내장 모듈임을 확실히 알게 합니다.",
            },
          ],
        },
      ],
      // 진단은 stderr 에, 결과는 반환값에. 라이브러리가 stdout 을 오염시키면
      // 이 패키지를 파이프라인에 끼운 호출자의 JSON 이 깨진다.
      'no-console': ['error', { allow: ['warn', 'error'] }],
    },
  },
  {
    // 테스트·예제·생성기는 콘솔 출력과 픽스처 조작이 정상 동작이다.
    files: ['test/**/*.ts', 'examples/**/*.ts', 'tools/**/*.ts', '*.config.ts'],
    rules: {
      'no-console': 'off',
      '@typescript-eslint/no-explicit-any': 'off',
      '@typescript-eslint/no-non-null-assertion': 'off',
    },
  },
];
