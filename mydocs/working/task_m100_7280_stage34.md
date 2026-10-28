# Task #7280 Stage 34 — R2 잔여 책임 점검과 어울림 계약 추가 검증

- Issue: [#7280](https://github.com/edwardkim/rhwp/issues/7280)
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md)
- 이전 절편: [R2ae](task_m100_7280_stage33.md).
- 점검 head: `990e7475dadbfc21c1558311fb55191ad1f1f57a`.
- 동일 제품 SHA: `6df03ebdf048b5a9914c4aa631d2aeb0b7123b3c`.
- 고정 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 상태: 잔여 경계 점검·추가 focused 10건 완료. 이번 절편은 제품 코드 변경이 없다.

## 1. 점검 결론

일반 문단과 표 소유 문단의 최상위 진입은 분리됐지만, 어울림 문단 경로에는 판별·상태 변경·
구역 반복문 제어가 섞여 있다. R2 구조 완료나 책임 묶음 통합 검증 완료를 아직 선언하지 않는다.
반대로 구역 순회 전체와 각주/미주·표 분할 본체를 R2에 계속 흡수하지 않는다.
승인된 R3/R4/R5 소유권을 유지하고 다음과 같이 경계를 고정한다.

아래 줄 번호는 점검 head의 `src/renderer/typeset.rs` 기준이다.

| 현재 책임/위치 | 실제 호출·효과 | 후속 소유권 |
| --- | --- | --- |
| `typeset_no_table_paragraph_tail` (6376), 호출 8753 | 표 없는 문단의 Picture/Shape 저장 밴드 준비, host anchor 등록, 저장 밴드가 없으면 유도 lane 준비 | R2: controls의 읽기 전용 준비 판단 + state 적용. 구역 내 호출 위치는 R5까지 유지 |
| `typeset_wrap_around_paragraph` (6644), 호출 8164 | 저장 cs/sw·개체 종류 매칭, anchor 등록, 표 옆 문단 기록, prefix/tail 분리, 밴드 종료와 쪽 전환 | R2: 판별·조정·상태 적용 분리. 반환 true가 구역 루프의 continue라는 계약 보존 |
| `native_hwp5_square_picture_next_page_owner` (6515), 호출 8889 | 현재 쪽 본문/각주를 보고 다음 physical page 소유 그림 후보 산출; 호출자는 지연 큐에 넣고 continue | R2: 그림 소유 후보 Query. 큐 materialize와 페이지 수명은 R5 |
| `square_picture_wrap_anchor_for_para` (2004), register (2080), activate (2101) | 미주 경로의 호출 13921/14140, 본문+미주 전역 문단 인덱스 해석 | R4 연결 경계. 본문 전용 매칭과 성급히 병합하지 않음 |
| `close_square_band` (4953), extend (5018), record (5098) | 현재 높이 변경, 저장 밴드 바닥 확장, 첫 표 조각이 있는 과거 쪽으로 소급 기록 가능 | R2 이동 시 필요한 명령 경계만 state로; 최종 접근 제어는 R5 |
| `typeset_section_with_variant` (7001), `push_new_page` (5249) | 구역 순회·계측·continue·페이지 초기화·지연 그림 anchor 확정 순서 | R5. R2에서 구역 전체를 통째로 이동하지 않음 |

`typeset_no_table_paragraph_tail`의 현재 본체는 paragraphs/composed/styles를 사용하지 않는다.
이는 구조 점검 결과이지 이번 절편에서 시그니처를 바꾼 것이 아니다.
기존 미사용 인자 제거와 본체 이동을 함께 할 경우 호출부 정적 대조에 별도로 표시한다.

## 2. 다음 분리에서 반드시 보존할 순서와 비대칭

1. 후속 문단의 어울림 처리(8164)는 `ensure_page`와 일반 배치 전에 실행된다.
   그 반환값을 잃거나 배치 뒤로 옮기면 같은 문단을 다시 배치할 수 있다.
