# Task 1284 Stage 12: PR 준비 CI 기대값 보정

## 배경

- stage11 커밋 후 `upstream/devel` 기준 rebase를 완료했다.
- PR 준비용 전체 CI 흐름에서 `cargo test --verbose`를 수행하던 중 단위 테스트 1건이 실패했다.

## 실패 항목

```text
renderer::height_cursor::tests::compact_endnote_page_path_title_bottom_backtrack_allows_safe_title
got=922.6666666666666, expected=926.6666666666666
```

## 원인 판단

- stage11에서 page-path compact 미주 제목 하단 보정에 4px title tail 여유를 추가했다.
- 해당 테스트는 기존 저장 `vpos` 위치(`end_y`)를 그대로 기대하고 있어, 새 정책의 `end_y - 4px` 결과와 어긋났다.
- `RHWP_VPOS_DEBUG=1` 재현 결과:
  - `title_bottom=true`
  - `page_tail=false`
  - `result=922.67`
- 즉, 구현 회귀가 아니라 stage11 의도값에 맞춘 테스트 기대값 갱신 누락이다.

## 수정 계획

- 해당 테스트 주석과 기대값을 4px title tail pad 기준으로 정정한다.
- 이후 `cargo test --lib renderer::height_cursor::tests::compact_endnote_page_path_title_bottom_backtrack_allows_safe_title -- --nocapture`로 단일 실패를 재검증한다.
- 전체 PR 준비 CI는 처음부터 다시 확인한다.
