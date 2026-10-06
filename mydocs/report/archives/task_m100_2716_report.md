# 최종 결과보고서 — Task M100 #2716

**이슈**: [#2716](https://github.com/edwardkim/rhwp/issues/2716) HWPX 각주/미주 저장 시 `prefixChar`/`suffixChar`/`instId`/`flag` 4개 속성 전량 유실
**마일스톤**: v1.0.0 (M100)
**브랜치**: `task/m100-2716-note-hwpx-attrs` (← `origin/devel` @ `49f38446`)
**범위**: HWPX 각주/미주 컨트롤 속성 왕복 보존 (파서 1개 분기 × 2 + 직렬화기 속성 방출)

---

## 1. 문제

HWPX 직렬화기 `render_note_sublist()` 가 `<hp:footNote>` / `<hp:endNote>` 에 `number` **하나만**
방출한다. 한컴이 자기 저장본에 항상 쓰는 `prefixChar`(앞 장식) · `suffixChar`(뒤 장식) ·
`instId`(주석 고유 ID) · `flag`(번호 모양) 4개가 전부 빠진다.

사용자 관점 증상: 미주가 있는 한컴 문서를 rhwp 로 열어 저장하면 본문 미주 마커
**「문1）」이 「1)」로 퇴화**한다(앞 장식 탈락 + 전각 괄호 → 반각 괄호). 이는
[#1199](https://github.com/edwardkim/rhwp/issues/1199) 가 파서 쪽에서 고친 바로 그 증상으로,
저장 경로가 같은 값을 버리므로 **저장 한 번에 #1199 이전 상태로 되돌아간다**.

---

## 2. 분석

### 2-1. 결함 지점과 대조군

| 위치 | 상태 |
|---|---|
| `src/serializer/hwpx/section.rs:1961 render_note_sublist` | `number` 스칼라만 인자로 받아 나머지 4개 필드가 **구조적으로 방출 불가** |
| `src/serializer/hwpx/section.rs:1539 render_header_footer` (같은 파일 형제) | `HeaderFooterFields` 묶음으로 IR 값을 **전부** 방출 |
| `src/serializer/control.rs:783 serialize_footnote` (HWP5 형제) | `number`+`before`+`after`+`numberShape`+`instanceId` **5개 전부** 기록 |
| `src/model/footnote.rs:16 Footnote` / `:35 Endnote` | 4개 필드를 이미 보유 |
| `src/parser/hwpx/section.rs:4563/4612 parse_ctrl_(foot\|end)note` | `flag` 분기 부재 → `number_shape` 가 HWPX 경로에서 항상 0 |

즉 **HWP5 ↔ IR 은 무손실이고 HWPX 만 편도 손실**이다.

### 2-2. 패스스루가 가려주지 않는 이유

`grep -n "raw_stream\|raw_xml\|raw_section_xml" src/serializer/hwpx/*.rs` → 매치 0.
`Contents/sectionN.xml` 은 100% IR 에서 재조립되므로, 레코드 `raw_data` · `Section::raw_stream`
(`src/serializer/body_text.rs:26-30`) · DocInfo `raw_stream_dirty` 어느 층도 HWPX 출력 경로에
관여하지 않는다. 4절 실측이 이를 직접 확인해 준다.

### 2-3. 한컴 실측 계약 확정

`samples/3-09월_교육_통합_2023.hwp`(HWP5) 와 같은 문서의 한컴 HWPX 저장본
`samples/3-09월_교육_통합_2023.hwpx` 의 각주/미주 46개를 필드 단위로 전수 대조 →
**일치 46 / 불일치 0**. 이로써 다음이 확정됐다.

- HWPX `flag` == HWP5 `CTRL_FOOTNOTE/CTRL_ENDNOTE` 의 `numberShape`(UInt4)
- `flag` 존재 ⟺ `number_shape != 0`
- `prefixChar` 존재 ⟺ `before_decoration_letter != 0`
- `number` / `suffixChar` / `instId` 는 **항상** 존재 (코퍼스 828/828)
- 속성 순서: `flag → number → prefixChar → suffixChar → instId`

`suffixChar` 를 0 일 때 생략하면 파서 기본값 `0x0029` `)` 가 들어가 오염된다
(`src/parser/hwpx/section.rs:4571`). `src/serializer/control.rs:789-792` 가 HWP5 쪽에서
같은 이유로 이미 고친 문제이므로 **0 이어도 항상 방출**한다.

---

## 3. 변경

| 파일 | 변경 |
|---|---|
| `src/serializer/hwpx/section.rs` | `NoteAttrs` 구조체 + `render_note_attrs()` 신설, `render_note_sublist` 시그니처를 묶음 전달로 교체, `render_footnote`/`render_endnote` 가 IR 4개 필드 전달 (+75 / −5) |
| `src/parser/hwpx/section.rs` | `parse_ctrl_footnote` / `parse_ctrl_endnote` 에 `b"flag" → number_shape` 분기 추가 (+20 / −0) |
| `src/serializer/hwpx/mod.rs` | 회귀 테스트 `footnote_endnote_decoration_attrs_roundtrip` 추가 (+112 / −0) |

합계 3파일 207 추가 / 5 삭제. 모델 · 렌더러 · HWP5 파서 · HWP5 직렬화기 · HWP3 ·
`hwpx_to_hwp.rs` 는 **무변경**. `render_note_sublist` 는 각주/미주 전용 영역이라 동시 작업 중인
`serializer/hwpx/section.rs` 의 shape/table 영역과 겹치지 않는다.

---

## 4. 검증

### 4-1. red → green (실제 실행 · 원문 캡처)

수정 2파일(`src/serializer/hwpx/section.rs`, `src/parser/hwpx/section.rs`)을 `git stash` 로
되돌리고 테스트만 남긴 상태에서 실행:

```
$ cargo test --lib footnote_endnote_decoration_attrs_roundtrip
running 1 test
test serializer::hwpx::tests::footnote_endnote_decoration_attrs_roundtrip ... FAILED

failures:

---- serializer::hwpx::tests::footnote_endnote_decoration_attrs_roundtrip stdout ----

thread 'serializer::hwpx::tests::footnote_endnote_decoration_attrs_roundtrip' (40568)
panicked at src\serializer\hwpx\mod.rs:1734:9:
footNote 속성이 한컴 계약대로 방출되지 않음: <?xml version="1.0" ... >
  ... <hp:ctrl><hp:footNote number="1"><hp:subList id="" textDirection="HORIZONTAL" ...
  ... <hp:ctrl><hp:endNote number="2"><hp:subList id="" textDirection="HORIZONTAL" ...

failures:
    serializer::hwpx::tests::footnote_endnote_decoration_attrs_roundtrip

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 2471 filtered out; finished in 0.01s

error: test failed, to rerun pass `--lib`
```

`git stash pop` 으로 수정 복원 후 재실행:

```
$ cargo test --lib footnote_endnote
running 4 tests
test serializer::hwpx::section::tests::footnote_endnote_beneath_text_reflects_ir ... ok
test serializer::hwpx::section::tests::footnote_endnote_numbering_and_start_reflect_ir ... ok
test serializer::hwpx::tests::footnote_endnote_roundtrip ... ok
test serializer::hwpx::tests::footnote_endnote_decoration_attrs_roundtrip ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 2468 filtered out; finished in 0.02s
```

### 4-2. 실파일 왕복 — 361개 note 속성 완전일치

`export-hwpx` 후 한컴 원본과 note 요소 속성 집합을 전수 비교:

| 케이스 | note 수 | 수정 전 | 수정 후 |
|---|---|---|---|
| `samples/3-09월_교육_통합_2022.hwpx` → hwpx | 46 | `{number}` 만 (46/46 손실) | **속성 완전일치 True** |
| `samples/3-09월_교육_통합_2023.hwpx` → hwpx (`flag` 포함) | 46 | `{number}` 만 | **속성 완전일치 True** |
| `samples/SO-SUEOP.hwpx` → hwpx | 223 | `{number}` 만 | **속성 완전일치 True** |
| `samples/3-09월_교육_통합_2022.hwp` → hwpx | 46 | `{number}` 만 | **속성 완전일치 True** |

수정 후 출력 예시:

```
{'flag':'3211264','number':'1','prefixChar':'47928','suffixChar':'65289','instId':'1085611790'}
```

### 4-3. HWP5 바이트 왕복 — 46/46 페이로드 완전일치

`samples/3-09월_교육_통합_2022.hwp` → HWPX → HWP5(`convert`) 후 `hwp5-inventory` 의
`CTRL_HEADER('en  ')` 페이로드 비교:

```
원본 [0]: 20 20 6e 65 01 00 00 00 38 bb 09 ff 00 00 00 00 1d 22 b5 40
왕복 [0]: 20 20 6e 65 01 00 00 00 38 bb 09 ff 00 00 00 00 1d 22 b5 40
페이로드 완전일치 46/46
```

수정 전에는 `... 00 00  29 00  00 00 00 00  00 00 00 00` 이었다
(`before` 소실, `after` 가 `0x0029` 로 변조, `instance_id` 0 리셋).

### 4-4. 렌더 회귀 — 마커 46개 복원

`export-text` 23쪽 전량 스캔:

```
원본 HWPX 렌더 "문N）" 마커: 46
수정 후 재저장본 렌더 "문N）" 마커: 46   (문1）,문2）,문3）,문4）,문5） …)
```

수정 전 재저장본은 `문N）` 0개 / `N)` 46개였다.

### 4-5. CI 3종

| 항목 | 결과 |
|---|---|
| `cargo clippy --all-targets -- -D warnings` | **통과** — 경고 0 (`Finished dev profile in 57.10s`) |
| `cargo test --profile release-test --tests` | **통과** — exit 0, 테스트 바이너리 291개, **3481 passed / 0 failed / 23 ignored** |
| 변경 `.rs` 3파일 `rustfmt --edition 2021` 후 `git diff --numstat` | 재포맷 churn 0 (20/0, 112/0, 75/5 — 의도한 hunk 뿐) |

`cargo fmt --all -- --check` 는 CRLF 체크아웃에서 `Incorrect newline style` 만 내고 diff 를
내지 않는 false pass 이므로 사용하지 않았다.

### 4-6. `cargo test --profile release-test --tests` 상세

lib 유닛 테스트 블록에 신규 테스트가 포함되어 실행됐다.

```
     Running unittests src\lib.rs (target\release-test\deps\rhwp-9b807afca6f4a468.exe)
running 2472 tests
...
test serializer::hwpx::tests::footnote_endnote_decoration_attrs_roundtrip ... ok
...
test result: ok. 2465 passed; 0 failed; 7 ignored; 0 measured; 0 filtered out; finished in 8.82s
```

각주/미주가 포함된 HWPX 구조 보존 게이트(`samples/hwpx/` 안 note 13개 — `footnote-01.hwpx` 9,
`footnote-tbox-01.hwpx` 2, `143E433F503322BD33.hwpx` 1, `aift.hwpx` 1)도 통과했다.

```
     Running tests\hwpx_roundtrip_baseline.rs
running 4 tests
test xfail_entries_still_fail ... ok
test grade_lists_are_consistent ... ok
test baseline_large_samples_roundtrip ... ok
test baseline_all_samples_roundtrip ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.29s
```

전체 집계: `test result:` 블록 291개 합산 **3481 passed / 0 failed / 23 ignored**, `FAILED` 문자열
등장 0회.

---

## 5. 미실행 항목

- 한컴 오피스 실물 열기 대조는 하지 않았다. 대신 한컴 저장본 원본과의 **속성 전수 일치**(4-2)
  와 **HWP5 페이로드 바이트 일치**(4-3) 로 대체했다.
- 시각(SVG/PNG) 픽셀 diff 는 하지 않았다. 마커 문자열 수준(4-4)까지만 확인했다.

---

## 6. 잔여 (범위 밖)

- **공개 OWPML 스키마 불일치**: `mydocs/manual/OWPML SCHEMA/ParaList XML schema.xml:2735
  NoteType` 은 `instId` 만 선언하고 `number`/`prefixChar`/`suffixChar`/`flag` 는 선언하지 않는다.
  본 작업은 공개 스키마가 아니라 한컴 실측 계약(828/828, 46/46)을 기준으로 했다. 스키마 문서
  갱신은 별건.
- **`flag` 상위 바이트 의미**: 실측값 `0x00310000`, `0x00480000`, `0x00380000`, `0x00200000` 등
  하위 바이트(번호 모양 코드)는 항상 0 이고 byte 2 만 변한다. 의미 미상 — 본 작업은 raw 왕복
  보존만 다뤘다. 현행 렌더러는 `number_shape as u8` 만 쓰므로
  (`src/renderer/layout/paragraph_layout.rs:529`) 이 샘플군 렌더에는 영향이 없다.
- **각주/미주 `<hp:subList>` 의 `textWidth`/`textHeight`/`hasTextRef`/`hasNumRef`**: 여전히 `"0"`
  하드코딩이다. 머리말/꼬리말과 달리 `Footnote`/`Endnote` IR 에 대응 필드 자체가 없어 모델 확장이
  필요하다. 별건.
- **HWP3 각주 경로**(`src/parser/hwp3/mod.rs:3394`): `after_decoration_letter = ')'`,
  `number_shape = 0` 고정. HWP3 전용 해석은 `src/parser/hwp3/` 안에 머물러야 한다는 `CLAUDE.md`
  규칙에 따라 다루지 않았다.
