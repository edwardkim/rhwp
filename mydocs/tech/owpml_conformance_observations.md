---
kind: reference
status: active
canonical: mydocs/tech/owpml_conformance_observations.md
last_verified: 2026-08-09
---

# HWP/OWPML 정합 관찰 노트

이 문서는 **관찰 기록**이지 권위 스펙이 아니다. 각 항목은 rhwp 파서 기여(코드 주석·PR·테스트)에서
나온 실측 사례를 모은 것으로, 한컴 비공개 동작에 의존한 결론을 포함한다. 항목이 이 문서와 다른
결과를 관찰했다면, 권위는 [HWP 5.0 스펙 문서 정오표](hwp_spec_errata.md)와
[한컴 공식 OWPML 모델 참조 가이드](hwpx_hancom_reference.md) 순으로 우선한다. 이 문서는 그 둘의
"HWP 바이너리 ↔ OWPML(HWPX)" 교차 항목만 별도로 모아 공개하는 목적이다.

> **모든 항목의 공통 경고**: OWPML 서면 스펙(Core/Header/Body/ParaList XML schema)과 실제 한컴
> 익스포터·에디터 동작은 다르다. 아래는 "스펙이 이렇게 쓰여 있다"가 아니라 "실측 결과 이렇더라"다.

## 1. HWPX `strikeout shape="3D"` — OWPML에 없는 값이 실제 파일의 기본 placeholder

- **관찰**: 한컴오피스 HWPX 익스포터는 본문 `charPr`에 `<hh:strikeout shape="3D"/>`를
  placeholder 기본값으로 채워 넣는 경우가 많다. `"3D"`는 OWPML `LineSym2`(표 27 선 종류) 열거에
  유효한 값이 아니며, 한컴 뷰어도 이를 취소선으로 그리지 않는다.
- **한컴 동작 의존**: 화이트리스트 방식 — 한컴이 실제 렌더링하는 `LineSym2` 13종만 취소선으로
  인정하고, `"NONE"`·`"3D"`·미지값은 fail-closed(취소선 없음)로 처리해야 한다.
- **근거**: `src/parser/hwpx/header.rs:22-39`(`is_real_strike_shape` 독스트링), `:742-748`.
- **HWP5 쪽 동형 현상**: 바이너리에서도 취소선 없는 문자에 bit 18~20=1(기본값)이 박히고 실판별자는
  모양 bit 26~29다 — `hwp_spec_errata.md` §34, PR #2258 실증.

## 2. OWPML `LineType3` 리터럴 — 스펙 표기와 rhwp 내부 이름이 달라 탭 리더가 유실

- **관찰**: OWPML `LineType3`(Core XML schema.xml 335~349행)의 정식 리터럴은
  `SLIM_THICK`/`THICK_SLIM`/`SLIM_THICK_SLIM`이다. rhwp는 한동안 내부 전용 이름
  `THIN_THICK`/`THICK_THIN`/`TRIM`만 인식해, 한/글이 실제로 저장한 정상 문서의 이중/삼중선 탭
  리더가 `NONE(0)`으로 유실됐다.
- **근거**: `src/parser/hwpx/header.rs:1799-1806`, 회귀 테스트 `:2441-2447`. 수정 PR
  #2868/#3088/#3091(#2857).

## 3. OWPML 문단번호 서식 리터럴 6종 미인식 + `HANGUL_JAMO`/`HANGUL_SYLLABLE` 코드 충돌

- **관찰**: `numFormat` 값 중 `CIRCLED_LATIN_CAPITAL` 등 OWPML Core XML schema.xml
  `NumberType1` 열거의 정식 리터럴을 인식하지 못해 `DIGIT(0)`로 조용히 유실되던 사례,
  그리고 `HANGUL_JAMO`(한글문서파일형식 5.0 표 41 값 10)가 `HANGUL_SYLLABLE`(8)과 같은 코드로
  뭉개지던 사례. 값 매핑은 `mydocs/tech/한글문서파일형식_5.0_revision1.3.md:978-993`과
  OWPML `NumberType1` 열거가 1:1 대응한다는 것이 근거.
- **근거**: `src/parser/hwpx/header.rs:2179-2183`, `:2513-2518`. 수정 PR #2879(#2877).

## 4. `CIRCLED_DIGIT` vs `CIRCLE_DIGIT` — 한컴 실물 파일의 오탈자 호환

- **관찰**: 스펙 표기는 `CIRCLED_DIGIT`(NumberType1)이지만, 한컴이 실제로 내보낸 파일에
  `CIRCLE_DIGIT` 오탈자가 저장된 사례가 있어 rhwp는 둘 다 인식해야 했다(문단 번호·쪽번호 양쪽).
- **근거**: `src/parser/hwpx/section.rs:4494-4495`, `:4905`. 수정 PR #3007/#3015/#3110(#3005/#3011).

## 5. 수식(Equation) 속성 생략 시 OWPML 스키마 기본값 — `version`/`baseLine`/`font`

