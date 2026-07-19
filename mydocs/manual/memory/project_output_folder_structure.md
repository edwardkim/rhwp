---
kind: memory
status: historical
canonical: mydocs/manual/memory/MEMORY.md
last_verified: 2026-07-17
name: output 폴더 서브폴더 구조
description: output/ 하위 용도별 서브폴더 규칙 — re/, svg/, debug/
type: project
originSessionId: 1f035a49-cf55-4427-a5b6-ba6a493aa832
---
output/ 폴더는 용도별 서브폴더로 분리한다. .gitignore에 등록되어 Git에 포함되지 않음.

| 폴더 | 용도 |
|------|------|
| `output/re/` | 재현검증용 샘플 (`re_sample_gen.rs` 테스트 자동 생성) |
| `output/svg/` | SVG 내보내기 기본 출력 (`rhwp export-svg`) |
| `output/svg/pr{N}_before/` + `output/svg/pr{N}_after/` | PR 검토 시 정정 전/후 SVG 비교용 — 작업지시자 시각 판정 효율을 위해 `/tmp/` 사용 금지 |
| `output/debug/` | 디버그 오버레이 HTML (`rhwp export-svg --debug-overlay`) |

**Why:** output/ 한 폴더에 모든 파일이 혼재하면 구분하고 찾기가 어려움. #138에서 도입. PR 검토 SVG 를 `/tmp/` 에 두면 작업지시자가 직접 IDE 에서 열어 시각 판정하기 어려움 — `output/svg/pr{N}_*` 로 프로젝트 폴더 안에 두어야 IDE/브라우저에서 즉시 비교 가능.

**How to apply:** 새 출력 파일을 생성하는 코드 작성 시 반드시 용도에 맞는 서브폴더에 저장. PR 검토에서 before/after SVG 생성 시도 즉시 `output/svg/pr{N}_before/` + `output/svg/pr{N}_after/` 패턴 사용 (샘플별 서브폴더 분리). 새 용도가 생기면 서브폴더를 추가하고 canonical 문서(`mydocs/manual/cli_commands.md` 출력 폴더 절)도 갱신 (CLAUDE.md는 2026-07-17부터 부트로더 — 절차 기록 금지).
