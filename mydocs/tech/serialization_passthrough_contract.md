---
kind: reference
status: active
canonical: mydocs/tech/hwp_save_guide.md
last_verified: 2026-07-19
---

# 직렬화 passthrough·무효화 계약

## 이 문서의 목적

`Document` IR을 편집하는 코드(주로 `src/document_core/`의 command·query)가 **저장에서 편집이
사라지는 결함**을 반복해서 만들어 왔다. 원인은 하나의 계약을 몰랐기 때문이다.

> HWP 저장 경로는 성능과 완벽한 라운드트립을 위해 **원본 바이트 passthrough**를 쓴다.
> 파서가 채워 둔 원본 바이트가 남아 있으면 직렬화기는 **현재 IR을 무시하고 원본 바이트를 그대로
> 반환**한다. 따라서 IR을 편집한 뮤테이터는 반드시 해당 계층의 passthrough를 **무효화**해야
> 그 편집이 저장 결과에 반영된다.

[저장 기술 가이드](hwp_save_guide.md)는 "IR → 바이트"를 어떻게 쓰는지를 다룬다. 이 문서는 그
직렬화가 **애초에 실행되기 위한 전제조건**, 즉 무효화 계약을 다룬다. 두 문서는 상호 보완이다.

이 계약은 세 계층에 걸쳐 있다. 그중 **레코드 계층(`raw_data`)의 clone-staleness**는 이미
트러블슈팅 문서 [raw_data 를 문 clone 뮤테이션](../troubleshootings/raw_data_stale_clone_on_mutation.md)에
증상·원인·정정 관례로 정리되어 있다(PR #2416/#2421 계열). 이 문서는 그 확정된 패턴을 **세 계층 전체를
아우르는 단일 계약**으로 승격하고, 특히 트러블슈팅 문서가 다루지 않은 **섹션 계층(`raw_stream`)과
DocInfo 계층(`raw_stream_dirty`)**을 함께 문서화한다. 레코드 계층의 세부 메커니즘은 반복하지 않고 위
트러블슈팅 문서를 따른다.

## 왜 조용히 깨지는가 (핵심)

이 결함이 위험한 이유는 **화면은 맞고 저장만 틀리기** 때문이다.

- 편집 후 뮤테이터가 `recompose_section`을 호출하면 조판 결과(`composed[]`)와 dirty 플래그가
  갱신되어 **화면에는 편집이 정확히 반영**된다.
- 그러나 `recompose_section`은 `raw_stream`을 **건드리지 않는다**
  (`src/document_core/queries/rendering.rs`의 `recompose_section`, 약 2730행).
- 그 결과 저장 시 `serialize_section`은 파서가 넣어 둔 **원본 바이트를 그대로 반환**하고, 방금 한
  편집은 파일에서 사라진다.

즉 수동 테스트(편집→화면 확인)로는 절대 드러나지 않고, **저장→재로드** 또는 바이트 단위 검증에서만
드러난다. 이 비대칭이 이 결함 부류가 반복 유입되는 근본 이유다.

## 세 계층과 게이트

passthrough는 세 계층에 독립적으로 존재한다. 편집한 데이터가 어느 계층에 속하는지에 따라 무효화
대상이 다르다.

| 계층 | 원본 바이트 필드 | 직렬화 게이트(원본 반환 조건) | 파서가 채우는 곳 | 무효화 방법 |
| --- | --- | --- | --- | --- |
| 레코드 | `CharShape/ParaShape/BorderFill/TabDef/...`의 `raw_data: Option<Vec<u8>>` | `serialize_doc_info` — 각 레코드에서 `raw_data`가 `Some`이면 원본 사용 (`src/serializer/doc_info.rs`, 약 54–137행) | 파서 DocInfo 경로 | 해당 레코드의 `raw_data = None` |
| 섹션 | `Section.raw_stream: Option<Vec<u8>>` | `serialize_section` — `raw_stream`이 `Some`이면 즉시 원본 반환 (`src/serializer/body_text.rs`, 약 28행) | `src/parser/mod.rs` 약 465·521행 | 해당 섹션의 `raw_stream = None` |
| DocInfo | `DocInfo.raw_stream_dirty: bool` (+ `DocInfo.raw_stream`) | `serialize_doc_info` — `!raw_stream_dirty`이고 `raw_stream`이 `Some`이면 원본 반환 (`src/serializer/doc_info.rs`, 약 23–24행) | `src/parser/mod.rs` 약 213·501행 | `doc_info.raw_stream_dirty = true` |

파싱 직후 문서는 세 계층 모두 원본 바이트가 채워진 "clean" 상태다. 그래서 **아무것도 무효화하지 않은
편집은 전부 무시된다**는 점이 이 계약의 출발점이다.

> 레코드 계층은 clone·PartialEq·직렬화 우선순위가 얽혀 특히 은밀하다. 그 세부 메커니즘과 정정 관례는
> [raw_data 를 문 clone 뮤테이션](../troubleshootings/raw_data_stale_clone_on_mutation.md)이 권위다.
> 이 문서는 세 계층에 공통인 무효화 의무만 통합해 다룬다.

### 게이트 코드 (원본 그대로)

`serialize_section` — 섹션 계층:

```rust
pub fn serialize_section(section: &Section) -> Vec<u8> {
    // 원본 스트림이 있으면 그대로 반환 (완벽한 라운드트립)
    if let Some(ref raw) = section.raw_stream {
        return raw.clone();
    }
    // ... 여기부터가 IR → 바이트 재직렬화 ...
```

`serialize_doc_info` — DocInfo 계층:

```rust
pub fn serialize_doc_info(doc_info: &DocInfo, doc_props: &DocProperties) -> Vec<u8> {
    // 원본 스트림이 있고 변경되지 않았으면 그대로 반환 (완벽한 라운드트립)
    if !doc_info.raw_stream_dirty {
        if let Some(ref raw) = doc_info.raw_stream {
            // ... 원본 반환 (배포문서 해제 등 국소 수술만 적용) ...
```

두 게이트 모두 **초기 return**이다. IR을 아무리 정확히 바꿔도 게이트가 먼저 원본을 돌려주면 그 아래
재직렬화 코드는 실행조차 되지 않는다.

## 뮤테이터의 의무

IR을 편집하는 함수는 **편집한 데이터가 속한 계층을 무효화**해야 한다. 계층은 중첩될 수 있으므로,
하나의 편집이 둘 이상을 무효화해야 하는 경우가 있다.

- 섹션 본문(문단 텍스트, 컨트롤, 셀 내용, 필드 값 등)을 바꿨다 → 그 **섹션의 `raw_stream = None`**.
- DocInfo 레코드(글자모양·문단모양·테두리 등)를 새로 만들거나 바꿨다 → 그 **레코드의 `raw_data =
  None`** 그리고(레코드 목록 자체가 바뀌므로) **`doc_info.raw_stream_dirty = true`**.
- 새 스타일을 만들어 섹션 문단에 적용했다 → 레코드 계층(`raw_data`/`raw_stream_dirty`)과 섹션
  계층(`raw_stream`) **둘 다** 무효화 대상이다.

### 무효화하는 in-tree 선례 (모두 병합된 코드)

- 섹션 계층: `set_field_text_at`(`src/document_core/queries/field_query.rs`, 약 519행) —
  주석 그대로 "raw_stream 무효화: 직렬화 시 수정된 모델을 사용하도록 강제". 여러 command
  (`clipboard`, `footnote_ops`, `formatting`, `header_footer_ops`, `object_ops/*` 등)도 동일 패턴을
  따른다.
- 레코드 계층: `CharShapeModification::apply_to`·`ParaShapeModification::apply_to`
  (`src/model/style.rs`, 약 759·918행) — "수정된 CharShape/ParaShape는 원본 바이트와 달라지므로
  raw_data 무효화".
- DocInfo 계층: `find_or_create_char_shape`·`find_or_create_para_shape`·`find_or_create_tab_def`
  (`src/model/document.rs`, 약 455·485·501행) — 목록에 레코드를 추가하며
  `raw_stream_dirty = true`.

## 흔한 실수

1. **`recompose_section`이 무효화한다고 착각.** 하지 않는다. `recompose_section`은 조판·dirty
   플래그만 갱신한다(화면용). 저장 반영은 별개로 `raw_stream = None`이 필요하다.
2. **화면으로 검증 끝냈다고 판단.** 화면-정상/저장-오류 비대칭 때문에 화면 확인은 이 결함을
   못 잡는다. 반드시 저장→재로드 또는 직렬화 바이트를 확인한다.
3. **한 계층만 무효화.** 스타일 생성+적용처럼 레코드와 섹션을 동시에 바꾸는 편집은 한쪽만
   무효화하면 절반이 사라진다.
4. **헬퍼가 대신 해줄 것이라 가정.** 무효화를 수행하는 헬퍼(`set_field_text_at` 등)와 하지 않는
   헬퍼(`recompose_section`)가 섞여 있다. 호출하는 헬퍼가 무효화를 포함하는지 개별 확인한다.
5. **셀 내부 편집에서 상위 컨트롤 dirty만 처리.** 셀 내용도 섹션 본문 스트림에 속하므로 소속 섹션의
   `raw_stream` 무효화가 필요하다.

## 라운드트립 검증 패턴 (sentinel)

무효화 여부는 **sentinel 바이트**로 결정적으로 검증할 수 있다. 절대 나올 수 없는 원본 바이트를
심어두고, 뮤테이터 실행 후 직렬화 결과가 그 원본이 아님을 확인한다.

```rust
// 섹션 계층 예시
let mut core = /* 대상 편집이 가능한 최소 문서를 담은 DocumentCore */;
let sentinel = vec![0xABu8; 64];
core.document.sections[0].raw_stream = Some(sentinel.clone());

// 편집 대상 뮤테이터 실행 (예: 필드 값 수정, 책갈피 추가 등)
core.some_mutator_that_edits_section_0(/* ... */).unwrap();

// 무효화되었는가?
assert!(
    core.document.sections[0].raw_stream.is_none(),
    "뮤테이터가 섹션 raw_stream을 무효화하지 않았다 — 편집이 저장에서 사라진다"
);
// 게이트를 실제로 통과해 IR이 재직렬화되는가?
let out = serialize_section(&core.document.sections[0]);
assert_ne!(out, sentinel, "serialize_section이 여전히 원본 바이트를 반환한다");
```

DocInfo 계층은 `doc_info.raw_stream_dirty`가 `true`가 되었는지, 레코드 계층은 대상 레코드의
`raw_data`가 `None`이 되었는지로 동일하게 검증한다.

이 검증은 **red→green으로 신뢰도를 확정**할 수 있다: 뮤테이터에서 무효화 한 줄을 제거하면
테스트가 실패(red)하고, 되돌리면 통과(green)한다. 회귀 테스트를 추가할 때 이 절차로 실제로 그
결함을 잡는지 증명한다.

## 빠른 체크리스트

IR을 편집하는 함수를 추가·수정할 때:

- [ ] 무엇을 바꿨나? 섹션 본문 / DocInfo 레코드 / 둘 다 중 어디인가?
- [ ] 섹션 본문을 바꿨다면 그 섹션의 `raw_stream = None`을 했나?
- [ ] DocInfo 레코드를 바꿨다면 `raw_data = None`과 `raw_stream_dirty = true`를 했나?
- [ ] 내가 호출한 헬퍼가 무효화를 대신 해주는지 개별 확인했나?
- [ ] sentinel 라운드트립 테스트를 추가하고 red→green으로 증명했나?

## 관련 문서

- [HWP 저장 기술 가이드](hwp_save_guide.md) — 이 계약을 통과한 뒤의 "IR → 바이트" 레코드 구조.
- [raw_data 를 문 clone 뮤테이션](../troubleshootings/raw_data_stale_clone_on_mutation.md) — 레코드
  계층(`raw_data`) clone-staleness의 증상·원인·정정 관례 (PR #2416/#2421).
- [HWPX to HWP Hancom Compatibility Rules](../troubleshootings/hwpx2hwp-rule.md) — raw_data 보존 계약의
  한컴 호환 배경(historical).
- [포맷 파서와 공통 Document IR 경계](parser_architecture.md) — 파서가 원본 바이트를 채우는 반대 방향의
  책임 경계.

---

**작성일**: 2026-07-19 — 트러블슈팅에서 확정된 raw_data 무효화 관례를 세 계층(record·section·docinfo)
공통 계약으로 승격. 게이트·뮤테이터 의무·선례·sentinel 검증 패턴 정리.
