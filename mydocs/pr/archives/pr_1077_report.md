# PR #1077 처리 보고서

- PR: <https://github.com/edwardkim/rhwp/pull/1077>
- 관련 이슈: <https://github.com/edwardkim/rhwp/issues/698>
- 작성일: 2026-05-26
- 처리 방식: `-x` cherry-pick 수용 후보

## 1. 처리 요약

PR #1077의 4개 커밋을 현재 `local/devel`에 순서대로 체리픽했다.

```text
원본 커밋: 927b1003 fix: raw_ctrl_data 오프셋 4바이트 밀림 — 표 위치 편집 시 flags 오염 (#698)
반영 커밋: db25906d

원본 커밋: f46fab29 test: raw_ctrl_data 오프셋 레이아웃 회귀 방지 테스트 추가
반영 커밋: a0f776f7

원본 커밋: 7a8781e5 style: cargo fmt 포맷 정규화
반영 커밋: 3c060476

원본 커밋: a6a87455 fix: update_ctrl_dimensions/set_table_properties 오프셋 4바이트 밀림 정정 (#698 잔존)
반영 커밋: 2455584b
```

변경 내용:

```text
1. table_ops.rs의 v_offset/h_offset 직접 접근을 CommonObjAttr 레이아웃에 맞게 정정
   - [4..8] = vertical_offset
   - [8..12] = horizontal_offset
2. get/set table properties의 width/height, outer_margin, keepWithAnchor offset 정정
3. Table::update_ctrl_dimensions()가 width/height를 [12..16]/[16..20]에 쓰도록 정정
4. raw_ctrl_data offset 레이아웃 회귀 가드 추가
5. update_ctrl_dimensions()가 h_offset 슬롯을 오염시키지 않는지 검증하는 회귀 가드 추가
```

## 2. 컨트리뷰터 주장 검증

컨트리뷰터의 핵심 주장은 다음과 같다.

```text
table.raw_ctrl_data는 ctrl_id를 제외한 CommonObjAttr 본체이며,
[0..4]=flags, [4..8]=vertical_offset, [8..12]=horizontal_offset,
[12..16]=width, [16..20]=height 이다.
```

코드 검증 결과:

```text
src/parser/control.rs:
  table.raw_ctrl_data = ctrl_data.to_vec()

src/parser/control/shape.rs::parse_common_obj_attr:
  attr → vertical_offset → horizontal_offset → width → height 순서로 읽음
```

판정:

```text
주장은 확인되었다.
현재 local/devel의 기존 table_ops.rs/model/table.rs 일부 경로는 이 레이아웃보다 4바이트 밀려 있었다.
```

이전 메인테이너 수동 검증에서 발견한 잔존 버그도 최신 PR 커밋에서 반영되었다.

```text
update_ctrl_dimensions:
  before: width [8..12], height [12..16]
  after : width [12..16], height [16..20]

outer_margin:
  before: [20..28]
  after : [24..32]

keepWithAnchor / prevent_page_break:
  before: [32..36]
  after : [36..40]
```

## 3. 검증

자동 검증:

| command | result |
|---|---|
| `cargo fmt --check` | pass |
| `cargo check` | pass |
| `cargo test raw_ctrl_data_offsets_match_parser` | pass |
| `cargo test update_ctrl_dimensions_writes_correct_slots` | pass |
| `cargo test --lib` | pass, 1398 passed / 0 failed / 6 ignored |
| `docker compose --env-file .env.docker run --rm wasm` | pass |

메인테이너 수동 검증:

```text
통과
```

## 4. 판단

수용 후보 판단:

```text
PR #1077은 #698의 표 위치 저장 손상 원인인 table raw_ctrl_data CommonObjAttr offset 오염을 정정한다.
최초 PR 범위에서 누락되었던 resize/update_ctrl_dimensions 경로도 최신 커밋에서 보강되었다.
현재 local/devel 기준 자동 검증과 wasm 빌드는 통과했다.
```

따라서 PR #1077은 체리픽 수용으로 처리하는 것이 타당하다.

수동 검증 권장 시나리오:

```text
1. rhwp-studio에서 빈 문서 생성
2. 표 생성
3. 표 전체 셀 선택 후 일괄 크기 조절
4. 저장
5. 한컴 에디터에서 재오픈
6. 표가 줄 끝으로 밀리지 않고 정상 위치에 남는지 확인
```

## 5. 주의 사항

이번 PR은 `table_ops.rs`와 `model/table.rs`만 수정한다.

```text
src/document_core/html_table_import.rs에도 유사한 offset 패턴이 남아 있다.
이 경로는 PR #1078의 별도 검토 범위로 보고 이번 PR의 blocker로 두지 않는다.
```

또한 컨트리뷰터가 언급한 8바이트 legacy raw_ctrl_data 자동 마이그레이션 설명은 코드상 확인되지 않는다.
다만 이번 PR의 대상은 valid CommonObjAttr raw_ctrl_data를 직접 접근 경로에서 잘못 덮어쓰는 문제이므로,
legacy 마이그레이션은 별도 이슈로 분리 가능하다.

## 6. 다음 절차

승인 후 진행:

```text
1. pr_1077_review.md / pr_1077_report.md 커밋
2. local/devel → devel fast-forward merge
3. devel 기준 검증
4. origin/devel push
5. PR #1077에 체리픽 반영 댓글 작성 후 close
6. 이슈 #698 close(completed)
```
