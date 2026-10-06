# 완료 보고서 — Task M100-5920

- 이슈: [#5920](https://github.com/edwardkim/rhwp/issues/5920)
- 제목: 쪽 하단 중첩 표가 상자 아래 보이지 않는 이송 여백 때문에 다음 쪽으로 밀린다
- 대상 문서: `samples/issue3637/press_release_split_cell_nested_table.hwpx`
- 정본: `pdf/issue3637/press_release_split_cell_nested_table-2020.pdf` (한글 2020)
- 작성일: 2026-08-23
- 브랜치: `fix/issue3637-nested-split` (기준 `origin/devel` = `b9eb55107`)

## 1. 결함

문서 본문 전체가 3행 1열 `RowBreak` 표의 한 셀 안에 들어 있고, 8쪽의 두 상자는
그 셀 안의 중첩 표(문단당 1개, 분할 불가 atom 유닛)다.

정본 8쪽은 `< 은행의 위탁보증 포트폴리오 구성(가상사례) >` 상자와 그 아래
`☞ 그동안 지속적인 노력에도 …` 결론 상자를 **함께** 담는다.
rhwp 는 결론 상자를 9쪽으로 밀어 **8쪽 아래 절반이 통째로 비었다.**

### 잘린 지점의 산식

`advance_row_cut_inner()` 의 예산 판정
(`if j > start && h + u.height > avail_height { break; }`) 에 진단을 걸어 실측했다.

```
DIAG r=1 c=0 start=175 j=177 h=628.787 uh=369.507 avail=997.827 over=0.467
```

- 8쪽 조각이 이미 소비한 높이 `h = 628.787px`
- 다음 atom(결론 상자) 유닛 높이 `u.height = 369.507px`
- 셀에 남은 예산 `avail_height = 997.827px`
- **초과 = 0.467px (0.35pt)**

같은 셀의 다른 컷 11건의 초과분은 2.36 ~ 574.99px 였다. 0.467px 은 압도적으로 작다.

### 왜 0.467px 인가 — 보이지 않는 꼬리 여백

가시 텍스트 없이 표 control 만 든 문단의 유닛 높이는 `cell_units()` 에서
`max(중첩 표 높이, 줄 이송 높이)` 로 정해진다. 이 문단은 **줄 이송 높이가 더 커서**
상자 아래에 아무것도 그리지 않는 여백이 남는다.

정본에서 그 여백은 쪽 예산에 들어가지 않는다:

```
정본 8쪽 : 앞 내용 하단 508.4pt -> 결론 상자 테두리 512.0 ~ 778.0pt
           본문 하단 785.2pt  (상자 바닥까지 7.2pt 여유)
rhwp 수정전 : 508.4 + 277.13(= 369.507px, 여백 포함) = 785.5pt > 785.2pt -> 9쪽
```

즉 한글은 **보이는 상자**가 본문 안에 들어가면 그 쪽에 앉히고, 상자 아래 이송
여백은 쪽 경계 밖으로 흘린다. rhwp 는 그 여백까지 예산에 넣어 판정했다.

저장소에 같은 개념의 부분 계약이 이미 있다 —
`native_multirow_saved_reset_trailing_trim()`("저장 reset 직전 마지막 가시 줄의
trailing 공백을 물리 쪽 경계에서 제외")은 저장 reset 이 뒤따르는 **텍스트 줄**에만
적용되고, 중첩 표 atom 유닛이 조각의 마지막으로 놓이는 경우는 다루지 않았다.

## 2. 변경

`src/renderer/layout/table_layout.rs` 한 파일.

- `LayoutEngine::nested_atom_invisible_tail()` 추가 (`+50`)
  - 가시 텍스트 없이 표 control 만 든 문단의 유닛에서
    `unit.height - Σ calc_nested_table_height(t)` = **상자 아래 보이지 않는 여백**.
  - 다음은 모두 `0.0` 을 돌려주어 기존 판정을 그대로 둔다 —
    빈 spacer 유닛 / 분할된 중첩 행(`nested_row`) / 중첩 조각
    (`nested_table_fragment`, `mixed_nested_fragment`, `mixed_nested_trailing`) /
    표와 글자가 섞인 문단(높이가 `line_based_h + nested_h + 4.0` 이라 꼬리가 여백이 아님) /
    표 없는 문단.
- `advance_row_cut_inner()` 예산 정지 직전에 이 여백만큼의 유예를 둔다 (`+10`)
  - `nested_atom_tail > 0.0 && h + u.height - nested_atom_tail <= avail_height`
    이면 유닛을 현재 쪽에 앉힌다.
  - `h` 에는 **가시 높이만** 더한다(`h += (u.height - nested_atom_tail).max(0.0)`)
    — 조각 높이가 본문을 침범하지 않게 하는 부기로,
    `native_multirow_saved_reset_trailing_trim` 경로와 같은 방식이다.

`advance_row_block_cut` (블록 컷) 경로에는 손대지 않았다. 이 문서는 그 경로를
타지 않으며(진단으로 확인: `DIAG_BLKCUT` 0건), 근거 없는 확대를 피했다.

## 3. 전/후 수치

| 항목 | 수정 전 | 수정 후 | 한글 2020 정본 |
|---|---|---|---|
| 8쪽 내용 | 포트폴리오 상자만 | 포트폴리오 상자 + 결론 상자 | 포트폴리오 상자 + 결론 상자 |
| 8쪽 마지막 본문 baseline | 491.9pt | 767.4pt | 772.0pt |
| 8쪽 셀 조각 하단 | 508.4pt | 785.6pt (= 본문 하단) | — |
| 결론 상자 테두리 | **9쪽** 36.8 ~ 302.2pt | **8쪽** 508.4 ~ 773.7pt | **8쪽** 512.0 ~ 778.0pt |
| 상자 바닥 ~ 본문 하단 | — | 11.9pt 여유 (쪽 안) | 7.2pt 여유 |
| 정본과 쪽 정합 구간 | 1~7쪽 | **1~8쪽** | — |
| 총 쪽수 | 13 | 13 | 12 |

전/후 비교 이미지: `mydocs/report/edit_demo_5920/page8_before_after.png`

### 총 쪽수가 그대로인 이유 (남은 결함)

8쪽 정합은 회복됐지만 총 쪽수는 13 그대로다. **9쪽에 별개의 결함**이 있다 —
정본 9쪽은 `☞ 각 보증기관은 … 이해도를 바탕으로 보증 제공 가능`(baseline 744.0pt)
까지 담는데, rhwp 9쪽은 본문이 더 높게 쌓여 세 줄 앞
(`- (개인사업자) … 성장`, baseline 768.0pt)에서 쪽이 찬다.
rhwp 9쪽 셀 조각 하단은 780.6pt 로 **이미 가득 차 있다** — 컷 판정이 아니라
줄 높이 누적 축의 문제이므로 이번 변경 범위 밖이다.
`tests/fixtures/render_page_samples.tsv` 의 `13` 은 그대로 유효해 갱신하지 않았다.

## 4. 검증

| 게이트 | 결과 |
|---|---|
| `tools/render_page_gate.py` (samples 259건) 전/후 | 245/259 일치(94.6%) — **전/후 산출 TSV 바이트 동일, 회귀 0** |
| `cargo test --profile release-test --test overflow_cell_baseline` | 1 passed — `LAYOUT_OVERFLOW_CELL` 래칫 증가 없음 |
| `cargo test --profile release-test --test regression_suite_016 ir_field_sweep` | 4 passed (`ir_field_sweep` 는 016 으로 재배정돼 있다) |
| `cargo test --profile release-test --test regression_suite_030 issue_3637` | 1 passed — 같은 문서의 기존 계약 `nested_table_snap_stays_inside_the_split_cell` |
| `cargo test --profile release-test --test regression_suite_015 issue_5920` | 2 passed (신규, red→green 실증) |
| `cargo test --profile release-test --lib -p rhwp` | 3893 passed / 0 failed / 13 ignored |
| `rustfmt --edition 2021 --check` (변경 2파일) | exit 0 |
| `cargo clippy --all-targets -- -D warnings` | exit 0 |

### 코퍼스 자기 diff — 의도 외 변화 0

`samples/` **784건 전수**를, 수정 전(`origin/devel` 빌드)·후 두 바이너리로
`export-text --json` 하여 **쪽별 텍스트 해시**를 대조했다. 쪽 경계가 움직이면
반드시 이 해시가 달라진다.

```
DIFF samples/issue3637/press_release_split_cell_nested_table.hwpx pages 13->13 changed_pages=6
DONE docs=784 identical=783 differing=1 errors=3 elapsed=388s
```

**변한 문서는 대상 문서 1건뿐이다** (8쪽에 상자가 들어가며 8~13쪽 내용이 앞당겨짐).
`errors=3` 은 전/후 동일한 기존 파싱 실패 문서다.

### 쪽 밖 글자 직접 계수

대상 문서 13쪽 전부의 SVG `<text y=…>` 중 쪽 높이를 넘는 것:

```
before  pages=13 out_of_page_text_elems=0
after   pages=13 out_of_page_text_elems=0
```

증가 없음(둘 다 0).

### red → green

수정 전 바이너리(`origin/devel` `b9eb55107` 빌드)로 같은 계약을 재면:

```
8쪽 마지막 줄 : "  - 장기간 보증을 이용해 왔으나, … 보증의 축소가 필요한 기업(0.7억원 상환)"
9쪽 첫 줄    : " ☞ 그동안 지속적인 노력에도 불구하고 개선이 어려웠던 한계기업에"
```

`issue_5920_page8_holds_both_nested_boxes` 의 `CONCLUSION_HEAD` 단언과
`issue_5920_page9_does_not_restart_with_the_conclusion_box` 가 둘 다 실패한다.
수정 후 2건 모두 통과.

## 5. 재현 명령

```
rhwp export-text samples/issue3637/press_release_split_cell_nested_table.hwpx -p 7 -o out/
rhwp export-svg  samples/issue3637/press_release_split_cell_nested_table.hwpx -p 7 -o out/
python tools/render_page_gate.py --root . --fixture tests/fixtures/render_page_samples.tsv --exe <바이너리>
```
