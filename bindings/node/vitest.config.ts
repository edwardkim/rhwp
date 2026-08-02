import { defineConfig } from 'vitest/config';

/**
 * 단위와 통합을 **프로젝트로** 나눈다.
 *
 * 파일 이름 규칙(`*.integration.test.ts`)으로 가르는 이유: 태그(`describe.skipIf`)로만
 * 나누면 통합 파일이 어차피 로드되고, 모듈 최상단에서 바이너리를 찾는 코드가 있으면
 * 바이너리 없는 환경에서 수집 단계부터 터진다. 파일 단위로 갈라야 "바이너리 없이
 * 단위 테스트가 돈다"는 계약이 실제로 성립한다.
 *
 * 통합 프로젝트는 `RHWP_BIN` 이 없으면 대상 자체가 비어 조용히 건너뛴다.
 * 없는 걸 실패로 만들지 않는 이유는, 기여자 대부분이 Rust 툴체인 없이 TS 만
 * 만지기 때문이다. CI 의 통합 잡은 항상 `RHWP_BIN` 을 채워서 돌린다.
 */
const hasBinary = Boolean(process.env['RHWP_BIN']);

const INTEGRATION_GLOB = 'test/**/*.integration.test.ts';

export default defineConfig({
  test: {
    projects: [
      {
        test: {
          name: 'unit',
          // 통합 파일은 단위에서 완전히 제외 — 이 프로젝트는 바이너리 없이 돈다.
          include: ['test/**/*.test.ts'],
          exclude: [INTEGRATION_GLOB, '**/node_modules/**', '**/dist/**'],
          environment: 'node',
        },
      },
      {
        test: {
          name: 'integration',
          include: hasBinary ? [INTEGRATION_GLOB] : [],
          exclude: ['**/node_modules/**', '**/dist/**'],
          environment: 'node',
          // 실제 프로세스를 띄우고 문서를 왕복하므로 단위보다 느리다.
          testTimeout: 120_000,
          hookTimeout: 120_000,
        },
      },
    ],
    // 실패한 단언이 어떤 봉투에서 나왔는지 보이도록 진단을 자르지 않는다.
    reporters: ['default'],
  },
});
