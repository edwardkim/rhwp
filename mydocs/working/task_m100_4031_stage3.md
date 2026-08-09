# Task M100 #4031 Stage 3 완료보고서 — production wasm 실브라우저 검증

## 1. 방법

- production wasm(`wasm-pack build --target web --out-dir pkg`) + 이 worktree 전용
  Vite(포트 7710) + headless Chrome(puppeteer)
- 신규 e2e `rhwp-studio/e2e/cell-enter-pagination-issue4031.test.mjs`
  (`npm run e2e:issue-4031-cell-enter`, `VITE_URL`·`CHROME_PATH` 환경 변수)
- HWP/HWPX 각각: 115쪽 로드 → 거대 셀(문단 5, offset 130)로 이동 → 실키 `111` 입력으로
  pending deferred pagination 생성 → 실키 Enter → 계약 단언 → 사후 flush oracle →
  실키 `1` + ArrowDown으로 barrier 대조군 단언

## 2. 결과

| 형식 | Enter dispatch | `wasm.flushDeferredPagination` | split | 사후 flush oracle | ArrowDown barrier |
|---|---:|---:|---:|---|---:|
| HWP | 1015ms | 0회 | 1회 | pages 불변(fallback no-op) | flush 1회 |
| HWPX | 1813ms | 0회 | 1회 | pages 불변(fallback no-op) | flush 1회 |

추가 단언 전부 통과: pending 해소, 셀 문단 수 +1, 분할점 offset 133(삽입 3자 반영),
caret은 새 문단 시작(cellParaIndex+1, offset 0).

acceptance criteria 대응:

- admitted pending cell Enter의 pre-navigation full flush 0회 ✓ (총 wasm flush 호출 0회)
- structural full pagination 1회(split 소유) ✓
- pending Enter가 cold Enter와 같은 단일 pagination 구조로 수렴(Stage 1의 2×→1×) ✓
- 문자열·문단 split·caret·쪽수 정합 ✓, 사후 flush oracle pages 불변 ✓
- 비Enter boundary key(ArrowDown)의 기존 barrier 유지 ✓
- IME 예약 Enter 경로 flush 보존은 unit 계약(⑥)으로 고정

## 3. 최초 구현에서 잡힌 결함

첫 e2e 실행이 admitted Enter에서 wasm flush 1회를 검출했다: keydown의 명시적 완료
호출이 `executeOperation` 내부 refresh(`afterEdit`의 `before-full-edit` flush)보다 늦어
pending이 남아 있었다. `SplitParagraphInCellCommand`가
`IMMEDIATE_TEXT_MUTATION_EFFECTS`를 선언하는 방식으로 교체해 기존 effects 경로가
refresh 이전에 pending을 해소하도록 했다(stage2 §1.3). 재실행에서 0회 확인.

## 4. 게이트 요약

| 게이트 | 결과 |
|---|---|
| e2e HWP/HWPX 계약 전부 | 통과 |
| Studio `npm test` | 769 passed |
| `npx tsc --noEmit` / `npm run build` | 통과 |
| e2e MANIFEST 검사 (`check_e2e_manifest.py`) | 이상 없음 |
| `cargo fmt`·`clippy`(probe) | 통과 |

## 5. 남은 후속

- cold Enter 자체(~1s wasm)는 fragment 전량 drain이 지배 항 — 보드 Task #8 본체
  "편집 행 fragment부터 재개" 증분 재조판으로 해소하며 PR #4122 병합 후 착수
  (stage1 §6-2).
- 브라우저 direct/IME × 3회 반복 중앙값 리포트는 PR 리뷰 시점에 위 e2e를 반복 실행해
  첨부할 수 있다(계약 단언은 이미 자동화).
