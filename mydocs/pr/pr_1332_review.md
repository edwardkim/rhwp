# PR #1332 검토 — 빈 글머리표 줄 marker/caret 크기 보정

- PR: edwardkim/rhwp#1332 (author: postmelee)
- 연결 이슈: #1330 (빈 글머리표 줄에 입력 시 marker와 caret 크기가 커짐)
- base ← head: `devel` ← `issue-1330-bullet-marker-caret-size`
- 규모: +1168 / −21, 9 files (단, **기능 코드는 1 file**)
- mergeable: MERGEABLE / CLEAN, CI 전부 pass (Canvas visual diff 포함)

## 1. 문제

빈 글머리표 문단의 caret/marker TextRun 이 `resolved_to_text_style(styles, 0, 0)`
(기본 char shape id 0) 으로 생성되어 문단 실제 글꼴 크기와 달라 marker/caret 이 커짐.

## 2. 기능 변경 (검토 surface)

### `src/renderer/layout/paragraph_layout.rs` (+66/−21)
- 헬퍼 도입:
  - `paragraph_active_text_style(styles, para, char_offset)` — 문단의 `char_shape_id_at`
    (없으면 first char_shape) 로 실제 스타일 + id 반환.
  - `numbering_marker_text_style(styles, para, first_run)` — numbering 마커 스타일 통합.
- 빈 문단/분할 빈 줄의 caret TextRun 을 default(0) 대신 **문단 실제 char_shape** 로 생성
  (`char_shape_id: None` → 실제 id). numbering 마커 폭 계산도 동일 헬퍼로 일원화.

### `tests/issue_1330_bullet_marker_caret_size.rs` (신규)
- 빈 글머리표 줄이 입력 전/후 active char shape 를 쓰는지 회귀 1건.

## 3. 로컬 검증

- `cargo fmt --all -- --check`: 클린
- `cargo test --test issue_1330_bullet_marker_caret_size`: **1 passed**
- `cargo clippy --release` (paragraph_layout): 무경고
- `cargo test --release` 전체: **2108 passed, 0 failed** (layout 핵심부 변경 회귀 없음)
- CI 전부 pass (Canvas visual diff 통과)

## 4. 평가

- `paragraph_layout.rs` 는 모든 문단 렌더 공통 경로이나, 변경은 "빈/분할 빈 줄 TextRun
  스타일을 default→문단 실제" 로 의미가 명확하고 전체 테스트 + Canvas diff 로 회귀 없음 확인.
- #1331 과 파일 비중복(이쪽은 renderer, #1331 은 document_core)·독립.
- **이슈(통합 시 제외 필요)**: `mydocs/plans|working|report` 6개 + **`mydocs/orders/20260608.md`** 포함 — #1331 과 동일하게 통합에서 제외.

## 5. 판단

**기능 코드 Merge 권장** — `mydocs/` 제외. #1338 과 동일 방식의 squash 통합 PR 권장.
#1331 과 독립이므로 순서 무관하게 통합 가능.
