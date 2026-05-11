# Task #833 최종 결과 보고서

**제목**: rhwp-studio: "다른 이름으로 저장" 메뉴/단축키 추가 + 저장 권한 프롬프트 취소 시 fallback download 오발현 정정
**마일스톤**: v1.0.0 (M100)
**브랜치**: `local/task833` (base: `local/devel`)
**이슈**: [#833](https://github.com/edwardkim/rhwp/issues/833) (범위 확장 코멘트 포함)

## 결론

file 저장 흐름 2건 통합 정정 완료. 작업지시자 시각 판정 통과. PR 로 `closes #833`.

## 결함별 정정 내용

### (A) Save As 기능 부재
**본질**: rhwp-studio 에 Save As 기능 부재. `file:save` (Ctrl+S) 만 존재하며 `currentFileHandle` 자동 overwrite.

**정정**:
- `SaveDocumentOptions.forceSaveAs?: boolean` 옵션 추가
- `saveDocumentToFileSystem` 분기 — `forceSaveAs` 시 `currentHandle` 우회 → 항상 `showSaveFilePicker`
- `file:save-as` command 신규 (`file:save` 와 동일 패턴 + `forceSaveAs: true` + currentFileHandle/fileName 갱신)
- 메뉴 항목 추가 (파일 메뉴 "다른 이름으로 저장(A)...")
- 단축키 등록 (`Ctrl+Shift+S` 영문 + 한글 IME `ㄴ`)

### (B) 저장 권한 프롬프트 취소 시 fallback download 오발현
**본질**: Chrome "변경사항을 저장하시겠습니까?" 권한 프롬프트 → "취소" → catch 가 `AbortError` 만 swallow → `NotAllowedError` 등 다른 cancel 에러는 fallback path 진입 → blob URL download → chrome-extension viewer 자동 연결.

**정정**:
- `isUserCancelError(e)` 모듈-레벨 helper 신규 — `AbortError + NotAllowedError` 모두 검출
- `file:save` + `file:save-as` 양쪽 catch 에 동일 적용

## 검증

### 자동
| 항목 | 결과 |
|---|---|
| `cargo test --release` | 1351 passed, 0 failed (TS 만 변경, Rust 무영향) |
| `cargo clippy --release -- -D warnings` | clean |
| `npx tsc --noEmit` | clean |
| WASM 재빌드 | 불필요 (TS 만) |

### 시각 (작업지시자)

**(A) Save As**
- ✅ 임의 HWP 문서 로드 → 파일 메뉴 → "다른 이름으로 저장(A)..." 항목 표시
- ✅ 클릭 → showSaveFilePicker 다이얼로그 표시
- ✅ 새 파일명 입력 + 저장 → 새 파일 생성 + currentFileHandle 갱신 (이후 Ctrl+S 는 새 파일에 overwrite)
- ✅ Ctrl+Shift+S 단축키 동작 (영문 + 한글 IME)
- ✅ Ctrl+S 회귀 부재

**(B) Cancel fallback 정정**
- ✅ Ctrl+S → "변경사항 저장" 권한 프롬프트 → "취소" → download 미발현
- ✅ showSaveFilePicker 다이얼로그 → 취소 → download 미발현 (기존 AbortError swallow 회귀 부재)
- ✅ 정상 저장 path 회귀 부재

## 단계 진행 + commit 요약

| 단계 | commit | 내용 |
|---|---|---|
| 1 (GREEN) | (Stage 1) | file:save-as command + 권한 cancel fallback 우회 |
| 2 (회귀) | `08930c3` | tsc + cargo test 1351/1351 + clippy 회귀 부재 |
| 3 (최종) | (본 commit) | 시각 판정 통과 + 보고서 + closes #833 |

## 산출물

- `rhwp-studio/src/command/file-system-access.ts` — `forceSaveAs` 옵션
- `rhwp-studio/src/command/commands/file.ts` — `isUserCancelError` helper + `file:save` catch 정정 + `file:save-as` command
- `rhwp-studio/src/command/shortcut-map.ts` — Ctrl+Shift+S binding
- `rhwp-studio/index.html` — 파일 메뉴 항목
- 수행/구현 계획서 + 단계 1-2 보고서 + 본 최종 보고서

## 알려진 제약 / 후속 사항

- HWPX 출처 파일은 `file:save-as` 도 비활성 (`#196` 정합) — 별도 이슈 (HWPX 완전 변환기 #197 후 검토)
- 폴백 path (File System Access API 미지원 브라우저) — `showSaveAs` dialog → blob download (기존 패턴 유지)

## 메모리 룰 정합

- `feedback_visual_judgment_authority` ✅ — 작업지시자 시각 판정 게이트 통과 후 commit
- `feedback_process_must_follow` ✅ — 수행/구현 계획서 → 3단계 절차 + 범위 확장 시 코멘트 + 수행계획서 갱신
- `feedback_hancom_compat_specific_over_general` ✅ — `isUserCancelError` 가 `AbortError + NotAllowedError` 만 좁게 swallow (다른 에러는 기존 fallback 진입 유지)
