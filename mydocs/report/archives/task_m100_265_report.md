---
타스크: #265 HWP 3.0 파일 감지 + 친절한 에러 메시지
브랜치: local/task265
작성일: 2026-04-24
상태: 완료
---

# 최종 보고서

## 1. 배경

@jangster77 님 제보 (2026-04-23). 첨부 파일을 rhwp 의 3가지 경로 (HOP · Chrome 확장 · github.io) 어디에서도 열지 못함. 표시된 에러:

```
파일 로드 실패 문서 파싱 실패: 유효하지 않은 파일:
CFB 오류: CFB 열기 실패: Invalid CFB file
(wrong magic number): [48, 57, 50, 20, 44, 6f, 63, 75]
```

## 2. 진단

**매직 넘버 `48 57 50 20 44 6f 63 75` = ASCII `"HWP Docu"`** — HWP 5.0 의 CFB 시그니처 (`D0 CF 11 E0`) 가 아닌 **HWP 3.0** (한글 3.0, 1996~1998년 경) 바이너리 시그니처.

파일 처음 24바이트:
```
4857 5020 446f 6375 6d65 6e74 2046 696c   "HWP Document Fil"
6520 5633 2e30 3020 1a01 0203 0405 0000   "e V3.00ᚱ....."
```

현재 rhwp 는 HWP 5.0 (CFB) 과 HWPX (ZIP+XML) 만 지원. HWP 3.0 정식 지원은 이번 마일스톤 (v1.0.0) 범위 밖.

## 3. 해결 방침

작업지시자 지시: "HWP 3.0 은 이번 마일스톤 범위가 아님. 감지 + 친절한 에러 메시지까지만".

## 4. 변경 사항

### 4.1 `src/parser/mod.rs`

- `FileFormat::Hwp3` variant 추가
- `detect_format` 에 17바이트 프리픽스 체크 (`"HWP Document File"`) — V3.00 / 2.x / 초기 한컴 워디안까지 관대하게 포괄
- `ParseError::UnsupportedFormat { format, hint }` variant 추가
- `Display` 구현 확장: `"지원하지 않는 포맷입니다: {format}. {hint}"`
- `parse_document` 에 `Hwp3` 분기: `UnsupportedFormat("HWP 3.0", 한글 안내 힌트)`

### 4.2 `src/error.rs` — **추가 버그 수정**

`From<ParseError> for HwpError` 등 3개 `From` 구현이 `format!("{:?}", e)` (Debug) 를 사용하고 있어 Display 로 작성한 친절한 메시지가 사용자에게 도달하지 못하던 버그 발견.

**수정**: `format!("{:?}", e)` → `format!("{e}")` (Display) — 3 곳 모두.

회귀 방어 테스트 `parse_error_to_hwp_error_uses_display_not_debug` 추가: ParseError → HwpError 변환 시 Display 전파 + Debug 누출 0 검증.

### 4.3 `rhwp-studio/src/main.ts` — UI 전달

기존엔 상태 표시줄 (`#sb-message`, `height: 22px`, `overflow: hidden`, `text-overflow: ellipsis`) 한 줄에만 쓰고 있어 긴 에러 메시지가 거의 보이지 않았음. 작업지시자 지적으로 추가 발견.

**수정**: `showLoadError(error)` 유틸 함수 신규. 3 경로 (`loadFile` · `loadFromUrlParam` · `open-document-bytes` 이벤트 핸들러) 모두 통합:

- 우상단 토스트 (`showToast`, 기존 유틸 재사용) — 긴 메시지 줄바꿈 · 자동 페이드 없음 · "확인" 버튼으로 사용자 닫기
- 상태 표시줄 메시지 (기존 동작 유지, 흔적 표시)
- 콘솔 에러 로그

**추가 버그 발견·수정**: `open-document-bytes` 이벤트 핸들러 (`main.ts:522`) 가 WASM 파서 throw 를 catch 하지 않아 "파일 열기" 커맨드 경로에선 HWP 3.0 에러가 완전히 묵살되던 문제도 함께 해결.