2. host 밴드 준비(8753)는 기존 문단/표 처리 뒤이며, 비표 컨트롤 순회 전에 있다.
   이 순서를 일반 문단 `flow::place` 안으로 무조건 흡수하지 않는다.
3. `close_square_band`는 플래그 해제만 하는 함수가 아니다.
   `current_height = max(current_height, square_band_bottom)`을 수행한다.
   Query로 분류하거나 fit/쪽 전환 검사 뒤로 옮기지 않는다.
4. 표 옆 문단 기록은 현재 쪽에만 append하지 않는다. 첫 표 조각이 이전 쪽에 있으면
   `record_wrap_around_para`가 해당 column에 소급 기록한다.
5. prefix/tail 경로는 문단 포맷 후 suffix 높이를 계산하고, 기록·밴드 종료 뒤
   새 상태로 필요 시 쪽 전환을 판단한다. 모든 관측값을 진입 시 snapshot으로 대체하지 않는다.
6. 지연 그림은 현재 본문을 남긴 채 다음 쪽에 속할 수 있다. `push_new_page`는 상태 초기화 후
   다음 쪽 anchor를 등록하고 Shape materialize를 위한 목록에 넣는다.
   현재 `current_items`에 즉시 넣는 것으로 단순화하지 않는다.
7. 본문 매칭은 `paragraphs.get`을 사용하지만 미주 anchor 조회는
   `paragraph_by_global_index(body_paragraphs, endnote_paragraphs, ...)`를 사용한다.
   닮은 조건식을 이유로 두 경로를 하나의 새 조판 규칙으로 통합하지 않는다.

상수·문서 형상 조건·호환성 분기의 타당성은 이번 감사에서 새로 승인하지 않았다.
기존 조판 결함 수정, IR/public API 변경, baseline/golden/ignore 완화는 범위 밖이다.

## 3. 추가로 확인한 기존 테스트 계약

Stage33의 313건 선택에는 아래 10건이 포함되지 않았다. 이는 **집중 선택 범위의 공백**이며,
baseline 전체 회귀에서 누락됐거나 실행 실패했다는 뜻이 아니다.
본문과 assertion을 읽어 다음 의미를 확인하고 기존 테스트 그대로 실행했다.

| 기존 module | 건수 | 확인하는 의미 / 한계 |
| --- | ---: | --- |
| `issue_6175_square_band_uniform_ladder` | 1 | 본문 4줄 보존·그림 밴드 침범 방지 |
| `issue_6175_body_square_float_stored_rows` | 3 | 좁은 저장 폭·줄 수 보존, 그림을 아래로 옮긴 변형 입력에서는 전폭 복귀; CLI 소비 포함 |
| `issue_7158_square_wrap_continuation` | 3 | 저장 줄 간격, 누락/중복, 선행 여백 변경에 따른 이어받기·후속 문단의 동일 이동 |
| `issue_4090_square_table_left_wrap` | 1 | 표 왼쪽 prefix와 표 아래 full-width tail 존재 |
| `issue_4090_hwpx_tail_page_break` | 1 | 총 17쪽뿐 아니라 지정 문단 앞/뒤 줄 범위의 쪽 소유 확인; CLI 소비 포함 |
| `issue_4599_ladder_band_double_count` | 1 | 후속 문단 y 상한으로 float 높이 이중 소비 방지; 전체 기하 동등성 검사는 아님 |

재사용할 입력 경로:

- `samples/issue5809/156518601_p1_square_host.hwpx`
- `samples/issue6175/seed_expo_square_float_body.hwpx`
- `samples/issue4090/156492236_규제샌드박스_min.hwpx`
- `samples/issue4599/36374873_night_guard_log.hwpx`

변형 입력 계약과 실제 원본 출력 대조는 별개다. 위 검사로 모든 caption/빈 문단/다단/그림
지연/미주 분기가 실행됐다고 주장하지 않는다. branch coverage 계측이나 직접 시각 판독은 하지 않았다.

## 4. 실행 증거

Stage33의 고정 제품 review worktree를 재사용했다. 점검 head와 이 제품 SHA 사이의
`src`, `tests`, `crates`, Cargo 파일 차이가 없음을 확인했다. 제품을 다시 수정하지 않았다.

