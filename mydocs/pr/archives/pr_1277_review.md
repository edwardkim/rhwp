# PR #1277 검토: 교육 통합 미주 간격·overflow 보정

## 1. PR 개요

- PR: https://github.com/edwardkim/rhwp/pull/1277
- 제목: `task 1274: 교육 통합 미주 간격·overflow 보정`
- 작성자: `jangster77`
- base: `devel`
- head: `task_m100_1274` (`f6e8628ae725ad1d2a4dfa1a09735d993ebbaa65`)
- 상태: open, ready for review
- mergeable: true
- 변경 규모: 26 commits, 36 files, +3715 / -102

## 2. CI 상태

GitHub PR head 기준 checks:

| check | status |
|---|---|
| Build & Test | pass |
| Canvas visual diff | pass |
| CodeQL | pass |
| Analyze Rust | pass |
| Analyze JS/TS | pass |
| Analyze Python | pass |
| WASM Build | skipping |

## 3. 변경 파일 범위

코드:

| file | 내용 |
|---|---|
| `src/renderer/layout.rs` | 텍스트 없는 non-TAC 그림/도형 host 처리, InFrontOfText+Para host line advance 예약, compact 미주 bottom bleed overflow 판정 보정 |
| `src/renderer/layout/paragraph_layout.rs` | 빈 spacer/미주 가상 문단 overflow 오탐 제거, 텍스트 없는 TAC 수식 baseline 정렬, body column 영역 overflow 로그 제한 |
| `src/renderer/typeset.rs` | compact 미주 단 하단 fit, large between-notes 예약, 내부 vpos rewind, 제목/첫 줄 동반 넘김, TAC 그림 rewind 공통 보정 |
| `src/renderer/height_cursor.rs` | compact 미주 tail vpos backtrack 조건 다수 보정, equation-only tail frame-fit, debug 로그 가드 정리 |
| `src/main.rs` | `export-render-tree` CLI 추가 |
| `scripts/task1274_visual_sweep.py` | HWP/PDF/SVG/PNG/render tree 기반 시각 sweep helper 추가 |
| `tests/issue_1139_inline_picture_duplicate.rs` | #1274 관련 회귀 테스트 추가 |
| `tests/golden_svg/issue-677/bokhakwonseo-page1.svg` | 골든 SVG 갱신 |

문서:

- `mydocs/plans/task_m100_1274*.md`
- `mydocs/working/task_m100_1274_stage*.md`
- `mydocs/orders/20260603.md`

## 4. 코드 검토

### 4.1 텍스트 없는 float shape host 처리

`layout.rs` 에서 텍스트 없는 non-TAC 그림/도형 host 문단이 실제 `Shape` item으로 이미 렌더되는 경우, 보이지 않는 빈 text line을 다시 생성하지 않도록 했다.

의미:

- 저장 vpos 기준으로 페이지 밖에 phantom line이 생기며 overflow 후보가 되는 문제를 줄인다.
- `InFrontOfText + vert_rel_to=Para` host는 한컴처럼 line advance만 예약한다.
- `BehindText` 배경 성격 개체는 기존처럼 host 줄 진행량을 예약하지 않는다.

검토 결과, #241 도장 host와 2022-11 실전 문서의 그림 host 문제를 직접 겨냥한다.

### 4.2 compact 미주 하단 bleed/rewind 보정

`typeset.rs`, `height_cursor.rs`, `paragraph_layout.rs` 에 compact 미주 하단의 작은 bleed 허용, vpos 되감기, 제목/본문 동반 넘김 조건이 추가됐다.

장점:

- 2단 문제집에서 문항 제목만 하단에 남거나, 수식 tail이 다음 텍스트와 겹치는 문제를 줄인다.
- PR 본문에 제시된 6종 문서의 PDF/render tree 페이지 수 1:1 검증이 포함되어 있다.

리스크:

- 조건이 compact 미주, 다단, page-path/lazy-path, between-notes, vpos rewind 등 여러 휴리스틱의 조합이다.
- 직접 타깃 문서에는 효과가 크지만, 다른 미주 스타일 문서에 대한 회귀 가능성은 남는다.
- 다만 회귀 테스트와 sweep 스크립트가 함께 들어와 후속 추적 가능성은 좋아졌다.

