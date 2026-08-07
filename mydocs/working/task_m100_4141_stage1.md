# Stage 1 — task_m100_4141 재현·계측

- **이슈**: [#4141](https://github.com/edwardkim/rhwp/issues/4141)
- **계획서**: [`mydocs/plans/task_m100_4141.md`](../plans/task_m100_4141.md)
- **브랜치**: `task_m100_4141` (분기 기준 `upstream/devel` `0fdac31ba`)
- **작업 시각**: 2026-08-07 KST
- **프로덕션 코드 변경**: 0

## 1. 계측 방법

수정 전 바이너리(`cargo build --profile release-test --bin rhwp`, 분기 기준 그대로)로 `samples/`
하위 HWP3 서명(`HWP Document File`) 파일 전수를 `convert`(→HWP5)와 `export-hwpx`(→HWPX) 한 뒤,
산출물에서 **저장 바이트/XML 을 직접** 읽었다.

- HWP5: CFB `DocInfo` 스트림을 압축 해제(`FileHeader[36] & 0x01`)하고 레코드를 순회해
  `HWPTAG_CHAR_SHAPE`(= `HWPTAG_BEGIN + 5` = 21) payload 를 수집. 오프셋 **28..35** 가
  `relative_sizes` 다 (레이아웃 정본 `src/parser/doc_info.rs:520-577`).
- HWPX: `Contents/header.xml` 의 `<hh:relSz>` 태그.
- 인덱스 0 참조 여부: `BodyText/Section*` 의 `HWPTAG_PARA_CHAR_SHAPE`(= `HWPTAG_BEGIN + 52` = 68)
  payload 를 `(pos: u32, char_shape_id: u32)` 쌍으로 읽어 id 별 참조 건수를 집계.

계측 스크립트는 임시(scratchpad)이며 커밋하지 않는다.

## 2. HWP5 `convert` 축 — 전수 결함

| 표본 | CHAR_SHAPE | relSz≠100 | relSz=0 | ratios=0 | idx0 참조 | relSz 범위밖 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `SO-SUEOP.hwp` | 2,512 | 2,512 | 2,512 | 1 | 0 | 2,512 |
| `hwp3-pagedef-1915.hwp` | 53 | 53 | 53 | 1 | 0 | 53 |
| `hwp3-sample.hwp` | 747 | 747 | 747 | 1 | 0 | 747 |
| `hwp3-sample10.hwp` | 28,193 | 28,193 | 28,193 | 1 | 0 | 28,193 |
| `hwp3-sample11.hwp` | 20,445 | 20,445 | 20,445 | 1 | 0 | 20,445 |
| `hwp3-sample13.hwp` | 262 | 262 | 262 | 1 | 0 | 262 |
| `hwp3-sample14.hwp` | 445 | 445 | 445 | 1 | 0 | 445 |
| `hwp3-sample16.hwp` | 6,520 | 6,520 | 6,520 | 1 | 0 | 6,520 |
| `hwp3-sample19.hwp` | 93 | 93 | 93 | 1 | 0 | 93 |
| `hwp3-sample4.hwp` | 2,567 | 2,567 | 2,567 | 1 | 0 | 2,567 |
| `hwp3-sample5.hwp` | 5,831 | 5,831 | 5,831 | 1 | 0 | 5,831 |
| `issue1892_hwp3_drawing_group_roundtrip.hwp` | 110 | 110 | 110 | 1 | 0 | 110 |
| `issue1892_hwp3_tab_roundtrip.hwp` | 63 | 63 | 63 | 1 | 0 | 63 |
| `issue1950_hwp3_tab_charoffset.hwp` | 156 | 156 | 156 | 1 | 0 | 156 |
| `issue_265.hwp` | 747 | 747 | 747 | 1 | 0 | 747 |
| **합계 (15건)** | **68,744** | **68,744** | **68,744** | **15** | **0** | **68,744** |

`samples/HWP3-password-123456.hwp` 는 `convert` 가 비밀번호를 요구해 제외(16건 중 1건).

**판정: CHAR_SHAPE 68,744개 전건(100%)이 `relative_sizes = [0;7]` 이다.** OWPML `relSz` 는
`xs:positiveInteger` minInclusive=10 / maxInclusive=250 이므로(`Header XML schema.xml:716-728`)
0 은 타입 수준에서 이미 불법이다.

`SO-SUEOP.hwp` 의 2,512 는 이슈 본문의 "2512개, 최빈값 1000=10pt" 와 **정확히 일치**한다.

### 대표 레코드 실측 (`SO-SUEOP.hwp` 변환본)

```text
idx0  ratios=[0×7]   spacings=[0×7]    relative_sizes=[0×7]  char_offsets=[0×7]  base_size=0
idx1  ratios=[95×7]  spacings=[-1×7]   relative_sizes=[0×7]  char_offsets=[0×7]  base_size=1000
idx2  ratios=[95×7]  spacings=[-1×7]   relative_sizes=[0×7]  char_offsets=[0×7]  base_size=1000
```

`samples/SO-SUEOP.hwpx`(같은 문서의 한컴산 HWPX) 정답지와 대조:

| 필드 | 한컴 HWPX (charPr 51개) | rhwp 변환본 | 판정 |
| --- | --- | --- | --- |
| `relSz` | **51/51 = 100** (편차 0) | **0** | **결함** |
| `ratio` | 95×30, 90×11, 100×8, 97×2 | 95 | 정상 — HWP3 레코드의 진짜 데이터 |
| `spacing` | -1×25, 0×11, -2×10, -3×3 | -1 | 정상 — 진짜 데이터 |
| `offset` | 51/51 = 0 | 0 | 정상 — 0 이 유효값 |

→ **오직 `relative_sizes` 만 잘못됐다.** 계획서 §3 의 판단이 실측으로 확정됐다.

## 3. 인덱스 0 CharShape 는 참조되지 않는다

`src/parser/hwp3/mod.rs:3566` 이 인덱스 0 자리에 `CharShape::default()` 를 push 하므로
`ratios=[0;7]`·`base_size=0` 인 레코드가 표본마다 **정확히 1개** 생긴다. 이것이 위 표의
`ratios=0` 열이 전 표본 1인 이유다.

그런데 **그 레코드를 참조하는 문단이 없다.** `SO-SUEOP.hwp` 변환본 실측:

```text
PARA_CHAR_SHAPE 참조 총건수 1,918 / 고유 id 1,918
참조된 최소 id = 16     (id 0 참조 = 0건)
```

집계기가 실제로 참조를 읽고 있음은 총건수 1,918 로 확인된다(파싱 불량으로 0 이 나온 것이 아니다).
전 15표본에서 idx0 참조는 0건이다.

**따라서 `base_size=0`·`ratios=0` 은 이번 백지 증상의 2차 원인이 아니다.** 한컴 판정에서
`after` 가 여전히 비정상일 경우를 대비한 리스크 항목(계획서 §10)이 이 계측으로 해소됐다.

## 4. HWPX `export-hwpx` 축 — 동일하게 전수 결함

이슈 본문이 "미실측"으로 남긴 항목을 확정했다.

| 표본 | `<hh:relSz>` | 범위 밖 | 값 분포 |
| --- | ---: | ---: | --- |
| `SO-SUEOP.hwp` | 2,512 | 2,512 | 전부 `0` |
| `hwp3-sample10.hwp` | 28,193 | 28,193 | 전부 `0` |
| `hwp3-sample11.hwp` | 20,445 | 20,445 | 전부 `0` |
| (나머지 12건) | HWP5 축과 동수 | 동수 | 전부 `0` |

`src/serializer/hwpx/header.rs:638` 이 `write_lang_attrs(w, "hh:relSz", ...)` 를 가드 없이 부르고,
`export-hwpx` 는 HWP3 입력을 차단하지 않는다(`src/main.rs:272` → `:11311` → `:11358` →
`src/parser/mod.rs:1294`). 따라서 산출 HWPX 에 `<hh:relSz hangul="0" .../>` 가 그대로 나간다.

## 5. HML 축 — HWP3 경로는 도달 불가, 그러나 HML 왕복은 결함

계획서 §5 의 테스트 ④(HWP3 → HML)를 **재조정한다.**

`DocumentCore::export_hml_native()`(`src/document_core/commands/document.rs:1352-1359`)는
`self.hml_metadata` 를 요구하고, 그 값은 HML 출처에만 설정된다. HWP3 출처는
`hml_metadata_missing_error` 로 막힌다. CLI 도움말도 `export-hml <입력.hml>` 로 입력을 HML 로 한정한다.
→ **HWP3 → HML 은 도달 불가 경로다.**

대신 **HML → HML 왕복**이 실제로 결함이다. `RELSIZE` 자식이 없는 `<CHARSHAPE>` 를 읽으면
`src/parser/hml/reader.rs:599-605` 가 `[0;7]` 을 남기고, `src/serializer/hml/head.rs:131` 이
가드 없이 그대로 방출한다.

저장소 안에 그 fixture 가 있다 — `tests/fixtures/hml/exambank_math_equations_min.hml` 의
`<CHARSHAPE>` 는 `FONTID` 만 갖고 `RATIO`·`RELSIZE` 가 없다:

```xml
<CHARSHAPE Height="1000" Id="0" TextColor="0"><FONTID Hangul="0" .../></CHARSHAPE>
```

배포 CLI 로 왕복시킨 실측:

```bash
rhwp export-hml tests/fixtures/hml/exambank_math_equations_min.hml -o /tmp/exambank_rt.hml
```

```xml
<RELSIZE Hangul="0" Latin="0" Hanja="0" Japanese="0" Other="0" Symbol="0" User="0"/>
<RATIO   Hangul="0" Latin="0" Hanja="0" Japanese="0" Other="0" Symbol="0" User="0"/>
```

`RELSIZE="0"` 은 이번 이슈의 결함이고, **`RATIO="0"` 은 §7 후속 이슈의 결함**이다.
HML 은 렌더 경로이기도 하므로(`export-svg`/`export-pdf` 가 `.hml` 을 받는다) `RATIO="0"` 은
장평 0 = 글자 폭 0 으로 실제 렌더에 영향한다.

→ 테스트 ④를 **"RELSIZE 없는 HML 을 왕복하면 유효범위 안의 값이 나온다"** 로 바꾼다.
이 fixture 가 red→green 을 실제로 보여준다. 기존 HML 표본(`samples/hml/*.hml`)은 전부
`RELSIZE="100"` 이라 수정 전후 모두 통과해 red 가 되지 않는다.

## 6. 전수 스윕 비용

CLI 기준 단건 변환 시간(프로세스 기동 포함):

```text
samples/hwp3-sample10.hwp   8,416 ms   (CHAR_SHAPE 28,193 — 최대)
samples/hwp3-sample11.hwp   1,989 ms
samples/SO-SUEOP.hwp          783 ms
```

계약 테스트는 프로세스 기동 없이 in-process 로 도므로 이보다 빠르다. 15표본 전수 스윕은
십수 초 규모로 예상되며 통합 테스트 예산 안이다. 실측치는 Stage 2 에서 확정한다.

## 7. 후속 이슈 근거 (`ratios` 기본값)

계획서 §9 의 후속 이슈에 붙일 근거가 이 단계에서 확보됐다.

- `ratios` 기본값도 `[0;7]` 이고 OWPML `ratio` 는 default=100 / [50,200] 이라 0 은 범위 밖이다.
- **렌더러가 이 값을 읽는다** — `src/renderer/style_resolver.rs:355`
  `ratios.push(cs.ratios[lang] as f64 / 100.0)` → 장평 0 = 글자 폭 0.
- **HWP3 축은 안전하다** — `convert_char_shape`(`hwp3/mod.rs:540`)가 레코드에서 채우고, 유일하게
  `ratios=0` 인 인덱스 0 레코드는 §3 대로 **참조되지 않는다**(전 15표본 0건).
- **HML 축은 실제로 노출된다** — §5 의 CLI 실측이 `RATIO="0"` 을 재현한다. 이것이 후속 이슈의
  가장 강한 재현 사례다.

→ 별도 이슈로 분리하는 판단(계획서 §9)이 유지된다. HWP3 축과 인과가 없으므로 이번 PR 의 한컴
판정 결과 귀속을 흐리지 않는다.

## 8. Stage 1 게이트 판정

| 항목 | 기준 | 결과 |
| --- | --- | --- |
| SO-SUEOP CHAR_SHAPE 수 | 이슈 본문 2,512 와 일치 | **일치** (2,512) |
| SO-SUEOP.hwpx charPr 수 | 51 | **일치** (relSz 51/51 = 100) |
| relSz 결함 범위 | 전수 | **68,744/68,744 (100%)** |
| HWPX 축 (이슈 미실측) | 확정 필요 | **확정** — 전건 `relSz="0"` |
| 인덱스 0 참조 (2차 원인 후보) | 계측 필요 | **참조 0건** — 2차 원인 아님 |
| HML 축 | 확인 필요 | HWP3→HML **도달 불가**, HML 왕복은 **결함 확인** |

**게이트 통과.** Stage 2(TDD 수정)로 진행한다.

## 9. Stage 2 로 넘기는 변경

- 계약 테스트 ④를 "HWP3 → HML" 에서 **"RELSIZE 없는 HML 왕복"**으로 재조정
  (fixture: `tests/fixtures/hml/exambank_math_equations_min.hml`)
- 실패 메시지에 이 단계의 실측치(68,744 전건 / SO-SUEOP 2,512)를 인용
