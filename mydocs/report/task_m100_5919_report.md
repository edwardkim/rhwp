# 완료 보고서 — Task M100-5919

- 이슈: [#5919](https://github.com/edwardkim/rhwp/issues/5919)
- 제목: 부동 overlay ColumnDef 구분자의 vpos=0 이 억제한 단나누기를 되살려 허위 쪽 — 74312 별지 12쪽이 두 쪽으로 갈림 (19p vs 정본 18p)
- 대상 문서: `samples/hwpx/issue2019_floating_form_74312.hwpx`
- 정본: `pdf/issue2019/issue2019_floating_form_74312-2020.pdf` (한글 2020, 18쪽)
- 작성일: 2026-08-24
- 브랜치: `fix/columndef-overlay-false-page-5919` (기준 `upstream/devel` = `ad28677080`)

## 1. 결함

신구조문대비표 묶음 `pi=260..276` 은 앞뒤가 모두 부동 overlay 앵커이고 단 수가
불변(2단)인 ColumnDef-only 구분자(`pi=267`, `pi=271`)를 포함한다.
`[#2019 v3] overlay_columndef_separator_break` 가 이 문단들의 명시 단나누기를
억제하는데, 바로 뒤의 `[Task #321]` 저장 vpos 리셋 경로가 같은 문단을 다시
잡는다. 구분자의 lineseg `vertpos="0"`(미설정 마커값)이 다단 Normal 트리거
`cv < pv && pv > 5000` 를 만족해(`cv=0, pv=39035`) `advance_column_or_new_page()`
가 두 번 불리고, 두 번째에서 `col_count=2` 를 넘겨 허위 쪽이 만들어진다.

- rhwp 0기준 12쪽: 표 격자만 있고 아래 절반 빈 쪽 (꼬리말 없음)
- rhwp 0기준 13쪽: 격자 없는 백지에 본문 글상자 + `- 12 -` 꼬리말
- 정본 12쪽: 이 둘을 한 쪽에 담는다. 총 쪽수 19 vs 정본 18.

## 2. 변경

`src/renderer/typeset.rs` 한 파일, 트리거 게이트 한 곳.

- Task #321 vpos-reset 트리거의 최종 게이트에
  `!overlay_columndef_separator_break` 를 추가했다.
  - `[#2019 v3]` 로 명시 단나누기를 억제하기로 한 문단이면 저장 vpos 리셋으로도
    단/쪽 경계를 읽지 않는다는 것이 계약의 전문이다. 억제 경로(단나누기)와
    재개 경로(저장 vpos)가 같은 문단을 정반대로 판정하던 모순을 제거한다.
  - `empty_columndef_only_break`(단일 단)·부동 앵커 억제 등 나머지 억제 사유는
    손대지 않았다 — 이슈 재현 묶음은 `overlay_columndef_separator_break`
    경로만 타고, 다른 억제 사유의 vpos 의미는 이 이슈로 검증된 바 없다.

## 3. 전/후 수치

| 항목 | 수정 전 | 수정 후 | 한글 2020 정본 |
|---|---|---|---|
| 총 쪽수 | 19 | **18** | 18 |
| 12쪽(1기준) | 격자만 있고 하단 빈 쪽 + 백지 본문 쪽으로 분리 | 격자 + 본문 한 쪽 | 격자 + 본문 한 쪽 |
| `- 12 -` 꼬리말 위치 | 14쪽 | 13쪽 | 13쪽 |
| `render_page_samples.tsv` delta | +1 | **0** | — |

전/후 비교 이미지: `mydocs/report/edit_demo_5919/issue2019_p13_before_after.png`
(왼쪽 수정 전 13쪽 = 격자만, 오른쪽 수정 후 13쪽 = 정본과 같이 격자 + 본문 + `-12-`)

## 4. 검증

| 게이트 | 결과 |
|---|---|
| `tools/render_page_gate.py` (259건) 수정 전 | 249/259 일치(96.1%), +1 6건 |
| `tools/render_page_gate.py` (259건) 수정 후 | **250/259 일치(96.5%)**, +1 5건 |
| 게이트 A/B diff | 변한 문서는 대상 1건뿐 (`issue2019 19→18`), 신규 이탈 0 |
| `cargo test --profile release-test --lib -p rhwp` | 3889 passed / 0 failed / 13 ignored |
| regression_suite 004/007/009/012/013/021 | 767 passed / 0 failed (004=issue_2019 소속, 007/009/012=저장 vpos 리셋 핀, 013/021=페이지네이션 민감 편람) |
| `cargo clippy --all-targets -- -D warnings` | exit 0 |
| `cargo fmt --all -- --check` / `rustfmt --edition 2021 --check` (변경 파일) | exit 0 |
| `node scripts/rust-unit-test-tiers.mjs --check --base-ref upstream/devel` | 4221 tests 정합 |

### 핀 갱신 (전부 공개)

| 테스트/픽스처 | 갱신 | 근거 |
|---|---|---|
| `tests/issue_2019_floating_form_overpagination.rs` | `pages <= 20` 완화 핀 → `assert_eq!(pages, 18)` | 한글 2020 정본 PDF 18쪽과 정확히 일치. 부분 완화 상한을 정본 오라클로 강화 |
| `tests/fixtures/render_page_samples.tsv` | `18/19/1` → `18/18/0` | 게이트 재실행 결과, 오라클(hangul_pages=18)과 일치 |

## 5. 재현 명령

```
rhwp info samples/hwpx/issue2019_floating_form_74312.hwpx     # 페이지 수: 18 (수정 전 19)
rhwp export-svg samples/hwpx/issue2019_floating_form_74312.hwpx -o out/
python tools/render_page_gate.py --root . --exe <바이너리> --fixture tests/fixtures/render_page_samples.tsv
```