### 4.3 TAC 수식 baseline 정렬

`paragraph_layout.rs` 에서 공백 run 안에 TAC 수식만 있는 줄도 baseline 기준으로 배치하도록 바뀌었다.

기존에는 수식 bbox를 y에 직접 붙이면서 큰 루트/분수 수식이 아래 텍스트를 덮는 문제가 있었고, 이번 PR은 수식-only 줄도 `baseline - layout_box.baseline` 기준으로 정렬한다.

### 4.4 render tree export CLI

`src/main.rs` 에 `export-render-tree` 명령이 추가됐다.

용도:

- 페이지별 render tree bbox JSON 산출
- `task1274_visual_sweep.py`에서 수식/텍스트 겹침, line order overlap, frame overflow 후보를 분석

CLI 출력 로그는 명령형 도구의 정상 출력 범위다.

### 4.5 임시 디버그 출력 점검

PR diff에서 `dbg!`, `TODO`, `FIXME`, `eprintln!`, `RHWP_VPOS_DEBUG` 를 검색했다.

결과:

- 남은 `eprintln!`은 기존 overflow 로그, 새 CLI 오류 출력, `RHWP_VPOS_DEBUG` 환경변수 가드 내부 로그다.
- 문서에 `RHWP_VPOS_DEBUG=1 ...` 재현 명령 기록이 포함되어 있으나 코드 임시 출력으로 보이지 않는다.

## 5. 회귀 테스트/시각 검증

PR 본문 기준 검증:

- `cargo fmt --all -- --check`
- `cargo build --verbose`
- `cargo check --target wasm32-unknown-unknown --lib`
- `cargo test --features native-skia skia --lib --verbose`
- `cargo test --verbose`
- `cargo clippy -- -D warnings`
- `wasm-pack build --target web --release --out-dir pkg` fresh worktree 검증
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
- `python3 scripts/task1274_visual_sweep.py --target all`

PR head CI 기준:

- Build & Test pass
- Canvas visual diff pass
- CodeQL pass
- CodeQL language analyses pass

주의:

- GitHub CI의 `WASM Build`는 skipping 이므로, 수용 전/후 로컬 wasm 빌드와 rhwp-studio 시각 판정이 별도 게이트로 필요하다.

## 6. 권장 처리

수용 가능.

권장 절차:

1. 메인테이너 승인 후 로컬 `devel`에 통합
2. 충돌 여부 확인
3. 포커스 테스트 실행
   - `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
   - `cargo test --test issue_241 -- --nocapture`
   - `cargo fmt --check`
4. wasm 빌드
5. 메인테이너 시각 판정
   - PR 본문 6종 중 최소 대표 문서
   - 특히 2022-09 p18 문26, 2022-10 p11 문20, 2022-11 p11~12, `samples/hwpx/issue_241.hwpx`
6. 통과 시 `devel` push 및 PR #1277 종료 처리

병합 방식:

- PR이 26 commits와 merge commits를 포함하므로, 히스토리 정리 관점에서는 로컬 squash/cherry-pick 통합이 더 깔끔하다.
- 다만 기여자 작업 추적을 위해 PR에는 완료 코멘트를 남기고, `devel` 반영 커밋에서 PR 번호를 명시하는 방식이 적절하다.

## 7. PR 코멘트 초안

```text
검토했습니다. PR #1277은 compact 미주, 텍스트 없는 TAC 수식/그림 host, 문제집 다단 페이지네이션에서 반복적으로 발생하던 overflow/겹침/분기 차이를 넓게 보정하고 있습니다.

CI는 현재 통과 상태이며, 추가된 render-tree 기반 sweep과 포커스 테스트도 문제 영역을 잘 고정하고 있습니다.

변경 범위가 크고 GitHub CI의 WASM Build는 skipping 상태이므로, 메인테이너 쪽에서 로컬 통합 후 wasm 빌드와 대표 샘플 시각 판정을 추가 게이트로 진행하겠습니다.

감사합니다.
```
