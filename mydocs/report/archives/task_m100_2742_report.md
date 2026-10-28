# task_m100_2742 처리결과 보고서 — HWPX secPr 템플릿 고정 속성 전수 조사 + autoNumFormat 수정

- **이슈**: [#2742](https://github.com/edwardkim/rhwp/issues/2742)
- **브랜치**: `task/m100-2742-secpr-template` (base `origin/devel` @ `1658d0bb`)
- **범위**: `src/serializer/hwpx/section.rs` (secPr/템플릿 영역 + 테스트 2개), 본 보고서
- **분류**: 결함 수정 (저장 충실도) + 전수 조사 산출물

## 1. 문제

HWPX 구역 설정 `<hp:secPr>` 은 정적 템플릿 `templates/empty_section0.xml` 을 `replacen`
으로 부분 치환해 만든다. **치환 앵커가 걸린 속성만 IR 값을 받고, 나머지는 문서가 무엇을
지정했든 템플릿 상수가 그대로 출력물이 된다.** 단일 버그가 아니라 결함 클래스이므로
템플릿 secPr 속성 슬롯을 전수 분류한 뒤 실측 손실이 가장 큰 항목을 골라 고쳤다.

## 2. 분석 — secPr 속성 행렬 (103 슬롯)

템플릿의 `<hp:secPr>` 블록에서 속성 슬롯을 기계적으로 추출해(요소 인스턴스 단위,
`pageBorderFill` 3개·`offset` 3개·note pr 2개를 각각 별개로 셈) **103개**를 얻고,
`section.rs` 의 치환 앵커 유무와 `src/parser/hwpx/section.rs` 의 IR 수집 유무를 대조했다.

| 분류 | 슬롯 수 |
|---|---:|
| **IR 반영** | **76** |
| **고정(IR 필드 없음)** | **15** |
| **고정(IR 필드 있는데 미반영 = 결함)** | **12** |
| 합계 | 103 |

### 2.1 IR 반영 76

`secPr` 5(textDirection·spaceColumns·tabStop·outlineShapeIDRef·masterPageCnt) ·
`grid` 2 · `startNum` 5 · `visibility` 6 · `pagePr` 4 · `margin` 7 ·
`footNotePr` 자식 10 · `endNotePr` 자식 10 · `pageBorderFill` ×3 = 15 · `offset` ×3 = 12.
각각 #1166 / #1388 / #1505 / #1637 / #1984 / #1987 이 도입한 앵커다.

### 2.2 고정(IR 필드 없음) 15 — 직렬화기 단독 수정 불가

| 요소@속성 | 코퍼스 실측 | 판정 |
|---|---|---|
| `secPr@memoShapeIDRef` | 값 5종(0/1/2/3/4), 왕복 손실 **14 secPr / 9 파일** | **실측** — 파서·모델 변경 필요, 잔여 |
| `pageBorderFill@type` ×3 | 위치 기반 합성, 왕복 불일치 1 파일 | 실측(경미) — 잔여 |
| `secPr@id` / `@tabStopVal` / `@tabStopUnit` / `@textVerticalWidthHead` | 전수 동일값 | 잠재. `tabStopVal`/`tabStopUnit` 은 [Finding 14] `secpr_emits_tab_stop_val_and_unit` 로 이미 의도 고정 |
| `grid@wonggojiFormat` | 90/90 `0` | 잠재 |
| `visibility@hideFirstPageNum` / `@showLineNumber` | 90/90 `0` | 잠재. `render_visibility` 주석이 IR 미보존을 이미 명시 |
| `lineNumberShape@restartType/countBy/distance/startNumber` | **360/360 전부 `0`** | 잠재 (줄 번호 사용 실물 0건) |

### 2.3 결함 12 — 파서는 읽는데 직렬화가 안 씀

| 요소@속성 | 템플릿 고정값 | 대응 IR 필드 |
|---|---|---|
| `footNotePr/autoNumFormat@type` | `DIGIT` | `FootnoteShape.number_format` |
| `…@userChar` | `""` | `.user_char` |
| `…@prefixChar` | `""` | `.prefix_char` |
| `…@suffixChar` | `")"` | `.suffix_char` |
| `…@supscript` | `0` | `.number_code_superscript` |
| `endNotePr/autoNumFormat` 5속성 | 동일 | `SectionDef.endnote_shape` 의 같은 5필드 |
| `footNotePr/placement@place` | `EACH_COLUMN` | `.placement` |
| `endNotePr/placement@place` | `END_OF_DOCUMENT` | `.placement` |

HWPX 파서(`parse_note_pr_children`)뿐 아니라 HWP5 파서도 `HWPTAG_FOOTNOTE_SHAPE` 에서
같은 5필드를 채우며(`apply_attr_fields_from_raw`), `SectionDef.footnote_shape`/
`endnote_shape` 는 두 포맷이 공유하는 단일 슬롯이다.

**의도된 고정과의 구분.** `tabStop` IR 0 폴백(주석 + `secpr_keeps_template_tab_stop_when_ir_unset`),
`tabStopVal`/`tabStopUnit`([Finding 14] 테스트), `textDirection` 조건부 치환은 코드/테스트에
의도가 남아 있다. 반면 `git log -S "autoNumFormat" -- src/serializer/hwpx/section.rs` 는
템플릿 도입 커밋 2개와 `autoNum` **컨트롤** 커밋(#1326)뿐으로, secPr `autoNumFormat` 을
고정으로 두자는 결정 기록이 없다. 파일 머리말도 "IR에 대응 필드가 더 담길 때까지
점진적으로 동적화 예정"이라는 임시 상태를 명시한다.

## 3. 코퍼스 실측

### 3.1 XML 레벨 — `samples/hwpx/*.hwpx` 60개

`Contents/section*.xml` 에서 `<hp:secPr>` 블록 직접 추출. ZIP 유효 **59 파일 · secPr 90개**
(`hwpx-01.hwpx` 는 ZIP 아님).

실측 변동이 있는 고정 속성: `secPr@memoShapeIDRef`(9 파일),
`footNotePr/endNotePr autoNumFormat@supscript`(`0`×80 / `1`×10, 1 파일),
`pageBorderFill@type`(1 파일). 나머지 고정 속성은 전수 동일값 = **잠재**.

### 3.2 IR 레벨 — 330 파일 · note shape 828개

XML 코퍼스만 보면 `autoNumFormat` 은 거의 잠재로 보인다. 그러나 `SectionDef.footnote_shape`
는 HWP5 파서도 채우고 `export-hwpx`(HWP5→HWPX 저장)는 1급 기능이므로 IR 레벨로 다시 셌다.
`rhwp dump-note-shape` 를 `samples/*.hwp` 270 + `samples/hwpx/*.hwpx` 60 =
**330 파일 전부**에 돌려 **note shape 828개**(각주 414 / 미주 414) 수집, 파싱 실패 0.

```
endnoteShape   prefixChar  {'\x00': 397, '문': 17}           ← U+BB38
endnoteShape   suffixChar  {')': 382, '）': 17, '\x00': 15}   ← U+FF09 전각
endnoteShape   supscript   {False: 413, True: 1}
footnoteShape  supscript   {False: 413, True: 1}
footnoteShape/endnoteShape numberFormat {'Digit': 414 / 414}
footnoteShape/endnoteShape userChar     {'\x00': 414 / 414}
footnoteShape/endnoteShape placement    {'EachColumn': 414 / 414}
```

**템플릿 고정 출력값과 다른 note shape = 19개 / 18 파일**

- 미주 `prefixChar='문'` + `suffixChar='）'` — **17 파일**
  (`3-09월_교육_통합_*`, `3-10월_교육_통합_2022`, `3-11월_실전_통합_*` — 전부 한컴 생산 시험지)
- 각주·미주 `supscript=true` — **1 파일**
  (`hwpx/issue2019_floating_form_74312.hwpx`, `version.xml` = `Hancom Office Hangul 9,1,0,2172`)

**실측/잠재 분리** — `prefixChar`·`suffixChar`·`supscript` 3속성은 **실측**,
`type`·`userChar` 2속성과 `placement@place` 는 현 코퍼스에서 **잠재**(실측 0건)다.
이번 수정에 `type`/`userChar` 도 포함했지만 실측 효과는 없고 잠재 방어다.

### 3.3 raw_stream 지름길 — HWPX 경로에는 없다

HWP5 경로에는 `src/serializer/body_text.rs` 의 `Section::raw_stream` 조기 반환이 있어
미편집 저장이 직렬화기에 도달하지 않는다. HWPX 경로에는 대응물이 없다.

- `grep -rn "raw_stream" src/serializer/hwpx/` → **0건**
- `serialize_hwpx()` 의 `hwpx_aux_entry` 패스스루는 `version.xml`·`Preview/*`·`settings.xml`·
  `Contents/content.hpf` 에만 적용되고 `Contents/section{N}.xml` 은 **무조건**
  `section::write_section()` 으로 재조립된다.

아래 4.1 에서 왕복 불일치가 실제로 관측된 것 자체가 그 실증이다(패스스루가 있었다면 0건).

## 4. 왕복 실증

### 4.1 HWPX → HWPX 59 파일 전수 secPr diff (수정 전 → 수정 후)

59 파일 전부 `rhwp export-hwpx` 로 저장한 뒤 원본/저장본의 첫 secPr 속성을 1:1 대조
(rhwp IR 은 section XML 파트당 secPr 1개만 유지).

| | 수정 전 | 수정 후 |
|---|---:|---:|
| 총 불일치 (속성×secPr) | **59** | **57** |
| 속성 종류 | **22** | **20** |
| `footNotePr/endNotePr autoNumFormat@supscript` | `'1' → '0'` 각 1건 | **0건** |

남은 57건은 전부 이번 범위 밖 항목이다 — `memoShapeIDRef` 14,
`tabStopVal`/`tabStopUnit` 추가 26([Finding 14] 의도),
`autoNumFormat@prefixChar/suffixChar/userChar` 6(**값 변경이 아니라 한컴이 생략한 속성을
기본값으로 추가**), `noteLine@width` 표기 2, `margin` 4, `pageBorderFill@type` 2,
`placement@place` 1, `secPr@id` 1, `secPr@tabStop` 1.

#### 이론과 어긋난 관측 2건 — 원인 규명

1. **`margin@left/right/top/bottom` 4건.** `margin` 은 IR 반영 속성인데
   `issue2019_floating_form_74312.hwpx` 에서 `left 6750 → 7600` 등이 나왔다. 원인은
   직렬화기가 아니다. 이 파일은 `Contents/section0.xml` **하나에 `<hp:secPr>` 10개**를
   담고 있고 `parse_secpr_children` 이 같은 `sec_def` 를 순차 덮어쓰므로 IR 에는 **마지막
   secPr** 값이 남는다. 원본 10번째 secPr 의 `left="7600" right="5800" top="4135"
   bottom="2935"` 가 저장본과 정확히 일치함을 확인했다. "다중 secPr → 구역 1개" 파서
   구조 이슈이지 템플릿 고정과 무관하다.
2. **`secPr@tabStop` `0 → 8000` 1건.** 같은 파일. `replace_secpr_scalars` 가 IR 0 일 때
   템플릿 상수를 유지하도록 **의도적으로** 작성돼 있고 전용 테스트가 이를 고정한다. 결함 아님.

`supscript '1' → '0'` 은 위 두 예외에 해당하지 않는다. 원본의 secPr 10개가 **전부**
`supscript="1"` 이라 어느 secPr 이 IR 에 남든 값은 `true` 여야 하고 `dump-note-shape` 도
`numberCodeSuperscript: true` 를 보고했는데, 저장본만 `0` 이었다 → 템플릿 고정이 유일 원인.

### 4.2 HWP5 → HWPX 실파일 왕복 (수정 전)

```
$ rhwp export-hwpx "samples/3-09월_교육_통합_2023.hwp" edu2023.hwpx
$ rhwp dump-note-shape "samples/3-09월_교육_통합_2023.hwp"      # 저장 전
0 endnoteShape {'numberFormat':'Digit','prefixChar':'문','suffixChar':'）','userChar':'\x00','numberCodeSuperscript':False}
$ rhwp dump-note-shape edu2023.hwpx                             # 저장 후
0 endnoteShape {'numberFormat':'Digit','prefixChar':'\x00','suffixChar':')','userChar':'\x00','numberCodeSuperscript':False}
```

`prefixChar` `문`(U+BB38) → `\x00` 소멸, `suffixChar` `）`(U+FF09) → `)`(U+0029) 변조.

### 4.3 HWP5 → HWPX 실파일 왕복 (수정 후)

```
$ rhwp dump-note-shape edu2023_fixed.hwpx
0 endnoteShape {'numberFormat':'Digit','prefixChar':'문','suffixChar':'）','userChar':'\x00','numberCodeSuperscript':False}

# 저장본 Contents/section0.xml 의 secPr
<hp:autoNumFormat type="DIGIT" userChar="" prefixChar="" suffixChar=")" supscript="0"/>   ← 각주(원본대로 기본값)
<hp:autoNumFormat type="DIGIT" userChar="" prefixChar="문" suffixChar="）" supscript="0"/> ← 미주(복원)
```

동일 패턴 파일 17개 전수 재측정: **대상 17 / note shape 5필드 일치 17 / 불일치 0 / 변환 실패 0**
(수정 전 동일 스크립트: 일치 0 / 불일치 17).

### 4.4 사용자에게 보이는 결과 (과장 없이)

구역 각주/미주 모양은 **새 주석을 삽입할 때의 기본값**이다.
`document_core/commands/object_ops/note.rs` 가 `shape.prefix_char`/`suffix_char`/번호 모양을
읽어 새 미주의 `before_decoration_letter`/`after_decoration_letter`/`number_shape` 를 정한다.
저장본에서 이 값이 리셋되면 이후 삽입되는 미주가 「문1）」이 아니라 「1)」로 매겨지고,
한/글 [주석 모양] 대화상자도 원본과 다른 값을 보여준다. `number_code_superscript` 는
`FootnoteShape::encode_attr()` bit 12 라 HWPX→HWP5 재변환 시 `HWPTAG_FOOTNOTE_SHAPE` attr
까지 오염된다.

**이미 배치된 주석의 마커 렌더는 ctrl 쪽 값**(`Footnote::before_decoration_letter`,
`renderer/layout/paragraph_layout.rs::note_marker_text_from_control`)**으로 그려지므로 이
결함만으로 기존 마커 표시가 즉시 바뀌지는 않는다.** 본 건은 구역 기본 주석 모양(secPr)의
손실이며 ctrl 쪽은 별건이다.

## 5. 변경

`src/serializer/hwpx/section.rs` 한 파일, **+140줄 / -0줄**. secPr/템플릿 영역만 손댔고
공통 도형(`render_common_shape_xml`)·각주 sublist(`render_note_sublist`) 영역은 무변경.

| 추가 항목 | 내용 |
|---|---|
| `note_number_format_str()` | `NumberFormat` → HWPX `type` 토큰 19종. 파서 `number_format_from_name` 의 UPPER_SNAKE 분기 역매핑 (`note_line_type_str`·`note_numbering_str`·`page_num_format_to_str` 와 동형) |
| `note_deco_char_attr()` | 장식 문자 `char` → 속성값. `< 0x20` 제어문자(`'\0'` 포함, 미지정)면 템플릿 기본값 유지, 아니면 `xml_escape` 후 방출. `render_hp_t_content` 의 `< 0x20` 제외와 동일 정책 — XML 1.0 금지 문자 방출로 저장본이 안 열리는 것을 막는다 |
| `render_auto_num_format()` | `FootnoteShape` → `<hp:autoNumFormat .../>`. 속성 순서 type→userChar→prefixChar→suffixChar→supscript (템플릿·한컴 실물과 동일) |
| `TEMPLATE_AUTO_NUM_FORMAT` | 템플릿 앵커 상수 |
| `replace_footnote_shape()` | 앞머리에 `replace_first_two()` 치환 1회 추가 |

**설계 판단 2가지**

1. **`replacen` 이 아니라 `replace_first_two`.** 템플릿의 각주/미주 `autoNumFormat` 문자열이
   완전히 같아 연쇄 `replacen` 은 두 번째가 첫 슬롯을 다시 잡는다. 같은 이유로
   `numbering` 이 이미 쓰던 위치 기반 2회 치환을 재사용했다.
2. **`'\0'` 은 템플릿 기본값 유지.** `object_ops/note.rs` 가 `suffix_char == '\0'` 을 기본값
   폴백으로 해석하므로 `'\0'` 은 "없음"이 아니라 "미지정"이다. IR 값이 있을 때만 치환하는
   `tabStop`/`textDirection` 패턴과 동일하게 두어, 기본 문서의 출력이 템플릿과 바이트
   동일하도록 보장했다(테스트 `issue2742_auto_num_format_keeps_template_when_ir_unset`).

치환은 콘텐츠 삽입 **이전**에 일어나므로 본문 `<hp:ctrl><hp:autoNum>` 의 `autoNumFormat` 과
앵커가 충돌하지 않는다.

## 6. 검증

### 6.1 red → green (실제 실행)

**RED** — `replace_footnote_shape` 의 `replace_first_two` 호출만 되돌리고 실행:

```
running 2 tests
test serializer::hwpx::section::tests::issue2742_auto_num_format_keeps_template_when_ir_unset ... ok
test serializer::hwpx::section::tests::issue2742_auto_num_format_reflects_ir ... FAILED

---- serializer::hwpx::section::tests::issue2742_auto_num_format_reflects_ir stdout ----
thread '...' panicked at src\serializer\hwpx\section.rs:2624:9:
각주 autoNumFormat 이 IR 값이어야 함: <?xml version="1.0" ... <hp:footNotePr>
<hp:autoNumFormat type="DIGIT" userChar="" prefixChar="" suffixChar=")" supscript="0"/>
<hp:noteLine length="0" type="NONE" width="0.1 mm" color="#000000"/> ...

test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 2471 filtered out
```

`issue2742_auto_num_format_keeps_template_when_ir_unset` 은 RED 에서도 **통과한다** —
이 테스트는 "IR 미설정 시 템플릿과 바이트 동일" 을 고정하는 **회귀 방어용**이므로
수정 전후 모두 통과하는 것이 정상이다. 결함을 잡는 테스트는
`issue2742_auto_num_format_reflects_ir` 하나다.

**GREEN** — 수정 복원 후:

```
running 2 tests
test serializer::hwpx::section::tests::issue2742_auto_num_format_reflects_ir ... ok
test serializer::hwpx::section::tests::issue2742_auto_num_format_keeps_template_when_ir_unset ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 2471 filtered out
```

### 6.2 CI 3종

| 항목 | 결과 |
|---|---|
| `cargo clippy --all-targets -- -D warnings` | **통과** (경고 0) |
| `cargo test --profile release-test --tests` | **통과** (exit 0, 0 failed) — 통합 바이너리 248 passed. 추가로 `cargo test --lib` 전체 **2466 passed / 0 failed / 7 ignored**, `hwpx_roundtrip_baseline` 4 · `hwpx_roundtrip_integration` 22 · `hwpx_form_roundtrip` 1 · `issue_1172_para_margin_roundtrip` 3 개별 통과 |
| `rustfmt --edition 2021` (변경 `.rs`) 후 idempotency 확인 | **변경 없음** (포맷 위반 0) |

`cargo fmt --all -- --check` 는 이 Windows 체크아웃에서 CRLF 때문에
`Incorrect newline style` 만 찍고 diff 를 내지 않는 거짓 통과이므로 사용하지 않았다.

### 6.3 실파일 회귀

- HWPX 왕복 59 파일 전수 secPr diff: 불일치 59 → **57** (autoNumFormat 값 불일치 소멸,
  다른 속성 증가 0)
- HWP5→HWPX 17 파일 note shape 5필드: **17/17 일치**

## 7. 미실행 항목

- 한/글 실물 열기 검증(설치 환경 없음). XML 레벨 + IR 재파싱 레벨로 갈음했다.
- 렌더 이미지 회귀(`render-diff`) 미실행 — 이번 변경은 secPr 속성값만 바꾸고
  레이아웃 입력(`separator_*`, `note_spacing`)은 건드리지 않는다.

## 8. 잔여

| 항목 | 실측 | 왜 남기나 |
|---|---|---|
| `secPr@memoShapeIDRef` | **14 secPr / 9 파일** | 파서 미수집 + `SectionDef` 필드 없음. 파서·모델 변경 필요 |
| `footNotePr/endNotePr placement@place` | 0건(정규화 1건) | 파서가 `END_OF_SECTION`/`BELOW_TEXT` 를 같은 `BelowText` 로 접어 무손실 역매핑 불가. 컨텍스트별 역매핑 재정의 필요 |
| `pageBorderFill@type` ×3 | 1 파일 | IR 이 `type` 문자열 미보존(위치로만 저장). 한컴이 `BOTH`×3 을 쓰는 사례 의미 확인 필요 |
| 한 section 파트에 `<hp:secPr>` 다수 | 1 파일(10개 → 1개, 9개 소실) | "section 파트 1 = 구역 1" 전제를 바꿔야 함. 별도 이슈 |
| `noteLine@width` `4 mm` → `4.0 mm` | 1 파일 | 굵기 코드 → mm 문자열 포맷 차이. 값 손실은 아니나 한컴 표기와 불일치 |
| `lineNumberShape` 4속성, `grid@wonggojiFormat`, `visibility@hideFirstPageNum`/`showLineNumber`, `secPr@textVerticalWidthHead`, `secPr@id` | 0건 | 코퍼스 전수 기본값 + IR 필드 없음 — 잠재로 기록만 |
