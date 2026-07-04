# Stage 2 완료보고서 — Task #1534

> `attr_str` unescape 적용 (수정) → 회귀 테스트 green

- **이슈**: [#1534](https://github.com/edwardkim/rhwp/issues/1534)
- **브랜치**: `local/task1534`
- **단계**: 2/4 (수정 적용)
- **작성일**: 2026-06-25

---

## 1. 수정 내용

`src/parser/hwpx/utils.rs` `attr_str` 가 속성값을 XML 엔티티 unescape 하도록 변경.

```rust
pub fn attr_str(attr: &quick_xml::events::attributes::Attribute) -> String {
    let raw = String::from_utf8_lossy(&attr.value);
    match quick_xml::escape::unescape(&raw) {
        Ok(value) => value.into_owned(),
        Err(_) => raw.into_owned(),
    }
}
```

이제 IR 은 시맨틱 값(`R&&D`)을 보관하고, 직렬화 측 writer 가 유일한 escape 권위가
되어 대칭이 성립한다.

### API 선택 근거

- quick-xml 0.40 의 `Attribute::unescape_value()` 는 **deprecated**(→ `normalized_value`).
- 그러나 `normalized_value(version)` 는 엔티티 해제 외에 **공백(`\t`/`\n`/`\r`)을 space
  로 접는** XML 속성값 정규화까지 수행 → 범위 밖 거동 변화(잠재 회귀)라 비채택.
- **`quick_xml::escape::unescape(&str)`** (non-deprecated, free 함수)는 **엔티티만**
  해제하고 공백을 보존 → escape 측(`escape_xml_text`/writer)과 정확히 대칭. 채택.
- 미정의 엔티티/malformed 는 원문 lossy 로 안전 폴백.
- 빌드 경고 0건 확인.

## 2. 검증 결과

### 회귀 테스트 (Stage 1) — 전 red → green 전환

```
test fixture_has_ampersand_caption ... ok
test saved_xml_has_no_double_escape ... ok
test form_caption_survives_roundtrip ... ok
test form_caption_stable_across_two_roundtrips ... ok
```

### 바이트 레벨 2회 누적 수동 검증 (`form-002.hwpx`)

| 단계 | section0.xml caption | 디코딩 값 |
|------|----------------------|-----------|
| 원본 | `caption="IP R&amp;&amp;D연계"` | `IP R&&D연계` |
| 1회 저장 | `caption="IP R&amp;&amp;D연계"` | `IP R&&D연계` |
| 2회 저장 | `caption="IP R&amp;&amp;D연계"` | `IP R&&D연계` |

- 1·2회 저장 모두 `&amp;amp;` 이중 이스케이프 **0건**.
- 저장할수록 자라던 결함이 **바이트 안정**으로 해소됨.

### 기존 테스트 무회귀 (smoke)

- `hwpx_form_roundtrip` : 1 passed
- `hwpx_roundtrip_baseline` : 4 passed (xfail 0 유지)

> 전체 `cargo test` 및 전수 export-text 재비교는 Stage 3 에서 수행.

## 3. 산출물 / 커밋

- `src/parser/hwpx/utils.rs` (수정)
- `mydocs/working/task_m100_1534_stage2.md` (본 보고서)

## 4. 다음 단계

Stage 3 — 전수 회귀 검증(전체 `cargo test` + baseline + 전수 export-text 재비교 +
golden 스냅샷 점검) 및 가드 보강 확정.
