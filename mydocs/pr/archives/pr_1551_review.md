# PR #1551 검토 문서 — 캡션 내 인라인 이미지 렌더링 복구

## 1. PR 메타

| 항목 | 내용 |
|------|------|
| PR | #1551 — Task #1270: 캡션 내 인라인 이미지 렌더링 복구 |
| 작성자 | @postmelee |
| base | `devel` |
| head | `postmelee:local/task1270` |
| 검토 경로 | collaborator self-merge 후보 예외 경로 |
| 문서 작성 위치 | `mydocs/pr/archives/pr_1551_review.md` |
| 문서 작성일 | 2026-06-27 |

작성 시점 참고값:

| 항목 | 값 |
|------|----|
| base SHA | `d8e792fe41f50f9e8977db9b73df3d1cf5c53c95` |
| code 검증 대상 head SHA | `fdfe568863095d42145709077bd1f2fe55f85e0a` |
| draft | false |
| mergeable | MERGEABLE |
| merge state | CLEAN |
| review decision | 없음 |
| review request | 없음 |

위 값은 문서 작성 시점 참고값이다. merge 전에는 최신 PR head, mergeable 상태, GitHub Actions 상태를 반드시 다시 확인한다.

## 2. 경로 판정

이 PR은 외부 contributor PR 이 아니라 collaborator 본인 PR이다. 따라서 `mydocs/manual/pr_review_workflow.md` 8장의 collaborator self-merge 후보 예외 경로를 적용한다.

- PR 번호가 이미 확정되어 review 문서를 PR head에 포함할 수 있다.
- merge 후 별도 문서 커밋을 만들지 않기 위해 archive 경로의 review 문서를 PR diff에 포함한다.
- 작업지시자 승인 전에는 approve review, merge, issue close를 수행하지 않는다.
- 작업지시자 요청에 따라 review request는 등록하지 않는다.

이번 PR은 규모와 성격상 별도 `pr_1551_review_impl.md`를 만들지 않고, 이 문서에 리뷰 계획과 검토 결과를 통합한다.

## 3. 관련 이슈

| 이슈 | 상태 | 비고 |
|------|------|------|
| #1270 — HWPX: 온라인 데모에서 HWPX 1페이지 상단 이미지가 표시되지 않음 | OPEN | 이 PR은 `Refs #1270`만 사용한다. merge 후에도 닫지 않는다. |
| #1585 — HWPX: 캡션 내 플로팅 이미지 렌더링 지원 | OPEN | 이번 범위 밖인 플로팅 캡션 이미지를 후속 추적한다. |

PR 본문은 #1270을 자동 close하지 않도록 `Refs #1270`으로 연결한다. #1270의 첨부 샘플에서 오른쪽 상단 `SEOUL MY SOUL` 로고는 캡션 내부 그림이지만 `textWrap="TOP_AND_BOTTOM"`인 플로팅 배치이므로 이번 PR의 해결 범위가 아니다.

## 4. 변경 범위

핵심 변경:

- `layout_caption()`이 캡션 문단을 `layout_composed_paragraph()`에 넘길 때 원본 `para`와 `bin_data_content`를 함께 전달한다.
- 본문 그림, 표, 분할 표, 도형 캡션 호출부에 `bin_data_content` 인자를 추가한다.
- 표 캡션의 picture-only TAC 문단을 위해 `paragraph_layout.rs`의 빈 줄 TAC picture 분기에 `cell_index == 65534` 캡션 센티널 예외를 추가한다.
- `tests/issue_1270_caption_inline_image.rs`를 추가해 텍스트 포함 캡션과 picture-only 캡션의 인라인 TAC picture 방출을 검증한다.

변경 파일:

| 파일 | 검토 요약 |
|------|-----------|
| `src/renderer/layout/picture_footnote.rs` | `layout_caption()`에 `bin_data_content` 추가, 캡션 문단의 실제 `para` 전달 |
| `src/renderer/layout.rs` | 본문 picture caption 호출부 인자 정합화 |
| `src/renderer/layout/shape_layout.rs` | shape caption 호출부 인자 정합화 |
| `src/renderer/layout/table_layout.rs` | table caption 호출부 인자 정합화, caption sentinel context 유지 |
| `src/renderer/layout/table_partial.rs` | partial table caption 호출부 인자 정합화 |
| `src/renderer/layout/paragraph_layout.rs` | picture-only 표 캡션 TAC 방출을 위한 caption sentinel 예외 추가 |
| `tests/issue_1270_caption_inline_image.rs` | 캡션 인라인 TAC picture 회귀 테스트 2건 추가 |
| `mydocs/**/task_m100_1270*` | 내부 task 계획, 단계 보고, 최종 보고 |

## 5. 범위 밖

이번 PR은 다음을 해결하지 않는다.

- 캡션 안 `textWrap="TOP_AND_BOTTOM"` 플로팅 그림 배치
- 캡션 내부 플로팅 그림의 앵커, 줄 매핑, wrap 배치 일반화
- 캡션 속 그림의 caption 재귀 렌더링
- #1270 issue close

