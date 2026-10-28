# 최종 결과보고서 — Task #1534

> HWPX 저장 시 폼 컨트롤 속성값(caption) XML 특수문자 이중 이스케이프 누적 — 해결

- **이슈**: [#1534](https://github.com/edwardkim/rhwp/issues/1534)
- **마일스톤**: v1.0.0 (M100)
- **브랜치**: `local/task1534` (base: `stream/devel`)
- **작성일**: 2026-06-25
- **상태**: 완료 (머지·이슈 클로즈 승인 대기)

---

## 1. 문제

HWPX 직렬화(저장) 시 폼 컨트롤(`<hp:checkBtn>` 등)의 **속성값**(`caption`)에 포함된
XML 특수문자(`&`/`<`/`>`/`"`)가 **이중 이스케이프**되어, 저장할 때마다 손상이 한 겹씩
누적됐다. 추가로 렌더링에서도 caption 의 `&` 가 화면에 `&amp;` 로 노출됐다.

```
원본 R&&D → 1회 저장 R&amp;&amp;D → 2회 저장 R&amp;amp;&amp;amp;D → …
```

`samples/hwpx/` 전수 53개 중 폼 caption 에 `&` 를 가진 `form-002.hwpx` 에서 표면화.

## 2. 근본 원인

| 측 | 위치 | 동작 |
|----|------|------|
| 파싱 | `src/parser/hwpx/utils.rs` `attr_str` | quick-xml 원문 바이트를 그대로 보관, **unescape 없음** → IR 에 `R&amp;&amp;D` 저장 |
| 직렬화 | `src/serializer/hwpx/form.rs` `write_form` → writer | 속성값을 **자동 escape** → 다시 escape → `R&amp;amp;&amp;amp;D` |

속성값을 "읽을 때 unescape 안 함 / 쓸 때 escape 함" 의 **비대칭**이 근본 원인. IR 이
시맨틱 값이 아닌 부분 이스케이프된 원문을 들고 있었다(SSOT 위반). 본문 텍스트
(`<hp:t>`)는 `GeneralRef` 이벤트로 정상 디코딩되어 대칭이라 무손상이었다.

## 3. 해결

`attr_str` 가 속성값을 XML 엔티티 unescape 하도록 수정. 직렬화 측 writer 가 유일한
escape 권위가 되어 대칭 성립.

```rust
pub fn attr_str(attr: &quick_xml::events::attributes::Attribute) -> String {
    let raw = String::from_utf8_lossy(&attr.value);
    match quick_xml::escape::unescape(&raw) {
        Ok(value) => value.into_owned(),
        Err(_) => raw.into_owned(),
    }
}
```

- **API 선택**: `Attribute::unescape_value()` 는 deprecated, 대체 `normalized_value()`
  는 공백(`\t`/`\n`/`\r`)을 space 로 접는 부작용이 있어 비채택. 엔티티만 해제하는
  `quick_xml::escape::unescape`(non-deprecated) 채택.
- **범위**: `attr_str` 는 폼 caption 외 모든 문자열 속성(name/command/폰트명 등)의
  동일 잠재 결함을 일괄 해소. 숫자/열거형 속성은 엔티티가 없어 no-op.
- **자기교정**: verbatim 재출력 경로(`<hp:parameters>` `escape_xml_text(&attr_str())`)도
  unescape 후 단일 escape 로 대칭화되어 함께 정상화.

## 4. 검증

| 검사 | 결과 |
|------|------|
| 신규 회귀 테스트 `issue_1534_hwpx_form_caption_escape` | 4/4 green (caption 불변·이중escape 0·2회누적 안정) |
| 전체 `cargo test` (--no-fail-fast) | golden 1건 외 전부 통과, golden 교정 후 **전건 통과** |
| `hwpx_roundtrip_baseline` | 4/4 (xfail 0 유지) |
| batch roundtrip (전수 57) | PASS 56 / IR_DIFF·SERIALIZE·REPARSE·ROUND2 = 0 (hwpx-01 제외) |
| 원본↔저장본 export-text | PASS **50→51** (form-002 전환, 무회귀) |
| form-002 2회 누적 (바이트) | 원본=1회=2회 `R&&D`, `&amp;amp;` 0건 |
| `cargo clippy` / 빌드 경고 | 0 |

**golden 교정**: `tests/golden_svg/form-002/page-0.svg` 에 이중 이스케이프 버그가
박혀 있었음(렌더러도 `&amp;` 노출). caption escape 레벨 3줄만 교정(좌표 무변화) →
화면 표시가 `R&&D` 로 한컴과 일치하게 정상화.

> **잔여 별건**: `k-water-rfp.hwpx` 저장본 페이지 경계 빈 줄 5개 증가(문단 구조·본문
> 보존, 페이지네이션 미세 차이). 본 이슈와 무관 — 필요 시 별도 이슈로 분리 권장.

> **한컴 호환 게이트**: 자기 roundtrip·테스트 통과는 구조 보존을 의미하며, 저장본의
> 한컴 편집기 수동 로드 검증은 작업지시자 환경(Windows + 한컴)에서 최종 확인 권장.

## 5. 변경 파일

| 파일 | 변경 |
|------|------|
| `src/parser/hwpx/utils.rs` | `attr_str` 엔티티 unescape (핵심 1함수) |
| `tests/issue_1534_hwpx_form_caption_escape.rs` | 신규 회귀 테스트(4건) |
| `tests/golden_svg/form-002/page-0.svg` | golden 교정(caption 3줄) |
| `mydocs/plans/task_m100_1534*.md`, `mydocs/working/task_m100_1534_stage{1..3}.md`, 본 보고서 | 문서 |

## 6. 커밋 이력

- Stage 1 `c532b549` — 회귀 테스트(red) + 계획서
- Stage 2 `86ed7827` — `attr_str` unescape 수정
- Stage 3 `838a42ea` — 전수 회귀 + golden 교정
- Stage 4 (본 보고서)

## 7. 결론

폼 caption 의 `&` 이중 이스케이프 누적 결함을 **파싱 측 1함수 수정**으로 근본 해소.
저장·렌더 양쪽이 함께 교정됐고, 회귀 가드(전용 테스트 + golden)를 추가해 재발을
차단했다. 기존 ir-diff/baseline 사각지대(폼 속성 미비교)는 신규 테스트로 보강.

이슈 클로즈 및 `local/task1534` → `local/devel` 머지 승인 요청.
