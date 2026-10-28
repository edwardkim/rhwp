# #5846 처리 결과 — 혼재 문단 셀의 컷 조각이 중첩 표 꼬리 행을 반쪽만 담던 결함

- 이슈: <https://github.com/edwardkim/rhwp/issues/5846>
- 문서: `samples/task2097/75544_pii_bunseok.hwpx`
- 정답지: `pdf/task2097/75544_pii_bunseok-2020.pdf` (한글 2020, 66쪽)
- 변경 파일: `src/renderer/layout/table_layout.rs` (+34/-1)
- 신규 계약 테스트: `tests/cases/issue_5846_mixed_nested_partial_row_duplicate.rs`
  (`regression_suite_030` 배정)

## 1. 증상

59쪽(0-based 58)에서 셀 안 중첩 표의 내용이 부모 셀을 넘어 용지 밖까지 흘렀다.

```
rhwp export-svg samples/task2097/75544_pii_bunseok.hwpx -p 58 -o out
```

| 항목 | 수정 전 | 수정 후 |
| --- | ---: | ---: |
| 59쪽 `<text>` 총수 | 1,083 | 469 |
| 본문 하한 1,046.9px 초과 `<text>` | **549** | **4** (꼬리말 `- 59 -`, 정상) |
| 최하단 `<text>` y | **1,725.6px** | **1,083.6px** |
| 59쪽 `LAYOUT_OVERFLOW_CELL` | 17줄 | **0줄** |
| 문서 전체 `LAYOUT_OVERFLOW_CELL` | 19줄 | 2줄 (§6 잔여) |
| 쪽수 | 66 | 66 (정답지 66) |

용지 밖으로 나간 내용은 유실이 아니라 **중복**이었다. 60쪽이 같은 내용을
이미 온전히 그리고 있었고, 59쪽 스텁에 그려진 사본은 셀 클립에 걸려
`․ 단기연체정보` 한 줄과 뒤따르는 중첩 표들의 테두리 잔재만 상자 닫는 선 위에
겹쳐 보였다.

## 2. 원인

문제 구조는 `pi=527` 의 `1×1 바깥 표 → 셀 → 2행 중첩 표` 다.

`RHWP_DIAG_FRAG` 실측 — 컷 부기 자체는 두 쪽이 정확히 맞물려 있었다.

```
p59: DIAG_FRAG pi=527 ci=0 rows=0..1 cont=false start_cut=[]   end_cut=[45] tbl_h=733.4
p60: DIAG_FRAG pi=527 ci=0 rows=0..1 cont=true  start_cut=[45] end_cut=[]   tbl_h=754.7
```

즉 59쪽은 유닛 45까지, 60쪽은 유닛 45부터다. 틀린 것은 그 컷을 **자식 표의 행
범위로 옮기는 단계**였다.

`LayoutEngine::mixed_nested_split_from_cut`(`src/renderer/layout/table_layout.rs`)
은 혼재 문단(텍스트 + 중첩 표) 셀의 유닛 컷을 자식 표 행 범위로 환산할 때
`calc_nested_split_rows(row_heights, cs, offset, visible)` 를 쓴다. 그 함수의
꼬리 행 탈락 규칙은 이랬다.

```rust
let min_threshold = (last_h * 0.5).min(10.0);
if available_for_last < last_h && available_for_last < min_threshold {
    end_row -= 1;
}
```

슬라이버가 **10px 을 넘으면 부분 행을 그대로 둔다**. 그런데 같은 함수는 연속
조각(`offset > 0`)의 `start_row` 를 **행 처음부터** 다시 그린다
(`offset_within_start = 0`). 두 규칙이 겹치면 컷 조각이 남긴 부분 행은 다음
쪽이 반드시 통째로 재렌더한다 — 구조적으로 중복이 확정된다.

이 문서의 실측값:

