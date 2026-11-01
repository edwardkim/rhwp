# #8424 scaffold 셀 저장 줄 정보 — 시각 검증 근거

- 측정일: 2026-11-01 (KST)
- 측정 소스: 수정 커밋 `eba319761` (base `upstream/devel` `20c9678b0`). 최종 head 재측정 결과는 아래 "최종 head 재측정" 절
- 회귀 테스트: `tests/cases/issue_8424_scaffold_cell_wrap.rs`

## 입력과 독립 기준

입력은 저장소 `samples/`가 아니라 scaffold 명세에서 생성한다. 명세 원문은 `specs/`에 있고,
회귀 테스트의 인라인 명세(`CELLWRAP_SPEC`, `business_plan_spec()`)와 같은 내용이다.

| 문서 | 생성 명령 | 수정 후 HWPX SHA-256 | 수정 전 HWPX SHA-256 |
|---|---|---|---|
| cellwrap | `rhwp scaffold specs/cellwrap.json -o cellwrap.hwpx` | `65820ea9c2b86fd9cf4453a4f7f79318d4b1f97b1a313d5a53fdc9637b1324c3` | `f06922f86e293da41655c9b27cdd85b404c29dba4f8cedf787479748f219933f` |
| business_plan | `rhwp scaffold specs/business_plan.json -o business_plan.hwpx` | `fc0c5d86d4b5e4cf35649d286740081894849dec6dc37a7e34a95f278c9c6c1c` | `76fde1351d8aaa805c6e61ca0a0f7f6fbb249686b93a973bec104e15aa246e1d` |

독립 기준은 **수정 후 HWPX**를 한컴오피스 2024 `13.0.0.3901`에서 Print 경로로 출력한 PDF다
(원격 변환 MCP `engine: 2024`, `pdf_print_method: 0`, `pdf_output_mode: frame_print_to_pdf_ex_one_up`,
한컴 글꼴 범위 `verified`).

| PDF | SHA-256 | 쪽수 | job |
|---|---|---|---|
| `pdf/issue-8424/cellwrap-2024.pdf` | `23d2ac9f7f43a82bbd8d0522869c114e3aaa0d1d6a43e3870e355b5a1f143bbd` | 1 | `39b0e123-4c6d-427a-b8d3-79bbd3b0e194` |
| `pdf/issue-8424/business_plan-2024.pdf` | `cde3d98418992f6236007fe6e6f56c28d52aca8613caa7c6388300ebb7a5ec05` | 3 | `c3772c4a-98e0-4f8b-98af-3e1ac7101968` |

수정 전 HWPX의 한컴 Print 출력도 같은 쪽수(1쪽·3쪽)다. 수정 전 cellwrap HWPX에서
`<hp:linesegarray>`를 모두 지운 파일과 원본의 한컴 Print 출력은 144dpi 래스터가 픽셀 단위로 같았다.
즉 한컴은 저장 줄과 무관하게 다시 조판하며, 저장 줄을 비워도 한컴 출력은 바뀌지 않는다.

## 측정

`scripts/visual_sweep.py --silhouette-only --compat 2024 --dpi 96` (pdftoppm), raw = `silhouette_raw_match_percent`.
rhwp 글꼴은 `RHWP_FONT_PATH=ttfs/opensource`(로컬에 함초롬바탕 없음 — RHWPPublicBatang·Noto Sans KR 대체).
Native는 `--features native-skia` release 바이너리, fresh WASM은 같은 소스에서
`scripts/wasm-pack-locked.sh --target web --out-dir pkg`로 새로 만든 package다.

| 문서 | 쪽 | 수정 전 Native | 수정 후 Native | 수정 후 fresh WASM |
|---|---|---|---|---|
| cellwrap | 1 | 56.73% | 99.99% | 100.00% |
| business_plan | 1 | 측정 불가 (rhwp 2쪽 / 한컴 3쪽) | 97.74% | 97.71% |
| business_plan | 2 | 〃 | 98.59% | 98.63% |
| business_plan | 3 | 〃 | 100.00% | 100.00% |

- 수정 후 전쪽 최저: Native 97.74%, fresh WASM 97.71% (90% 미만 0쪽), 전체 쪽수 rhwp = 한컴.
- Native 전체 모드(`visual_sweep.py` 기본) `pr_review_gate`: cellwrap·business_plan 모두 `passed`.
- TSV 원본: `tsv/native/`, `tsv/wasm/`, `tsv/base_native/`.

## 직접 판독

- `png/cellwrap_before_review_p1.png`: 수정 전 rhwp는 "세부 과업 1: 요구사항"까지 한 줄로 그리고
  나머지를 셀 경계에서 자른다. 행 높이도 1줄이라 표 뒤 문단이 한컴보다 위에 온다.
- `png/cellwrap_after_review_p1.png`, `png/cellwrap_after_overlay_p1.png`: 수정 후
  "세부 과업 1: 요구사 / 항 정의, 설계, 구현 / 및 시험" 3줄, 행 높이·표 뒤 문단 위치가 한컴과 일치한다.
- `png/business_plan_after_review_p2.png`, `png/business_plan_after_overlay_p2.png`: 6행 표 전체의
  줄 나눔·행 높이·표 아래 본문이 한컴과 겹친다.
- 남은 차이: 개요 제목 번호 뒤 간격(rhwp "3.Ⅲ." / 한컴 "3. Ⅲ.")은 이 수정과 무관한
  기존 차이이며 #8425로 분리했다.

## 회귀 테스트 수정 전·후

수정 전 `src/scaffold/builder.rs`(base `20c9678b0`)로 같은 테스트를 실행하면 3건 모두 의도한 원인으로 실패한다.

| 테스트 | 수정 전 | 수정 후 |
|---|---|---|
| `scaffold_does_not_fabricate_line_segs_for_text_paragraphs` | FAIL: "셀 줄바꿈 재현": 저장 줄 1개 | PASS |
| `long_cell_text_is_kept_inside_its_cell_on_several_lines` | FAIL: 글 조각 x+w=402.5px > 칸 오른쪽 396.8px | PASS |
| `page_counts_match_hancom_print` | FAIL: business_plan 2쪽 ≠ 3쪽 | PASS |
