---
name: rhwp-cli
description: rhwp CLI 바이너리로 HWP/HWPX 파일을 분석·내보내기·디버깅합니다. SVG/PNG/PDF/텍스트/Markdown 내보내기, 페이지네이션·조판부호·render tree 덤프, IR 비교(HWPX↔HWP), HWPX→HWP 저장 계약 분석을 적절한 명령으로 수행합니다. 트리거 — 사용자가 ".hwp/.hwpx 파일을 SVG/PNG/PDF/텍스트로 내보내", "페이지네이션/조판부호/구조 덤프", "두 파일 IR 비교", "render tree 추출", "레이아웃/간격/겹침 버그 디버깅", "HWPX→HWP 저장 차이 분석", "rhwp <명령>" 등을 요청할 때. 전체 명령 레퍼런스는 mydocs/manual/cli_commands.md. gym 이 아니라 실사용 에이전트 경로다.
---

# rhwp-cli — rhwp CLI 분석·디버깅 Skill

`rhwp` 바이너리로 HWP/HWPX 문서를 내보내기·덤프·비교·진단한다. 사용자 요청을
**적절한 기존 명령으로 매핑**하고, 레이아웃·겹침은 **권장 디버깅 순서**로 좁힌다.

이 스킬은 **gym 이 아니다.** 실사용 에이전트가 문서를 분석·디버깅하는 경로다.
**새 CLI 를 만들지 않는다.** 새 명령을 만들지 않는다. DocumentCore 편집 로직을
발명하지 않는다. 권위는 `src/main.rs` 디스패치 = `rhwp --help` =
[`mydocs/manual/cli_commands.md`](../../../mydocs/manual/cli_commands.md).

처리 기록: [`mydocs/working/agent_cli.md`](../../../mydocs/working/archives/agent_cli.md).

## 바이너리 실행

```bash
cargo build --release
./target/release/rhwp <명령> [옵션]
```

개발 중이면 `cargo run --quiet --bin rhwp -- <명령> [옵션]`.
네이티브는 **항상 로컬 cargo**. Docker 는 WASM 전용(분석 명령에 쓰지 않음).
`export-png` 는 `--features native-skia` 가 필요하다. 없으면 exit 2.
산출은 `output/poc/<주제>/` 로 분리(gitignore).

## 요청 → 명령 매핑

| 사용자 요청 | 명령 | 레퍼런스 |
|---|---|---|
| SVG / 시각 확인 / overlay | `export-svg <파일> [-p N] [-o 폴더] [--debug-overlay]` | 02_export_svg.md |
| PNG / VLM 입력 | `export-png <파일> [-p N] [--vlm-target claude]` | 03_export_png.md |
| PDF | `export-pdf <파일> [-o out.pdf] [-p N] [--profile print]` | 04_export_pdf.md |
| 텍스트 추출 | `export-text <파일> [--json] [--max-chars N]` | 05_export_text.md |
| 마크다운 | `export-markdown <파일> [-p N]` | 06_export_markdown.md |
| 이 페이지 배치 | `dump-pages <파일> -p N` | 07_dump_pages.md |
| 조판부호 / 문단 속성 | `dump <파일> -s N -p M` | 08_dump.md |
| raw record | `dump-records <파일>` | 09_dump_records.md |
| 번호 / 글머리표 / 개요 | `diag <파일>` | 10_diag.md |
| 파일 정보 (버전/구역) | `info <파일> [--json]` | 11_info.md |
| render tree / bbox | `export-render-tree <파일> -p N` | 12_export_render_tree.md |
| HWPX↔HWP IR 비교 | `ir-diff <a.hwpx> <b.hwp> [--json]` | 13_ir_diff.md |
| 썸네일 | `thumbnail <파일> [--data-uri]` | 14_thumbnail.md |
| 배포용 → 편집 가능 HWP | `convert <입력> <출력.hwp> [--verify]` | 15_convert.md |
| 한컴 저장과 다름 | `hwp5-inventory-diff oracle.hwp generated.hwp` | 16_hwp5_family.md |

전체 표: [01_request_command_map.md](references/01_request_command_map.md).
hwp5-* 가족(`hwp5-inventory`, `hwp5-table-probe`, `hwp5-anchor-trace`,
`hwp5-char-shape-audit`, `hwp5-roundtrip` 등)은 [16_hwp5_family.md](references/16_hwp5_family.md).

## 레이아웃·간격·겹침 디버깅 (권장 순서)