## 5. 검증 결과

### 5.1 자동

| 항목 | 결과 |
|---|---|
| `cargo test --lib` | **963 passed / 0 failed / 1 ignored** (기존 957 + 신규 6) |
| `cargo test --test svg_snapshot` | 3 passed |
| `cargo clippy --lib -- -D warnings` | clean |
| `tsc --noEmit` (rhwp-studio) | clean |

신규 테스트 6건 breakdown:
- `test_detect_format_hwp3`, `_exact_17_bytes`, `_too_short`
- `test_parse_document_hwp3_returns_unsupported_error`
- `test_parse_document_issue_265_sample` (실파일 end-to-end)
- `parse_error_to_hwp_error_uses_display_not_debug` (Display 회귀 방어)

### 5.2 실파일 UX 검증

**Before** (실제 제보자 로그):
```
[main] 파일 로드 실패: 유효하지 않은 파일: CFB 오류: CFB 열기 실패:
Invalid CFB file (wrong magic number): [48, 57, 50, 20, 44, 6f, 63, 75]
```

**After** (Stage 3 확인, Stage 3 UI 수정 후 Stage 3 재확인):
```
[main] 파일 로드 실패: 유효하지 않은 파일: 지원하지 않는 포맷입니다:
HWP 3.0. 현재 rhwp 는 HWP 5.0 과 HWPX 만 지원합니다. 한컴오피스
또는 LibreOffice 에서 파일을 연 뒤 HWP 5.0 포맷으로 다시 저장하여
시도해주세요.
```

**우상단 토스트** 로 사용자 확인 완료 (작업지시자 확인).

### 5.3 회귀 검증 (Stage 4)

HWP 5.0 2건 (`text-align.hwp`, `biz_plan.hwp`) + HWPX 2건 (`hwpx-02.hwpx`, `blank_hwpx.hwpx`) export-svg 정상 작동 — 회귀 0.

## 6. 산출물

### 코드
- `src/parser/mod.rs`
- `src/error.rs`
- `rhwp-studio/src/main.ts`

### 빌드
- `pkg/rhwp_bg.wasm` 재빌드 (+4.88 KB)

### 문서
- `mydocs/plans/task_m100_265{,_impl}.md`
- `mydocs/working/task_m100_265_stage{1,2,3,4}.md`
- `mydocs/report/task_m100_265_report.md` (본 문서)
- `mydocs/orders/20260424.md` (신규)

## 7. 예상 시나리오 대비 실제 작업량

- 예상: 60분 (Rust 만 변경)
- 실제: 추가 2건 (error.rs Debug 누출 버그 · UI 전달 누락) 이 연쇄 발견되어 약 2.5시간
- **교훈**: 범위 외로 분리하지 않고 연쇄 추가 수정한 판단이 옳았음. 이 2건을 제외하고 머지했으면 "Stage 2 에서 메시지 바꿨으나 사용자 UX 변화 0" 이라는 의미 없는 PR 이 될 뻔함. Root cause 를 끝까지 추적한 것이 유익했음.

## 8. 이번 마일스톤 외 후속 이슈 후보

- HWP 3.0 정식 파싱·렌더 지원 (M101+ 이후. 스펙 접근성 문제 · 수요 확인 필요)
- HWPML 1.x (별도 XML 기반 포맷, 시그니처 `<?xml...`)
- `open-document-bytes` 이외 다른 WASM 호출 경로의 에러 UX 통일 점검

## 9. 결론

- 사용자 친화적 에러 메시지로 "왜 안 열리는지 + 어떻게 해결할지" 명확히 안내
- Display vs Debug 치환 버그까지 뿌리 뽑아 에러 경로 전체 건강성 개선
- 기존 기능 회귀 0
- 3 프론트엔드 (studio · Chrome ext · github.io) 자동 반영

Task #265 종료 승인 요청.
