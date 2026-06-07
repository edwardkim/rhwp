---
name: rhwp-cli
description: rhwp CLI 바이너리로 HWP/HWPX 파일을 분석·내보내기·디버깅합니다. SVG/PNG/PDF/텍스트/Markdown 내보내기, 페이지네이션·조판부호·render tree 덤프, IR 비교(HWPX↔HWP), HWPX→HWP 저장 계약 분석을 적절한 명령으로 수행합니다. 트리거 — 사용자가 ".hwp/.hwpx 파일을 SVG/PNG/PDF/텍스트로 내보내", "페이지네이션/조판부호/구조 덤프", "두 파일 IR 비교", "render tree 추출", "레이아웃/간격/겹침 버그 디버깅", "HWPX→HWP 저장 차이 분석", "rhwp <명령>" 등을 요청할 때. 전체 명령 레퍼런스는 mydocs/manual/cli_commands.md.
---

# rhwp-cli — rhwp CLI 분석·디버깅 Skill

## 목적

`rhwp` 바이너리로 HWP/HWPX 문서를 내보내기·덤프·비교·진단한다. 사용자 요청을 **적절한
명령으로 매핑**하고, 레이아웃 버그는 **권장 디버깅 순서**로 좁힌다.

권위 출처: `src/main.rs` 명령 디스패치 = `rhwp --help` = [`mydocs/manual/cli_commands.md`](../../../mydocs/manual/cli_commands.md).
이 셋은 항상 함께 현행화한다(CLI 추가/변경 시 help 문자열 + 매뉴얼 동시 갱신).

## 바이너리 실행

```bash
# release 빌드(권장, 빠름)
cargo build --release        # 최초 1회 또는 소스 변경 후
./target/release/rhwp <명령> [옵션]

# 개발 중(빌드 안 됐을 때)
cargo run --quiet --bin rhwp -- <명령> [옵션]
```
- 네이티브 빌드/실행은 **항상 로컬 cargo**. Docker 는 WASM 전용(분석 명령에 쓰지 않음).
- `export-png`/`export-pdf` 는 `--features native-skia` 또는 svg2pdf 의존 — release 바이너리에 포함됨.
- 출력 기본 폴더 `output/`. 분석 산출물은 `output/poc/<주제>/` 로 분리 권장(gitignore 됨).

## 요청 → 명령 매핑

| 사용자 요청 | 명령 |
|------------|------|
| "SVG로 내보내" / 시각 확인 | `export-svg <파일> [-p N] [-o 폴더]` |
| "PNG로" / VLM 입력용 이미지 | `export-png <파일> [-p N] [--vlm-target claude]` |
| "PDF로" | `export-pdf <파일> [-o out.pdf] [-p N]` |
| "텍스트/마크다운 추출" | `export-text` / `export-markdown` |
| "이 페이지 배치/페이지네이션 보여줘" | `dump-pages <파일> -p N` |
| "조판부호/문단/표 속성 구조" | `dump <파일> -s N -p M` |
| "raw record 덤프" | `dump-records <파일>` |
| "번호/글머리표/개요 진단" | `diag <파일>` |
| "파일 정보(버전/구역)" | `info <파일>` |
| "render tree / bbox 좌표" | `export-render-tree <파일> -p N` |
| "HWPX 와 HWP 차이(IR) 비교" | `ir-diff <a.hwpx> <b.hwp> [-s N] [-p M] [--summary]` |
| "썸네일 추출" | `thumbnail <파일> [--data-uri]` |
| "읽기전용 HWP 편집가능하게" | `convert <입력> <출력.hwp>` |
| "HWPX→HWP 저장이 한컴과 다름" | `hwp5-inventory-diff` 등 (4절) |

## 레이아웃·간격·겹침 버그 디버깅 (권장 순서)

코드 무수정으로 결함을 좁힌다:

1. `export-svg <파일> --debug-overlay -p N` → 문단/표 식별 (`s{섹션}:pi={인덱스} y={좌표}`)
2. `dump-pages <파일> -p N` → 해당 페이지 문단/표 배치 목록 + 높이(vpos/lh/ls)
3. `dump <파일> -s N -p M` → ParaShape / LINE_SEG / 표·도형 속성 상세
4. (HWPX↔HWP 불일치) `ir-diff a.hwpx b.hwp -s N -p M`
5. (정밀 좌표) `export-render-tree <파일> -p N` → bbox JSON. 보정 전/후 또는 두 파일 diff
6. (저장 계약) `hwp5-inventory-diff oracle.hwp generated.hwp`

### 보정 전/후 시각 회귀 확인 패턴
레이아웃 변경의 회귀 검출은 **시각 판정이 핵심**(전체 테스트·golden 통과해도 미검출 가능).
```bash
# 보정 전(기준 브랜치) vs 후(변경) 전체 페이지 SVG diff → 변화 페이지만 식별
rhwp export-svg <파일> -o output/poc/before/      # 기준 상태
rhwp export-svg <파일> -o output/poc/after/       # 변경 상태
diff <(ls before) <(ls after); 페이지별 diff 로 변화 줄 수·좌표 이동 확인
```
- render tree JSON 비교가 SVG 문자열 비교보다 좌표 분석에 정확하다.
- 셀 내부/transform 그룹은 절대 y 가 아닌 `translate(x,y)` 단위로 본다.

## HWPX→HWP 저장 계약 분석 (hwp5-*)

HWPX 로 연 문서를 HWP 로 저장(#178 어댑터)했을 때 한컴이 다르게 여는 경우의 record 축별 분석.
oracle = 한컴 저장본, generated = rhwp 저장본.

```bash
rhwp hwp5-inventory-diff oracle.hwp generated.hwp --report hints --focus table
rhwp hwp5-table-probe oracle.hwp generated.hwp --out-dir output/poc/probe/
rhwp hwp5-anchor-trace 파일.hwp --needle "특정텍스트" --section N
```
전체 hwp5-* 목록·옵션은 [cli_commands.md §4](../../../mydocs/manual/cli_commands.md).

## 검증·주의

- **자기 round-trip / 테스트 통과 ≠ 한컴 호환**: 저장·렌더 결함은 한컴 수동 검증이 최종 게이트.
- 명령은 코드 무수정 분석이 원칙. 결함 정정은 별도 절차(이슈→브랜치→계획) 따른다.
- 페이지 번호는 0부터. PDF/한컴 페이지 표기(1부터)와 혼동 주의.
- 단위: 1인치=7200 HWPUNIT=96px, 1px=75 HWPUNIT, 1mm≈283.46 HWPUNIT.

## 상세 레퍼런스

- 전체 명령·옵션: [`mydocs/manual/cli_commands.md`](../../../mydocs/manual/cli_commands.md)
- `dump`: [`mydocs/manual/dump_command.md`](../../../mydocs/manual/dump_command.md)
- `export-png`: [`mydocs/manual/export_png_command.md`](../../../mydocs/manual/export_png_command.md)
- `ir-diff`: [`mydocs/manual/ir_diff_command.md`](../../../mydocs/manual/ir_diff_command.md)
