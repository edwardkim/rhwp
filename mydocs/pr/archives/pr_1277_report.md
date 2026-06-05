# PR #1277 처리 보고서

## 1. 대상

- PR: https://github.com/edwardkim/rhwp/pull/1277
- 제목: `task 1274: 교육 통합 미주 간격·overflow 보정`
- 작성자: `jangster77`
- 처리일: 2026-06-04

## 2. 처리 내용

PR head를 현재 `devel` 위에 squash 통합했다.

- PR head: `f6e8628ae725ad1d2a4dfa1a09735d993ebbaa65`
- 통합 방식: `git merge --squash origin/pr/1277`
- 충돌: 없음

메인테이너 웹 시각 판정까지 통과했다. 최종 커밋/push/PR 종료 절차를 진행한다.

## 3. 코드 변경 요약

### 3.1 compact 미주/다단 페이지네이션 보정

`src/renderer/typeset.rs`, `src/renderer/height_cursor.rs`, `src/renderer/layout/paragraph_layout.rs` 에 compact 미주 하단 배치, vpos 되감기, 제목/본문 동반 넘김, 수식-only tail frame-fit 관련 조건을 보강했다.

주요 대상:

- `3-09월_교육_통합_2022.hwp`
- `3-09월_교육_통합_2024-미주사이20.hwp`
- `3-10월_교육_통합_2022.hwp`
- `3-11월_실전_통합_2022.hwp`

### 3.2 텍스트 없는 non-TAC 그림/도형 host 처리

`src/renderer/layout.rs` 에서 텍스트 없는 non-TAC 그림/도형 host가 실제 `Shape` item으로 렌더되는 경우 phantom text line을 만들지 않도록 했다.

또한 `InFrontOfText + vert_rel_to=Para` host는 한컴처럼 line advance를 예약한다.

### 3.3 TAC 수식 baseline 정렬

텍스트 없는 TAC 수식 문단도 baseline 기준으로 배치하도록 보정했다. 큰 루트/분수 수식이 다음 줄을 덮는 회귀를 방지한다.

### 3.4 render tree export 및 sweep 도구 추가

`src/main.rs` 에 `export-render-tree` CLI를 추가했다.

`scripts/task1274_visual_sweep.py` 는 HWP/PDF/SVG/PNG/render tree를 생성하고 다음 후보를 수집한다.

- frame overflow
- red marker drift
- line band drift
- equation/text overlap
- question title overlap
- line order overlap

### 3.5 회귀 테스트 추가

`tests/issue_1139_inline_picture_duplicate.rs` 에 #1274 관련 회귀 테스트를 추가했다.

대표 검증:

- 2022-09 page 18 문26 수식 문단 높이 예약
- 2022-10 page 11 문20 수식 tail frame-fit
- 2022-10 page 16 문30 제목/첫 줄 유지
- 2022-11 page 11 빈 float picture host phantom overflow 제거
- 2022-11 page 11/12 partial endnote tail 분기 유지
- issue_241 HWPX 도장 host flow line height 유지

## 4. 검증

| command | result |
|---|---|
| `cargo fmt --check` | pass |
| `cargo test --test issue_241 -- --nocapture` | pass |
| `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture` | pass |
| `docker compose --env-file .env.docker run --rm wasm` | pass |

WASM 산출물:

- `rhwp-studio/public/rhwp_bg.wasm`
- size: 5.3M
- sha256: `d82c6a96fe80fc0860e04059179b1f05c666815e11d701d576a21d27462fb105`

GitHub PR head CI:

| check | result |
|---|---|
| Build & Test | pass |
| Canvas visual diff | pass |
| CodeQL | pass |
| Analyze Rust | pass |
| Analyze JS/TS | pass |
| Analyze Python | pass |
| WASM Build | skipping |

## 5. 메인테이너 시각 판정

메인테이너가 rhwp-studio 웹 캔버스에서 직접 시각 판정을 수행했고 통과했다.

확인 대상:

- `3-09월_교육_통합_2022.hwp` page 18 문26
- `3-10월_교육_통합_2022.hwp` page 11 문20
- `3-11월_실전_통합_2022.hwp` page 11~12
- `samples/hwpx/issue_241.hwpx`

판정:

```text
2026-06-04 통과
```

## 6. 추가 확인: 21_언어 머리말 표 셀 배치

메인테이너 시각 판정 중 `samples/21_언어_기출_편집가능본.hwp` page 1
머리말 영역의 수험번호 표 배치 문제가 발견됐다.

확인 결과:

- PR #1277 적용 후 SVG: `output/poc/pr1277-regression-lang-header-current/21_언어_기출_편집가능본_001.svg`
- PR 적용 전 `origin/devel` 기준 SVG: `/tmp/rhwp-pr1277-base-out/svg/21_언어_기출_편집가능본_001.svg`
- 두 SVG의 SHA-256: `2f1567069cd85e5ce1005a4bd1ce783dfd7b921e80ca8c62dc62b055e15e1d6e`

따라서 native/SVG 렌더 경로 기준으로는 #1277에서 새로 발생한 회귀가 아니라,
PR 이전 `devel`에도 존재하던 배치 문제로 판단한다.

구조 분석:

- 해당 부모 셀은 오른쪽 정렬 문단 안에 TAC 표 2개가 한 줄에 배치되는 구조다.
- `성명` 표와 `수험번호` 표가 같은 부모 셀 문단의 treat-as-char 컨트롤로 배치된다.
- `수험번호` 셀 자체의 inner 폭은 약 92.8px, textRun 폭은 약 77.0px이다.
- 오른쪽 정렬이라면 시작 x가 약 813.5px 근처여야 하나, 현재 x는 약 805.6px이다.
- 원인은 TAC 표 2개와 중간 공백을 부모 문단의 inline sequence 폭으로 합산하는 경로가
  정밀하지 않은 기존 결함으로 추정된다.

이 항목은 #1277 수용 판단과 분리해 별도 이슈 #1285 로 등록했다.

## 7. 남은 절차

1. `mydocs/pr/pr_1277_review.md`, `mydocs/pr/pr_1277_report.md` 포함 커밋
2. `origin/devel` push
3. PR #1277에 메인테이너 코멘트 등록
4. PR #1277 close
5. 연결 이슈 #1274 close 여부 확인 및 처리

## 8. 판정

자동 검증, wasm 빌드, 메인테이너 웹 시각 판정을 모두 통과했다.

PR #1277은 수용한다.
