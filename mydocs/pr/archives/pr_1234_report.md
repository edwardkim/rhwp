# PR #1234 처리 보고서

- PR: `#1234`
- 제목: `폰트 충실도: 한컴 돋움 폴백을 Noto Sans KR ExtraLight로 (closes #1224)`
- 기여자: `planet6897`
- 관련 이슈: `#1224`
- 처리일: 2026-06-02

## 1. 처리 결론

PR #1234는 현재 `local/devel` 기준 검증 브랜치에서 병합 시뮬레이션과 자동 검증을 통과했다.

수용 후보 범위:

```text
1. 한컴 돋움/고딕/굴림 계열 폰트 미설치 환경에서 Noto Sans KR ExtraLight 우선 폴백
2. SVG 네이티브 렌더링 경로의 opensource 폰트 탐색 보강
3. rhwp-studio 웹폰트 로더의 ExtraLight 매핑 추가
4. Noto Sans KR ExtraLight TTF/WOFF2 및 OFL 라이선스 문서 추가
5. 관련 golden SVG 갱신
6. #1224 폰트 충실도 측정/전략/작업 문서 추가
```

현재 단계의 결론은 **자동 검증 및 메인테이너 시각 판정 통과**이다.

## 2. 로컬 반영

```text
base: local/devel @ 914ee139
PR head: pr/1234 @ dbf8aa87
verify branch: local/pr1234-verify
merge commit: 9ec04efd Merge PR 1234 verification
```

충돌:

```text
없음
```

추가 보완:

```text
mydocs/tech/font_fidelity_measurement_1224.md
mydocs/working/task_m100_1224_stage2.md
```

Stage 1/2 문서의 `Noto Sans KR Light(wght 300)` 기록이 최종 선택으로 오해되지 않도록,
Stage 3에서 `Noto Sans KR ExtraLight(wght 200)`로 재확정되었다는 주석을 추가했다.

## 3. 검증 결과

통과:

```text
PR 범위 whitespace check
cargo fmt --all --check
cargo test --lib renderer::tests::test_generic_fallback
cargo test --test svg_snapshot
cargo test --lib
cargo test --tests
docker compose --env-file .env.docker run --rm wasm
cd rhwp-studio && npm run build
```

주요 결과:

```text
renderer::tests::test_generic_fallback: 1 passed
svg_snapshot: 8 passed
cargo test --lib: 1523 passed; 0 failed; 6 ignored
cargo test --tests: 전체 통과
WASM build: success
rhwp-studio build: success
```

비고:

```text
rhwp-studio build는 기존과 동일한 CanvasKit browser externalize 안내와 chunk size warning만 출력했다.
추가로 포함하기로 한 `mydocs/manual/memory/*` 문서의 trailing whitespace도 정리했다.
```

## 4. 확인된 리스크

이번 PR은 레이아웃 계산을 직접 바꾸지 않고 폰트 폴백과 표시용 glyph weight를 조정한다.
자동 테스트 기준으로는 snapshot과 전체 테스트가 통과했고, 메인테이너가 `samples/3-09월_교육_통합_2023.hwp` 6페이지 SVG 산출물로 본문 굵기 차이를 시각 판정했다.

네이티브 CLI/rsvg 경로는 fontconfig 환경에 `Noto Sans KR ExtraLight`가 잡혀야 실제 렌더링 결과에 반영된다. 웹 경로는 `@font-face` 등록으로 번들 폰트를 사용한다.

## 5. 메인테이너 시각 판정 대상

권장 판정 대상:

```text
1. samples/3-09월_교육_통합_2023.hwp 6쪽 문26 주변 본문
2. pdf/3-09월_교육_통합_2023.pdf 동일 구간
3. rhwp-studio 웹 캔버스에서 Haansoft Dotum/돋움 계열 본문 굵기
4. 기존 golden SVG 대상 문서의 과도한 굵기/폭 회귀 여부
```

판정표:

| 항목 | 기대 동작 | 판정 | 비고 |
|---|---|---|---|
| 한컴 돋움/고딕 계열 본문 | 과도하게 굵지 않게 출력 | 통과 |  |
| 문단 폭/줄바꿈 | 기존과 동일한 흐름 유지 | 통과 | `textLength`/advance 변경 없음 |
| 웹 캔버스 폰트 로딩 | ExtraLight 웹폰트 적용 | 통과 | WASM/Studio 빌드 후 판정 |
| snapshot 대상 문서 | 시각적 회귀 없음 | 통과 |  |

메인테이너 시각 판정:

```text
2026-06-02 통과
```

시각 판정 산출물:

```text
output/poc/pr1234-visual-font-fidelity/3-09월_교육_통합_2023_006.svg
output/poc/pr1234-visual-font-fidelity/font-style/3-09월_교육_통합_2023_006.svg
output/poc/pr1234-visual-font-fidelity/embed-full/3-09월_교육_통합_2023_006.svg
```

## 6. 다음 절차

```text
1. 메인테이너 시각 판정
2. 완료 보고서 승인
3. 검증 브랜치 변경을 local/devel에 반영
4. devel push 및 PR #1234 종료 처리
```
