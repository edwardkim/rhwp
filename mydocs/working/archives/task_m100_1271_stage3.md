# Stage 3 보고 — Task M100-1271

## 범위

- 이슈: [#1271](https://github.com/edwardkim/rhwp/issues/1271)
- 단계: 최종 쪽번호 기준 masterpage 선택 보정
- 수정 파일: `src/document_core/queries/rendering.rs`

## 변경

기존 바탕쪽 선택 블록을 `assign_master_pages_for_section` helper 로 분리했다.

호출 시점을 다음처럼 변경했다.

```text
기존:
  section-local page_number 기준 바탕쪽 선택
  -> 구역 간 page_number carry 보정

변경:
  구역 간 page_number carry 보정
  -> 최종 page_number 기준 바탕쪽 선택
```

helper 는 재호출 가능하도록 각 페이지의 `active_master_page` 와 `extra_master_pages` 를 먼저 초기화한다.
기존 선택 규칙은 유지했다.

- 기본 바탕쪽: `Odd`/`Even` > `Both`
- 첫 쪽 바탕쪽 감추기 유지
- 마지막 쪽 확장 바탕쪽 처리 유지
- `overlap`/`replace_base` 처리 유지

## 회귀 테스트

`src/document_core/queries/rendering.rs` 에 synthetic 문서 테스트를 추가했다.

```text
master_page_selection_uses_final_carried_page_number_parity
```

검증 내용:

- section 0 이 1쪽으로 끝난다.
- section 1 은 `NewNumber(Page)` 없이 이어진다.
- section 1 첫 페이지는 section-local 로는 1쪽이지만 carry 후 최종 `page_number=2` 가 된다.
- 기대: section 1 첫 페이지는 Odd 바탕쪽이 아니라 Even 바탕쪽을 선택해야 한다.

## 대상 샘플 확인

명령:

```text
cargo run --quiet --bin rhwp -- dump-pages "samples/hwpx/[2027] 온새미로 1 본교재.hwpx" -p 1
cargo run --quiet --bin rhwp -- dump-pages "samples/hwpx/[2027] 온새미로 1 본교재.hwpx" -p 3
```

결과:

```text
문서 로드: samples/hwpx/[2027] 온새미로 1 본교재.hwpx (46페이지)

page 2:
  FullParagraph pi=4 "MEMO"

page 4:
  section=1, page_num=4
  FullParagraph pi=0 "강의 01."
```

## 검증

```text
cargo fmt --check
cargo test --lib master_page_selection_uses_final_carried_page_number_parity -- --nocapture
cargo test --lib test_1098_hwpx_last_page_master_replaces_base_master -- --nocapture
cargo test --test issue_1271_hwpx_behind_text_table -- --nocapture
cargo test --test issue_703 -- --nocapture
cargo test --test issue_775 -- --nocapture
```

결과:

- `cargo fmt --check`: 통과
- `master_page_selection_uses_final_carried_page_number_parity`: 1 passed
- `test_1098_hwpx_last_page_master_replaces_base_master`: 1 passed
- `issue_1271_hwpx_behind_text_table`: 1 passed
- `issue_703`: 3 passed
- `issue_775`: 1 passed

참고: `issue_775` 실행 중 기존 overflow 진단 로그가 출력되지만 테스트는 통과했다.

## 판단

- section-local 홀짝으로 선선택한 바탕쪽이 carry 후 최종 홀짝과 달라지는 구조적 위험을 제거했다.
- Stage 2 로 맞춘 #1271 앞부분 페이지 대응은 유지된다.
- Stage 4 에서는 대상 샘플의 구조/시각 검증을 진행한다.
