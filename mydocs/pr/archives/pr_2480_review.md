# PR #2480 검토 기록

| 항목 | 내용 |
|---|---|
| 원 PR | [#2480](https://github.com/edwardkim/rhwp/pull/2480) |
| 작성자 / base | kevin9327 / `devel` |
| 검토자 | @jangster77 (검토 전 지정) |
| 규모 / 검토 스냅샷 | 2026-07-20 GitHub 조회: +36/-4, 1 file, `maintainerCanModify=true`, `mergeStateStatus=BEHIND` (동적 참고값) |
| 범위 | 문단 병합 undo 시 새 문단으로 이관되어야 할 `field_ranges` 보존 |
| 판단 | 누적 통합 PR에 수용 |

## 변경 범위와 통합
- PR 본문은 문단 split/merge undo가 field range를 새 문단으로 옮기지 않아 누름틀 필드가 소실되는 문제를 다룬다.
- PR 코멘트는 검토 시점에 없었다.
- 기여자 원 변경 `b90614163`과 회귀 `e43b74182`를 적용했다.

## 렌더 영향 판정
- 편집 모델의 undo 의미 보정이며 renderer·typeset 변경이 아니다. focused undo 회귀가 직접 근거다.

## 검증
- 최초 누적 PR CI에서 기존 `issue_258_clickhere_form_mode`의 복사/붙여넣기 회귀 세 건이 실패했다. `Field`를 일반 이동형 문자 컨트롤로 분류해 보이는 문자 offset에 포함한 것이 원인이었다.
- 보정 뒤 `FieldRange`가 새 문단으로 이관될 때만 연결된 `Field` control을 함께 옮기도록 분리했다.
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_258_clickhere_form_mode` 13건과 [#2417](https://github.com/edwardkim/rhwp/issues/2417) 병합 undo 회귀, visible-offset 분할 회귀를 통과했다.
- headless undo-contracts E2E의 실제 입력·Ctrl+Z 5개 계약과 text-flow E2E의 2페이지 생성·Backspace 문단 병합을 통과했다. 최종 수용은 보정이 포함된 최신 PR head CI 전체 성공을 조건으로 한다.

## 리스크와 권고
- split 경로의 field range 소유권만 보정하며, 다른 undo 단위와 섞지 않았다.
- **권고**: 누적 통합 PR에 수용. 최신 통합 PR head의 CI가 성공한 뒤에만 merge한다.
