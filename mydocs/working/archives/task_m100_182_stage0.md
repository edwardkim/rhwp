# Stage 0 단계별 완료보고서: 라운드트립 하네스 + 분기 제거 + 한컴 기본값 상수화

- **타스크**: [#182](https://github.com/edwardkim/rhwp/issues/182)
- **마일스톤**: M100 (v1.0.0)
- **브랜치**: `local/task182`
- **일자**: 2026-04-17
- **단계**: Stage 0 / 5 (기반 공사)

## 1. 목표 (수행계획서 기반)

"단편적 실패"를 구조적으로 방지하는 기반 공사:

1. 빈 문서 특수 분기(`mod.rs:61-66`) 제거 — 거짓-양성 테스트 경로 차단
2. `SerializeContext` 뼈대 구축 — 1-pass ID 풀 + `assert_all_refs_resolved()`
3. `IrDiff` 라운드트립 하네스 — 바이트 비교 대신 IR 의미 비교
4. 한컴 OWPML 공식 기본값 상수화 — Stage 1~5의 출력 스펙 근거

## 2. 산출물

### 2.1 신규 파일

| 파일 | 줄 수 | 역할 |
|---|---|---|
| `src/serializer/hwpx/canonical_defaults.rs` | 약 200 | 한컴 OWPML 공식 기본값·enum 상수 테이블 (Apache 2.0 참조 고지) |
| `src/serializer/hwpx/context.rs` | 약 250 | `SerializeContext`, `IdPool<T>`, `BinDataEntry`, `assert_all_refs_resolved` |
| `src/serializer/hwpx/fixtures.rs` | 약 30 | `EMPTY_HEADER_XML`, `EMPTY_SECTION0_XML`, `EMPTY_CONTENT_HPF` — 테스트 대조용만 |
| `src/serializer/hwpx/roundtrip.rs` | 약 230 | `IrDiff`, `roundtrip_ir_diff`, `diff_documents` (Stage 0 최소 필드) |
| `tests/hwpx_roundtrip_integration.rs` | 약 30 | `stage0_blank_hwpx_roundtrip` — 누적 하네스 진입점 |

### 2.2 수정 파일

- `src/serializer/hwpx/mod.rs`:
  - 빈 문서 분기(`doc.sections.len() == 1 && doc.bin_data_content.is_empty()`) **제거**
  - `SerializeContext::collect_from_document(doc)` 호출 + 끝에서 `ctx.assert_all_refs_resolved()` 단언
  - 모듈 선언 추가: `canonical_defaults`, `context`, `fixtures`, `roundtrip`
  - 문서 주석 업데이트 (Stage 0~5 로드맵 반영)

- `THIRD_PARTY_LICENSES.md` (이전 커밋에서 반영 완료):
  - "참조한 오픈소스 프로젝트" 섹션: hancom-io/hwpx-owpml-model, hancom-io/dvc 명시

## 3. 검증 결과

### 3.1 단위 테스트

```
serializer::hwpx 관련 단위 테스트: 25개 (기존 10 + 신규 15)
- canonical_defaults::tests: 5개 통과
- context::tests: 4개 통과
- fixtures::tests: 2개 통과
- roundtrip::tests: 3개 통과
- 기존 mod::tests: 11개 유지 통과
```

전체 라이브러리 테스트: **818 passed, 0 failed, 1 ignored** (회귀 없음).

### 3.2 통합 테스트

```
running 1 test
test stage0_blank_hwpx_roundtrip ... ok

test result: ok. 1 passed; 0 failed
```

`blank_hwpx.hwpx` 라운드트립 IrDiff 0 달성.

### 3.3 참조 정합성 단언

빈 Document에 대해 `assert_all_refs_resolved()` 통과. 고의로 등록되지 않은 ID를 참조하는 테스트(`unresolved_char_pr_fails`)는 기대대로 `SerializeError::XmlError`로 실패하며, 에러 메시지에 `charPrIDRef`와 누락 ID가 포함됨을 확인.

### 3.4 빈 문서 분기 제거 확인

```bash
$ rg "bin_data_content.is_empty" src/serializer/
# (no matches) — 분기 완전 제거
```

## 4. 완료 기준 대조

수행계획서 Stage 0 완료 기준:

| 기준 | 상태 | 근거 |
|---|---|---|
| 기존 단위 테스트 10개 green | ✅ | `mod::tests` 11개 전부 통과 (기존과 동일) |
| `stage0_blank_hwpx_roundtrip` IrDiff 0 | ✅ | `tests/hwpx_roundtrip_integration.rs` 통과 |
| `ctx.assert_all_refs_resolved()` 전 경로 통과 | ✅ | `serialize_hwpx` 끝부분에서 단언, 818개 기존 테스트 모두 통과 |
| `canonical_defaults.rs` 주요 20+ 상수 | ✅ | 60여 개 상수 등록 (charPr/paraPr/borderFill/cellSpan/RunType/Table/Pic/Sz/Numbering/PageBorderFill/BeginNum/Font + enum 9종) |
| 분기 `mod.rs:61-66` 제거 확인 | ✅ | `rg` 검색 무매칭 |
| THIRD_PARTY_LICENSES.md 한컴 참조 명시 | ✅ | 이전 커밋 반영 |

## 5. 주요 설계 결정

### 5.1 `IdPool<T>`의 양방향 추적

```rust
pub struct IdPool<T: Copy + Eq + std::hash::Hash> {
    registered: HashSet<T>,  // header.xml에서 정의됨
    referenced: HashSet<T>,  // section.xml 등에서 참조됨
}
```

Stage 1~4에서 writer가 `reference(id)` 호출하면 자동으로 미등록 참조를 탐지. "section에선 쓰는데 header엔 없는" 단편적 실패를 컴파일 타임에는 아니지만 **런타임 단언**으로 차단.

### 5.2 `IrDiff`의 누적 확장 설계

Stage 0에선 뼈대 필드(섹션 수·리소스 카운트)만 비교. Stage 1~5에서 `IrDifference` enum variant를 추가하면서 자동으로 확장되는 구조. **삭제·완화 금지** — `is_empty()` 는 하드 요구사항.

### 5.3 `canonical_defaults.rs`의 Apache 2.0 고지

파일 헤더 주석에 명시:
```rust
//! Default values and enum definitions referenced from
//! hancom-io/hwpx-owpml-model (Apache License 2.0, © 2022 Hancom Inc.).
```

각 상수마다 출처 `.cpp:라인` 주석으로 추적 가능. 코드를 복사하지 않고 스펙 정보만 참조하는 범위를 명시.

### 5.4 `fixtures.rs`로 템플릿 격리

Stage 0 이후 `templates/*.xml`은 실코드 경로에서 사용되지 않고 **테스트 대조용 fixture**로만 남는다. `fixtures::EMPTY_HEADER_XML` 등으로 접근. 점진적으로 Stage 1~2에서 동적 생성 결과와 비교하는 단위 테스트에서 활용 예정.

## 6. 알려진 제한

- **Stage 0은 기반 공사**다. 실제 ID 스캔·참조는 Stage 1~4에서 각 writer가 활성화되면서 채워진다. 현재 `SerializeContext`는 리소스 카운트만 등록하고 참조는 추가되지 않는다 (= 현재 모든 단언이 자동 통과).
- `IrDiff`도 현재 뼈대 필드만 비교한다. Stage 1~5가 진행되며 비교 대상 필드가 누적 확장된다.

이것은 설계된 의도이며, 이번 Stage 0의 "통과"가 거짓-양성이 아닌 이유는:
1. 분기 제거로 **실문서도 동일 경로**를 탄다
2. `blank_hwpx.hwpx`는 실제 HWPX 샘플이며, 기존 분기에선 fixture로 우회되던 경로를 지금은 동적 경로로 통과
3. Stage 1 이후 IrDiff 비교 범위가 점진적으로 강화되어 회귀 방지망이 조여진다

## 7. 다음 단계 (Stage 1)

**Stage 1 — header.xml IR 기반 동적 생성**:

- `src/serializer/hwpx/header.rs` 13줄 → ~400줄로 확장
- `HeaderWriter<'a>` 구조체 + 9개 writer 함수 (fontfaces/borderFills/charProperties/tabProperties/numberings/paraProperties/styles/compatDoc/docOption)
- 한컴 canonical 속성·자식 순서 준수 (CharShapeType.cpp:59-86, ParaShapeType.cpp:50-68 기준)
- 완료 기준: `ref_empty.hwpx` 라운드트립 IrDiff 0 + charPrIDRef/paraPrIDRef 전 참조 resolve

## 8. 승인 요청

본 Stage 0 완료보고서 검토 후 승인 시 Stage 1 착수. 피드백 요청 시 `mydocs/feedback/` 에 등록 부탁드립니다.
