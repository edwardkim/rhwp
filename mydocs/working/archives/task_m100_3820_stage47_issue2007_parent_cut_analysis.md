---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-07
---

# Task #3820 Stage 47 — issue2007 부모 RowCut 소유 경계 분석

## 목적

PR #4122 통합 뒤 남은
`samples/basic/issue2007_nested_cell_pagination_42065.hwp` p12–p15의 페이지 소유권
오차를 한컴 2020 PDF와 직접 대조한다. 이번 단계는 실패한 휴리스틱을 코드에 남기지 않고,
재현 수치와 다음 수정 경계를 문서·집중 회귀로 고정하는 체크포인트다.

## 기준 자료

- 입력 HWP: `samples/basic/issue2007_nested_cell_pagination_42065.hwp`
- 한컴 2020 기준 PDF:
  `pdf/basic/issue2007_nested_cell_pagination_42065-2020.pdf`
- 리베이스 판정: `mydocs/working/task_m100_3820_stage46_pr4122_rebase_resolution.md`
- 전용 target: `target/task-3820-3821-fidelity-rebase`

## 확인된 소유권 결함

현재 p12 하단에는 기준 PDF상 p13에서 시작해야 하는 `4 국가인권위원회` 제목이 먼저
그려진다. 약 27.7px 높이의 제목이 앞쪽 조각에 잘못 소유되면서 이후 페이지가 연쇄적으로
더 많은 중첩 셀 unit을 소비한다.

### p13 → p14 감사원 경계

감사원 scalar stream은 p13에서 8 units / 113.52px를 소비하고 p14에서 그 offset으로
재개한다. 부모 outer unit의 의미는 다음과 같다.

| outer unit | 의미 | 높이(px) |
| ---: | --- | ---: |
| 276 | padding | 3.76 |
| 277 | `감사원법` | 19.093 |
| 278 | spacing | 4.0 |
| 279 | 제27조 본문 | 17.333 |
| 280–281 | 항목 1 두 줄 | 17.333 × 2 |
| 282 | 항목 2 | 17.333 |
| 283 | 항목 3 | 17.333 |

현재 부모 cut은 `[273, 284)`여서 항목 2·3까지 소유한다. 실제 paint에서는 항목 2가 p13에
보이고 항목 3은 reservation처럼 소비된다. 기준 PDF 소유 경계는 `end=282` exclusive이며,
p14가 unit 282부터 시작해야 한다.

### p14 → p15 금융위원회 경계

금융위원회 scalar stream은 p14에서 32 units / 516.2px를 소비한다.

| outer unit | 의미 | 높이(px) |
| ---: | --- | ---: |
| 331–332 | 항목 7 두 줄 | 17.333 × 2 |
| 333–334 | 항목 8 두 줄 | 17.333 × 2 |
| 335 | 다음 문단 spacing | 4.0 |
| 336 | 제427조 첫 줄 | 17.333 |
| 337 | 제427조 둘째 줄 | 17.333 |

현재 부모 cut은 `[284, 337)`여서 항목 8, spacing, 제427조 첫 줄까지 소비한다. 기준 PDF
소유 경계는 `end=333` exclusive이며, p15가 unit 333부터 시작해야 한다. 따라서 모든
continuation을 한 줄 이동하는 보정으로는 해결할 수 없다.

## 유지할 검증된 변경

1. `mixed_nested_split_from_cut`의 terminal 여부는 host cell 전체 끝이 아니라 같은
   `para_idx`의 mixed nested stream 잔여 여부로 판정한다.
2. terminal 재귀 조각의 맨 앞 contentless trailing reservation은 바깥 flow 높이에는
   유지하되 child cursor에서는 건너뛴다. 이 보정은 p17 상단의 불필요한 약 32px 공백과
   본문 누락을 제거한다.
3. scalar 첫 가시 unit 보정은 재귀 child cursor가 없고 non-terminal인 1×1 continuation에만
   적용한다.
4. p8 제목 위치 회귀는 한컴 PDF bbox `yMin=88.610521pt`, 96dpi 환산 118.147px를 기준으로
   `117.5..=119.0px`를 허용한다.
5. p11 frame 검사는 기준 PDF에서 실제로 다음 쪽에 소유되는 문장으로 검사한다.

## 폐기한 가설

1. 재귀 조각 전체에 flow extra를 더하는 방식은 p3/p8 등 기존 페이지를 함께 이동시켰다.
2. terminal stream 전체에 같은 extra를 주는 방식은 집중 회귀 5건을 깨뜨렸다.
3. scalar viewport를 고정 1 unit 이동하는 방식은 p13의 2 units와 p14의 4 units 초과를
   동시에 설명하지 못한다.
4. `mixed_nested_starts_after_table` 뒤의 짧은 꼬리를 부모에서 일괄 되감는 실험은 p12 제목을
   옮길 가능성은 보였으나 p11 continuation viewport도 함께 바꿔 집중 회귀를 만들었다.
5. `authoritative_source_cut` 플래그로 contentless trailing을 terminal에서 무시하는 실험은
   부모 경계 결함과 scalar child 경계를 섞으므로 체크포인트에서 제거한다.

## Stage 48 수정 경계

다음 단계에서는 p12 제목을 뒤의 page-scale recursive block과 함께 넘기는 source 의미를
부모 `RowCut`에서 좁게 식별한다. p10→p11과 p17의 기존 계약은 바꾸지 않는다. 수정 뒤에는
clip 밖 source text까지 포함하는 정규화 문자열이 아니라 실제 paint tree를 사용해 다음
소유권을 고정한다.

- p13에는 감사원 항목 2가 없고 p14에는 있다.
- p14에는 금융위원회 항목 8이 없고 p15에는 있다.
- 금융위원회 제목은 p14에 있고 p15에서 반복되지 않는다.
- p15에는 조달청 마지막 문장 `제기할 수 있다.`가 남는다.

## 검증 결과

다음 명령을 전용 target에서 실행했다.

```bash
CARGO_INCREMENTAL=0 \
CARGO_TARGET_DIR=target/task-3820-3821-fidelity-rebase \
cargo test --profile release-test --test issue_2007_nested_cell_pagination

cargo fmt --all -- --check
git diff --check
```

- `cargo fmt --all` 및 `git diff --check`: 통과
- issue2007 집중 회귀: **11 passed / 1 failed / 0 ignored**
- 유일한 실패:
  `issue_2007_continuation_frame_restarts_and_drops_previous_page_residual`
- 실패 위치: p15 continuation에서
  `금융위원회는 관계자에 대한 조사실적`을 포함하는 상단 프레임을 찾지 못함

이 1건은 Stage 47에서 발견한 부모 source cut의 잔존 결함을 그대로 고정한 gate다. assertion을
완화하거나 baseline으로 승인하지 않고 Stage 48의 완료 조건으로 넘긴다. 나머지 11건이
통과하므로 실패한 parent carry 휴리스틱을 제거한 뒤의 검증된 체크포인트와 일치한다.