- **관찰**: `<hp:equation>`의 `version`/`baseLine`/`font` 속성이 생략된 파일에서, zero 계열
  값(빈 문자열·0)으로 복원하면 직렬화기가 이 값을 그대로 방출해 라운드트립마다
  `version=""`/`baseLine="0"`/`font=""`로 문서가 변형된다. OWPML(ParaList 스키마
  EquationType) 기본값 `version="Equation Version 60"`, `baseLine=85`, `font="HYhwpEQ"`로
  복원해야 한다.
- **근거**: `src/parser/hwpx/section.rs:5513-5525`, 회귀 테스트 `:6536-6570`. 수정 PR #3155(#3149).

## 6. `CharShape.relative_sizes` — `<hh:relSz>` 생략 시 OWPML 기본값은 100, 0이 아니다

- **관찰**: `<hh:relSz>` 자식이 없는 `charPr`은 OWPML 기본값 100(장평 100%)으로 남아야 하는데,
  `CharShape::default()`의 파생값 0이 그대로 IR에 실려 라이터가 이를 방출하면 유효범위 밖 상대크기
  0이 저장되고, HWP3→HWP5/HWPX 변환본에서 한컴이 백지로 뜨는 원인이 됐다.
- **근거**: `src/parser/hwpx/header.rs:3209-3235`("Header XML schema.xml:716-728" 인용).
  수정 PR #4160(#4141), 통합 #4171.

## 7. HWPX 체크박스 `value` — `INDETERMINATE`(3상태) 뭉개짐

- **관찰**: `<hp:formObject>` 체크박스의 `value`는 `UNCHECKED`/`CHECKED`/`INDETERMINATE`
  3상태(OWPML `AbstractButtonObjectType`) 열거인데, `INDETERMINATE`를 `UNCHECKED`로 뭉개면
  라운드트립 시 tri-state 체크박스의 중간 상태가 유실된다.
- **근거**: `src/parser/hwpx/section.rs:5700-5706`. 수정 PR #2996.

## 8. 필드 고유 ID — OWPML `id` vs `fieldid`, 후자를 우선하면 필드 구분 불가

- **관찰**: 필드 모델 계약은 "문서 내 고유 ID"를 요구하는데, OWPML `id`는 필드마다 고유하고
  `<hp:fieldEnd beginIDRef>`가 이 `id`를 참조하는 반면, `fieldid`는 같은 종류 필드(예: FORMULA
  다수)에서 공유될 수 있다. `fieldid`를 우선하면 같은 종류 필드가 모두 동일 ID로 반환되어
  누름틀 구분이 불가능해진다.