- 2행 중첩 표, 행 높이 650.9 / 767.6px, 셀 가용 `nested_h=1419.0`
- 59쪽 가시 `visible = 688.0`
- `row_y[1] = 652.7` → `available_for_last = 688.0 - 652.7 = 35.3px`
- `min_threshold = min(767.6 * 0.5, 10.0) = 10.0` → `35.3 < 10.0` 이 거짓이라 **탈락하지 않음**
- 결과 `end_row = 2` → 행 1 이 **35.3px 스텁**으로 붙고, 그 안의 25문단 전체가 거기 그려짐

render tree 실측(수정 전):

```
Table y=293.8 h=733.9 bot=1027.7          ← 바깥 1×1
  Cell  y=293.8 h=733.4 bot=1027.2
    Table y=337.3 h=688.0 bot=1025.3      ← 2행 중첩 표
      Cell(row=0) y=337.3 h=650.9 bot=988.2
      Cell(row=1) y=988.2 h=37.1  bot=1025.3   ← 35.3px 스텁
        Table y=1023.4 h=51.3 / 51.3 / 68.4 / 51.3 / 85.0   ← 전부 셀 하단으로 클램프돼 겹침
```

`cell-clip-373`(`y=988.17 h=37.09`)이 나머지를 가렸을 뿐, 좌표상 글줄은
`y=1,715.4`(TextRun 윗변)까지 내려가 있었다.

## 3. 수정

`mixed_nested_split_from_cut` 의 `nt.row_count > 1` 분기에서, **비종료
조각**(`!terminal`, 즉 뒤에 연속 조각이 있는 경우)의 꼬리 행이 가시 창에
일부만 걸치면 그 행을 이 조각에서 뺀다.

```rust
if !terminal && rows.end_row > rows.start_row + 1 {
    let last = rows.end_row - 1;
    // ... last_top 누적 (calc_nested_split_rows 와 동일 산식)
    let available_for_last = offset + visible - last_top;
    if available_for_last + 0.5 < row_heights[last] {
        rows.end_row = last;
    }
}
```

설계 근거:

- **온전한 행이 최소 하나 남을 때만** 뺀다(`end_row > start_row + 1`). 조각이
  통째로 비면 행 이월이 무한히 미뤄질 수 있다.
- **높이 필드는 건드리지 않는다.** `visible_height` / `flow_height` 는 유닛
  회계 값을 그대로 둔다. 조각 경계와 부모 flow 소비는 이미 페이지네이터의
  `RowCut` 이 정했고, 이 환산은 "그 조각이 어느 행들을 담는가" 만 정한다
  (원 주석의 계약 그대로).
