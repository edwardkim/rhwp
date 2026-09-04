# Stage 1 완료보고서 — Task #1534

> 회귀 테스트(red) 작성 + 결함 재현 고정

- **이슈**: [#1534](https://github.com/edwardkim/rhwp/issues/1534)
- **브랜치**: `local/task1534`
- **단계**: 1/4 (회귀 테스트 red)
- **작성일**: 2026-06-25

---

## 1. 작업 내용

신규 회귀 테스트 `tests/issue_1534_hwpx_form_caption_escape.rs` 작성. 픽스처는
`samples/hwpx/form-002.hwpx` (폼 체크박스 caption 에 `&` 포함하는 유일 샘플).

| 테스트 | 검증 | 비고 |
|--------|------|------|
| `fixture_has_ampersand_caption` | 원본에 `&` 포함 caption 존재 | 전제 가드 |
| `form_caption_survives_roundtrip` | parse→serialize→reparse 후 caption 불변 | 핵심 |
| `saved_xml_has_no_double_escape` | 저장본 XML 에 `&amp;amp;` 없음 | XML 직접 |
| `form_caption_stable_across_two_roundtrips` | 2회 저장 후 caption 누적 없음 | 누적 가드 |

## 2. red 확인 결과

```
test fixture_has_ampersand_caption ... ok
test saved_xml_has_no_double_escape ... FAILED
test form_caption_survives_roundtrip ... FAILED
test form_caption_stable_across_two_roundtrips ... FAILED
```

전제 가드 1건 green, 결함 검증 3건 red — 의도대로 결함을 고정했다.

## 3. 분석 입증

`form_caption_survives_roundtrip` 실패 출력에서 **원본 IR caption 값**이 다음과 같이
확인됨:

```
"IP R&amp;&amp;D연계", "R&amp;&amp;D 자율성트랙(일반)", "R&amp;&amp;D 자율성트랙(지정)"
```

즉 현재 파서가 속성값을 **unescape 하지 않고 원문(`&amp;`)을 IR 에 그대로 저장**함을
입증한다. 직렬화 측이 이를 다시 escape → `&amp;amp;` 이중 이스케이프. 구현계획서의
근본원인 분석과 정확히 일치한다.

> 수정(Stage 2) 후 원본 IR caption 은 `IP R&&D연계` 로 정규화되고, 직렬화→재파싱
> 후에도 동일하게 유지되어 위 3건이 green 으로 전환되어야 한다.

## 4. 산출물 / 커밋

- `tests/issue_1534_hwpx_form_caption_escape.rs` (신규)
- `mydocs/working/task_m100_1534_stage1.md` (본 보고서)
- 소스 수정 없음 — red 상태 고정 커밋.

## 5. 다음 단계

Stage 2 — `src/parser/hwpx/utils.rs` `attr_str` unescape 적용 → 위 3건 green 전환.
**소스 수정 단계이므로 본 보고서 승인 후 착수한다.**
