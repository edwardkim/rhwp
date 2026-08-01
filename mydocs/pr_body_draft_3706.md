# fix(convert): HWP3 변환본 FileHeader 에 HWP5 버전을 실체화한다 (#3706)

## 요약

HWP3 → HWP5 변환본의 FileHeader 가 버전 3 을 선언하던 규격 위반(#3706, #3676 조사에서
확정)을 수정합니다. 저장 정규화가 `header.raw_data` 를 버리기 전에 그 안의 HWP5
버전(5.0.3.0)을 필드로 회수하고, 회수 불가면 파서 기본값 5.0.3.0 으로 실체화합니다.

- 전: 변환본 FileHeader 버전 바이트 `00000003` (major=3) — HWP5 규격 위반
- 후: `00030005` (5.0.3.0) — 한컴 저장본과 같은 5.x 선언

## 근인

두 코드의 전제가 어긋났습니다.

1. HWP3 파서(`parser/hwp3/mod.rs`)는 `version.major = 3` 을 메모리 전용
   표시(`assign_auto_numbers()` 의 HWP3 문단 카운팅 선택용)로 두고, 실제 저장용 HWP5
   헤더는 `raw_data` 에 기록 — `serialize_file_header` 의 raw_data 우선 규칙 전제.
2. 저장 정규화(`normalize_file_header_for_hwp`)가 압축 플래그 실체화 과정에서
   raw_data 를 무조건 폐기 → 직렬화가 필드 경로로 떨어져 major=3 이 디스크에 기록.

## 범위

- 이 수정은 #3676(한컴 열기 거부)의 근인이 **아닙니다** — #3676 코멘트의 버전 4종
  실험으로 반증됐고, 거부 근인 3종은 PR #3685 담당입니다. 본 건은 그 조사에서 "별도로
  정리할 값"으로 남은 독립 규격 위반입니다.
- 이미 5.x 인 경로(HWPX 파서는 5.1.0.0 을 필드에 직접 기록)는 무변경.
- 압축 플래그 실체화(기존 동작) 유지 — raw_data 의 flags=0 은 회수하지 않아
  헤더-스트림 자기모순을 만들지 않습니다.
- `AdapterReport.file_header_version_materialized` 카운터 신설.

## 검증 (red → green)

회귀 테스트 `tests/issue_3706_hwp3_convert_file_header_version.rs` 3건 — CLI `convert`
와 같은 경로(parse_hwp3 → convert_if_hwpx_source → serialize_hwp)로 실변환 후 CFB 의
FileHeader 스트림 바이트를 검사:

1. 변환본이 5.0.3.0 을 선언한다
2. 버전 회수가 압축 플래그(bit0) 실체화를 깨지 않는다
3. 재파스·문서 재로드 왕복에서 major=5

| 게이트 | 결과 |
| --- | --- |
| red (수정 제거 상태) | **2 failed / 1 passed** — ①·③이 `left: 3, right: 5` 로 실패, ②는 기존 동작이라 통과 |
| green (수정 적용) | **3 passed** |
| `--test hwpx_to_hwp_adapter` (변경 파일의 통합 타깃) | **50 passed / 0 failed** (15 ignored) |
| `--lib document_core::converters::hwpx_to_hwp` (모듈 단위 테스트) | **49 passed / 0 failed** |
| rustfmt --check (변경 파일 2건) | diff 0 |

처리 기록: `mydocs/report/task_m100_3706_report.md`

Closes #3706

🤖 Generated with [Claude Code](https://claude.com/claude-code)
