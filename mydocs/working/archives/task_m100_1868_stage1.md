# Task M100 #1868 1단계 완료보고서 — export-hwpx 서브커맨드 구현

- 이슈: #1868 / 브랜치: `local/task1868`
- 작성일: 2026-07-04 / 단계: 1/3

## 변경 내용 (`src/main.rs`)

1. dispatch: `Some("export-hwpx") => export_hwpx(&args[2..])` (export-markdown 다음).
2. `fn export_hwpx(args)` 신설 (convert_hwp 뒤):
   - 위치 인자 `<입력> [출력]`, 출력 생략 시 `input.with_extension("hwpx")`.
   - 출력 확장자 비-.hwpx 경고(진행), **입력==출력 경로 시 거부**(원본 덮어쓰기 방지 —
     HWPX 입력 + 출력 생략 조합이 정확히 이 케이스).
   - `HwpDocument::from_bytes`(포맷 자동 감지) → `export_hwpx_native()` → `fs::write`.
   - 메시지 관례 convert 정합("저장 완료: <경로> (NKB)").
3. `print_usage` 등재 (export-pdf 다음).

## 검증 (스모크)

| 케이스 | 결과 |
|---|---|
| `export-hwpx samples/pr-1674.hwp <out>.hwpx` | 저장 완료 424KB, **PK 매직**, 재파싱 OK, **페이지 35**(정답지 35 일치) |
| 출력 생략 | `<stem>.hwpx` 생성 ✓ |
| 입력==출력 | "원본을 덮어쓰지 않습니다" 거부 ✓ |
| `cargo build --release` | 성공 |

## 다음 단계

2단계: 통합 테스트(`tests/issue_1868_export_hwpx_cli.rs`) + 전체 회귀 + #1879 하니스 스모크.