코드 무수정으로 결함을 좁힌다. **이 순서가 계약**이다.

1. `export-svg <파일> --debug-overlay -p N` → 문단/표 식별 (`s{섹션}:pi={인덱스} y={좌표}`)
2. `dump-pages <파일> -p N` → 해당 페이지 문단/표 배치 + 높이(vpos/lh/ls)
3. `dump <파일> -s N -p M` → ParaShape / LINE_SEG / 표·도형 속성
4. `ir-diff a.hwpx b.hwp -s N -p M` → HWPX↔HWP IR (형식 쌍이 있을 때)
5. `export-render-tree <파일> -p N` → bbox JSON. 셀은 `translate(x,y)`
6. `hwp5-inventory-diff oracle.hwp generated.hwp` → 저장 계약

답이 나오면 다음 단으로 내려가지 않는다. 상세: [17_layout_debug_order.md](references/17_layout_debug_order.md).

```bash
# 사용자가 "3쪽이 겹친다" — 한컴 3쪽 = -p 2
rhwp export-svg 보고서.hwp --debug-overlay -p 2 -o output/poc/overlap/
rhwp dump-pages 보고서.hwp -p 2
rhwp dump 보고서.hwp -s 0 -p 14
rhwp ir-diff 보고서.hwpx 보고서.hwp -s 0 -p 14 --json
rhwp export-render-tree 보고서.hwp -p 2 -o output/poc/overlap/tree/
rhwp hwp5-inventory-diff oracle.hwp generated.hwp --report hints --focus table
```

## 페이지는 0부터 · 단위는 HWPUNIT

- `-p` / `pages[].page` / `dump-pages -p` / `export-render-tree -p` 는 **0부터**.
- 사용자가 "4쪽" 이라고 하면 한컴·PDF 표기이므로 `-p 3`.
- `dump -p` / `ir-diff -p` 는 **문단** 인덱스(0부터). 페이지가 아니다.
- `extract-pages --from/--to` 만 1부터 — 이 스킬의 기본 축이 아니다.
- 1인치 = 7200 HWPUNIT = 96px, 1px = 75 HWPUNIT, 1mm ≈ 283.46 HWPUNIT.

상세: [18_page_units.md](references/18_page_units.md).

## 자기 round-trip ≠ 한컴 호환

`hwp5-roundtrip` / `hwpx-roundtrip` / `render-diff` 자기 비교 / `convert --verify`
통과는 **우리 직렬화가 닫혔다**는 뜻이다. 한컴이 같은 파일을 열거나 같은 화면을
보여준다는 뜻이 아니다. 저장·렌더 결함의 최종 게이트는 한컴 수동 검증이다.

- 하지 말 것: "라운드트립 통과했으니 한컴에서 열립니다"
- 할 것: "자기 직렬화는 닫혔다. 한컴 검증은 남아 있다."

상세: [19_roundtrip_vs_hangul.md](references/19_roundtrip_vs_hangul.md).

## HWPX→HWP 저장 계약 (oracle vs generated)

oracle = 한컴 저장본, generated = rhwp 저장본. 인자 순서는 항상 oracle 먼저.

```bash
rhwp hwp5-inventory-diff oracle.hwp generated.hwp --report hints --focus table
rhwp hwp5-table-probe oracle.hwp generated.hwp --out-dir output/poc/probe/
rhwp hwp5-anchor-trace generated.hwp --needle "특정텍스트" --section 0
```

가짜 oracle 을 만들지 않는다. 한컴 저장본이 없으면 6단을 건너뛴다.
상세: [20_hwpx_hwp_save_contract.md](references/20_hwpx_hwp_save_contract.md).

## 예외 봉투

실패는 추측하지 않고 네 종류로 먼저 분류한다. 메시지 문자열은 `src/main.rs` 정본.

| kind | stderr | exit |
|---|---|---|
| missing-file | `오류: 파일을 읽을 수 없습니다 - {path}: {os}` | 1 |
| bad-page-index | `오류: 페이지 번호가 범위를 벗어났습니다 (0~{max})` | 2 |
| native-skia-missing | `오류: export-png 명령은 native-skia feature 가 활성화되어야 합니다.` | 2 |
| load-fail | `오류: 문서 파싱 실패 - {msg}` | 1 |

