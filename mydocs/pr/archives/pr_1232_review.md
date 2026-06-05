# PR #1232 검토 — task 1209: task 1139 후속 미주·그림 흐름 보정

- **작성일**: 2026-06-02
- **PR**: #1232 (OPEN)
- **제목**: `task 1209: task 1139 후속 미주·그림 흐름 보정`
- **컨트리뷰터**: @jangster77
- **연결 이슈**: #1209
- **base/head**: `devel` ← `task_m100_1209`
- **Head SHA**: `ef0330a44e5a13422c7737e5de7630b496b4155f`
- **PR 기준 base SHA**: `f83c43b57ee4e9bf3e5ecb8be73f53a81f290430`
- **현재 local/devel**: `f6ffe9d6` (#1183/#1237 반영 후)
- **규모**: 38 files, +1695 / -156, 11 commits
- **mergeable**: false (PR base가 현재 devel보다 뒤처져 있어 rebase/merge 시뮬 필요)
- **PR 댓글**: 없음

## 1. PR 요약

PR은 #1139/#1189 후속인 #1209를 처리한다. 핵심은 시험지 계열 HWP에서 미주 간격, 미주 구분선 아래 여백, internal rewind 수식, Square/어울림 그림 주변 문단 흐름을 한컴/PDF 기준에 맞추는 것이다.

부가적으로 rhwp-studio의 여러 모달 대화상자 드래그 동작을 `dialog-drag.ts` 공통 유틸로 정리한다.

## 2. 주요 변경 범위

| 영역 | 주요 변경 |
|---|---|
| `src/renderer/typeset.rs` | compact 미주 흐름, 미주 사이/구분선 아래 margin, Square 그림 wrap anchor, TAC 수식 포함 guide line 처리 |
| `src/renderer/layout.rs` | endnote title gap 보존, `Square/어울림 + 문단 기준` 그림을 첫 narrow `LINE_SEG.vertical_pos`에 맞추는 보정 |
| `src/renderer/height_cursor.rs` | compact endnote stale forward/backtrack 처리, vpos base 이동 helper |
| `src/renderer/layout/paragraph_layout.rs` | 문단 레이아웃 관련 보정 포함 |
| `tests/issue_1139_inline_picture_duplicate.rs` | #1209 회귀 가드 다수 추가 |
| `src/diagnostics/hwp5_anchor_trace.rs`, `src/main.rs` | HWP5 개체 배치 진단 보강 |
| `rhwp-studio/src/ui/*` | 주요 modal dialog title bar drag 공통화 |
| `mydocs/tech/*` | CommonObjAttr text wrap bit 실측 정오표/보완 주석 |
| `samples/test-image*.hwp`, `samples/test-image.hwpx` | 개체 배치 fixture 추가/갱신 |

## 3. 타당한 부분

### 3.1 미주 간격 보정 방향

PR은 파일별 예외가 아니라 다음 공통 개념으로 정리하려 한다.

- `FootnoteShape`의 `betweenNotes`, separator below 값을 흐름 계산에 반영
- 저장된 HWP5 `LINE_SEG.vertical_pos`가 이미 포함한 간격과 렌더 단계에서 소비하는 간격을 분리
- compact 미주에서 stale forward jump를 버리더라도 직전 line spacing 기반 gap은 보존

이 방향은 기존 #1139/#1189/#1209 계열에서 반복된 “미주 제목이 붙거나 과하게 밀리는 문제”를 공통 로직으로 수습하려는 시도라 수용 가능성이 높다.

### 3.2 Square/어울림 그림 처리

HWP5 `Square/어울림` 그림이 문단 중간부터 본문을 감싸는 경우, 그림 상단을 문단 시작이 아니라 처음 좁아지는 `LINE_SEG.vertical_pos`에 맞추도록 보정한다.

이는 한컴의 “어울림” UI 의미와 `LINE_SEG.column_start/segment_width` 계약을 함께 따르려는 방향이다. #1237에서 정리한 `LineSeg` 계약과도 방향이 맞다.

### 3.3 모달 드래그 공통화

`dialog-drag.ts`를 추가해 각 대화상자의 title bar drag를 공통 처리한다. 시작 시 `position: fixed`, `left/top`, `margin=0`, `transform=none`으로 전환하는 방식은 기존 중앙 정렬 modal을 드래그 가능한 floating modal로 전환하는 일반적인 방식이다.

## 4. 위험 및 확인 필요 사항

### 4.1 현재 devel과 PR base 차이

PR의 base SHA는 `f83c43b5`이고, 현재 `local/devel`/`origin/devel`은 `f6ffe9d6`이다.

그 사이 최소 다음 커밋이 추가되었다.

- `885eb58e` 셀내 이미지 삽입 샘플
- `c7b301aa` #1237 `LineSeg` 저장 계약 정리
- `f6ffe9d6` #1237 merge commit

따라서 PR 자체의 `mergeable=false`는 충돌/behind 가능성을 포함한다. 반드시 현재 `local/devel` 기준 로컬 머지 시뮬레이션 또는 contributor branch rebase 후 검증이 필요하다.

### 4.2 페이지네이션/렌더링 공통 경로 리스크

`typeset.rs`, `layout.rs`, `height_cursor.rs`는 HWP/HWPX/HWP3 공통 렌더링과 페이지네이션에 직접 영향을 준다. 타깃 샘플에는 강한 테스트가 추가되었지만, 다음 계열 회귀를 별도 확인해야 한다.

- #1082 multicolumn endnote drift
- #1139 inline picture duplicate
- #1189 endnote/equation tail
- #1209 대상 `3-09월`, `3-10월`, `3-11월` 시험지
- 최근 분할표/페이지네이션 가드 (#1145, #1153 계열)

### 4.3 `endnote_between_notes_pagination_margin = extra * 3 / 4`

미주 사이 초과분의 3/4만 pagination budget에 예약하는 로직은 실측 기반 휴리스틱으로 보인다. PR 본문과 테스트가 타깃 파일을 방어하지만, 코드 주석대로 “lineSeg vpos에 일부 반영됨”이라는 전제를 더 넓은 문서에서 검증해야 한다.

수용 시에는 시각 판정과 회귀 테스트를 게이트로 두는 것이 안전하다.

### 4.4 UI drag 공통화의 이벤트 범위

현재 `dialog-drag.ts`는 mouse event 중심이다. 데스크톱 editor modal에는 충분하지만, touch/pointer 기반 환경까지 포괄하지는 않는다. 이번 PR의 blocker는 아니며 후속 개선 항목으로 둘 수 있다.

### 4.5 문서 변경 범위

PR은 공식 스펙 문서 `한글문서파일형식_5.0_revision1.3.md`에 보완 주석을 직접 추가하고 `hwp_spec_errata.md`에도 항목을 추가한다. 이 프로젝트의 관행상 공식 스펙 오류/실측 정정은 `hwp_spec_errata.md`에 기록하는 것이 맞다. 공식 스펙 문서 원문에 보완 주석을 붙이는 현재 관행도 이미 존재하므로, 이번 PR만의 blocker는 아니다.

## 5. 검증 계획

PR 수용 후보로 진행한다면 다음 순서가 필요하다.

1. 현재 `local/devel` 기준 임시 브랜치에서 PR을 로컬로 가져온다.
2. 충돌 여부와 #1237 `LineSeg` 상수화 변경과의 간섭을 확인한다.
3. 최소 검증:

```text
cargo fmt --all --check
git diff --check
cargo test --test issue_1082_endnote_multicolumn_drift -- --nocapture
cargo test --test issue_1139_inline_picture_duplicate -- --nocapture
cargo test --tests
cargo test --lib renderer::equation -- --nocapture
cargo test --test issue_1219_equation_line_hangul_advance -- --nocapture
cargo test --test issue_301 -- --nocapture
```

4. WASM/Studio 검증:

```text
wasm-pack build --target web --out-dir pkg
cd rhwp-studio && npm run build
```

5. 작업지시자 시각 판정:
   - #1209 대상 시험지 페이지
   - `samples/test-image.hwp/.hwpx`
   - rhwp-studio modal drag 동작

## 6. 권장 처리

권장안: **PR 기능 방향은 수용 후보로 본다. 단, 현재 devel 기준 로컬 머지 시뮬레이션과 회귀 검증 후 최종 판단한다.**

이 PR은 렌더링 공통 경로를 건드리지만, 문제 정의와 테스트 가드는 충분히 구체적이다. 현재 base가 뒤처져 있으므로 바로 머지하지 말고, maintainer 쪽에서 현재 `local/devel` 기준으로 체리픽/머지 시뮬레이션 후 검증하는 방식이 안전하다.

## 7. 다음 승인 요청

다음 단계로 진행하려면 작업지시자 승인이 필요하다.

권장 절차:

1. `local/pr1232-verify` 같은 임시 검증 브랜치를 현재 `local/devel`에서 생성
2. PR #1232 변경을 가져와 충돌 해결
3. #1237 `LineSeg` 상수화와 겹치는 부분이 있으면 현재 devel 계약을 유지하도록 조정
4. 로컬 테스트와 wasm/studio 빌드 진행
5. 작업지시자 시각 판정 후 merge/push 여부 결정
