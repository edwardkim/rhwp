---
타스크: #178 HWPX→HWP IR 매핑 어댑터
단계: Stage 5 — 통합 진입점 + 전 영역 결합 검증
브랜치: local/task178
작성일: 2026-04-19
선행: Stage 4 완료
---

# Stage 5 단계별 완료 보고서

## 1. 목표 (구현계획서 §2 Stage 5)

`DocumentCore::export_hwp_with_adapter()` 통합 진입점 추가 + 3개 디버그 샘플의 페이지 수
회복을 단일 호출 경로로 검증.

## 2. 산출물

### 2.1 신규 메서드

[src/document_core/commands/document.rs:454-465](src/document_core/commands/document.rs#L454-L465)

```rust
pub fn export_hwp_with_adapter(&mut self) -> Result<Vec<u8>, HwpError> {
    use crate::document_core::converters::hwpx_to_hwp::convert_if_hwpx_source;
    let _report = convert_if_hwpx_source(&mut self.document, self.source_format);
    self.export_hwp_native()
}
```

`source_format` 검사 → HWPX 출처면 어댑터 호출 → HWP 직렬화. HWP 출처는 어댑터 no-op
이므로 `export_hwp_native` 와 동일 결과.

### 2.2 HWP 직렬화기 변경

**0줄 수정** — 정체성 (구현계획서 §1.0.1) 준수.
`export_hwp_native` 도 변경 없음 (수정계획서 §1.2 정책 유지).

## 3. 검증 결과

### 3.1 통합 테스트 (21개, +4)

```
stage5_export_hwp_with_adapter_hwpx_source_recovers_pages ... ok
  ← HWPX 출처 → 통합 진입점 → 페이지 수 보존
stage5_export_hwp_with_adapter_hwp_source_unchanged ... ok
  ← HWP 출처 → 통합 진입점 → export_hwp_native 와 bytes 동일
stage5_export_hwp_with_adapter_idempotent_on_repeated_calls ... ok
  ← 같은 DocumentCore 에 2회 호출 → 동일 bytes
stage5_all_three_samples_recover_via_unified_entry_point ... ok
  ← 3개 디버그 샘플 모두 단일 호출 경로로 회복
```

### 3.2 회귀 (전체 라이브러리)

```
test result: ok. 891 passed; 0 failed; 1 ignored; 0 measured
```

### 3.3 페이지 회복 측정 (Stage 4 와 동일 — 통합 진입점으로 재검증)

| 샘플 | 원본 | 통합 진입점 결과 |
|---|---:|---:|
| hwpx-h-01 | 9 | **9** ✅ |
| hwpx-h-02 | 9 | **9** ✅ |
| hwpx-h-03 | 9 | **9** ✅ |

## 4. 핵심 설계 결정

### 4.1 `export_hwp_native` 비변경 유지

기존 호출자 (CLI `main.rs`, 테스트, `wasm_api`) 가 사용하는 `export_hwp_native` 는
손대지 않음. 새 진입점만 추가. 기존 코드 회귀 위험 0.

### 4.2 어댑터는 IR 을 변경 — `&mut self` 요구

`export_hwp_native` 가 `&self` 인 반면 `export_hwp_with_adapter` 는 `&mut self`.
어댑터가 `paragraph.controls` 에 SectionDef 컨트롤을 삽입하고 `table.raw_ctrl_data` 를
채우므로 IR 자체가 변경됨. idempotent 가드가 있어 같은 호출을 반복해도 결과 동일.

### 4.3 source_format 분기는 호출자 책임

`convert_if_hwpx_source(doc, source_format)` 가 분기 담당. 어댑터 본체는 IR 만 받고
변환을 수행. 분리로 어댑터 함수의 단위 테스트 용이성 보존.

## 5. 정체성 셀프 체크

- [x] HWP 직렬화기 0줄 수정
- [x] 어댑터는 IR 만 만짐 (`&mut Document`)
- [x] idempotent (`stage5_export_hwp_with_adapter_idempotent_on_repeated_calls`)
- [x] HWP 출처 보호 (`stage5_export_hwp_with_adapter_hwp_source_unchanged` 가 native 와 bytes 동일 단언)
- [x] `export_hwp_native` 비변경 정책 유지 (구현계획서 §1.2 + §7.3)

## 6. WASM API 노출 (작업지시자 추가 지시)

본 단계 통합 테스트가 native 검증에 그치면 사용자 시나리오 (브라우저에서 HWPX 저장)
검증 부재 — 작업지시자 지적으로 wasm_api 노출까지 Stage 5 범위로 확장.

### 6.1 변경

[src/wasm_api.rs:2782-2789](src/wasm_api.rs#L2782) `export_hwp`:
- `&self` → `&mut self`
- `export_hwp_native` → `export_hwp_with_adapter`

→ **HWPX 출처 문서를 HWP 로 저장하는 모든 사용자 경로에 어댑터 자동 적용**.
HWP 출처는 어댑터 no-op 이므로 기존 동작과 동일.

### 6.2 추가 통합 테스트 (+1, 총 22개)

```
stage5_wasm_api_export_hwp_uses_adapter ... ok
  ← HwpDocument::from_bytes → export_hwp_with_adapter → 페이지 수 보존
```

### 6.3 wasm_api 회귀

```
test result: ok. 154 passed; 0 failed; 0 ignored; 0 measured (wasm_api lib)
```

## 7. E2E 자동화 검토 결과 — 부적합

작업지시자 추가 통찰:
> "문제는 저장입니다. e2e 에서는 파일 저장시 처리되지 않겠다 라는 생각이 듭니다."

### 7.1 E2E 의 파일 저장 한계

[rhwp-studio/src/command/commands/file.ts:60-85](rhwp-studio/src/command/commands/file.ts#L60-L85) 의 `file:save`:
1. File System Access API (`showSaveFilePicker`) — 사용자 다이얼로그 의존
2. 폴백: Blob → `<a download>` → 브라우저 다운로드 폴더

→ 두 경로 모두 OS 파일 시스템에 의존. puppeteer headless 에서 자동 처리 불가.

### 7.2 E2E 가 검증할 수 있는 것

- WASM `exportHwp()` 호출 + 반환 `Uint8Array` bytes 검증
- 그러나 파일 저장 자체는 우회됨 → **본 타스크의 핵심 사용자 시나리오 (저장 후 한컴 오픈) 미검증**
- native 통합 테스트가 이미 `export_hwp_with_adapter` (WASM 진입점이 호출하는 함수) 를 검증
- E2E 추가 가치 거의 없음

### 7.3 결정 — Stage 7 에서 작업지시자 수동 검증

WASM 빌드 + rhwp-studio 빌드 + 작업지시자가 hwpx-h-01.hwpx → 저장 → 한컴2020 오픈 검증을
Stage 7 에서 한 번에 수행. Stage 5 는 native + wasm_api 노출까지로 종료.

## 8. 다음 단계

Stage 6: 명시적 검증 함수.

`serialize_hwp_with_verify(&mut self) -> HwpExportVerification` — 어댑터 적용 후
직렬화 → 자기 재로드 → 페이지 수 비교를 명시적으로 호출자가 요청할 수 있도록.
구현계획서 Q3 (B) 결정대로 `export_hwp_with_adapter` 와 별개 진입점.

## 9. 승인 요청

본 단계 완료 보고서 승인 후 Stage 6 착수.
