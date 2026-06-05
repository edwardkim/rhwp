# PR #1077 재검토 문서

- PR: <https://github.com/edwardkim/rhwp/pull/1077>
- 제목: `fix: 표 위치 편집 시 raw_ctrl_data 오프셋 4바이트 밀림 — flags 오염 (#698)`
- 관련 이슈: <https://github.com/edwardkim/rhwp/issues/698>
- 작성일: 2026-05-26
- 작성자: Codex

## 1. PR 상태

| 항목 | 값 |
|---|---|
| 상태 | open |
| base | `devel` |
| head | `contrib/fix-table-position-offset` |
| head sha | `a6a8745506949807f81ebd474129ef2848642ea8` |
| mergeable | mergeable |
| 작성자 | `oksure` |
| 변경 파일 | 2개 |
| 변경 범위 | `src/document_core/commands/table_ops.rs`, `src/model/table.rs` |
| 관련 이슈 | `#698` |

CI 확인:

| check | status |
|---|---|
| Build & Test | pass |
| Analyze (javascript-typescript) | pass |
| Analyze (python) | pass |
| Analyze (rust) | pass |
| CodeQL | pass |
| WASM Build | skipped |

## 2. PR 변경 이력

PR #1077은 4개 커밋으로 구성되어 있다.

```text
927b1003 fix: raw_ctrl_data 오프셋 4바이트 밀림 — 표 위치 편집 시 flags 오염 (#698)
f46fab29 test: raw_ctrl_data 오프셋 레이아웃 회귀 방지 테스트 추가
7a8781e5 style: cargo fmt 포맷 정규화
a6a87455 fix: update_ctrl_dimensions/set_table_properties 오프셋 4바이트 밀림 정정 (#698 잔존)
```

중요한 점:

```text
처음 3개 커밋은 v/h offset 경로만 수정했다.
이후 메인테이너 수동 검증에서 표 resize 경로의 width/height 오염이 남는 것이 확인되었다.
마지막 커밋 a6a87455가 그 지적 사항을 반영했다.
```

따라서 수용 여부는 4개 커밋 전체 기준으로 판단해야 한다.

## 3. 컨트리뷰터 주장 검증

### 3.1 권위 레이아웃 주장

컨트리뷰터 주장:

```text
table.raw_ctrl_data는 CommonObjAttr payload이며,
[0..4]=flags, [4..8]=vertical_offset, [8..12]=horizontal_offset,
[12..16]=width, [16..20]=height 이다.
```

코드 검증:

```text
src/parser/control.rs:
  table.raw_ctrl_data = ctrl_data.to_vec()

src/parser/control/shape.rs::parse_common_obj_attr:
  read_u32() -> attr
  read_u32() -> vertical_offset
  read_u32() -> horizontal_offset
  read_u32() -> width
  read_u32() -> height
  read_i32() -> z_order
  read_i16 x4 -> outer_margin
  read_u32() -> instance_id
  read_i32() -> prevent_page_break
```

판정:

```text
확인됨.
rhwp의 raw_ctrl_data는 ctrl_id를 제외한 CommonObjAttr 본체로 보는 것이 맞다.
따라서 현재 local/devel의 table_ops.rs와 table.rs 일부 인덱싱은 4바이트 밀려 있다.
```

### 3.2 현재 local/devel 버그 존재 여부

현재 `local/devel` 확인 결과:

```text
src/document_core/commands/table_ops.rs:
  move_table_offset_native:
    vertical_offset   [0..4]  사용
    horizontal_offset [4..8]  사용

  get_table_properties_json:
    table_width       [8..12] 사용
    table_height      [12..16] 사용
    keepWithAnchor    [32..36] 사용

  set_table_properties_json:
    vertOffset        [0..4]  사용
    horzOffset        [4..8]  사용
    outer_margin      [20..28] 사용

src/model/table.rs:
  update_ctrl_dimensions:
    width             [8..12] 사용
    height            [12..16] 사용
```

이는 `parse_common_obj_attr` 기준과 불일치한다.

판정:

```text
확인됨.
특히 update_ctrl_dimensions()는 resize_table_cells_native()에서 호출되므로,
표 크기 조절 후 total_width가 h_offset 슬롯에 저장되는 문제가 실제로 발생할 수 있다.
```

### 3.3 메인테이너 수동 검증 지적 반영 여부

이전 메인테이너 지적:

```text
1. update_ctrl_dimensions width/height를 [12..16]/[16..20]으로 정정
2. set_table_properties outer_margin을 [24..32]로 정정
3. keepWithAnchor/prevent_page_break 위치를 [36..40]으로 정정
4. 회귀 가드를 width/height까지 확장
```

PR 최신 커밋 `a6a87455` 확인 결과:

| 항목 | 반영 여부 |
|---|---|
| `update_ctrl_dimensions` width/height 슬롯 정정 | 반영 |
| `get_table_properties_json` width/height 읽기 정정 | 반영 |
| `outer_margin` 읽기/쓰기 `[24..32]` 정정 | 반영 |
| `keepWithAnchor` `[36..40]` 정정 | 반영 |
| `update_ctrl_dimensions_writes_correct_slots` 테스트 추가 | 반영 |

