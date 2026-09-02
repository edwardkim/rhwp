---
kind: working
status: done
canonical: mydocs/working/task_m100_4100_stage4.md
last_verified: 2026-08-11
---

# #4100 Stage 4 — `set_chart_data_native`, ①② 동시 기록

- **계획서**: [`mydocs/plans/task_m100_4100.md`](../plans/task_m100_4100.md)
- **기준 커밋**: `devel = dd9ecdc4b`
- **산출**: `src/document_core/commands/object_ops/chart.rs`(쓰기 경로) ·
  `src/document_core/commands/document.rs`(`bin_data_epoch` 주석 갱신) ·
  `tests/issue_4100_chart_data_edit.rs`(Stage 4 테스트 6건)

## 1. 결정과 근거

### 1-1. 입력은 행렬이다 — CSV 한 장과 같은 모양

```json
{ "labels": ["항목 1", …], "series": [{"name": "계열 1", "values": ["91.7", …]}], "dryRun": false }
```

행·열 수 대조가 본래 행렬 모양의 검증이고, 이슈가 *"행·열 수가 다르면 한 칸도 쓰지 않고
`invalid[]` + exit 2"* 를 요구한다. 코어가 행렬을 받으면 그 검증이 **코어 한 곳**에 남고
`csv-to-chart` 는 CSV→행렬 변환만 하는 얇은 층이 된다.

값은 **문자열로만** 받는다. JSON 숫자로 받으면 `4.3` 이 `4.30` 으로 되쓰일 수 있어 무편집
왕복의 바이트 동일이 깨진다.

`labels` 는 축에 따라 뜻이 갈린다 — 카테고리형이면 **대조만** 하고(다르면 `categoryMismatch`,
라벨 변경은 B2), 분산형이면 X 라 **기록한다**.

### 1-2. ② 를 특정하지 못하면 아무것도 쓰지 않는다

`chart_switch_fallback` 이 없어 ②를 못 찾으면 `nestedCopyNotFound` 로 거부하고 **①에도 쓰지
않는다.** "①만이라도 쓴다"는 선택지가 없는 이유는 #4099 의 fold 가 확정했다 — HWP5 변환은
①을 IR 에서 지우므로 ①만 새 값이면 변환 즉시 편집이 사라진다. 반쪽만 새 값인 파일이 최악이다.

`wrote[]` 를 항상 봉투에 실어 어느 표현에 썼는지 드러낸다. HWPX 는 `["zipPart","nestedCopy"]`,
HWP5 는 `["nestedCopy"]`, 거부·dry-run·무변경은 `[]`.

**이슈가 적은 `ambiguousNestedCopy` 는 구현하지 않았다.** 해소가 결정적이라 모호해질 자리가
없다 — 조인 키는 `chart_switch_fallback` 하나뿐이고, 슬롯 조회는 인덱스 우선 → id 폴백으로
답이 하나다. 조인 키가 없으면 `nestedCopyNotFound` 이고 그 사이의 상태가 없다.

### 1-3. ② 재포장을 먼저 시도한다

실패할 수 있는 것(CFB 재포장)을 **먼저** 돌리고, 성공한 뒤에야 두 슬롯을 갈아끼운다. 순서가
반대면 ①만 새 값인 문서가 메모리에 남는다.

### 1-4. 바뀐 칸이 없으면 한 바이트도 건드리지 않는다

`changedCount == 0` 이면 슬롯 대입 자체를 건너뛴다. 되쓰기만 해도 중첩 CFB 재포장이 섹터
배치를 바꿔 무편집 왕복의 바이트 동일이 깨진다(Stage 2 §2-4 와 같은 이유, 한 층 위에서).

### 1-5. `bin_data_epoch` — 네 번째 자리를 만들었다 (계획서 R1)

주석이 *"id→바이트는 append-only 라 세션 중 안정하고, 그 안정성을 깨는 것은 문서를 통째로
갈아끼우는 연산뿐"* 이라며 세 곳(스냅샷 복원·새 문서·`set_document`)만 허용해 뒀다.

차트 편집은 **문서를 갈아끼우지 않으면서 기존 id 의 바이트를 제자리에서 바꾸는 첫 연산**이라
그 전제를 정면으로 깬다. 올리지 않으면 편집 후 재렌더가 옛 차트를 그린다. 그래서
`bump_bin_data_epoch()` 를 부르고 **주석의 목록에 이 경우를 추가**했다 — 다음 사람이 "왜
여기서 올리지?"로 되돌리지 않게 근거를 같은 자리에 남긴다.

대가는 바이트가 그대로인 다른 그림의 캐시 키도 함께 무효화되는 것인데, 그것은 성능이고
이쪽은 정확성이다.

## 2. 판정

### 2-1. 코퍼스 56건 — 전건 green

