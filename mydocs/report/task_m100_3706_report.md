---
kind: report
status: active
canonical: mydocs/report/task_m100_3706_report.md
last_verified: 2026-08-01
---

# #3706 처리 기록 — HWP3 → HWP5 변환본 FileHeader 버전 3 기록 수정 (#3676 후속)

- Issue: [#3706](https://github.com/edwardkim/rhwp/issues/3706) —
  [#3676](https://github.com/edwardkim/rhwp/issues/3676) 조사에서 확정된 규격 위반의
  분리 등록분 (한컴 열기 거부 근인 3종은 PR #3685 담당, 본 건은 독립)
- 브랜치 `task/3676-hwp5-hancom-compat`

## 증상

`rhwp convert` 로 만든 HWP3 → HWP5 변환본의 FileHeader 버전 바이트가
`00000003`(major=3)이다. HWP5 컨테이너는 5.x 를 선언해야 하며(같은 문서의 한컴 저장본
`samples/hwp3-sample-hwp5.hwp` 는 `01000105` = 5.1.0.1), 버전 3 은 규격 위반이다.
rhwp 자신은 읽기에서 버전을 강제하지 않아 왕복 검증으로는 드러나지 않는다 — #3676 과
같은 사각.

## 근인 — 두 코드의 전제 어긋남

1. HWP3 파서(`src/parser/hwp3/mod.rs`)는 `version.major = 3` 을 **메모리 전용
   표시**(assign_auto_numbers 의 HWP3 문단 카운팅 선택용)로 두고, 실제 저장용 HWP5
   헤더(5.0.3.0)는 `header.raw_data` 에 넣는다 — `serialize_file_header` 의
   raw_data 우선 규칙을 전제한 설계다(코드 주석에 명시).
2. 저장 정규화(`document_core/converters/hwpx_to_hwp.rs::normalize_file_header_for_hwp`)가
   압축 플래그 실체화 과정에서 **raw_data 를 무조건 버린다.** 직렬화가 필드 경로로
   떨어지고 메모리 전용 표시값 major=3 이 그대로 디스크에 기록된다.

## 수정

`normalize_file_header_for_hwp` 에서 raw_data 를 버리기 **전에**, `version.major < 5`
이면 raw_data 바이트 32..36(revision/build/minor/major)의 5.x 버전을 필드로 회수한다.
회수 불가(raw_data 없음·짧음·major<5)면 파서 기본값 5.0.3.0 으로 실체화한다.

- 이미 5.x 인 경로는 무변경 — HWPX 파서는 5.1.0.0 을 필드에 직접 기록하므로 이 분기에
  들어오지 않는다.
- 압축 플래그 실체화(기존 동작)는 그대로 유지 — raw_data 의 flags=0(비압축)은 회수하지
  않는다. 헤더-스트림 자기모순 방지.
- `AdapterReport.file_header_version_materialized` 카운터 신설로 관측 표면 유지.

## 검증 — red → green

회귀 테스트 `tests/issue_3706_hwp3_convert_file_header_version.rs` 3건. CLI `convert`
와 같은 경로(parse_hwp3 → convert_if_hwpx_source → serialize_hwp)로 실변환 후 CFB 에서
FileHeader 스트림을 꺼내 바이트를 검사한다.

1. `converted_file_header_declares_hwp5_version` — 변환본이 5.0.3.0 을 선언
2. `converted_file_header_keeps_compressed_flag_materialized` — 압축 플래그(bit0)
   실체화 유지 (버전 회수가 flags 를 되살리지 않음을 격리)
3. `converted_reloads_with_hwp5_version` — 재파스·문서 재로드 왕복에서 major=5

| 단계 | 결과 |
| --- | --- |
| red (수정 제거 상태) | **2 failed / 1 passed** — ①·③이 `left: 3, right: 5` 로 실패, ②(압축 플래그)는 기존 동작이라 통과 |
| green (수정 적용) | **3 passed** (0.04s) |
| `--test hwpx_to_hwp_adapter` (변경 파일의 통합 타깃) | **50 passed / 0 failed** (15 ignored) |
| `--lib document_core::converters::hwpx_to_hwp` (모듈 단위 테스트) | **49 passed / 0 failed** |
| rustfmt --check (변경 파일 2건) | diff 0 — "Incorrect newline style" 은 autocrlf 작업본 CRLF 오탐(저장 blob 은 LF) |

수정이 기본값 문서(`Document::default()`, version 0.0.0.0)에도 5.0.3.0 실체화를
적용하므로, 어댑터 단위 테스트(`empty_doc_normalizes_file_header_once`,
`idempotent_when_called_twice`)와 통합 타깃 전체로 회귀를 확인했다.

## 남긴 것

- 잔여 위반 여지: HWP3 파서가 raw_data 를 만들지 못하는 경로는 없지만, 방어적으로
  기본값 5.0.3.0 실체화를 두었다.
- PR #3685(#3676 근인 3종)와 같은 함수의 인접 영역을 수정한다 — merge 순서에 따라
  단순 충돌 가능. 본 수정은 3685 의 3종과 독립이므로 어느 쪽이 먼저 들어가도 의미
  충돌은 없다.
