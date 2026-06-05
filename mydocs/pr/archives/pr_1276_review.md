# PR #1276 검토: HWPX 글뒤로 표 분할로 인한 바탕쪽 홀짝 밀림 수정

## 1. PR 개요

- PR: https://github.com/edwardkim/rhwp/pull/1276
- 제목: `Task #1271: HWPX 글뒤로 표 분할로 인한 바탕쪽 홀짝 밀림 수정`
- 작성자: `postmelee`
- base: `devel`
- head: `fix/issue-1271-hwpx-master-page` (`e2b84d796cbaf5caf3cf80183c85b0e4a8ad9f78`)
- 상태: open, ready for review

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

## 3. 변경 파일

| file | 내용 |
|---|---|
| `src/renderer/typeset.rs` | HWPX paper-anchored `BehindText`/`InFrontOfText` 표를 본문 표 분할 대상으로 보지 않도록 보정 |
| `src/document_core/queries/rendering.rs` | 구역 간 page number carry 이후 최종 쪽번호 기준으로 바탕쪽 odd/even 선택 |
| `tests/issue_1271_hwpx_behind_text_table.rs` | `[2027] 온새미로 1 본교재.hwpx` 앞쪽 페이지 밀림 회귀 가드 |
| `mydocs/...task_m100_1271...` | 계획/작업/완료 보고 문서 |
| `mydocs/orders/20260603.md` | #1271 작업 행 추가 |

## 4. 코드 검토

### 4.1 글뒤로 표 분할 억제

`typeset.rs` 변경은 종이 기준 절대좌표 표가 페이지 배경/전경 역할을 하는 경우, 다행 표라는 이유만으로 `PartialTable` 로 분할하지 않도록 한다. 실제 단축 분기 조건은 `InFrontOfText | BehindText` 로 제한되어 있어 `Through` 표까지 넓게 영향을 주지는 않는다.

확인 포인트:

- `treat_as_char=false`
- `vert_rel_to=Paper`
- `horz_rel_to=Paper`
- `text_wrap=BehindText` 또는 `InFrontOfText`
- `row_count > 1` 이고 본문 높이보다 큰 경우에도 배경/전경 표로 유지

이 조건은 #1271 재현 문서의 cover/background 라벨 표와 맞는다.

### 4.2 바탕쪽 odd/even 선택 시점

`rendering.rs` 변경은 section pagination 직후 바로 바탕쪽을 고르는 기존 흐름을 분리해, 구역 간 쪽번호 carry 보정 이후 `assign_master_pages_for_section` 에서 최종 `page.page_number` 기준으로 odd/even 바탕쪽을 선택하게 한다.

검토 결과:

- 이전 구역의 page number carry 때문에 다음 구역 첫 페이지의 홀짝이 바뀌는 케이스를 직접 겨냥한다.
- 마지막 바탕쪽 extension/overlap 처리도 기존 의미를 보존하도록 helper 내부로 이동했다.
- `master_page_selection_uses_final_carried_page_number_parity` 단위 테스트가 추가되어 핵심 케이스를 고정한다.

## 5. 병합 충돌

현재 `devel` 기준으로 PR head를 병합하면 코드 충돌은 없고, 다음 문서만 충돌한다.

- `mydocs/orders/20260603.md`

충돌 원인:

- 현재 `devel`: #1205, #1251 행이 추가되어 있음
- PR head: #1271 행이 추가되어 있음

해결 방향:

- #1205, #1251 행을 유지
- PR의 #1271 행을 함께 추가

## 6. 권장 처리

수용 권장.

단, 현재 `devel`이 PR base보다 앞서 있으므로 GitHub auto-merge 대신 로컬에서 다음 순서로 처리하는 것이 안전하다.

1. 현재 `devel`에서 PR head를 병합 또는 cherry-pick
2. `mydocs/orders/20260603.md` 충돌을 수동 해결
3. 포커스 테스트 실행
   - `cargo test --test issue_1271_hwpx_behind_text_table -- --nocapture`
   - `cargo test master_page_selection_uses_final_carried_page_number_parity -- --nocapture`
4. 필요 시 SVG/wasm 시각 판정
5. 결과 보고서 작성 후 커밋

## 7. PR 코멘트 초안

```text
검토했습니다. #1271 재현 조건인 HWPX paper-anchored 글뒤로/글앞으로 표가 본문 표 분할 대상으로 들어가면서 앞쪽 페이지와 바탕쪽 홀짝이 함께 밀리는 문제를 잘 겨냥하고 있습니다.

현재 PR head의 CI는 통과했습니다. 다만 최신 devel과는 `mydocs/orders/20260603.md` 문서 충돌이 있어, 메인테이너 쪽에서 #1205/#1251/#1271 행을 모두 보존하는 방식으로 통합하겠습니다.

감사합니다.
```
