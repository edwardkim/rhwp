---
kind: working
status: active
issue: 6872
---

# x2x 왕복에서 각주 번호가 밀리고 표시 모양이 바뀐다 (#6872)

작업 브랜치: `fix/6872-hwpx-footnote-autonum`
대상: `src/serializer/hwpx/section.rs` · `src/parser/hwpx/section.rs`

## 한 줄

HWPX 저장이 **한컴이 쓰지 않는 numbering 토큰**(`RESTART_PAGE`)을 내보내고, **빈 장식
문자**(`suffixChar=""`)를 기본값 `)` 로 되돌려 놓는다. 한글은 전자를 못 알아들어 각주
번호를 연속으로 매기고, 후자로 사용자 기호 `*` 가 `*)` 가 된다.

## 이슈가 요구한 것

- `x2x` 왕복에서 각주 번호(`1)` → `2)`)와 표시 모양(`*` → `*)`)을 원본대로 보존한다.
- `h2h`·`h2x`·`x2h` 는 이미 OK 이므로 깨지 않는다.

## 왜 rhwp 자기 눈에는 무결했나

`rhwp export-text` 로 원본과 왕복본을 견주면 **양쪽 다 `1)`** 이다. rhwp 파서가 자기가
낸 토큰을 도로 받아주기 때문이다.

```rust
// src/parser/hwpx/section.rs — 관대한 수용
"ON_PAGE" | "RESTART_PAGE" | "restartPage" => …
```

그래서 이 결함은 **한글을 정답지로 세워야만** 보인다. 이슈가 한글 2024 텍스트 추출을
쓴 이유가 이것이다.

## 축 1 — 한컴이 쓰지 않는 numbering 토큰

```rust
// 종전
RestartSection => "RESTART_SECTION",
RestartPage    => "RESTART_PAGE",
```

주석은 "파서도 이 토큰을 수용한다" 고 적었는데, 수용하는 것은 **rhwp 파서**이지 한글이
아니다. 코퍼스 실측이 토큰을 확정한다.

```text
  원본 HWPX 3,391 파일의 <hp:numbering type>
    CONTINUOUS   7,640
    ON_PAGE         12
    RESTART_*        0      ← 한컴은 한 번도 쓰지 않는다
```

한글은 `RESTART_PAGE` 를 못 알아듣고 **연속 번호로 떨어진다**. `ON_PAGE`/`ON_SECTION`
으로 바꿨다.

## 축 2 — 빈 장식 문자가 `)` 로 되살아난다

파서와 직렬화기 **양쪽**이 빈 값을 잃는다.

```rust
// 파서: 빈 문자열이면 chars().next() 가 None → 그냥 넘어가 기본값이 남는다
if let Some(c) = s.chars().next() { shape.suffix_char = c; }

// 직렬화기: '\0' 을 ")" 로 되돌린다
note_deco_char_attr(shape.suffix_char, ")"),
```

### 첫 시도가 깬 것 — `'\0'` 에 두 뜻이 겹쳐 있다

처음에는 "`'\0'` 은 없음" 이라는 HWP3 축의 해석(`footnote_bracket == 0` → `'\0'`,
`issue_hwp3_endnote_suffix_char_wires_footnote_bracket_flag`)을 그대로 가져와 폴백을
`""` 로 바꿨다. **회귀 2건이 났다.**

```
issue2742_auto_num_format_keeps_template_when_ir_unset  FAILED  (left 0, right 2)
issue2742_auto_num_format_reflects_ir                   FAILED
```

`#2742` 는 HWPX 직렬화 경로에 **반대 규약**을 세워 두었다 — 주석 그대로 "`'\0'` = 미지정"
이고, IR 미설정이면 템플릿 문자열(`suffixChar=")"`)을 바이트 동일하게 유지해야 한다.
즉 `'\0'` 하나에 **"원본이 명시적으로 비웠다"** 와 **"IR 이 설정된 적 없다"** 가 겹쳐 있고,
직렬화기만 보고는 가를 수 없다.

### 신호를 분리한다

```rust
/// [#6872] HWPX `<hp:autoNumFormat>` 의 장식 문자 속성을 원본에서 실제로 읽었는지.
pub deco_chars_from_source: bool,
```

HWPX 파서가 `<hp:autoNumFormat>` 을 만나면 `true` 로 세운다. 직렬화기는 그때만 빈 값을
빈 채로 낸다.

