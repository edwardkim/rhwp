# 단계별 완료보고서 — Task #1499 단계 2: render-diff CLI

- **이슈**: #1499 · **브랜치**: `local/task1499`

## 작업 내용

`render_geom_diff.rs` 에 `pub fn run(args)` 추가 + `main.rs` dispatch/help 배선.

3개 모드:
- `render-diff <a.hwpx>` — 자기 라운드트립 (orig vs `serialize_hwpx`→재parse). 사람 가독 요약.
- `render-diff <a.hwpx> <b.hwpx>` — 두 파일 직접 비교.
- `render-diff --batch <dir> [-o out]` — 재귀 수집·전수 자기 라운드트립 → `geom_inventory.tsv`.
- `--threshold <px>` (기본 0.5). 하드 실패(비-PASS) 존재 시 종료코드 1.

자기 라운드트립 바이트 생성은 `parse_hwpx`+`serialize_hwpx` (선례 `hwpx_roundtrip_batch`).
외부 이미지는 양쪽 모두 비주입(대칭성 확보 — 오탐 방지).

## 검증 (스모크)

- 두 파일 **동일 파일** 비교 → PASS, 변위 0.000 (비교기 정합성 확인).
- 자기 라운드트립 `143E...hwpx` → STRUCT_MISMATCH (ins=1 del=5), IrDiff=0 인데도 검출.
- `--batch samples/hwpx` 57건 → PASS 36 · STRUCT_MISMATCH 15 · DISP_OVER 5 · ERROR 1.

## 게이트 유효성 입증 (핵심)

`el-school-001.hwpx`: `hwpx-roundtrip` IrDiff=0(구조 보존) 인데 렌더 트리 비교 결과
**Image 노드 1→0 (라운드트립이 임베디드 이미지 누락)**. 기존 baseline 이 못 잡는
시각 회귀를 본 게이트가 정확히 검출 — 이슈의 공백 메우기 목표 달성.

## 다음 단계

단계 3: `tests/visual_roundtrip_baseline.rs` 회귀 게이트 (36 PASS baseline + 21 XFAIL/EXCLUDED).
