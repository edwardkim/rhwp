# 타스크 133 최종 결과보고서: 빈 문서 만들기 + 저장

## 개요

빈 문서 생성, 편집, 저장까지의 전체 흐름을 구현하고, 이 과정에서 발견된 직렬화 버그를 수정했다.

## 구현 내용

### 1. 선결 버그 수정 (엔터키 + 직렬화)

| 문제 | 수정 | 파일 |
|------|------|------|
| `split_at()` char_count에 컨트롤 미포함 | 컨트롤 문자도 char_count에 포함하도록 수정 | `src/model/paragraph.rs` |
| `split_at()` raw_header_extra 유실 | 분할된 문단에 원본 raw_header_extra 복사 | `src/model/paragraph.rs` |
| 빈 문단 has_para_text 불일치 → 파일 손상 | cc ≤ 1이고 콘텐츠 없으면 PARA_TEXT 생략 | `src/serializer/body_text.rs` |
| control_mask 고아 → 파일 손상 | 직렬화 시 controls 배열에서 control_mask 재계산 | `src/serializer/body_text.rs` |

### 2. 빈 문서 생성

- 내장 템플릿(`blank2010.hwp`) 기반 `createBlankDocument()` WASM API
- `file:new-doc` 커맨드 활성화 (확인 대화상자 포함)
- `WasmBridge.createNewDocument()` + `_fileName = '새 문서.hwp'`

### 3. 파일 저장 기능

저장 흐름:
1. **showSaveFilePicker** (Chrome/Edge, Secure Context) → OS 네이티브 저장 대화상자 (폴더 + 파일이름 선택)
2. **폴백** (Firefox/Safari/비보안 컨텍스트) → 새 문서면 자체 파일이름 대화상자 → Blob 다운로드

| 구성 요소 | 파일 |
|-----------|------|
| SaveAsDialog (자체 파일이름 입력 대화상자) | `rhwp-studio/src/ui/save-as-dialog.ts` (신규) |
| File System Access API + 폴백 로직 | `rhwp-studio/src/command/commands/file.ts` |
| isNewDocument, set fileName | `rhwp-studio/src/core/wasm-bridge.ts` |

### 4. 셀 나누기 기능 (타스크 135)

- `Table::split_cell_into()` — N×M 분할 알고리즘
- `splitTableCellInto` WASM API + TypeScript 브릿지
- 셀 나누기 대화상자 UI (줄 수/칸 수 + 옵션)
- `table:cell-split` 커맨드 연결

### 5. 셀 나누기 후 저장 손상 수정

- **근본 원인**: 고아 문단의 `control_mask=0x800`(TABLE 비트)이 실제 `controls=[]`과 불일치 + 빈 문단에 PARA_TEXT 기록
- **수정**: `compute_control_mask()` 함수로 직렬화 시 재계산 + `has_para_text` 보정
- 트러블슈팅 문서: `mydocs/troubleshootings/cell_split_save_corruption.md`

## 변경 파일

| 파일 | 변경 |
|------|------|
| `src/model/paragraph.rs` | split_at() char_count/raw_header_extra 수정 |
| `src/model/table.rs` | split_cell_into() + 단위 테스트 |
| `src/serializer/body_text.rs` | compute_control_mask() + has_para_text 보정 |
| `src/wasm_api.rs` | splitTableCellInto WASM API + 진단 테스트 |
| `rhwp-studio/src/command/commands/file.ts` | showSaveFilePicker + 폴백 저장 |
| `rhwp-studio/src/command/commands/table.ts` | 셀 나누기 커맨드 연결 |
| `rhwp-studio/src/core/types.ts` | 셀 나누기 관련 타입 추가 |
| `rhwp-studio/src/core/wasm-bridge.ts` | splitTableCellInto, isNewDocument, set fileName |
| `rhwp-studio/src/engine/input-handler.ts` | 셀 나누기 단축키 |
| `rhwp-studio/src/ui/cell-split-dialog.ts` | 셀 나누기 대화상자 (신규) |
| `rhwp-studio/src/ui/save-as-dialog.ts` | 파일이름 입력 대화상자 (신규) |
| `rhwp-studio/src/ui/table-cell-props-dialog.ts` | 셀 속성 대화상자 개선 |

## 검증

- 582개 테스트 전부 통과
- WASM 빌드 성공
- TypeScript 컴파일 성공
- 한컴오피스에서 저장 파일 정상 오픈 확인
- localhost에서 showSaveFilePicker 네이티브 대화상자 정상 동작 확인
- HTTP 환경에서 폴백(자체 대화상자 + Blob 다운로드) 정상 동작 확인
