# Task 1293 Stage 42: 0/0/0 미주 잔여 overflow 정리

## 배경

Stage41에서 `2024-11-practice-above0-between0-below0` target의 render-tree overflow를
14건에서 6건으로 줄였다. 기존 7mm/8mm 계열 focused target은 0건을 유지했다.

남은 overflow는 다음과 같다.

| page | paragraph | 유형 | overflow |
|---:|---:|---|---:|
| 9 | `pi=510` | FullParagraph line 3 | 32.2px |
| 10 | `pi=537` | Shape | 31.3px |
| 12 | `pi=616` | FullParagraph | 3.2px |
| 14 | `pi=712` | FullParagraph | 3.0px |
| 14 | `pi=713` | FullParagraph | 21.0px |

## 판단

남은 후보는 `0/0/0` profile에서 구분선 주변 여백이 사라졌을 때 저장 `vpos`와 실제 line/object
advance가 얼마나 소비되어야 하는지의 문제다. 기존 7mm 계열을 회귀시키지 않으려면 Stage41처럼
정규화된 미주 모양 profile 조건 안에서만 보정해야 한다.

## 확인 계획

1. `pi=510`은 page 10 오른쪽 단의 긴 마지막 문단 tail이다. PDF와 비교해 전체 문단을 다음
   쪽으로 넘겨야 하는지, 마지막 line만 넘겨야 하는지 확인한다.
2. `pi=537`은 비TAC Shape 문단이다. 0/0/0 profile에서도 현재 단에 남는 것이 맞는지, 아니면
   다음 단 첫 항목으로 보내야 하는지 확인한다.
3. `pi=616`은 TAC 그림 문단의 미세 overflow다. 허용 가능한 bleed인지, 다음 단 분기가 필요한지
   PDF와 compare PNG로 확인한다.
4. `pi=712~713`은 page 14 왼쪽 단 하단 tail이다. 다음 문단 시작 전 saved-vpos/line advance가
   어느 기준으로 소비되는지 dump-pages와 render tree를 대조한다.

## 실험 결과

### `pi=510` own-span overfull 강제 advance

`RHWP_ENDNOTE_FIT_DEBUG=1` 임시 로그로 남은 후보의 fit 판단을 확인했다.

```text
pi=510 cur=885.8 avail=1001.6 raw=135.9 fit=135.9 advfit=141.9 own_span=true adv_fit=false
pi=537 cur=845.6 avail=1001.6 raw=12.0 fit=155.9 advfit=155.9 obj=Some(155.92) own_span=true
pi=616 cur=867.8 avail=1001.6 raw=130.9 fit=130.9 advfit=137.0 own_span=false
pi=712 cur=855.6 avail=1001.6 raw=12.0 fit=12.0 advfit=18.0 own_span=true
pi=713 cur=873.6 avail=1001.6 raw=12.0 fit=12.0 advfit=18.0 own_span=true
```

`pi=510`은 `cur + fit > available`인데도 `own_span=true`라 advance하지 않는다. 이를 근거로
0/0/0 profile에서 `cur + en_fit > available`이면 `compact_endnote_own_vpos_span_fits_for_flow`를
false로 만드는 실험을 했다.

- 실험 산출물: `output/task1293_stage42_probe_ownspan_overfull/summary.json`
- 결과: overflow 6건이 9건으로 증가
- 새 overflow chain: page 14 `pi=705~706`, `pi=753`

따라서 `pi=510`을 단순히 다음 단/쪽으로 넘기는 방향은 한컴/PDF 흐름과 맞지 않는다. 이 후보는
pagination advance 문제가 아니라 renderer의 saved-vpos 하단 기준 또는 overflow 판정 쪽으로
계속 봐야 한다.

### 남은 후보의 현재 판단

- `pi=510`: 넘기면 후속 문항 흐름이 크게 깨진다. 현재 쪽에 남아야 하며, 하단 draw 기준 보정이 필요하다.
- `pi=537`: `cur + object fit`이 거의 available과 같아 pagination은 맞지만, shape y가 31.3px 낮게
  기록된다. 비TAC shape의 render overflow 기준을 별도로 확인해야 한다.
- `pi=616`, `pi=712`, `pi=713`: pagination상 충분히 들어가지만 render overflow 로그가 난다.
  saved-vpos base와 render tree overflow 판정의 차이를 봐야 한다.

## 검증 계획

- `cargo build --bin rhwp`
- `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-above0-between0-below0 --out output/task1293_stage42_zero_profile --rhwp-bin target/debug/rhwp`
- focused 4종 sweep으로 기존 7mm/8mm target 회귀 확인
- `cargo test --lib compact_endnote -- --nocapture`

## 상태

`pi=510`을 단순 advance로 해결하는 방향은 폐기했다. 다음 단계에서는 renderer saved-vpos 하단 기준,
비TAC shape overflow 판정, TAC 그림 미세 bleed 판정을 분리해 본다.