```bash
# /home/edward/mygithub/rhwp-review-7280-r2ae, HEAD 6df03ebdf048b5a9914c4aa631d2aeb0b7123b3c
CARGO_BUILD_JOBS=4 cargo nextest run --locked --cargo-profile release-test \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review \
  --test regression_suite_001 --test regression_suite_004 \
  --test regression_suite_012 --test regression_suite_019 \
  --test regression_suite_020 --test regression_suite_021 \
  -E 'test(issue_6175_square_band_uniform_ladder::) | test(issue_6175_body_square_float_stored_rows::) | test(issue_4599_ladder_band_double_count::) | test(issue_7158_square_wrap_continuation::) | test(issue_4090_square_table_left_wrap::) | test(issue_4090_hwpx_tail_page_break::)' \
  --no-fail-fast
```

- **10 passed / 0 failed**, 6 binaries, 필터 비선택 1,176건, exit 0.
- 캐시 재사용 build 0.16초, test 0.256초.
- run ID: `00f2c465-d1a7-45d6-99c4-0c3ece6ad380`.
- `output/7280/stage34/run-wrap-audit.sh`, `nextest-wrap.log`에 명령과 로그 보존.
- `verify-wrap-audit.mjs` / `regression-comparison.json`: baseline의 같은 10개 PASS 이름과
  일치, Stage33 선택과 교집합 0건, 제품 차이 없음 확인.
- nextest 0.9.137 권장 버전 및 observation 설정 경고는 기존과 동일하다.

Stage33의 313건은 이번에 재실행하지 않았다. 같은 제품 SHA에 10건의 추가 증거를 붙인 것이며,
323건을 이번에 한 번에 실행했다거나 전체 회귀를 통과했다고 보고하지 않는다.
이번에는 문서만 변경하므로 Rust lint를 재실행하지 않았다. Stage33 lint 결과는 그 제품 SHA의
기존 증거로만 참조한다. review worktree tracked 변경은 없고 파생 suite는 stage하지 않는다.

## 5. 다음 절편과 통합 게이트

다음 구현 절편 R2af는 **표 없는 host 문단의 밴드 준비**로 한정한다.

1. `typeset_no_table_paragraph_tail`의 저장 밴드/host anchor/유도 lane 선택을 읽기 전용 입력과
   준비 결과로 분리한다. 기존 값이 유지되는 분기와 새로 적용되는 분기를 구분한다.
2. state에는 의미 있는 준비 결과 적용 명령을 두고 구역 호출 위치·계측 순서는 유지한다.
   뒤따르는 전체 어울림 문단 처리·페이지 이월 본체는 같은 절편에 섞지 않는다.
3. 전체 원문 대조 + 기존 313건과 이번 10건을 선택한 집중 검증으로 확인한다.
   추가 분기에 관련된 미주/지연 그림 계약은 별도로 확인하며 323건만으로 완전성을 주장하지 않는다.

그 뒤 후속 문단 매칭/흡수·prefix/tail 경계와 지연 그림 후보를 순서대로 분리한다.
R4 미주와 R5 페이지 전이의 연결부는 책임 소유권을 기록하고 해당 묶음에서 마무리한다.

R2 책임 묶음 종료에는 고정 baseline/변경본의 동일 입력·설정·쪽에서 내용 소유권·컷·기하와
native/fresh Docker WASM 출력을 직접 비교해야 한다. 위 네 입력은 우선 대조 후보이며,
대응 PDF의 위치·유효성과 실제 영향 쪽은 실행 전에 확인한다. 이번에는 PDF 확인/시각 통과를
선언하지 않았다. 기존 TAC/float/빈 문단/각주 후보도 이전 절편 기록에서 함께 선택한다.
전체 회귀·WASM/workspace lint·workspace build·Native Skia·최종 시각 증적은 구현계획 §7과
local_validation §4.3을 따른다. 원격 push·PR·댓글은 실행하지 않았다.