부가: 인자 없음(2) · 비밀번호 없음(2) · 비밀번호 틀림(1) · `ir-diff --json` 차이(3) ·
`convert --verify`(3) · `convert --verify-pages`(4).
export-pdf `--backend direct` 의 skia 부재는 **exit 1** (png 스텁의 2 와 다름).
실패 경로 `--json` 의 stdout 은 0바이트. 상세: [21_exception_envelopes.md](references/21_exception_envelopes.md).

## 종료 코드 — 판정은 데이터

| 코드 | 의미 |
|---:|---|
| 0 | 성공. ir-diff **텍스트** 모드는 차이가 있어도 0 |
| 1 | 런타임 (파일 없음, 파싱 실패, 쓰기 실패) |
| 2 | 사용법 (인자, 페이지 범위, export-png feature 부재) |
| 3 | `ir-diff --json` 차이, `convert --verify` |
| 4 | `convert --verify-pages` |

exit 3 을 크래시로 읽지 않는다. `identical:false` 가 데이터다.
상세: [22_exit_codes.md](references/22_exit_codes.md).

## 하지 않는 것

- 새 rhwp CLI 하위명령·플래그를 만들지 않는다.
- DocumentCore 편집 구현을 건드리지 않는다.
- gym/ 팩을 실행하거나 점수를 내지 않는다.
- 다른 스킬(`rhwp-doc-triage` · `rhwp-visual-regression` · `rhwp-safe-edit` 등)을 여기서 고치지 않는다.
- 자기 라운드트립 통과를 한컴 호환으로 승격하지 않는다.
- oracle/generated 순서를 뒤집지 않는다.
- 페이지 기본값을 1로 문서화하지 않는다.

## 레퍼런스 목차

1. [00_tree.md](references/00_tree.md) — 판단 트리
2. [01_request_command_map.md](references/01_request_command_map.md) — 요청→명령
3. [02_export_svg.md](references/02_export_svg.md)
4. [03_export_png.md](references/03_export_png.md)
5. [04_export_pdf.md](references/04_export_pdf.md)
6. [05_export_text.md](references/05_export_text.md)
7. [06_export_markdown.md](references/06_export_markdown.md)
8. [07_dump_pages.md](references/07_dump_pages.md)
9. [08_dump.md](references/08_dump.md)
10. [09_dump_records.md](references/09_dump_records.md)
11. [10_diag.md](references/10_diag.md)
12. [11_info.md](references/11_info.md)
13. [12_export_render_tree.md](references/12_export_render_tree.md)
14. [13_ir_diff.md](references/13_ir_diff.md)
15. [14_thumbnail.md](references/14_thumbnail.md)
16. [15_convert.md](references/15_convert.md)
17. [16_hwp5_family.md](references/16_hwp5_family.md)
18. [17_layout_debug_order.md](references/17_layout_debug_order.md)
19. [18_page_units.md](references/18_page_units.md)
20. [19_roundtrip_vs_hangul.md](references/19_roundtrip_vs_hangul.md)
21. [20_hwpx_hwp_save_contract.md](references/20_hwpx_hwp_save_contract.md)
22. [21_exception_envelopes.md](references/21_exception_envelopes.md)
23. [22_exit_codes.md](references/22_exit_codes.md)
24. [23_pitfalls.md](references/23_pitfalls.md)
25. [24_anti_patterns.md](references/24_anti_patterns.md)
26. [25_journeys.md](references/25_journeys.md)
27. [26_cli_surface.md](references/26_cli_surface.md)
28. [27_field_catalog.md](references/27_field_catalog.md)
29. [28_worked_traces.md](references/28_worked_traces.md)

예제: [examples/](examples/README.md).
기계 픽스처: [fixtures/skill_index.json](fixtures/skill_index.json).

## 인계

- 긴 문서 파악(info→digest→search) → `rhwp-doc-triage`
- 시각 회귀 숫자(render-diff) → `rhwp-visual-regression`
- 원본 편집 → `rhwp-safe-edit`
- 배포 전 스윕 → `rhwp-security-sweep`
- 폴더 일괄 → `rhwp-bulk-pipeline`

이 스킬 안에서 그 스킬들의 파일을 고치지 않는다.

## 권위

- [`cli_commands.md`](../../../mydocs/manual/cli_commands.md)
- [`dump_command.md`](../../../mydocs/manual/dump_command.md)
- [`export_png_command.md`](../../../mydocs/manual/export_png_command.md)
- [`ir_diff_command.md`](../../../mydocs/manual/ir_diff_command.md)
- `src/main.rs` 종료 코드·stderr 문자열
