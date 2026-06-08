# PR #1331 검토 — 빈 글머리표 줄 caret 위치 보정

- PR: edwardkim/rhwp#1331 (author: postmelee)
- 연결 이슈: #1329 (글머리표 Enter 직후 빈 줄 caret이 글머리표 앞에 표시됨)
- base ← head: `devel` ← `issue-1329-bullet-caret`
- 규모: +1189 / −11, 9 files (단, **기능 코드는 2 files**)
- mergeable: MERGEABLE / CLEAN, CI 전부 pass

## 1. 문제

번호/글머리표 문단에서 Enter 로 만든 빈 줄의 caret 이 marker(글머리표) **앞쪽** x 에
표시됨. body anchor TextRun(char_start=None)의 bbox.x 가 marker 왼쪽이라 발생.

## 2. 기능 변경 (검토 surface)

### `src/document_core/queries/cursor_rect.rs` (+178)
- 문단이 list(Outline/Number/Bullet head_type)인지 판정 + marker char_shape_id 취득.
- `ParaLineHit` 구조 도입: `line_x`/`first_body_x`/`marker_end_x` 수집 후
  `cursor_x(is_list_para, char_offset)` 로 caret x 결정 — 빈 list 문단 offset 0 은
  `marker_end_x`(marker 오른쪽 끝) 우선, 없으면 first_body_x, 그래도 없으면 line_x.
- marker 폭은 `estimate_text_width` + marker char_shape 로 산출(렌더 bbox 폴백).
- exact_only 경로에 `empty_list_anchor` 예외 추가, TextRun 폴백 보존.

### `tests/issue_1329_bullet_caret.rs` (신규)
- bullet/number Enter 빈 줄 caret 이 marker 뒤에 오는지 + 일반 빈 문단은 기존 동작
  유지 회귀 3건.

## 3. 로컬 검증

- `cargo fmt --all -- --check`: 클린
- `cargo test --test issue_1329_bullet_caret`: **3 passed**
- `cargo clippy --release` (cursor_rect): 무경고
- `cargo test --lib cursor`: 52 passed (회귀 없음)
- CI 전부 pass

## 4. 평가

- 기능 변경은 정확하고 잘 격리됨(폴백 경로 보존, 일반 문단 영향 없음). 테스트로 가드.
- **이슈(통합 시 제외 필요)**: PR 에 `mydocs/plans|working|report` 7개 + **`mydocs/orders/20260608.md`** 포함. orders 는 메인테이너 영역으로 외부 기여가 편집하면 안 됨(선례 #1324/#1325: 기능 코드만 반영, 작업 문서 제외).

## 5. 판단

**기능 코드 Merge 권장** — 단, `mydocs/` 전체(특히 orders)는 통합에서 **제외**.
권한 제약상 #1338 과 동일하게 기능 코드만 담은 squash 통합 PR 로 제출 권장.
