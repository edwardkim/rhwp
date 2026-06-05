# PR #1235 처리 보고서

- PR: `#1235`
- 제목: `수식 큰 연산자(Σ/∏/∫) 피연산자 간격 추가 (closes #1233)`
- 기여자: `planet6897`
- 관련 이슈: `#1233`
- 처리일: 2026-06-02

## 1. 처리 결론

PR #1235는 현재 `local/devel` 기준 검증 브랜치에서 병합 시뮬레이션과 자동 검증을 통과했다.

수용 후보 범위:

```text
1. 수식 큰 연산자 계열(Σ/∏/∫)의 trailing pad 추가
2. SVG 렌더러에서 limits 연산자 중앙정렬 기준 보정
3. Canvas 렌더러에서 SVG와 동일한 중앙정렬 기준 보정
4. 큰 연산자 width 회귀 테스트 추가
5. #1233 작업 계획/단계/보고서 문서 추가
```

현재 단계의 결론은 **자동 검증 및 메인테이너 시각 판정 통과**이다.

## 2. 로컬 반영

```text
base: local/devel @ 5752a1f7
PR head: pr/1235 @ cec32c77
verify branch: local/pr1235-verify
merge commit: 1031b209 Merge PR 1235 verification
```

충돌:

```text
없음
```

추가 보완:

```text
mydocs/working/task_m100_1233_stage3.md
mydocs/report/task_m100_1233_report.md
```

Stage 3 문서와 완료 보고서 일부에 남아 있던 `pad 0.1 확정` 표현을 최종 구현값인 `BIG_OP_TRAIL_PAD = 0.45` 확정 흐름으로 보완했다.
실제 검토 결과는 `0.1 → 0.25 → 0.45` 비교 후, 첫 `∑(...)`, 둘째 `∑b_n`, 적분 `∫` 모두 PDF 기준에 가장 근접한 `0.45`를 채택한 것이다.

## 3. 검증 결과

통과:

```text
git diff --check HEAD
cargo fmt --all --check
cargo test --lib equation
cargo test --test issue_1219_equation_line_hangul_advance
cargo test --lib
cargo test --tests
docker compose --env-file .env.docker run --rm wasm
cd rhwp-studio && npm run build
```

주요 결과:

```text
cargo test --lib equation: 139 passed
issue_1219_equation_line_hangul_advance: 1 passed
cargo test --lib: 1524 passed; 0 failed; 6 ignored
cargo test --tests: 전체 통과
WASM build: success
rhwp-studio build: success
```

비고:

```text
rhwp-studio build는 기존과 동일한 CanvasKit browser externalize 안내와 chunk size warning만 출력했다.
```

## 4. 확인된 리스크

이번 PR은 수식 box의 자연폭을 조정한다.
TAC 수식의 본문 advance는 control width에 맞춰 압축되는 경로가 있어 일반 문단 페이지네이션 영향은 제한적일 가능성이 높다.

다만 사용자가 보는 문제는 실제 수식의 시각 간격이므로, 자동 테스트만으로 종료하지 않고 `samples/3-09월_교육_통합_2023.hwp` 6페이지 문26 주변 수식에 대해 메인테이너 시각 판정을 거쳐야 한다.

## 5. 메인테이너 시각 판정 대상

권장 판정 대상:

```text
1. samples/3-09월_교육_통합_2023.hwp 6쪽 문26 주변 첫 ∑(...)
2. 같은 페이지 둘째 ∑b_n
3. 문25 적분 ∫ 주변
4. rhwp-studio 웹 캔버스에서 SVG와 동일한 간격으로 보이는지
```

판정표:

| 항목 | 기대 동작 | 판정 | 비고 |
|---|---|---|---|
| 첫 `∑(...)` | 큰 연산자와 피연산자 사이 간격 확보 | 통과 |  |
| 둘째 `∑b_n` | `∑`와 `b_n`이 붙지 않음 | 통과 |  |
| 적분 `∫` | `∫`와 피적분 함수가 붙지 않음 | 통과 |  |
| SVG/canvas 정합 | 웹 캔버스도 SVG와 유사 | 통과 | WASM 빌드 후 판정 |

메인테이너 시각 판정:

```text
2026-06-02 통과
```

시각 판정 산출물:

```text
output/poc/pr1235-eq-bigop-spacing/3-09월_교육_통합_2023_006.svg
output/poc/pr1235-eq-bigop-spacing-debug/3-09월_교육_통합_2023_006.svg
```

## 6. 다음 절차

```text
1. 완료 보고서 승인
2. 검증 브랜치 변경을 local/devel에 반영
3. devel push 및 PR #1235 / Issue #1233 종료 처리
```