이 범위 제한은 PR 본문과 task 보고서에 명시되어 있다.

## 6. 리뷰 계획

검토는 다음 순서로 진행했다.

1. PR metadata 확인
   - base/head, 작성자, label, milestone, assignee, review request, mergeable, check rollup 확인
2. 변경 코드 검토
   - `layout_caption()` 인자 스레딩 경로 확인
   - 모든 호출부의 `bin_data_content` 보유 여부 확인
   - 표 캡션 sentinel context와 실제 표 셀 중복 렌더 방지 조건 확인
3. 테스트 검토
   - 신규 테스트가 렌더 트리의 `ImageNode`와 payload 존재를 직접 확인하는지 검토
   - 텍스트 포함 캡션과 picture-only 캡션 양쪽을 고정하는지 확인
4. 문서 검토
   - task 계획/보고서가 PR 범위와 검증 결과를 일관되게 설명하는지 확인
   - #1270 유지와 #1585 후속 추적 조건 확인
5. 검증
   - PR head 기준 GitHub Actions 통과 확인
   - 로컬 targeted test와 diff hygiene 확인

## 7. 코드 리뷰 결과

### 7.1 `layout_caption()` 스레딩

기존 결함은 캡션 문단을 compose한 뒤 paragraph layout에 넘길 때 원본 `Paragraph`와 `BinDataContent`가 `None`으로 들어가 인라인 picture 방출 경로가 활성화되지 않는 점이었다.

수정 후 `layout_caption()`은 캡션 문단 루프 안의 실제 `para`와 상위 렌더 경로의 `bin_data_content`를 `layout_composed_paragraph()`에 전달한다. 이 변경은 기존 paragraph layout의 TAC picture 방출 경로를 재사용하므로, 별도 캡션 전용 이미지 렌더러를 추가하지 않는다.

판정: 적절함.

### 7.2 호출부 정합성

`layout_caption()` 호출부는 다음 경로에 존재한다.

- `src/renderer/layout.rs`
- `src/renderer/layout/picture_footnote.rs`
- `src/renderer/layout/shape_layout.rs`
- `src/renderer/layout/table_layout.rs`
- `src/renderer/layout/table_partial.rs`

각 호출부는 이미 상위 함수 scope에서 `bin_data_content`를 보유하고 있어 신규 인자 전달만으로 정합화된다. 캡션 좌표, 높이 계산, y advance 정책은 변경하지 않는다.

판정: 적절함.

### 7.3 picture-only 표 캡션 보완

초기 리뷰에서 표 캡션의 picture-only TAC 문단 경계 조건을 확인했다. 표 캡션은 실제 표 셀이 아니지만 `cell_ctx`에 `cell_index = 65534` 센티널을 사용한다. 기존 빈 줄 TAC picture 분기는 `cell_ctx.is_none()`일 때만 동작하므로, 텍스트 없이 TAC picture만 있는 표 캡션 문단은 여전히 누락될 수 있었다.

보완 커밋 `fdfe568863095d42145709077bd1f2fe55f85e0a`는 다음을 반영했다.

- `CAPTION_CELL_SENTINEL = 65534` 상수 추가
- `is_caption_cell_context()` helper 추가
- 빈 줄 TAC picture 렌더링 조건을 `cell_ctx.is_none() || is_caption_cell_context(cell_ctx.as_ref())`로 좁게 확장
- 실제 표 셀 내부 중복 렌더링 방지 조건은 유지
- picture-only 캡션 TAC 회귀 테스트 추가

판정: 이전 P2급 경계 조건은 해소됨.

### 7.4 테스트 적합성

신규 테스트는 기존 fixture `samples/hwpx/hy-001.hwpx`에서 실제 파싱된 TAC picture 문단을 찾은 뒤 첫 top-level 표의 TOP caption으로 복제한다. 이후 `build_page_render_tree(0)` 결과를 순회해 caption sentinel context의 `ImageNode`가 정확히 1개 방출되고, `bin_data_id`와 image payload가 존재하는지 확인한다.

검증 케이스:

- 텍스트가 함께 있는 캡션 TAC picture 문단
- 텍스트 없이 TAC picture만 있는 캡션 문단

판정: 이번 변경 범위에 맞는 직접 회귀 테스트다.

## 8. 검증 결과

### 8.1 GitHub Actions

작성 시점 참고값으로, code 검증 대상 head SHA `fdfe568863095d42145709077bd1f2fe55f85e0a` 기준 GitHub check rollup은 다음 상태였다.

| Check | 상태 |
|-------|------|
| CI preflight | SUCCESS |
| CodeQL preflight | SUCCESS |
| Render Diff preflight | SUCCESS |
| Build & Test | SUCCESS |
| CodeQL | SUCCESS |
| Analyze (javascript-typescript) | SUCCESS |
| Analyze (python) | SUCCESS |
| Analyze (rust) | SUCCESS |
| Canvas visual diff | SUCCESS |
| WASM Build | SKIPPED |

