# #2019 Stage3 — v3 재착수, 기준 PDF 보존, false confidence 정정

- 브랜치: `task/2019-absolute-extent-pagination`
- 목적: PR #2035 검토로 확인된 "18쪽이지만 PI-page/시각 정합 미완" 상태를 정정하고, 근본 개선 착수 기준선을 만든다.

## 1. 정정

- `task_m100_2019_report.md` 상단에 2026-07-08 정정 절을 추가했다.
- `tests/issue_2019_floating_form_overpagination.rs` 는 한컴 정합 테스트가 아니라 81쪽 산란 재발 방지 smoke guard 로 재정의했다.
- `src/renderer/layout.rs`, `src/renderer/typeset.rs` 의 #2019 주석도 "부분 완화"와 v3 TODO 로 고쳤다. 동작 변경은 없다.
- 신규 계획서 `mydocs/plans/task_m100_2019_impl_v3.md` 는 `task/2019-repursue` 의 교훈을 반영해 "zero footprint" 가 아니라 Paper 앵커 절대 object extent 기반 page-local pagination 을 목표로 삼는다.

## 2. 한컴 PDF 기준 보존

HWP 2020 MCP 로 재현 fixture 를 PDF 변환해 `pdf/issue2019/` 에 보존했다.
이 PDF 가 #2019 v3 의 기준이다. 페이지 수나 PI-page map 은 보조 지표이며, 수용 판단은 이 PDF 와 rhwp
렌더 결과의 페이지별 시각 대조로 한다.

| 항목 | 값 |
|------|----|
| 원본 | `samples/hwpx/issue2019_floating_form_74312.hwpx` |
| 원본 sha256 | `092651a9cd41b9ece41abf0b4431a0e5c0a07d811ebb88375d208a1c92ba3d54` |
| PDF | `pdf/issue2019/issue2019_floating_form_74312-2020.pdf` |
| pages | 18 |
| sha256 | `6730453913d087ef152291e0b0b8de97217a1b6f80a00a3184a2d5fc43091bf2` |
| MCP validation | `ok` |

MCP 서버 URL/IP 와 토큰은 공개 문서에 기록하지 않았다.
#2035 에서 이 HWPX fixture 삭제가 제안되었으나, #2045 에서는 같은 원본을 기준 샘플로 유지한다.

## 3. v3 구현 결과

재빌드 후 `task/2019-absolute-extent-pagination` 의 v3 보정은 18쪽을 유지한다.
PDF 기준 시각 검증에서 문제가 된 5쪽 Paper 기준 Square 표와 본문 겹침도 해소했다.

```text
target/debug/rhwp dump-pages samples/hwpx/issue2019_floating_form_74312.hwpx
→ 18페이지
```

최신 검증:

- `cargo build --bin rhwp`
- `CARGO_INCREMENTAL=0 cargo test --test issue_2019_floating_form_overpagination -- --nocapture`
- `cargo test --profile release-test --test visual_roundtrip_baseline visual_baseline_all_samples -- --nocapture`
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`
- `target/debug/rhwp render-diff samples/hwpx/issue2019_floating_form_74312.hwpx --via hwpx` → PASS, 18→18, 구조 불일치 0, 최대 변위 0px
- visual sweep: `output/task2019_v3_visual_scaffold_skip2/`
- result: SVG/PDF/render-tree 18/18/18, `flagged=0/18`
- p5 좌표: `pi78` bottom 279.5px, table top 281.8px → 겹침 없음

보조 산출:

- `output/task2019_v3_visual_scaffold_skip2/summary.json`
- `output/task2019_v3_visual_scaffold_skip2/issue2019/review_contact_sheet.png`
- `output/task2019_v3_visual_scaffold_skip2/issue2019/compare/compare_005.png`
- `output/poc/task2019/hwp_paramap.py` — Windows+한컴+pyhwpx paramap 재현 도구

현재 rhwp PI 범위:

| page | pi range |
|------|----------|
| 1 | 0-19 |
| 2 | 20-37 |
| 3 | 38-48 |
| 4 | 49-72 |
| 5 | 73-79 |
| 6 | 80-94 |
| 7 | 95-127 |
| 8 | 128-137 |
| 9 | 138-154 |
| 10 | 155-186 |
| 11 | 187-223 |
| 12 | 224-259 |
| 13 | 260-276 |
| 14 | 277-288 |
| 15 | 289-321 |
| 16 | 322-351 |
| 17 | 352-389 |
| 18 | 390-396 |

## 4. visual sweep 기준선과 개선 결과

초기 v3 기준선:

```text
python3 scripts/task1274_visual_sweep.py \
  --key issue2019 \
  --hwp samples/hwpx/issue2019_floating_form_74312.hwpx \
  --pdf pdf/issue2019/issue2019_floating_form_74312-2020.pdf \
  --out output/task2019_v3_visual
```

결과:

- SVG/PDF pages: 18/18
- flagged: 8/18
- large drift: `[6, 7, 10, 11, 14, 15, 16]`
- tail overflow 후보: `[5, 6, 7, 10, 11, 14, 15, 16]`
- 대표 산출:
  - `output/task2019_v3_visual/issue2019/review/review_007.png`
  - `output/task2019_v3_visual/issue2019/review/review_008.png`

대표 확인:

- page 7: rhwp 는 큰 프레임/선과 하단 일부만 보이고, PDF 기준은 같은 페이지에 변경사항/신청서/처리절차 텍스트와 하단 회색 bar 가 함께 보인다.
- page 8: `신ㆍ구조문대비표` 프레임과 하단 라벨 위치가 PDF 기준과 크게 다르다.

최신 v3 결과:

```text
python3 scripts/task1274_visual_sweep.py \
  --key issue2019 \
  --hwp samples/hwpx/issue2019_floating_form_74312.hwpx \
  --pdf pdf/issue2019/issue2019_floating_form_74312-2020.pdf \
  --out output/task2019_v3_visual_scaffold_skip2
```

결과:

- SVG/PDF/render-tree pages: 18/18/18
- flagged: 0/18
- 대표 육안 확인: `compare_005.png` 에서 5쪽 본문과 Paper 기준 Square 표 상단 겹침 없음
- 잔여 차이: 한컴 PDF와 rhwp의 글꼴/래스터 굵기 차이. 페이지 귀속·큰 표/본문 흐름 판단에는 영향 없음

## 5. 구현 요약

1. Paper/Page 기준 비-TAC 글상자 내부 stored vpos 에서 object origin 을 빼서 이중 vertical offset 을 제거했다.
2. 단나누기 뒤 Paper overlay tail 이 page-local body 하단을 넘는 경우에만 다음 page 로 넘기고, 부동 overlay 사이 ColumnDef-only 구분자는 컬럼 진행에서 제외했다.
3. 빈 앵커에 매달린 Paper/Page 기준 Square 표는 선언 y 에 절대 배치하고 flow cursor 를 전진시키지 않는다.
4. 해당 절대 Square 표 앞뒤의 쪽나누기/ColumnDef/Bookmark, ColumnDef-only 구분자 스캐폴드는 PDF 기준으로 별도 빈 줄을 렌더하지 않는다.