- `terminal` 조각은 손대지 않으므로 #3658 의 `shown_band` 경로는 불변이다.
- `recursive_cut` 이 있는 경로(#4069)는 앞선 `match` 팔에서 갈라지므로 불변이다.

효과: `split=(start_row 0, end_row 2)` → `(0, 1)`.

## 4. red → green

`tests/cases/issue_5846_mixed_nested_partial_row_duplicate.rs` 는 세 가지를 본다.

1. 쪽수가 정답지와 같은 66인가 (전제)
2. 59쪽 **Body 안**에 본문 하한을 넘은 `TextRun` 이 하나도 없는가 (본체 판정)
3. 60쪽 첫 항목 `단기연체정보` 가 59쪽에 **없고** 60쪽에 **있는가** (정답지 대조 + 유실 방지)

수정만 되돌리고(`git stash push -- src/renderer/layout/table_layout.rs`) 같은
테스트를 돌린 실제 출력:

```
thread '...issue_5846_cut_fragment_defers_partial_nested_tail_row' panicked at
tests\generated\..\cases\issue_5846_mixed_nested_partial_row_duplicate.rs:99:5:
#5846 회귀 — 59쪽 본문 하한 1046.9px 밖 TextRun 45개 (최하단 y=1715.4px)

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 124 filtered out
```

수정을 복원한 뒤:

```
test issue_5846_mixed_nested_partial_row_duplicate::issue_5846_cut_fragment_defers_partial_nested_tail_row ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 124 filtered out
```

(SVG `<text>` 549개는 글리프 단위, 테스트의 `TextRun` 45개는 줄·글자모양
단위라 세는 단위가 다르다. 같은 내용을 다른 층에서 잰 값이다.)

## 5. 검증 게이트

| 게이트 | 명령 | 결과 |
| --- | --- | --- |
| lib 유닛 | `cargo test --lib -p rhwp` | **ok — 3,893 passed / 0 failed / 13 ignored** |
| 신규 계약 | `cargo test --test regression_suite_030 issue_5846` | **ok — 1 passed** |
| 중첩 표 회귀 6스위트 | `regression_suite_{005,015,018,020,025,030}` | 760 중 **759 passed**, 1 실패는 사전 실패(§6) |
| 259문서 쪽수 게이트 | `python tools/render_page_gate.py --root . --fixture tests/fixtures/render_page_samples.tsv` | 수정 전/후 모두 **245/259 일치(94.6%)**, 저장 TSV **문서별 전건 동일 → 회귀 0** |
| clippy | `cargo clippy --all-targets -- -D warnings` | **exit 0** |
| unit tier 래칫 | `node scripts/rust-unit-test-tiers.mjs --check --base-ref origin/devel` | **확인 완료 — 4,225 tests / 299 modules** |
| rustfmt | `rustfmt --edition 2021 --check <LF 사본>` | 변경 2파일 **차이 없음** |

259문서 게이트는 `--save` 로 문서별 결과를 남겨 `diff` 로 비교했다.

```
diff output/_s5846/gate_before.tsv output/_s5846/gate_after.tsv
→ 차이 없음
```

쪽수 분포도 전후 동일하다: `-6:1, -3:1, -2:2, +0:245, +1:8, +2:2`.
이 수정은 **페이지네이션이 아니라 그리기 범위만** 바꾸므로 쪽수가 움직이지
않는 것이 기대 동작이다.

### 문서 전체 부작용 점검

- 66쪽 전량 SVG 재생성 → **59쪽 하나만 달라짐**, 나머지 65쪽 바이트 동일
- 60쪽 SVG는 **바이트 동일** (연속 조각 불변)
- `export-text` 66쪽 비교 → 59쪽만 89줄 감소, 그 89줄은 전부 60쪽이 이미 갖고 있는 중복
- 59쪽 래스터 픽셀 diff: 변경 4,096px, bbox `(123, 927, 983, 1436)` — 전/후가 실제로 다르다

## 6. 남은 것 (이 PR 범위 밖)

- `wmf_emf_goldens::wmf_emf_goldens_lock_current_engine` (`regression_suite_005`)
  는 **사전 실패**다. 이 수정을 되돌린 base 에서 같은 명령으로 동일하게
  실패하는 것을 확인했다. 본 변경과 무관하다.
- 같은 문서에 `LAYOUT_OVERFLOW_CELL` 2줄이 남는다 — `pi=43` (19.9px 초과),
  `pi=71` (499.4px 초과). 둘 다 59쪽·`pi=527` 과 다른 위치의 별건이라 이
  이슈에서 손대지 않았다. 별도 이슈로 다룰 값이다.
- `calc_nested_split_rows` 의 `min(last_h*0.5, 10.0)` 규칙 자체는 다른 호출부
  (`table_partial.rs` 의 `available_h` 휴리스틱 경로)에 그대로 남아 있다. 이
  PR 은 원인이 실증된 혼재 문단 경로만 좁게 고쳤다.

## 7. 전/후 스크린샷

- 쪽 하단부 3단 비교: `mydocs/report/edit_demo_5846/issue_5846_p59_before_after_oracle.png`
- 결함 밴드 확대 3단: `mydocs/report/edit_demo_5846/issue_5846_p59_zoom_bottom_band.png`

각각 위에서부터 수정 전(rhwp devel) / 수정 후(본 PR) / 한글 2020 정본이며,
같은 문서 좌표 구간을 잘라 배치했다.
