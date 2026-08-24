# task_m100_5918 보고 — 쪽 경계 표 꼬리 조각의 이중 쪽 경계 제거

- 이슈: [#5918](https://github.com/edwardkim/rhwp/issues/5918)
- 브랜치: `fix/table-tail-fragment-page-5918`
- 샘플: `samples/issue4514/sample1-repro.hwp` (한글 2020 정본 46쪽, rhwp 48쪽)

## 원인

RowBreak 표(pi=578)의 꼬리 조각(rows 3..4)은 앞 조각이 쪽을 채운 시점에
continuation drain이 **새 쪽 상단에 배출**한다. 그런데 그 새 쪽이 저장 사다리의
리셋 지점(pi=608, `vpos=0`)과 같은 물리 경계였다. 메인 루프의 저장 vpos 리셋
트리거(`cv==0 && prev_vpos_end>5000`)가 pi=608에서 한 번 더
`advance_column_or_new_page()`를 불러 꼬리 조각+빈 줄(used≈78px)만 담근 근빈
쪽을 만들었고, 뒤따르는 PMR-002 표(701.4px)가 남은 ≈874px 공간으로 흐르지 못해
문서 전체가 48쪽(+2)이 됐다. 같은 이중 경계가 pi=750(38쪽, used≈220px)에서도
반복됐다.

계측으로 확인한 발동 지점(`RHWP_DIAG_COMPAT24 reset-trigger pi=608 cur=77.6`):

```
CONT-TERMINAL para=578 pages=29 cur_h=77.6   ← 꼬리 조각이 drain이 열어둔 쪽에 배치
reset-trigger pi=608 cur=77.6                ← 같은 경계를 리셋이 재차 주장
ADVANCE para=608                             ← 이중 쪽 경계 → 근빈 쪽
```

## 수정

저장 vpos 리셋 트리거 지점(src/renderer/typeset.rs)에 이중 경계 가드를 넣는다.

- `page_holds_only_fresh_table_continuation`: 현재 단이
  - `PartialTable { is_continuation: true }` 조각(들)과
  - 빈 필러 문단(`text`·컨트롤 없는 Full/PartialParagraph)만 담고 있는지 판정.
- 추가로 꼬리 조각이 쪽의 30% 이하만 차지할 때 한정한다. 조각이 작으면 그 쪽은
  저장 사다리상 다음 경계의 내용을 흡수할 예약 쪽이지만(sample1-repro pi=608:
  78px, pi=750: 220px), 조각이 쪽을 대부분 채웠다면 이미 소진된 독립 경계라
  리셋을 존중해야 한다(task2097/75544 pi=316: 909px·pi=525: 826px,
  hwpx_sample2 pi=138: 1042px — 전부 한글 정답지 존중 요구).

실 내용이 함께 놓인 쪽의 저장 경계는 종전대로 존중하므로, 리셋 의미론 자체는
변경하지 않는다.

## 결과

| 항목 | before | after |
|---|---|---|
| sample1-repro 쪽수 | 48 | **46 (정본 일치)** |
| 29쪽 | 꼬리 조각만(58자, 874px 공백) | 조각 + PMR-002 전체 + PMR-003 머리(정본과 동일 구성) |
| 38쪽 | 꼬리 조각만(97자, 734px 공백) | 조각 + 기술능력 표 + 보안분석 머리 |
| LAYOUT_OVERFLOW | 0 | 0 |

증적: `mydocs/report/edit_demo_5918/p29_before_after.png`,
`p38_before_after.png`

## 게이트

### 259문서 쪽수 게이트(tools/render_page_gate.py)

| | before | after |
|---|---|---|
| 일치(delta 0) | 249 | **250** |
| -3/-2/-1 | 각 1 | 각 1 (변화 없음) |
| +1 | 6 | 6 (변화 없음) |
| +2 | 1 | **0** |

행 단위 비교에서 delta가 바뀐 문서는 의도 대상인 sample1-repro 하나뿐이다
(`+2 → 0`). `tests/fixtures/render_page_samples.tsv`의 해당 행을
`46/48/+2 → 46/46/0`으로 갱신(정본 46쪽 = 오라클).

### 회귀 스위트

- `cargo test --profile release-test --lib -p rhwp`: **3889 passed / 0 failed**
- regression_suite_001~032: 전부 통과(단일 실행). 이 중 핀 갱신 1건:
  - `tests/issue_4514_overlay_table_flow.rs` — sample1-repro를 48로 핀한
    `assert_eq!(total, 48)`을 46으로. 테스트 주석이 "한컴 수렴 개선 시 이 값을
    좁혀 갱신한다"고 예고한 핀으로, 본 수정이 그 수렴이다(오라클: 정본 46쪽).
- `issue_4179_cursor_rect_text_host_para_pages`(suite_001)는 프로세스 전역
  perf 카운터 테스트로, cargo 기본 병렬 실행에서는 타 테스트의 트리 빌드가
  카운터에 섞여 간헐 실패한다. **베이스 커밋에서도 동일하게 재현**되는 기존
  취약성이며 본 변경과 무관하다 — 본 문서에서 가드는 issue1949 샘플에서 한 번도
  발동하지 않고(TRACE 계측 0건), 115쪽 전 페이지 텍스트가 base와
  바이트 단위로 동일하다. `--test-threads=1` 및 단독 실행에서는 통과하며,
  CI의 nextest(프로세스 분리)에서도 해당하지 않는다.

### 정적 검사

- `cargo clippy --all-targets -- -D warnings`: 0 error / 0 warning
- `cargo fmt --all` + `cargo fmt --all -- --check`: 통과
- `node scripts/rust-unit-test-tiers.mjs --check`: 4221 tests 이상 없음

## 회귀 테스트

`tests/cases/issue_5918_table_tail_fragment_page.rs` (regression_suite 번들은
`--prepare`로 재생성):

- `page_count() == 46` (정본 일치)
- 29쪽(0-based 28)에 PMR-001 꼬리 조각(`pi=578`)과 PMR-002 표(`pi=612`)가
  함께 존재하는지 검사 — 근빈 쪽 회귀를 잡는다.

## 스킵/보류

- 이슈 본문이 든 43쪽(pi=763/764 경계)은 저장 사다리에 실제 경계가 있어
  정답지상 분할이 맞다 — 48→46이 정본과의 정확한 수렴이며, 추가 병합은
  오라클 위반이다.
