# Task M100 #2124 Stage 1 완료 보고 - metrics 도구와 모집단 고정

- 이슈: #2124
- 단계: Stage 1 - metrics 도구와 모집단 고정
- 작성일: 2026-07-10
- 브랜치: `task2124-frontend-baseline`
- 기준 커밋: `upstream/devel` `3077f96d1f9931c50d6d62be77b389d4f66470a9`
- 선행 구현 계획: `mydocs/plans/task_m100_2124_impl.md`

## 1. 완료 요약

제품 프론트 복잡도 모집단, 공통 제외 규칙, schema v2 측정 도구를 고정했다. 초기 draft의 threshold
개수 중심 산식을 보강해 총량, 고복잡도 합, 함수별 diff와 재현성 metadata를 포함한다.

#1904에서 최대·상위 분포는 개선됐지만 CC 총합이 2.3% 증가한 결산과, 이를 보정한 #2130의 총량
순감소·함수별 diff 규칙을 frontend 측정 방식에 반영했다.

## 2. 변경 사항

| 파일 | 변경 내용 |
|------|-----------|
| `rhwp-studio/package.json` | metrics 실행 script 추가 |
| `scripts/frontend-metrics/package.json` | ESLint/SonarJS/parser와 호환 TypeScript 고정 |
| `scripts/frontend-metrics/package-lock.json` | 측정 도구 버전 고정 |
| `scripts/frontend-metrics.mjs` | schema v2 frontend metrics 수집·비교 도구 |
| `mydocs/tech/investigations/issue-2124/task_m100_2124_frontend_metrics_scope.md` | 공식 모집단, 제외군, 산식 문서화 |

루트 `package.json`이나 `@rhwp/editor` runtime dependency는 추가하지 않았다.

## 3. 고정한 산식

- 제품 파일/물리 LOC/code LOC/함수 LOC.
- Total CC, 전체·group Top 20 합, CC>25/100 count와 sum, Max CC.
- 파일+함수 종류+이름+occurrence 기반 function id와 `--compare` diff.
- TypeScript AST 기반 `any`, `as any`, `this: any`, export surface.
- browser duplicate candidate의 byte/hash/symlink/동일 내용 여부.
- font asset의 정확한 byte/SHA-256과 license 문서.
- commit, upstream, 전체 dirty path, 측정 소스 dirty path, script/lock hash, toolchain.

모든 test/tests/e2e 디렉터리, `*.test.*`, `*.spec.*`, `web/clipboard_test.html`은 제품 지표에서 제외한다.
결과는 advisory이며 숫자만으로 SOLID 점수나 CI 실패를 결정하지 않는다.

## 4. 검증 결과

| 항목 | 값 |
|------|---:|
| 포함 제품 파일 | 203 |
| Total CC / 전체 Top 20 합 | 11,805 / 2,581 |
| CC>25 count / sum | 62 / 3,932 |
| CC>100 count / sum | 6 / 1,732 |
| Max CC | 453 |
| parse/fatal diagnostics | 0 |

schema v2 snapshot 자체 비교에서 aggregate delta 0, function diff 0을 확인했다. 이후 Stage 2에서 이
도구로 공식 snapshot과 manifest를 저장했다.
