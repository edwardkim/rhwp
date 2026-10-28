# Stage 1 — task_m100_4161 재현·계측

- **이슈**: [#4161](https://github.com/edwardkim/rhwp/issues/4161)
- **계획서**: [`mydocs/plans/task_m100_4161.md`](../../plans/archives/task_m100_4161.md)
- **브랜치**: `task_m100_4161` (분기 기준 `upstream/devel` `0bc05ef81`)
- **작업 시각**: 2026-08-18 KST
- **프로덕션 코드 변경**: 0 (신규 계약 테스트 1파일만 추가 — red 상태)

## 1. 계측 방법

- **계약 테스트 red**: `tests/cases/issue_4161_ratio_default_contract.rs` 를 먼저 작성하고
  (`node scripts/rust-test-suite-manifest.mjs --generate` 로 로컬 배선 — 생성물 미커밋)
  `cargo test --test regression_suite_021 issue_4161` 로 실패 원문을 확보했다.
  검사 방식은 #4141 stage1 과 동일 — CFB `DocInfo` 의 `HWPTAG_CHAR_SHAPE` payload
  오프셋 **14..21**(ratios), HWPX `Contents/header.xml` 의 `<hh:ratio>`, HML `<RATIO>`.
- **XML 전수 스윕**: scratchpad 임시 Python 스크립트(커밋하지 않음, #4141 stage1 §1 관례)로
  `samples/**/*.hwpx` 276건의 charPr 15,120개와 HML 표본·fixture 의 CHARSHAPE 를 전수 판독.
- **before 산출물**: 분기 기준 소스 그대로의 바이너리(`cargo build --bin rhwp`)로
  exambank fixture 의 HML 왕복·SVG, `samples/SO-SUEOP.hwp` 의 PDF 를 확보.

## 2. 계약 테스트 red 실측 — 위반은 전 표본에서 idx0 placeholder 뿐이다

`cargo test --test regression_suite_021 issue_4161` (2026-08-18):

```text
test result: FAILED. 0 passed; 5 failed; 0 ignored; 0 measured; 81 filtered out; finished in 12.19s

failures:
    issue_4161_ratio_default_contract::hml_roundtrip_without_ratio_child_emits_valid_ratio
    issue_4161_ratio_default_contract::hwp3_convert_emits_valid_ratios_for_every_sample
    issue_4161_ratio_default_contract::hwp3_export_hwpx_emits_valid_hh_ratio
    issue_4161_ratio_default_contract::public_document_core_export_also_emits_valid_ratios
    issue_4161_ratio_default_contract::so_sueop_convert_ratios_within_valid_range
```

대표 실패 사유(발췌):

```text
HWP3 표본 22건 중 22건 실패 (통과분 CHAR_SHAPE 0개):
  SO-SUEOP.hwp: CHAR_SHAPE 2512개 중 위반 7건 — 첫 위반 id=0 한글=0. …
  hwp3-sample10.hwp: CHAR_SHAPE 28193개 중 위반 7건 — 첫 위반 id=0 한글=0. …
samples/SO-SUEOP.hwp: <hh:ratio> 2512개 중 1개가 유효범위 50~200 밖이다
  (첫 위반: `<hh:ratio hangul="0" latin="0" hanja="0" japanese="0" other="0" symbol="0" user="0"/>`)
```

**판독**: HWP3 표본은 #4141 시점 15건에서 **22건**으로 늘었고, 22건 전부에서 위반이
표본당 **정확히 7건 = 인덱스 0 placeholder 1레코드의 7개 언어 슬롯**이다.
`convert_char_shape` 가 레코드에서 복사한 실데이터 위반은 **0건** — 이슈의 원인 진단
(placeholder 만 결함, HWP3 실데이터 축은 안전)이 확장 표본에서도 유지된다.

## 3. HWPX/HML 표본 전수 스윕 — 범위 단언 [50,200] 선판정

`samples/**/*.hwpx` 276건 (ZIP 아닌 손상 표본 6건 제외), charPr 15,120개:

| 항목 | 실측 |
| --- | ---: |
| `<hh:ratio>` 속성값 총건수 | 105,840 |
| 값 분포 상위 | 100×84,655 / 95×9,177 / 98×3,956 / 97×1,941 / 90×1,799 … |
| **min / max** | **50 / 154** |
| 유효범위 [50,200] 밖 | **0건** |
| `<hh:ratio>` 자식 없는 charPr | **0건** |
| `height` 속성 없는 charPr | 0건 |
| `height="0"` (명시값) charPr | 1건 (`samples/hwpx/issue1948_cross_para_fieldend.hwpx`) |
| charPr id 불연속(갭 filler 발동) | 1건 (`samples/issue3460/svg_picture_repro.hwpx`, charPr 37개) |

HML (`samples/hml/*` 2건 + fixture): RATIO 값 분포 100×70, 95×14 — 전부 유효.
RATIO 자식 없는 CHARSHAPE 는 `tests/fixtures/hml/exambank_math_equations_min.hml` 의 2개뿐.
`Height` 속성 없는 CHARSHAPE 0건.

**판정 1 (범위 단언)**: 한컴산 실데이터 105,840건의 min=50 / max=154 가 [50,200] 안이고,
HWP3 변환 축(§2)의 위반도 전부 placeholder 유래다. **계약 테스트의 [50,200] 소속 단언은
실데이터와 충돌하지 않는다** — 계획서의 완화 컨틴전시(`!=0` 로 좁힘)는 불발동.

**판정 2 (ratios 기본값 실도달)**: 실표본에서 `ratios` 기본값 경로에 도달하는 것은
① HWP3 placeholder(전 22표본, 단 참조 0건 — #4141 stage1 §3), ② HWPX 갭 filler 1건,
③ RATIO 부재 HML fixture 뿐이다. 그런데 **방출 축은 전수 발화한다** — HWP3→HWP5/HWPX
변환본 전부와 HML 왕복이 스키마 불법값 0 을 실어 나른다(§2). 저장 정합 결함이 핵심이고
자체 렌더는 폭 경로 `ratio > 0.0` 폴백이 가린다는 이슈 진단과 일치한다.

## 4. before 산출물 (분기 기준 소스 빌드)

- `exambank_math_equations_min.hml` HML 왕복 → **`<RATIO Hangul="0" Latin="0" Hanja="0"
  Japanese="0" Other="0" Symbol="0" User="0"/>`** 방출 확인 (이슈 재현 명령 그대로).
- 같은 fixture `export-svg`, `samples/SO-SUEOP.hwp` `export-pdf` 를 scratchpad
  `task_m100_4161_evidence/before/` 에 보존 — Stage 3 에서 after 와 대조한다.

## 5. base_size 계측과 결정 — **이번 PR 에서 제외** (후속 이슈 분리 제안)

계획서 §3 의 결정 기준에 따라 계측했다.

### 5.1 스키마 판정 — ratios 와 결함의 부류가 다르다

OWPML `charPr@height` 는 **`xs:integer` (제약 없음), default="1000"** 이다
(`Header XML schema.xml:985-990`). 즉 **`base_size=0` 은 스키마 합법**이다.
`ratio` 가 `xs:positiveInteger` [50,200] 이라 0 이 **타입 수준 불법**인 것과 다르다 —
#4161 을 정당화하는 "OWPML 정합" 논거가 base_size 에는 성립하지 않는다.

### 5.2 실표본 도달 계측 — 기본값 base_size 가 소비되는 실사례 0건

| 경로 | 실측 |
| --- | --- |
| HWPX charPr `height` 속성 부재 (Default 잔류) | **0건** / 276표본 (§3) |
| HML CHARSHAPE `Height` 부재 | **0건** — 리더 자체도 `unwrap_or(1000)` (`src/parser/hml/reader.rs:602`) |
| HWP3 idx0 placeholder 참조 | **0건** / 전 표본 (#4141 stage1 §3, PARA_CHAR_SHAPE 집계) |
| HWPX 갭 filler | 1건 잠재 (`issue3460`, 갭 id 가 문단에서 참조될 때만) |
| HTML import 기본 CharShape | 합성 경로 (문서에 char_shape 이 0개일 때만) |

`height="0"` 1건(`issue1948`)은 **명시값**이라 Default 변경과 무관하다 (파싱값 유지).

### 5.3 행동 반전 지점 — 셋 다 "이득 없는 계약 변경"

1. `src/doclang/adapter/inline.rs:421` — `cs.base_size != 0` 이 "보고할 폰트 정보 있음"
   판정의 일부다. 기본값이 1000 이 되면 기본형(정보 없는) CharShape 도 손실 항목
   `font_id=0, base_size=1000` 을 방출한다. 행동 보존 재작성이 없다 — `!= 1000` 으로
   바꾸면 명시적 10pt(최빈값) 문서의 손실 보고가 소실되는 실회귀다.
2. `src/document_core/queries/hidden_text.rs:255-257` — 실효 pt `base_size.max(0)/100.0`.
   기본형 문자의 은닉(ZeroSize) 판정이 0pt→10pt 로 반전된다.
   `tests/hidden_text_contract.rs:36,367` 이 `CharShape::default()` 로 이 계약을 고정 중이다.
3. `src/renderer/style_resolver.rs:341` — 폰트 크기의 유일한 원천이고 `ratios` 와 달리
   0 가드 폴백이 없다. 기본값 경로 문서의 줄높이·레이아웃이 실제로 움직인다.

### 5.4 결정

**제외한다.** 근거: (1) 스키마 위반이 아니므로 정합 수정의 명분이 없고(§5.1),
(2) 실표본에서 기본값 base_size 소비 사례가 0건이라 사용자 가시 이득이 없으며(§5.2),
(3) 반면 손실 보고·은닉 판정·레이아웃 세 축의 계약이 움직인다(§5.3). 올바른 해법은
"기본값 유래 여부" 프로버넌스(`Option` 화 등 IR 설계 변경)로 별도 이슈 규모다.
Stage 2 에서 잠금 테스트의 `base_size == 0` 단언은 유지하고 사유 메시지를 위 3논거로
교체하며, 최종 보고서에서 후속 이슈 분리를 제안한다.

## 6. Stage 1 게이트 판정

| 항목 | 기준 | 결과 |
| --- | --- | --- |
| 계약 테스트 red | 5건 전부 예상 사유(기본값 0)로 실패 | **통과** (§2 원문) |
| 범위 단언 선판정 | 실데이터와 충돌 없음 | **통과** — min 50 / max 154, 위반 전부 placeholder |
| 스윕 하한 가드 | `MIN_SWEPT_SAMPLES=10` | **통과** — 22건 스윕 |
| base_size 결정 | 계획서 §3 기준으로 판정 | **제외 확정** (§5) |
| before 산출물 | HML 왕복·SVG·PDF 확보 | **통과** (§4) |
| 프로덕션 변경 0 | `git status` 에 src/ 변경 없음 | **통과** |

## 7. Stage 2 로 넘기는 변경

- `src/model/style.rs:213` `ratios: [0; 7]` → `[100; 7]` + doc 주석(:187-191) 교체
- 잠금 테스트(:1154-1195) — ratios 단언 `[100;7]` 반전, base_size 단언 유지 + 사유 교체
- 계약 테스트 green 확인 → fmt·clippy·유닛
