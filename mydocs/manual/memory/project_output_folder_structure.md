---
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
| `output/debug/` | 디버그 오버레이 HTML (`rhwp export-svg --debug-overlay`) |

**Why:** output/ 한 폴더에 모든 파일이 혼재하면 구분하고 찾기가 어려움. #138에서 도입.

**How to apply:** 새 출력 파일을 생성하는 코드 작성 시 반드시 용도에 맞는 서브폴더에 저장. 새 용도가 생기면 서브폴더를 추가하고 CLAUDE.md도 갱신.