- **근거**: `src/parser/hwpx/section.rs:4590-4594`, 테스트 `:7858-7861`(이슈 #1512).

## 9. 각주/미주 배치 — OWPML 정식 토큰 `MERGED_COLUMN`/`RIGHT_MOST_COLUMN` 미인식

- **관찰**: `place` 속성의 OWPML 정식 토큰은 컨텍스트마다 다르지만 HWP5 attr bits 8-9 코드
  공간을 공유한다: 각주 `EACH_COLUMN(0)`·`MERGED_COLUMN(1, 통단)`·`RIGHT_MOST_COLUMN(2, 가장
  오른쪽 단)`. 토큰 표에 없어 `_ => continue`로 떨어지면 통단/오른쪽단 각주가 파싱 단계에서
  기본값(각 단마다, 코드 0)으로 소실된다.
- **근거**: `src/parser/hwpx/section.rs:1086-1089`, `:6769-6772`(이슈 #2779).

## 10. HWPX `hatchStyle` 생략 = 무늬없음(`-1`) — HWP 쪽 sentinel과의 비대칭

- **관찰**: HWP 쪽 `pattern_type`은 `-1`이 무늬없음이고 1~6이 OWPML 스키마의 6개 `hatchStyle`
  값에 대응한다. HWPX에서 `hatchStyle`이 생략되면 "무늬없음"으로 저장해야 하므로 호출자는
  기본값으로 `-1`을 명시적으로 써야 한다 — 단순 `unwrap_or(0)` 등은 0번째 무늬로 오염된다.
- **근거**: `src/parser/hwpx/utils.rs:121-126`.

## 11. 용지 방향 — OWPML `landscape` 값이 이름과 반대로 매핑됨

- **관찰**: OWPML `landscape` 속성값 `WIDELY`가 세로(Portrait), `NARROWLY`가 가로(Landscape)다.
  이름만 보면 직관과 반대로 느껴지는 매핑이며, hwplib `ForSecPr`의
  `Portrait→WIDELY`/`Landscape→NARROWLY` 매핑이 권위다.
- **근거**: `src/parser/hwpx/section.rs:315-318`(이슈 #1166).

## 12. `breakNonLatinWord` — OWPML 열거명과 한컴 실동작이 반전

- **관찰**: HWP5 `ParaShape.attr1` bit 7은 스펙대로 `0=어절`/`1=글자`가 한컴 실동작과 일치한다.
  반면 HWPX/OWPML `breakNonLatinWord` 열거명은 직관적으로 `KEEP_WORD=어절`/`BREAK_WORD=글자`처럼
  보이지만, 한컴 202x 계열 실제 import/export와 rhwp 통제 실측에서는 **반대**로 매핑된다
  (`KEEP_WORD→bit7=1`, `BREAK_WORD→bit7=0`).
- **근거**: `mydocs/tech/hwp_spec_errata.md` §33 (PR #2194, 이슈 #2185). 검증:
  `cargo test --profile release-test --test issue_2185_korean_break_unit -- --nocapture`.

## 13. `PAGE_BORDER_FILL` 위치 기준 — OWPML `textBorder`와 HWP5 `attr bit 0`, 그리고 한컴 UI 용어 충돌

- **관찰**: HWP5 표 136 `attr bit 0`(`0=본문`, `1=종이`)과 OWPML
  `hp:pageBorderFill@textBorder`(`CONTENT`/`PAPER`)는 대응하지만, 한컴 UI가 표시하는 "쪽 기준"
  용어는 `CommonObjAttr::Page`(일반 개체 위치 기준의 `Paper/Page/Para/Column`)와 **같은 계층의
  계약이 아니다**. HWP3 전용 보정과 HWP5/HWPX 렌더링 계약을 하나의 `paper_based` 플래그로 섞으면
  둘 중 하나가 회귀한다(PR #956 vs Task #987 이력).
- **근거**: `mydocs/tech/hwp_spec_errata.md` §31.

## 14. HWPX→HWP5 저장 시 한컴이 요구하는 OLE contract 스트림 — OWPML에 대응물 없음

- **관찰**: 한컴 HWP 5.0이 정상으로 인식하려면 HWPX 컨테이너에 동등 데이터가 없는
  contract 스트림(예: `Scripts/DefaultJScript`, `HwpSummaryInformation`, `DocOptions/_LinkDoc`)이
  필요하다. `samples/form-01.hwp`(한컴 정답지)와 `saved/blank2010.hwp`에서 사전 추출한 fallback을
  강제로 채워 넣지 않으면 한컴이 파일을 거부하거나 Form 컨트롤 JS 핸들러가 동작하지 않는다.
- **근거**: `src/parser/hwpx/contract_streams.rs:1-25`, `:39-42`, `:61-65`, `:91-121`(Task #852).

## 15. 한컴 2020 HWPX 희소 Odd 바탕쪽 — HWP5 저장 시 압축 인코딩

- **관찰**: 기본 바탕쪽 순서는 1번째=양쪽(Both)·2번째=홀수(Odd)·3번째=짝수(Even)다. 그런데
  한컴 2020이 HWPX의 희소 Odd 바탕쪽을 HWP5로 저장할 때는 `LIST_HEADER` 하나와
  `SECTION_DEF` 상위 플래그 `0x80000000`만 써서, 앞 구역의 짝수 쪽을 상속하고 현재 구역의
  홀수 쪽만 바꾸는 압축 인코딩을 쓴다. 첫 목록을 그대로 Both로 해석하면 안 된다.
- **근거**: `src/parser/hwpx/body_text.rs:665-669`(경로 표기: `src/parser/body_text.rs`).

## 16. LAST_PAGE 바탕쪽의 overlap bit — `pageDuplicate="0"`이어도 함께 세팅됨

- **관찰**: 한컴 HWPX→HWP5 저장본은 LAST_PAGE 바탕쪽을 확장 바탕쪽으로 저장하면서
  `pageDuplicate="0"`(중복 없음)인 경우에도 overlap bit를 함께 세운다 — OWPML 속성 이름의
  액면 의미(중복 없음)와 실제 저장 비트가 어긋난다.
- **근거**: `src/parser/hwpx/section.rs:226-230`.

---

## 이 문서가 하지 않는 것

- 위 항목은 **한컴 특정 버전(202x 계열 위주)의 실측**이지, OWPML/HWPX 표준 자체의 결함 판정이
  아니다. 다른 한컴 버전이나 서드파티 OWPML 구현에서는 다르게 관찰될 수 있다.
- HWP5 바이너리 단독 스펙 오류(OWPML과 무관한 항목)는 이 문서가 아니라
  [HWP 5.0 스펙 문서 정오표](hwp_spec_errata.md)가 원천이다. 겹치는 항목(§31·§33·§34)은 그쪽이
  원본이고 이 문서는 발췌·재수록이다.
- 이 문서는 1차 편집본이다 — 파서 기여가 계속되며 항목이 늘어난다. 새 관찰은 발견한 PR에서
  코드 주석으로 먼저 남기고, 이 문서에는 나중에 배치로 반영한다(개별 PR마다 이 문서를 고치도록
  강제하지 않는다 — 트랙 I 운영 규칙과 동일하게 열린 PR 캡을 늘리지 않는다).
