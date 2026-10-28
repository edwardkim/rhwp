# task_m100_4321 Stage 1 — injection_scan 문단 리스트 소유자 커버리지

- **이슈**: [#4321](https://github.com/edwardkim/rhwp/issues/4321)
- **PR**: [#4365](https://github.com/edwardkim/rhwp/pull/4365)
- **브랜치**: `fix/issue-4321-injection-scan-coverage`
- **분기 기준**: `upstream/devel` `e48fe8694`
- **상태**: 로컬 전체 검증 통과, PR 게시
- **기록일**: 2026-08-09 KST

## 1. 근거 — OWPML 의 8개 소유자

OWPML 스키마상 문단 리스트를 소유하는 자리는 8개다 — 루트 `ParaListType` 와 `subList` 를 갖는 7개
요소(field parameters, hiddenComment, caption, tc, drawText, HeaderFooterType, NoteType).

`injection_scan.rs` 의 `visit_control` 은 표·글상자·수식·각주·미주·머리말·꼬리말만 돌고 나머지를
`_ => {}` 로 흡수했다. 빠진 자리는 캡션, `Control::Picture`, `Field.memo_paragraphs` 다.

주입 신호가 이 자리에 있으면 스캔이 통째로 놓친다.

## 2. 캡션이 어느 필드에 남는가 — 변형마다 다르다

`get_caption_from_shape` 헬퍼를 만들면서 전수 확인했다.

| 변형 | HWP5 | HWPX |
|---|---|---|
| Line/Rectangle/Ellipse/Arc/Polygon/Curve | `drawing.caption` | `drawing.caption` |
| Group | `group.caption`(이동) | 직접 채움 |
| Picture | `picture.caption`(이동) | 직접 채움 |
| **Chart** | **`chart.caption`** — `chart.drawing.caption.take()` (`parser/control/shape.rs:213`) | 파싱 안 함(#4319) |
| **Ole** | **`ole.caption`** — `.take()` (`:222`) | 파싱 안 함(#4319) |

**첫 구현이 Chart/Ole 을 놓쳤다.** `.drawing()` 이 `Some` 이지만 파서가 `.take()` 로 캡션을 옮겨
`drawing.caption` 이 항상 `None` 이기 때문이다. 리뷰에서 지적받아 arm 을 추가했고, 전 변형을
개별 검증하는 테스트(`every_shape_variant_with_a_caption_is_scanned`)를 붙였다 — 수정 전 실행 시
`["Chart", "Ole"]` 로 정확히 실패한다.

## 3. 구현

- `helpers.rs::get_caption_from_shape` 신설 — 위 표를 근거로 변형별 분기.
- `visit_control` 에 표·도형 캡션 순회와 `Control::Picture` arm 추가.
- `scan_injection` 필드 루프에 `memo_paragraphs` 순회 추가.
- `Scope` 에 `Caption`(기본), `FieldMemo`(`--include-fields` 필요) 추가.
- `main.rs::injection_scan_scopes` 의 봉투 자기선언을 실제 스캔 범위와 맞췄다.

## 4. 범위 밖 (조사만)

같은 계통의 미스캔이 `field_query.rs::collect_max_field_id`(도형 캡션), `grep.rs`, `pii_scan.rs`,
`extract_data.rs`, `search_query.rs` 에도 있다. 이 작업은 `injection_scan` 만 고쳤다.

HWPX 파서가 Chart/OLE `<hp:caption>` 자체를 읽지 않는 것은 #4319 로 이미 열려 있는 별개 결함이다.

## 5. 검증 (완료)

- 단위 시험 7건 신설. `injection_scan` 28건, `field_query` 14건(미변경 확인),
  `injection_scan_contract` 14건(`every_normal_sample_is_clean` 포함, 오탐 없음) 통과.
- `cargo test --profile release-test --tests` 전체 통과.
- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` 통과.

남은 미래 조건은 GitHub Actions 와 작업지시자 승인, merge 다.