판정:

```text
확인됨.
이전 수동 검증에서 발견한 "width가 h_offset 슬롯에 들어가는 잔존 버그"는 최신 PR에서 처리되었다.
```

### 3.4 컨트리뷰터의 8바이트 legacy 설명

컨트리뷰터는 Copilot 피드백에 대해 다음 취지로 답했다.

```text
기존 8바이트 raw_ctrl_data는 이미 잘못된 레이아웃이었고,
parse_common_obj_attr가 바이트 수에 맞게 파싱하므로 raw_ctrl_data 직접 접근 경로만 수정하면 된다.
```

코드 기준 검토:

```text
parse_common_obj_attr는 legacy 8바이트 레이아웃을 별도로 마이그레이션하지 않는다.
항상 [0..4]=attr, [4..8]=v_offset, [8..12]=h_offset 순서로 읽는다.
```

판정:

```text
컨트리뷰터의 "parser가 바이트 수에 맞게 legacy를 처리한다"는 설명은 코드상 확인되지 않는다.
다만 PR이 겨냥한 HWP5 CommonObjAttr 저장/편집 경로의 권위 레이아웃은 명확하며,
현재 버그도 valid CommonObjAttr raw_ctrl_data를 잘못 덮어쓰는 문제다.
```

따라서 legacy 8바이트 호환성은 이번 PR의 blocker로 보지는 않되, 별도 이슈가 되면 마이그레이션
정책을 따로 정해야 한다.

## 4. 적용 방식 검토

PR branch는 현재 `devel`보다 오래된 base에서 출발한다.

```text
merge-base = 8a1f9fd2240a7c9372abeadc381164f05a33e541
현재 local/devel에는 #1084, #1091, #1093 등 이후 변경이 이미 반영되어 있음
```

따라서 PR branch를 그대로 merge하면 현재 devel의 많은 변경과 문서가 사라지는 diff가 된다.
수용 시에는 반드시 4개 커밋을 순서대로 `-x` cherry-pick 해야 한다.

권장 커맨드:

```text
git cherry-pick -x 927b1003a0537cc3f795e13950b640376a92f638
git cherry-pick -x f46fab299e342a54f30cf26fbc072dc57f2c54da
git cherry-pick -x 7a8781e56f00242c3720d325e316e703ceaf6521
git cherry-pick -x a6a8745506949807f81ebd474129ef2848642ea8
```

`git merge-tree` 기준 충돌은 예상되지 않는다.

## 5. 리스크

### 5.1 html_table_import 미포함

현재 `src/document_core/html_table_import.rs`에도 비슷한 offset 패턴이 남아 있다.

```text
raw_ctrl_data[8..12]  = total_width
raw_ctrl_data[12..16] = total_height
raw_ctrl_data[20..28] = outer_margin
raw_ctrl_data[28..32] = instance_id
```

이 PR은 `table_ops.rs`와 `model/table.rs`만 수정한다.
PR 본문과 메인테이너 코멘트 모두 자매 PR #1078을 별도 점검 대상으로 언급하고 있으므로,
HTML 테이블 import 경로는 이번 PR 수용 범위 밖으로 둔다.

### 5.2 회귀 가드 범위

추가된 테스트는 raw offset layout과 `update_ctrl_dimensions()` 슬롯 오염을 직접 검증한다.
하지만 실제 한컴 재오픈까지 포함한 end-to-end 검증은 자동화되어 있지 않다.

따라서 최종 수용 전에는 다음 수동 검증이 필요하다.

```text
1. rhwp-studio에서 빈 문서 생성
2. 표 생성
3. 표 전체 셀 선택 후 일괄 크기 조절
4. 저장
5. 한컴 에디터에서 재오픈
6. 표가 줄 끝으로 밀리지 않고 정상 위치에 남는지 확인
```

## 6. 권장 처리 방향

권장안:

```text
1. 4개 커밋을 현재 local/devel에 -x cherry-pick 한다.
2. cargo fmt --check
3. cargo check
4. cargo test raw_ctrl_data_offsets_match_parser update_ctrl_dimensions_writes_correct_slots
5. cargo test --lib
6. docker compose --env-file .env.docker run --rm wasm
7. 메인테이너 수동 한컴 검증
8. 통과 시 devel에 반영하고 PR #1077 및 이슈 #698 close
```

현재 판단:

```text
컨트리뷰터의 핵심 기술 주장(raw_ctrl_data CommonObjAttr offset)은 코드와 스펙 기준으로 맞다.
이전 메인테이너 수동 검증에서 발견한 누락 영역도 최신 커밋에서 반영되었다.
따라서 체리픽 수용 후보로 보는 것이 타당하다.
```

## 7. 승인 요청

위 권장안대로 진행해도 되는지 승인 요청한다.
