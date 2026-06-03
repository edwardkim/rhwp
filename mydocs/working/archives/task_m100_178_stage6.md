---
타스크: #178 HWPX→HWP IR 매핑 어댑터
단계: Stage 6 — 명시적 검증 함수
브랜치: local/task178
작성일: 2026-04-19
선행: Stage 5 완료
---

# Stage 6 단계별 완료 보고서

## 1. 목표 (구현계획서 §2 Stage 6, Q3 결정 B)

자기 재로드 검증을 별도 명시 호출로. 운영 경로 (`export_hwp_with_adapter`) 는 검증 비용을
부담하지 않으며, 진단·테스트·사용자 경고가 필요한 경우에만 본 함수 사용.

## 2. 산출물

### 2.1 신규 구조체

[src/document_core/commands/document.rs](src/document_core/commands/document.rs):

```rust
pub struct HwpExportVerification {
    pub bytes: Vec<u8>,
    pub bytes_len: usize,
    pub page_count_before: u32,
    pub page_count_after: u32,
    pub recovered: bool,
}
```

### 2.2 신규 메서드 (네이티브)

```rust
pub fn serialize_hwp_with_verify(&mut self) -> Result<HwpExportVerification, HwpError> {
    let page_count_before = self.page_count();
    let bytes = self.export_hwp_with_adapter()?;
    let bytes_len = bytes.len();
    let reloaded = DocumentCore::from_bytes(&bytes)?;
    let page_count_after = reloaded.page_count();

    Ok(HwpExportVerification {
        bytes, bytes_len,
        page_count_before, page_count_after,
        recovered: page_count_before == page_count_after,
    })
}
```

### 2.3 WASM 노출

[src/wasm_api.rs](src/wasm_api.rs):

```rust
#[wasm_bindgen(js_name = exportHwpVerify)]
pub fn export_hwp_verify(&mut self) -> Result<String, JsValue> { ... }
```

JSON 반환:
```json
{"bytesLen":678912,"pageCountBefore":9,"pageCountAfter":9,"recovered":true}
```

### 2.4 HWP 직렬화기 변경

**0줄 수정** — 정체성 (구현계획서 §1.0.1) 준수.

## 3. 검증 결과

### 3.1 통합 테스트 (25개, +3)

```
stage6_verify_recovered_for_hwpx_h_01 ... ok
  → before=9, after=9, recovered=true, bytes=678912
stage6_verify_recovered_for_all_three_samples ... ok
  → 3개 디버그 샘플 모두 recovered=true
stage6_verify_for_hwp_source_also_recovered ... ok
  → HWP 출처 (어댑터 no-op) 도 자기 재로드 일치
```

### 3.2 회귀 (전체 라이브러리)

```
test result: ok. 891 passed; 0 failed; 1 ignored; 0 measured
```

## 4. 핵심 설계 결정 — 운영 경로와 검증 경로 분리

### 4.1 `export_hwp_with_adapter` (운영) vs `serialize_hwp_with_verify` (진단)

| 함수 | 호출 비용 | 용도 |
|---|---|---|
| `export_hwp_with_adapter` | paginate + 직렬화 | 사용자 저장 (운영) |
| `serialize_hwp_with_verify` | 위 + from_bytes (= +paginate) | 진단·테스트·경고 |

운영 경로에서 매번 검증을 수행하면 큰 문서에서 수백 ms 추가 비용. 작업지시자 결정 (B) 대로
선택적 명시 호출.

### 4.2 WASM 측은 메타데이터만 반환

`export_hwp_verify` 는 JSON 메타데이터만 반환하며 bytes 자체는 별도 `exportHwp` 호출.
이는 호출자 (UI) 가 검증 실패 시 어떤 동작을 취할지 결정할 수 있게 함:
- `recovered=true` → 정상 다운로드
- `recovered=false` → 사용자 경고 후 다운로드 또는 취소

## 5. 정체성 셀프 체크

- [x] HWP 직렬화기 0줄 수정
- [x] 어댑터 본체 변경 없음 (Stage 5 완성된 진입점 재사용)
- [x] HWP 출처 보호 (verify 도 어댑터 no-op 분기 그대로)
- [x] 운영 경로 영향 없음 (verify 는 별도 함수, `export_hwp` 는 변경 없음)

## 6. 다음 단계

Stage 7: UI 복원 + 한컴 검증 + 배포 준비.

`rhwp-studio` 의 `file:save` / `hwpctl.SaveAs` 분기를 변경하여 HWPX 출처도 HWP 저장
(파일명 정규화 포함). WASM 빌드 (이미 완료) → 작업지시자가 hwpx-h-01.hwpx → 저장 →
한컴2020 으로 정상 오픈 확인. 통과 시 #178 완료.

## 7. 승인 요청

본 단계 완료 보고서 승인 후 Stage 7 착수.
