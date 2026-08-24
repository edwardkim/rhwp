---
kind: working
status: done
canonical: mydocs/working/task_m100_5652_stage5.md
last_verified: 2026-08-23
---

# #5652 Stage 5 — 엔진 산출 회귀 · 한컴 판정 번들 · (판정 후) 자산·원장·보고서

- **계획서**: [`mydocs/plans/task_m100_5652.md`](../plans/task_m100_5652.md) §5·§8
- **브랜치**: `task5652` (`upstream/devel` `bf30bd792` 기준)
- **번들**: `output/issue_5652_b2_engine_judgment/` (gitignored) — 32 파일 + `PANJEONG.md`
- **생성기**: `tests/issue_4100_chart_data_edit.rs::generate_b2_engine_judgment_bundle` (`#[ignore]`)

## 1. 무엇을 만들었나 (S5-a, `tests/issue_4100_chart_data_edit.rs`)

| 항목 | 내용 |
|---|---|
| `b2_engine_edits(core, stem, label)` | #5447 변종 카탈로그(`b2_variants`) 14종 → 엔진 편집 입력(`structure:true` 행렬). 경계 2종(원형 계열추가·주식형 계열삭제)은 `None` — 가드가 막는다 |
| `b2_engine_write_variant(out_dir?, v, sheet)` | 변종 1건을 엔진으로 양 포맷 생성 + 4단 자기검증(재개방·`v.check`·①==②·③④ 불변). `out_dir: None` 이면 상시 회귀 |
| `b2_engine_row_and_series_edits_render_after_reopen` | 행추가(「추가항목」 렌더)·행삭제(「항목 2」 부재)·계열추가(「추가계열」 범례) — 엔진 산출이 재렌더에 반영 |
| `b2_engine_output_passes_the_scanner_for_every_variant` | 12변종 × 2포맷 상시 자기검증 |
| **`engine_documents_match_spike_documents_except_positional_series_delete`** | 엔진 경로와 스파이크 경로(문자열 수술 + 주입)를 **같은 현재 라이터**로 저장하면 **문서 바이트** 가 10변종 × 2포맷 = 20건 동일, 계열삭제 2종 × 2 = 4건은 바이트는 다르되 논리(이름·라벨·값) 동일 |
| `generate_b2_engine_judgment_bundle` (`#[ignore]`) | `output/issue_5652_b2_engine_judgment/` — 대조군 7 + 변종 12 × 2포맷 + HWPX→HWP 변환본 1 = 32 파일 + `PANJEONG.md`. 기존 `generate_b2_structure_judgment_bundle`·38건 원장 무수정 |

## 2. 실측 — 엔진 번들 vs #5447 자산

| 대조 | 결과 |
|---|---|
| 엔진 번들 vs **현재 바이너리로 재생성한** 스파이크 번들 | 32 파일 중 계열삭제 4 파일(`묶은세로막대형-계열삭제`·`누적세로막대형-계열삭제` × 2포맷) 제외 **전건 바이트 동일** |
| 엔진 번들 차트 XML(①) vs 커밋된 `samples/issue5447/` 자산의 ① | 행추가·계열추가·행삭제·계열명변경·라벨변경·점추가 **XML 동일**, 계열삭제만 다름(설계) |
| 엔진 번들 문서 vs 커밋된 `samples/issue5447/` 문서 | 대조군만 동일. 변종 전건 DIFF — 차이는 `BinData/image1.OLE`(중첩 CFB)의 디렉터리 red/black 플래그·FAT 꼬리뿐(차트 스트림 동일). **스파이크를 지금 재생성해도 같은 DIFF** → 이 브랜치와 무관한 CFB 라이터 변화(#5647 이후 devel) |

의미 — 한컴이 #5447 에서 판정한 차트 XML 과 엔진 산출 XML 이 10종에서 같다. S5-b 재판정의 실질
관심은 (1) 계열삭제 2종(위치 기반 — 뒤 계열이 앞으로 당겨짐, 색은 위치를 따름)과 (2) 현재 CFB
라이터 포장 상태의 개봉이다.

### 게이트 실측 (2026-08-23)

| 게이트 | 결과 |
|---|---|
| `issue_4100_chart_data_edit` | 56 passed / 3 ignored |
| fmt `--check` · suite-manifest `--prepare`→`--check` · unit-tiers `--base-ref upstream/devel`(4225 불변) · clippy `-D warnings` | 통과 |

## 3. S5-b — 한컴 재판정 (2026-08-23 완료)

작업지시자가 번들 32 파일을 한글 2022 로 열어 같은 폴더에 PDF 로 저장했다(개봉 실패 0). 원본 32건은
번들과 SHA-256 전건 동일을 확인한 뒤 `samples/issue5652/` 로, PDF 는 `<기준문서>-<변종>-<포맷>-2022.pdf`
로 정규화해 `pdf/issue5652/` 로 옮겼다. 원장 `MANIFEST.json`(`rhwp/hancom-judgment-manifest@1`) —
PyMuPDF 1.28.2·poppler 26.06 2축 래스터, 판정(대조군 대비 변화) 13 단위 **전건 반영**, invariants
`raster_equal` 12쌍 + 변환본 + page_geometry 1190×1682 + counts, 교차 참조 `cross_reference_issue5447`
**25/25 픽셀 동일**. 재계산 `tools/hancom_chart_judgment_verify.py --manifest samples/issue5652/MANIFEST.json`
3모드 전건 통과. 트립와이어 `b2_engine_judgment_assets_match_the_manifest` 추가. 편집기 행·열 수는
원장 `editor_observation` 에 사람 관측 칸으로 남겼다(미기입).

### 원래 절차 (참고)

절차(#5447 §1 과 동일):

1. `output/issue_5652_b2_engine_judgment/` 32 파일을 한컴 2022 로 연다 — `PANJEONG.md` 의 (a) 개봉
   (b) 기대 모양 (c) 편집기 개봉 (d) **편집기 행·열 수** 네 가지를 파일별로 한 줄씩.
2. 각 파일을 **같은 폴더에 PDF 로 저장**한다(파일명 규약은 #5447: `<이름>-<포맷>-2022.pdf`).
3. 에이전트가 `tools/hancom_chart_judgment_verify.py` 로 144DPI 래스터 SHA-256 을 재계산해 대조군과
   가르고, `samples/issue5652/` + `pdf/issue5652/` + `MANIFEST.json`(`rhwp/hancom-judgment-manifest@1`)
   을 쌓은 뒤 트립와이어 `b2_engine_judgment_assets_match_the_manifest` 를 추가한다.
4. `mydocs/report/task_m100_5652_report.md` 에 수용 기준 7건 판정표를 쓰고 계획서 `status: done`.

## 4. 다음

전체 release-test 1회 실측(보고서 §6) → PR.
