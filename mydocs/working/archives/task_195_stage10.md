# Task #195 단계 10 완료보고서 — EMF 모듈 골격 + EMR_HEADER 파서

## 수행 내용

`src/emf/` 신규 모듈을 생성하고 `EMR_HEADER` 레코드를 구조체로 파싱하는 최소 구현을 추가했다. 레코드 디스패처는 헤더/EOF만 분기하고, 그 외 레코드는 `Record::Unknown { record_type, payload }`로 보존한다(후속 단계에서 확장).

## 추가 파일

| 파일 | 역할 |
|------|------|
| `src/emf/mod.rs` | 공개 API (`parse_emf`, `Error`, `Record`, `Header` 재노출) |
| `src/emf/parser/mod.rs` | 레코드 디스패처 + `Cursor` 리틀엔디언 스트림 리더 |
| `src/emf/parser/constants/mod.rs` | 상수 엔트리 |
| `src/emf/parser/constants/record_type.rs` | `RecordType` enum (1차 범위 발췌, 단계별 카테고리 구분) |
| `src/emf/parser/objects/mod.rs` | 공통 구조체 엔트리 |
| `src/emf/parser/objects/header.rs` | `Header`, `HeaderExt1`, `HeaderExt2` (Size 기반 확장 분기) |
| `src/emf/parser/objects/rectl.rs` | `RectL`, `PointL`, `SizeL` + `read` 헬퍼 |
| `src/emf/parser/records/mod.rs` | `Record` enum (Header/Eof/Unknown) |
| `src/emf/parser/records/header.rs` | EMR_HEADER 파서 래퍼 |
| `src/emf/tests.rs` | 단위 테스트 6건 |

## 수정 파일

| 파일 | 변경 |
|------|------|
| `src/lib.rs` | `pub mod emf;` 추가 |

## 검증 결과

```
$ cargo build --release
    Finished `release` profile [optimized] target(s) in 13.48s

$ cargo test --release emf::
running 6 tests
test emf::tests::parses_minimal_header_and_eof ... ok
test emf::tests::parses_header_with_extensions ... ok
test emf::tests::preserves_unknown_records_as_payload ... ok
test emf::tests::rejects_misaligned_record ... ok
test emf::tests::rejects_non_header_first_record ... ok
test emf::tests::rejects_bad_signature ... ok
test result: ok. 6 passed; 0 failed; 891 filtered out
```

전체 회귀 (`cargo test --release`):
- 라이브러리 단위 테스트: 896 passed (기존 890 → 신규 6 추가)
- 통합 테스트: 13 passed
- WMF 회귀: 영향 없음 (독립 모듈)

## 단위 테스트 6건 요약

1. **parses_minimal_header_and_eof** — 88B 헤더 + 20B EOF 픽스처 파싱, Signature/Bounds/Device/Handles 검증
2. **rejects_bad_signature** — offset 40 변조 시 `InvalidSignature` 오류
3. **rejects_non_header_first_record** — 선두 레코드 Type≠1 시 `InvalidFirstRecord` 오류
4. **rejects_misaligned_record** — Size 필드가 4의 배수가 아닐 때 `MisalignedRecord` 오류
5. **parses_header_with_extensions** — Size=108(ext1+ext2)에서 Extension 1/2 필드 읽기
6. **preserves_unknown_records_as_payload** — 미분기 레코드가 `Record::Unknown`으로 보존되는지

## 설계 결정 사항

- **Cursor 구조**: WMF 모듈과 독립적으로 `Cursor<'a>` 최소 리더 구현. `embedded-io`에 의존하지 않음(WMF의 `Read` 트레이트 매크로 시스템 미도입).
- **에러 타입**: `thiserror` 대신 수동 `Display` + `std::error::Error` 구현(신규 의존성 회피).
- **RecordType enum**: 1차 범위(GDI 기본 + 텍스트/비트맵) 값만 열거. `from_u32`는 단계 10 현재 Header/Eof만 반환하며, 후속 단계에서 필요한 항목 추가.
- **Extension 분기**: `Size` 필드 값(≥100, ≥108)으로 ext1/ext2 존재 여부 판정. Description/PixelFormat 페이로드는 스킵하지만 헤더 offset 필드는 보존.
- **Unknown 레코드**: 페이로드는 type/size 8B를 **제외한** 나머지. 후속 단계가 type 분기로 재파싱할 때 size 정보 불필요.

## 기존 테스트 영향

- 기존 890 단위 테스트 모두 통과
- `parser::control::shape::parse_ole_shape` doctest 실패는 **선행 단계(#195 단계 3) 코드의 기존 이슈**로 EMF 변경과 무관 (stash 후 재확인)

## 미해결 이슈

- doctest 실패 정리 — 단계 3 코드의 `parse_ole_shape` docstring이 Rust code block으로 파싱되는 것이 원인. 별도 처리 필요 (현 단계 범위 외)
- `RecordType::from_u32` 전체 분기 — 단계 11~13에서 필요한 값만 점진 확장

## 다음 단계

**단계 11**: 객체 레코드(펜/브러시/폰트 생성·선택·삭제) + 상태 레코드(DC 스택, WorldTransform, Window/Viewport) 파서 + `DeviceContext`/`ObjectTable` 컨버터 기반 구조.
