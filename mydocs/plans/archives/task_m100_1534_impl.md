# 구현계획서 — Task #1534

> HWPX 저장 시 폼 컨트롤 속성값(caption) XML 특수문자 이중 이스케이프 누적 — 해결 구현

- **이슈**: [#1534](https://github.com/edwardkim/rhwp/issues/1534)
- **브랜치**: `local/task1534` (base: `stream/devel`)
- **수행계획서**: [`task_m100_1534.md`](task_m100_1534.md) (승인 완료)
- **작성일**: 2026-06-25

---

## 1. 채택 방향 — A안 (`attr_str` unescape)

수행계획서 A/B/C 중 **A안 채택**. 감사 결과 A안이 근본 정합이면서 blast radius가
실질적으로 안전함을 확인했다.

### 감사 근거

- `attr_str()` 사용 **228건** (`utils.rs`/`header.rs`/`section.rs`).
- 대부분 `parse_u32`/`parse_bool` 등 **숫자·열거형** 변환의 전단계 → 값에 `&<>"`가
  없으므로 unescape는 **no-op** (무영향).
- 문자열 IR 저장 후 serializer가 다시 escape하는 경로(폼 caption/name, 폰트명,
  스타일명 등)는 현재 **동일 잠재 버그** 보유 → A안으로 일괄 해소.
- **verbatim 재출력 경로**(`section.rs` 4520/4539/4568, `<hp:parameters>` #1391
  재조립)는 `escape_xml_text(&attr_str(&attr))` 구조. 현재 attr_str(원문) + 재escape
  = 이중 이스케이프(잠재 버그). attr_str이 unescape하면 `값 → escape_xml_text` 단일
  escape로 **대칭·정상화**. 즉 A안이 이 경로도 고친다.
- raw 원문(escape된 형태)에 의존해 **escape 없이 verbatim 재출력**하는 경로는 감사
  결과 **없음** → under-escape 위험 없음.

### 구현

`src/parser/hwpx/utils.rs:21` `attr_str` 를 quick-xml 0.40 `unescape_value()` 기반으로
변경 (실패 시 기존 lossy 원문 폴백 — 미정의 엔티티/malformed 안전):

```rust
pub fn attr_str(attr: &quick_xml::events::attributes::Attribute) -> String {
    match attr.unescape_value() {
        Ok(v) => v.into_owned(),
        Err(_) => String::from_utf8_lossy(&attr.value).to_string(),
    }
}
```

- `unescape_value()`는 미리 정의된 5개 엔티티(`&amp; &lt; &gt; &quot; &apos;`)와
  숫자 문자 참조를 해석. 정상 HWPX 속성은 항상 escape되어 있으므로 손실 없음.
- 직렬화 측 writer(`start_tag_attrs`)가 **유일한 escape 권위**가 되어 대칭 성립.

> **폴백(리스크 시)**: 전수 golden 변동이 과다하거나 예기치 않은 회귀 발견 시,
> 신규 `attr_text()`(unescape) 를 도입해 폼 caption/name/command/selectedValue 등
> **문자열 속성에 한정** 적용(B안)으로 축소. 단계 3 검증에서 판단.

## 2. 구현 단계 (4단계)

### Stage 1 — 회귀 테스트(red) + 감사 확정

- 신규 테스트 `tests/issue_1534_hwpx_form_caption_escape.rs`:
  - (a) 폼 caption `R&D`(또는 `A<B>"&"`) 포함 픽스처 또는 `samples/hwpx/form-002.hwpx`
    parse→serialize 후 출력 XML에 `&amp;amp;` 가 **없음** 단언.
  - (b) parse→serialize→reparse 후 caption 값 == 원본 시맨틱 값 단언.
  - (c) **2회 누적** roundtrip 후 caption 불변(길이·값) 단언.
- 현재 코드에서 **실패(red) 확인** → 결함 재현 고정.
- 산출물: `mydocs/working/task_m100_1534_stage1.md`
- 커밋: 테스트 + 단계보고서 (소스 수정 전 red 상태 고정)

### Stage 2 — 수정 적용 (green)

- `attr_str` unescape 적용 (위 1절).
- Stage 1 테스트 green 확인.
- form-002 단독 roundtrip 2회 누적 무손상 수동 확인
  (`hwpx-roundtrip` + `unzip` caption 바이트 검증).
- 산출물: `mydocs/working/task_m100_1534_stage2.md`
- 커밋: `src/parser/hwpx/utils.rs` + 단계보고서

### Stage 3 — 전수 회귀 검증 + 가드 보강

- `cargo test --test hwpx_roundtrip_baseline` 통과 유지.
- **`cargo test`(전체, --lib 아님)** — golden/레이아웃 무회귀
  (메모리 `feedback_full_cargo_test_before_pr` 준수).
- `samples/hwpx/` 전수 원본↔저장본 export-text 재비교:
  form-002 **PASS 전환**, 기존 50건 무회귀(k-water-rfp 별건 제외).
- golden/baseline 스냅샷 변동 시 건별 "교정 여부" 확인 후 갱신.
- HWP5 어댑터(#852) 경로 caption 영향 없음 확인(`hwp5-*` 또는 관련 테스트).
- A안 리스크 판정 → 필요 시 B안 폴백 결정.
- 산출물: `mydocs/working/task_m100_1534_stage3.md` (검증 로그 포함)
- 커밋: 스냅샷 갱신(있을 경우) + 단계보고서

### Stage 4 — 최종 보고서

- `mydocs/report/task_m100_1534_report.md`:
  근본원인·수정·검증결과·잔여리스크·이슈 클로즈 판단.
- `git status` 로 미커밋 파일 0 확인 후 머지 준비.
- 산출물: 최종 보고서
- 커밋: 최종 보고서

> **orders 갱신 주의**: 메모리 룰 `feedback_orders_no_update` 에 따라 `mydocs/orders/`
> 는 작업지시자 명시 지시 없이는 변경하지 않는다. (CLAUDE.md 13절과 충돌 시 메모리 우선)

## 3. 변경 파일 (예정)

| 파일 | 변경 |
|------|------|
| `src/parser/hwpx/utils.rs` | `attr_str` unescape 적용 (핵심 수정) |
| `tests/issue_1534_hwpx_form_caption_escape.rs` | 신규 회귀 테스트 |
| (조건부) golden/baseline 스냅샷 | `&amp;` 잔존 교정 시 갱신 |
| `mydocs/working/task_m100_1534_stage{1..3}.md` | 단계보고서 |
| `mydocs/report/task_m100_1534_report.md` | 최종 보고서 |

## 4. 검증 기준 (Definition of Done)

1. 신규 테스트 (a)(b)(c) green.
2. form-002 2회 누적 roundtrip caption 불변 (`R&&D` 유지).
3. `cargo test` 전체 통과.
4. `hwpx_roundtrip_baseline` 통과 (xfail 0 유지).
5. 전수 export-text 비교: form-002 PASS, 기존 무회귀.
6. `cargo clippy` 무경고(변경 파일 범위).

## 5. 리스크 및 대응

| 리스크 | 대응 |
|--------|------|
| 전역 unescape로 예기치 않은 IR 변동 | Stage 3 전체 cargo test + golden 점검, 과다 시 B안 폴백 |
| `unescape_value` 미정의 엔티티 에러 | lossy 원문 폴백으로 안전 |
| 한컴 비호환(자기 roundtrip ≠ 한컴) | 저장본 한컴 수동 검증은 작업지시자 게이트로 별도 요청 |

---

> 본 구현계획서 승인 후 Stage 1 부터 착수한다. 각 단계 완료 시 단계보고서 작성 →
> 승인 요청 → 다음 단계 진행.