| 경우 | 플래그 | 방출 |
|---|---|---|
| 원본 HWPX `suffixChar=""` | true | `""` (`*` 유지) |
| 원본 HWPX `suffixChar=")"` | true | `")"` |
| IR 미설정 (#2742) | false | `")"` — **종전 계약 보존** |
| HWP5·HWP3 경로 | false | 종전 그대로 — **동작 불변** |

`prefixChar`·`userChar` 도 같은 형상이라 파서에서 함께 고쳤다.

## 검증 실측 — 한컴 2024 정답지

MCP 변환 서버(한컴 13.0.0.3901)로 **원본 / 수정 전 왕복 / 수정 후 왕복** 을 각각 PDF 로
뽑아 텍스트를 대조했다.

| 문서 | 원본 vs 수정 전 | 원본 vs 수정 후 |
|---|---|---|
| 156584446 (2333, 36쪽) | **diff 60줄** | **0줄** |
| 156513948 (00931, 32쪽) | **diff 379줄** | **0줄** |

수정 전 diff 원문(축 1):

```text
193c193
<                     1)
---
>                     2)
361c361
<      등은 증가하였으나, 전자· 통신1), 화학제품, 1차금속 등은 감소
---
>      은 증가하였으나, 전자· 통신3), 화학제품, 1차금속 등은 감소
467c467
<                              1)          →          4)
809c809
<        금속가공          전자·통신1)      →      전자·통신5)
```

쪽마다 `1)` 로 재시작하던 각주가 왕복 뒤 `1) 2) 3) 4) 5) 6) …` 으로 이어진다.

수정 전 diff 원문(축 2):

```text
911c911
<           *          →          *)
939c939
< * 표시는 전년동기대비 증감을 의미함   →   *) 표시는 전년동기대비 증감을 의미함
```

B 문서 diff 379줄에는 들여쓰기 차이도 섞여 있었는데 **그것도 이 두 결함의 결과**였다 —
각주 존 높이가 달라져 줄바꿈이 밀린 것이고, 수정 뒤 함께 사라졌다(0줄).

구조 대조로도 A 문서의 note 속성 12개가 원본과 **완전 일치**한다.

## 게이트

| 게이트 | 결과 |
|---|---|
| `cargo fmt --all -- --check` | OK (수정 없음) |
| `node scripts/rust-unit-test-tiers.mjs --check` | 4208 tests / 298 modules |
| native·WASM32·workspace all-targets Clippy | OK |
| `node scripts/rust-test-suite-manifest.mjs --check` | 48/48 targets |
| `cargo test --release --workspace` | **9424 passed / 0 failed / 49 ignored** (94 suites) |

## 코퍼스 영향 범위

| 축 | 조건 | 원본 문서 |
|---|---|---|
| 1 | `<hp:numbering>` 이 `ON_PAGE`/`ON_SECTION` | 9건 |
| 2 | `autoNumFormat suffixChar=""` | 24건 |

## 회귀 테스트 (신규 3건)

- `issue6872_numbering_uses_hancom_tokens` (직렬화기) — 토큰 3종 고정.
- `issue6872_empty_suffix_char_stays_empty` (직렬화기) — 원본이 비운 접미는 빈 채로,
  명시된 `)` 는 그대로, **IR 미설정은 `)` 유지**(#2742 계약 동시 고정).
- `issue6872_empty_deco_char_attrs_mean_none` (파서) — 빈 `suffixChar`/`prefixChar` 를
  `'\0'` 으로 기록하고 `userChar`·`numbering` 은 보존.

기존 `footnote_endnote_numbering_and_start_reflect_ir` 의 기대값도 실제 방출 토큰
(`ON_PAGE`/`ON_SECTION`)으로 갱신했다.

## 남긴 것 — 세 번째 축(이 PR 범위 밖)

156513948 은 `USER_CHAR` autoNumFormat 슬롯이 6개인데 그중 **5개가 `DIGIT` 으로
떨어진다.** `replace_footnote_shape` 가 한 구역에서 템플릿의 **앞 두 슬롯**(각주·미주)만
치환하기 때문이다 — 구역에 note 속성 블록이 셋 이상이면 나머지는 템플릿 상수로 남는다.

**수정 전 5개 · 수정 후 5개** 로 이 PR 과 무관하며(악화 아님), 정답지 diff 가 0줄인 것으로
보아 그 5개는 이 문서의 렌더링에 쓰이지 않는 슬롯이다. 템플릿 치환을 슬롯 수만큼 여는
별도 축이라 여기서 열지 않았다.
