# PR #670 처리 보고서

## 1. 처리 결과

| 항목 | 값 |
|------|-----|
| PR | #670 — samples: 한글 2022 PDF 변환본 추가 |
| 컨트리뷰터 | @planet6897 (Jaeuk Ryu / Jaeook Ryu) — 4번째 사이클 PR (PR #629/#620/#578 직후) |
| 연결 이슈 | 없음 (closes 키워드 부재) |
| 처리 옵션 | **옵션 D 변형** — cherry-pick + 본 환경 정합 보강 (CLAUDE.md + pdf/README.md) |
| 머지 commit | `69a7078` (한글 2022 PDF 199 + 변환 자동화 스크립트) + `12aa99c` (본 환경 정합 보강) |
| 처리 일자 | 2026-05-07 |

## 2. cherry-pick 결과

### 본질 commit (`69a7078`)
- 199 PDF binary (한글 2022 변환본, `{stem}-2022.pdf`)
- `pdf/_convert.ps1` (+106 LOC) — PowerShell 변환 스크립트 + 크래시 자동 복구
- `pdf/_watchdog.ps1` (+19 LOC) — Hang 감지 watchdog
- `pdf/_convert.log` (+221 LOC) — 변환 실행 전체 로그
- author: Jaeook Ryu (jaeook.ryu@gmail.com), committer: edward

### 본 환경 정합 보강 commit (`12aa99c`)
- `pdf/README.md` 신규 작성 (+155 LOC) — 폴더 구조 + 명명 규약 + 권위 등급 + 변환 자동화 스크립트 안내
- `CLAUDE.md` 갱신 (+33/-1) — 예제 폴더 영역 + PDF 권위 자료 명명 규약 + PDF 권위 등급 (컨트리뷰터 환경별) 추가

## 3. 본 환경 결정적 검증

| 항목 | 결과 |
|------|------|
| `cargo build --release` | ✅ |
| `cargo test --lib --release` | ✅ **1155 passed** (회귀 0) |
| `cargo clippy --lib -- -D warnings` | ✅ 0 |

> 본 PR 은 PDF binary + 스크립트 + 문서 영역만 추가, src 영역 변경 0 → 결정적 검증 영역 영향 0. PR 670 head 시점 (1141 passed) 이 devel head 시점 (1155 passed) 과 다른 영역은 PR base (5/7 09:19) 가 PR #659/#602/#668 cherry-pick 이전 시점이었던 영역의 차이 (cherry-pick 후 devel 위 1155 정합 일치).

## 4. 본 환경 정합 영역의 본질 정정 영역

### 4.1 메모리 룰 영역 갱신 (5/7 본 사이클의 통찰 영역)

**`reference_authoritative_hancom`** 갱신:
- (기존) "한컴 2010 + 한컴 2022 의 편집기 출력만 정답지"
- (갱신) **컨트리뷰터 환경별 권위 영역 분리**:
  - Windows + 한컴 편집기: 한컴 2010/2020/2022 편집기 1차 + PDF 보조
  - macOS / Linux: 한글 **2020/2022 PDF** 1차 / 한글 2010 PDF 등급 미달
  - 한컴 뷰어 / 외부 변환 / macOS 인쇄: 모든 환경 정답지 아님

**`feedback_pdf_not_authoritative`** 갱신:
- (기존) "PDF 출력은 환경별 차이로 정답지 미입증"
- (갱신) **PDF 정답지 등급 영역 분리**:
  - 정답지 가능: 한컴 한글 **2020/2022 편집기 → PDF 변환**
  - 정답지 등급 미달: 한글 2010 PDF / 한컴 뷰어 / 외부 변환 / macOS 인쇄

### 4.2 본 영역의 본질 영역 — 외부 컨트리뷰터 다양성 인프라

**작업지시자의 통찰 영역 (5/7)**:
> "맥과 리눅스에서 개발하는 컨트리뷰터의 경우 PDF 가 오히려 더 정확합니다."

본 통찰 영역의 본질:
- 한컴 편집기 영역은 Windows 전용 → 맥/리눅스 컨트리뷰터 영역 부재
- 본 환경 PDF (한글 2020/2022) 가 맥/리눅스 컨트리뷰터의 권위 정답지 역할 가능
- 향후 컨트리뷰터 증가 시 한컴 프로그램 부재 환경에서 기여 가능 인프라 영역 확보
- `project_dtp_identity` (rhwp 정체성) 영역 + "닫힌 포맷의 벽 깨기" 영역의 본질 영역 강화

### 4.3 폴더 구조 영역 결정 — 버전별 폴더 분리

| 폴더 | 한컴 버전 | 상태 |
|------|----------|------|
| `pdf/` | 한글 2022 | 본 PR 영역 영구 보존 |
| `pdf-2020/` | 한글 2020 | (예정) 향후 추가 환영 |
| `pdf-2010/` | 한글 2010 | (예정) 등급 미달 — 영역 외 |

> 컨트리뷰터의 현재 영역 (`pdf/`) 유지 + 향후 한글 2020 추가 시 `pdf-2020/` 별도 폴더. 한글 2010 PDF 는 등급 미달 영역으로 영구 보존 영역 부재 (보조 자료 영역).

### 4.4 명명 규약 영역 — `pdf/README.md` + CLAUDE.md 명시

| 폴더 | 패턴 |
|------|------|
| `pdf/` | `pdf/{원본 stem}-2022.pdf` |
| `pdf-2020/` | `pdf-2020/{원본 stem}-2020.pdf` |
| `pdf-2010/` | `pdf-2010/{원본 stem}-2010.pdf` (등급 미달, 권유 안 함) |

원본 파일이 하위 폴더 (`samples/basic/` / `samples/hwpx/`) 에 있는 경우 PDF 도 동일 하위 폴더 구조 유지.

## 5. 변환 결과 통계 (한글 2022 영역)

| 항목 | 값 |
|------|-----|
| 총 입력 (samples 재귀) | 207개 (.hwp + .hwpx) |
| 성공 | **199 (96.1%)** |
| 실패 | 8 (3.9%) |
| 크래시 자동 복구 | 3건 |
| 평균 변환 시간 | ~3.5초/파일 |
| 총 소요 시간 | ~15분 |

### 실패 8개 영역 (한글 2022)
| 파일 | 추정 원인 |
|------|-----------|
| `20250130-hongbo_saved.hwp` | RPC 0x800706BE |
| `20250130-hongbo-no.hwp` | SaveAs False |
| `basic\calendar_monthly.hwp` | 8분 hang |
| `honbo-save.hwp` | 한글 크래시 |
| `hwp_table_test_saved.hwp` | 한글 크래시 |
| `hwp-3.0-HWPML.hwp` | HWPML 형식, SaveAs 실패 |
| `hwpers_test4_complex_table.hwp` | 복잡한 테이블, 크래시 |
| `hwpspec.hwp` | 한컴 스펙 문서, 크래시 |

> 8 실패 영역은 향후 한글 2020 변환 영역 추가 시 동일 패턴 영역 발생 가능 영역 — 영구 보존 영역 부재가 영역 정합.

## 6. devel 머지 + push

### 진행
1. local/devel 에서 `8764b1b` cherry-pick → `69a7078` (충돌 0, author Jaeook Ryu 보존)
2. 본 환경 정합 보강 (CLAUDE.md + pdf/README.md) → `12aa99c` (committer edward)
3. devel ← local/devel ff merge
4. push: `9216781..12aa99c`

### 분기 처리
- 본 cherry-pick 시점 origin/devel 분기 0 — `feedback_release_sync_check` 정합

## 7. PR / 메모리 / 거버넌스 영역

### PR close + 댓글
- PR #670: 한글 댓글 등록 + close (`gh pr close 670`)
- 컨트리뷰터의 한글 2010 / 2020 추가 변환 제안 영역에 대한 응답:
  - 한글 2020 PDF 추가 변환 환영 (별도 PR 환영)
  - 한글 2010 PDF 영역은 정답지 등급 미달 영역 → 영구 보존 영역 부재

### 메모리 룰 갱신
- `reference_authoritative_hancom.md` — 컨트리뷰터 환경별 권위 영역 분리
- `feedback_pdf_not_authoritative.md` — PDF 정답지 등급 영역 분리
- `MEMORY.md` — 두 메모리 룰의 description 갱신

### 거버넌스 영역
- `pdf/README.md` 신규 작성 — 컨트리뷰터 친화 영역
- `CLAUDE.md` 갱신 — 향후 컨트리뷰터 증가 시 일관 규약 영역

## 8. 메모리 룰 적용

- `reference_authoritative_hancom` (갱신) — 컨트리뷰터 환경별 권위 영역 분리. 본 PR 영역의 본질 영역 정합
- `feedback_pdf_not_authoritative` (갱신) — PDF 정답지 등급 영역 분리. 한글 2020/2022 PDF 만 정답지 영역
- `feedback_assign_issue_before_work` — 본 PR 의 closes 키워드 부재 + 이슈 미연결 영역 (외부 컨트리뷰터 자기 등록 영역, 일차 방어선 부재 사례)
- `feedback_pr_comment_tone` — 작업지시자의 "와우! 고생하셨습니다." 영역 + 본 영역의 차분 + 사실 중심 영역 응답 영역 정합
- `project_dtp_identity` — 외부 컨트리뷰터 다양성 인프라 영역의 본질 영역 강화. "닫힌 포맷의 벽 깨기" 영역 정합
- `feedback_visual_regression_grows` — 본 PDF 영역은 시각 비교 자료 영역 (작업지시자 시각 판정 영역의 보조 자료 영역)

## 9. 다음 사이클 영역

- 한글 2020 PDF 추가 변환 영역 (외부 컨트리뷰터 환영 영역 또는 작업지시자 직접 영역)
- 향후 PDF 비교 자동화 영역 (이슈 #253 — 한컴 PDF 기준 Visual Diff 파이프라인) 의 한글 2020/2022 영역 한정 영역 활성화 영역
- 한글 2010 PDF 영역의 발견 영역 (정답지 등급 미달 영역의 활용 영역 — 보조 자료 영역의 별도 영역 정합)