| 테스트 | 무엇을 고정하나 |
|---|---|
| `an_edit_lands_in_both_representations_and_leaves_the_others_alone` | 56건. HWPX 는 ①②, HWP5 는 ②에 새 값. **③ 레거시와 ④ EMF 는 바이트 그대로** |
| `writing_the_current_values_back_changes_nothing` | 56건. 현재 값을 되쓰면 `changedCount == 0`, `wrote == []`, **전 슬롯 바이트 동일** |
| `the_edit_survives_save_and_reparse` | 3종(막대·원형·분산형). 저장→재파스 후에도 ①②에 새 값 |
| `every_refusal_writes_nothing` | 거부 5종(`seriesCountMismatch`·`valueCountMismatch`·`notANumber`·`seriesNameMismatch`·`categoryMismatch`) 각각에서 **바이트 무변경** |
| `dry_run_reports_the_diff_without_writing` | diff 만 내고 무기록 |
| `scatter_x_values_are_editable` | 분산형 X 편집 — 계열이 X 를 공유하므로 두 칸이 함께 바뀐다 |

### 2-2. 게이트

```text
issue_4100_chart_data_edit                   24 passed
cargo fmt --check                            Diff in 0건
cargo clippy --all-targets -- -D warnings    exit 0
```

## 3. 수용 기준 4 — 미리 쟀고, 다시 재도록 못박았다

### 3-1. 측정

`devel` 로는 잴 수 없다. 편집 없이 변환만 해도 변환본의 차트가 **0개**다(직접 확인:
`RenderError("차트 순번 1 범위 초과 (차트 0개)")`) — 그게 #4099 증상이라 새 값이든 옛 값이든
읽을 대상이 없다. 그래서 T4 는 `#[ignore]` 다. 사유를 추정으로 적지 않고 실행해 확인했다.

계획서 R6 이 허용한 경로대로 임시 워크트리에서 쟀다.

```text
워크트리 = task4100(5d8b7e91c) + merge origin/task4099(e34e6d8b1) + Stage 4 패치
머지 충돌 0건 — 두 작업이 건드리는 파일이 겹치지 않는다
test the_edit_survives_conversion_to_hwp5 ... ok
```

3종(묶은세로막대형·쪼개진원형·직선이있는분산형) 전건:

| | 무엇 | 결과 |
|---|---|---|
| 본 시험 | ①② 함께 기록 → HWP 변환 → 재파스 | **새 값 `91.7`** |
| **대조군** | **①만** 고침 → HWP 변환 → 재파스 | **옛 값 그대로** |

변환본에는 ①이 없으므로 재파스가 읽은 값이 곧 ② 바이트다. 대조군이 옛 값이라는 것이
**"②를 함께 써야 한다"를 추정에서 실측으로 바꾼다** — 이 작업 설계의 근간이다.

### 3-0. 그 뒤 — 착지본 기준으로 다시 쟀다 (2026-08-11)

아래 §3-2 가 경고한 그대로 됐다. **PR #4499 는 머지되지 않고 CLOSED 됐고**, 메인테이너가
다른 SHA 로 재착지시켰다(PR #4571, devel `e6a01730d`). 회수 게이트가 리베이스 직후 실패하며
재측정을 지시했고, `#[ignore]` 를 떼고 게이트를 지우고 **착지본 기준으로 다시 재서 통과**했다.
아래 워크트리 측정은 그 과정의 기록으로 남긴다 — 결론은 같았지만 그것을 **확인한** 것이 값이다.

### 3-2. 이 측정은 유통기한이 있다

잰 대상은 **지금의 `origin/task4099 = e34e6d8b1`** 이지 실제로 `devel` 에 들어올 것이 아니다.
이 저장소는 기여자 PR 을 닫고 메인테이너가 보정해 재착지시키는 관행이 있다 — 바로 이 작업의
선행인 #4097 이 그랬다(PR #4144 CLOSED → PR #4171 MERGED, 통합 중 CFB v4 오프셋 보정 **추가**).
스쿼시·체리픽·설계 보정 중 무엇이 오든 fold 가 달라질 수 있다.

### 3-3. 그래서 게이트가 스스로 깨어난다

잠든 `#[ignore]` 는 조용히 잊힌다. 그래서 조건이 사라지는 순간 시끄럽게 깨우는 짝을 뒀다 —
`the_conversion_gate_wakes_itself_when_4099_lands`. 편집 없이 변환만 해서 **차트가 살아남으면
실패**하며 "T4 의 `#[ignore]` 를 떼고 들어온 코드 기준으로 다시 재라"고 알린다.

신호가 "#4099 가 머지됐는가"가 아니라 **"변환이 차트를 보존하는가"라는 관측**이라, 머지든
스쿼시든 체리픽이든 통합 중 설계 보정이든 형태를 가리지 않는다.

양쪽 다 실제로 확인했다 — `devel` 에서는 조용하고(ok), #4099 를 얹은 워크트리에서는 위 문구를
띄우며 실패한다. 이 저장소가 결함을 테스트로 못박는 관행(#4097 이전의
`mini_cfb_repack_drops_the_ole_class_id`)과 같은 모양이다.

## 4. 다음 (Stage 5)

`chart-to-csv` / `csv-to-chart` + 배선 4곳. CSV→행렬 변환만 하면 되도록 검증은 이미 코어에
있다. `sharedXRequired` 도 코어에 붙어 있다.
