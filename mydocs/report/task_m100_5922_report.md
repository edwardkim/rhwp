# 완료 보고서 — Task M100-5922

- 이슈: [#5922](https://github.com/edwardkim/rhwp/issues/5922)
- 제목: CellBreak 자리차지 표의 연속 조각이 표 바깥 여백(0.5mm 상·하)을 다시 열지 않는다 — 쪽마다 3.2pt 과적재, 화성시 별표2 162쪽 vs 159쪽
- 대상 문서: `samples/issue2063_huge_cellbreak_table.hwp` (화성시 사무전결 처리규칙 [별표 2], 5,277행×10열)
- 정본: `pdf/issue2063_huge_cellbreak_table-2020.pdf` (한글 2020, 162쪽)
- 작성일: 2026-08-24
- 브랜치: `fix/cellbreak-margin-reopen-5922` (기준 `upstream/devel` = `ad28677080`)

## 1. 결함

연속 조각의 바깥 여백 재개방이 RowBreak 전용 좁은 계약(#2439 native 서명)으로만
구현돼 있었다. CellBreak 조각은 두 경로 모두에서 여백 0이라 본문 영역 맨 위에
붙고 본문 맨 아래까지 채워, 쪽마다 3.2pt 넓은 세로 예산으로 한글이 넘기는 행 한
줄을 더 밀어 넣었다(누적 −3쪽).

## 2. 변경

4 파일.

- `src/renderer/float_placement.rs`
  - `native_empty_host_cellbreak_fragment_repeats_outer_margin()` 추가 —
    native HWP5, 비-TAC TopAndBottom(vert=문단), 쪽나눔=CellBreak, 빈 host
    문단의 구조 게이트. #2439 처럼 저장 ladder 증거를 요구할 수 없다(거대 표의
    저장 vpos ladder 는 표 높이를 접음 — 본 문서 host ls vpos=5440 다음 문단은
    빈 문단이라 `next_vpos − host_vpos == advance` 검증 자체가 성립하지 않는다).
    대신 구조를 #2439 보다 좁힌다: CellBreak + 빈 host 한정. 여백 0 표는 no-op.
- `src/renderer/typeset.rs`
  - `partial_rowbreak_fragment_spacing_px()` — `repeats_outer_margin` 을
    쪽나눔별 게이트로 분리(RowBreak=#2439 플래그, CellBreak=신규 플래그,
    None=불가). 연속/중간/마지막 조각의 조판 예산(`page_avail`, `current_height`
    누계)이 `outer_margin_top`+`outer_margin_bottom` 을 다시 예약한다.
- `src/renderer/layout.rs`, `src/renderer/layout/table_partial.rs`
  - `repeat_fragment_outer_margin` 플래그 산출에 신규 게이트를 OR — 조각 그리기
    원점이 직전 조각 하단+바깥여백 위에 자기 `outer_margin_top` 을 다시 열고,
    흐름 복귀 y 에 `outer_margin_bottom` 을 더한다(콘텐츠 하단 판정 제외 —
    기존 #2439 경로와 동일 취급).

첫 조각의 상단 여백은 기존에도 host spacing 으로 계상되므로(자리차지 `outer_top`),
변경의 실질 대상은 **연속 조각의 상·하 재개방**이다.

## 3. 전/후 수치

| 항목 | 수정 전 | 수정 후 | 한글 2020 정본 |
|---|---|---|---|
| 총 쪽수 | 159 | **161** | 162 |
| 연속 쪽 표 상단 y | 42.74pt (전 쪽 동일) | **43.92pt** | 44.00pt |
| 표 하단 y 최댓값 | 554.93pt | **553.22pt** | 553.00pt |
| 연속 쪽 세로 span 최댓값 | 512.20pt | **509.30pt** | 509.00pt |
| `render_page_samples.tsv` delta | −3 | **−1** | — |

정본 PDF 좌표는 1pt 정수 반올림(±0.5pt 불확도)이라 상단·하단·span 모두
불확도 안의 정합이다. 쪽당 과적재 3.2pt 는 소멸했다.

### 잔여 −1 에 관하여

여백 축(본 이슈의 대상)은 해소됐고, 남은 −1 은 행 경계 sub-pt 적산 축이다 —
정본 괘선 좌표가 1pt 반올림인 반해 rhwp 예산은 무반올림이라, 행 피치(1282/1500HU
혼합) 누적 소수부가 쪽 경계 근방에 놓인 쪽에서 한 행 차이가 난다. 오라클
불확도(±0.5pt) 아래의 축이라 이번 변경 범위 밖으로 남기고, 핀과 픽스처에 근거를
기록했다.

전/후 비교 이미지: `mydocs/report/edit_demo_5922/issue2063_p50_before_after.png`
(위 수정 전 = 표가 본문 상단에 밀착, 아래 수정 후 = 상단 바깥 여백 재개방)

## 4. 검증

| 게이트 | 결과 |
|---|---|
| `tools/render_page_gate.py` (259건) 수정 전 | 249/259 일치(96.1%) |
| `tools/render_page_gate.py` (259건) 수정 후 | 249/259 일치(96.1%) — 분포 +1 6·+2 1·−1~−3 3 동일, 대상 문서만 −3→−1 |
| 게이트 A/B diff | 변한 문서는 대상 1건뿐, 신규 이탈 0 |
| `cargo test --profile release-test --lib -p rhwp` | 3889 passed / 0 failed / 13 ignored |
| regression_suite 013/021/026/028/031 | 639 passed / 0 failed (026=issue_1842 CellBreak, 028=issue_2070 밀도 핀, 031=issue_2063 원문, 013/021=페이지네이션 민감 편람) |
| `cargo clippy --all-targets -- -D warnings` | exit 0 |
| `cargo fmt --all -- --check` / `rustfmt --edition 2021 --check` (변경 파일) | exit 0 |
| `node scripts/rust-unit-test-tiers.mjs --check --base-ref upstream/devel` | 4221 tests 정합 (source-side test 총량 불변 — 신규 화이트박스 테스트는 정책상 금지라 통합 핀으로 검증) |

### 핀 갱신 (전부 공개)

| 테스트/픽스처 | 갱신 | 근거 |
|---|---|---|
| `tests/issue_2070_rowbreak_density.rs::huge_cellbreak_table_pin` | 159 → **161** | 파일 머리말의 "잠정 핀은 잔여 축 해소 시 기준 PDF 값으로 복귀" 축 중 여백 축 해소. 연속 쪽 기하 3축(상단/하단/span)이 정본 ±0.5pt 안. 잔여 −1 은 행 경계 sub-pt 적산 축으로 별도 기록 |
| `tests/fixtures/render_page_samples.tsv` | `162/159/−3` → `162/161/−1` | 게이트 재실행 결과 |

## 5. 재현 명령

```
rhwp info samples/issue2063_huge_cellbreak_table.hwp            # 페이지 수: 161 (수정 전 159, 정본 162)
rhwp export-svg samples/issue2063_huge_cellbreak_table.hwp -o out/
python tools/render_page_gate.py --root . --exe <바이너리> --fixture tests/fixtures/render_page_samples.tsv
```