이 문서 커밋이 PR head에 추가되면 최신 head SHA가 바뀐다. 따라서 merge 전에는 최신 PR head 기준으로 GitHub Actions 통과 또는 문서 전용 후속 커밋 fast-pass 조건을 다시 확인해야 한다.

### 8.2 로컬 검증

리뷰 중 확인한 로컬 검증:

| 명령 | 결과 |
|------|------|
| `cargo fmt --check` | 통과 |
| `cargo test --test issue_1270_caption_inline_image` | 통과 — 2 passed |
| `cargo test --test issue_1139_inline_picture_duplicate` | 통과 — 85 passed |
| `cargo test --test issue_1352_table_cell_tac_picture_text` | 통과 — 1 passed |
| `cargo test --test issue_1459_topbottom_picture_reflow` | 통과 — 3 passed |
| `cargo test --lib` | 통과 — 1959 passed, 6 ignored |
| `cargo clippy --lib -- -D warnings` | 통과 |
| `cargo test --test issue_530` | 통과 — 1 passed |
| `cargo test --test issue_1486_hwpx_partial_tac_table` | 통과 — 6 passed |
| `git diff --check upstream/devel...HEAD` | 통과 |

문서 작성 직전 재확인:

| 명령 | 결과 |
|------|------|
| `cargo test --test issue_1270_caption_inline_image` | 통과 — 2 passed |

## 9. 잔여 리스크

| 리스크 | 판단 |
|--------|------|
| 플로팅 캡션 이미지가 여전히 보이지 않음 | 의도된 범위 밖. #1585에서 추적 |
| #1270이 merge 후에도 OPEN 상태로 남음 | 의도된 상태. 플로팅 후속 범위가 남아 있으므로 닫지 않음 |
| 이 review 문서 커밋 후 최신 head CI가 다시 필요함 | merge 전 최신 head 기준 check 또는 문서 전용 fast-pass 확인 필요 |
| 그림 자체의 picture-only caption은 별도 sentinel 예외 대상이 아님 | 이번 회귀 테스트는 표 캡션을 겨냥한다. 그림 캡션의 picture-only TAC는 후속 확장 검토 여지 있음 |

현재 PR 범위에서 merge를 막는 잔여 코드 이슈는 발견하지 못했다.

## 10. 최종 권고

권고: merge 준비 가능.

단, 실제 merge 전 최종 조건은 다음을 모두 만족해야 한다.

- 최신 PR head 기준 GitHub Actions 통과 또는 문서 전용 후속 커밋 fast-pass 조건 충족
- `mydocs/pr/archives/pr_1551_review.md`가 PR diff에 포함됨
- merge 직전 `mergeable` / `mergeStateStatus` 재확인
- 작업지시자 merge 승인
- GitHub review 또는 PR comment를 남길지 작업지시자 최종 확인
- #1270은 merge 후에도 닫지 않음
- #1585는 플로팅 캡션 이미지 후속 이슈로 유지

## 11. merge 후 확인 계획

merge가 승인되고 완료되면 다음을 확인한다.

1. PR merge commit SHA와 merged timestamp 확인
2. #1270 상태 확인
3. #1270이 open이면 그대로 유지
4. #1585 상태 확인
5. 필요 시 PR merge comment 초안을 작성하되, 게시는 작업지시자 승인 후 수행
6. 로컬/원격 PR 작업 브랜치 정리 여부는 작업지시자 승인 후 결정

## 12. 리뷰 코멘트 초안

작업지시자가 직접 게시할 수 있는 코멘트 초안:

```md
리뷰 결과, 최신 코드 검증 대상 head `fdfe5688` 기준으로 merge를 막는 추가 이슈는 발견하지 못했습니다.

확인한 내용:
- `layout_caption()`이 캡션 문단의 실제 `para`와 `bin_data_content`를 paragraph layout에 전달해 인라인 TAC picture 방출 경로를 사용할 수 있음
- 표 캡션의 picture-only TAC 문단 경계 조건은 `cell_index == 65534` 캡션 센티널 예외와 회귀 테스트로 보완됨
- 실제 표 셀 내부 TAC picture 중복 방지 조건은 유지됨
- 신규 `issue_1270_caption_inline_image` 테스트가 텍스트 포함 캡션과 picture-only 캡션을 모두 검증함

검증:
- GitHub Actions: Build & Test / CodeQL / Render Diff 계열 성공
- 로컬: `cargo test --test issue_1270_caption_inline_image` 2 passed
- 기존 관련 테스트, `cargo test --lib`, `cargo clippy --lib -- -D warnings` 통과 확인

주의:
- 이 PR은 캡션 내 `treat_as_char` 인라인 이미지 복구만 다룹니다.
- 첨부 샘플의 `textWrap="TOP_AND_BOTTOM"` 플로팅 캡션 이미지는 #1585 후속 범위입니다.
- 따라서 #1270은 이 PR merge 후에도 닫지 않는 것이 맞습니다.
```
