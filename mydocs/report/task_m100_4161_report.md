# task_m100_4161 최종 보고서 — CharShape 장평 기본값 0 정합

- **이슈**: [#4161](https://github.com/edwardkim/rhwp/issues/4161) `[model] CharShape.ratios 기본값 0 이
  OWPML 유효범위 [50,200] 밖 — 렌더러가 장평 0 으로 소비한다`
- **브랜치**: `task_m100_4161` (분기 기준 `upstream/devel` `0bc05ef81`)
- **문서**: 계획 [`plans/task_m100_4161.md`](../plans/task_m100_4161.md) ·
  [stage1](../working/task_m100_4161_stage1.md) · [stage2](../working/task_m100_4161_stage2.md) ·
  [stage3](../working/task_m100_4161_stage3.md)
- **작성**: 2026-08-18 KST

## 1. 요약

`impl Default for CharShape` 의 `ratios` 를 `[0; 7]` → `[100; 7]` 로 고쳤다 —
**프로덕션 변경은 이 한 줄**이다. OWPML `ratio` 는 `xs:positiveInteger` [50,200],
default=100 이라 0 은 타입 수준 불법이고, HWP5 파서 폴백(`parser/doc_info.rs:528-532`)은
이미 100 이라 모델만 어긋나 있었다. HWPX ratio 자식 부재·HML RATIO 부재·HWP3 인덱스 0
placeholder·HTML import·charPr id 갭 filler — 미채움 5계열 전부가 `CharShape::default()` 를
통과하므로 한 곳 수정으로 전부 해소된다(#4141 과 같은 구조).

#4141(`relative_sizes`)과 달리 렌더러가 이 값을 소비하지만, 폭 계산 경로 5곳이 전부
`ratio > 0.0` 폴백이라 0→100 전환의 자체 렌더 산출은 **byte-identical** 로 실측됐다.
바뀌는 것은 저장 축뿐이다 — 스키마 불법값 0 의 방출이 사라진다.

## 2. 왜 지금까지 안 잡혔나

- HWPX 라운드트립 검증기는 char_shapes 를 개수만 비교(`ratios` 비교 0건), HML preflight
  비교 목록에도 없음 → `--verify` 무반응.
- HML 왕복 동등성 테스트는 before==after 만 봐서 0→"0"→0 도 통과.
- rhwp 자체 렌더는 `ratio > 0.0` 폴백이 결함을 가림 — 저장 바이트/XML 만이 증거.
- 골든 SVG 7건은 전부 ratio 명시 표본이라 기본값 경로 미경유.

## 3. 실측 (Stage 1)

- **red**: 신규 계약 5건 전부 실패 — HWP3 표본 **22건 전수**에서 위반이 표본당 정확히
  7건(= idx0 placeholder 1레코드 × 7언어 슬롯), **실데이터 위반 0건**.
- **한컴산 정답지**: HWPX 표본 276건 charPr 15,120개의 `ratio` 105,840건 —
  min=50 / max=154, 범위 밖 **0건**, ratio 자식 부재 **0건**. 범위 [50,200] 단언이
  실데이터와 충돌하지 않음을 선판정.
- **재현 실물**: `rhwp export-hml tests/fixtures/hml/exambank_math_equations_min.hml` →
  `<RATIO Hangul="0" …/>` (이슈 재현 명령 그대로).

## 4. 수정

`src/model/style.rs` — 기본값 1줄 + impl doc 주석 + 잠금 테스트.

### 함께 바꾸지 않은 것 — `base_size` (계측 후 제외 확정)

이슈의 "함께 검토" 요건은 stage1 §5 계측으로 이행했다. 제외 근거 3가지:

1. **스키마 합법** — OWPML `height` 는 `xs:integer`(제약 없음, default=1000)라 0 이
   불법값이 아니다. #4161 의 정합 논거가 성립하지 않는다.
2. **실도달 0건** — height 부재 charPr 0/15,120, HML 리더는 자체 `unwrap_or(1000)`,
   HWP3 idx0 참조 0건. 기본값 base_size 가 소비되는 실표본이 없다.
3. **계약 3축 반전** — `doclang/adapter/inline.rs:421` 의 "폰트 정보 없음" sentinel,
   hidden-text 은닉 판정(0pt→10pt), 무가드 레이아웃 소비(`style_resolver.rs:341`).
   행동 보존 재작성이 없고 "기본값 유래" 프로버넌스 설계가 선행돼야 한다.

잠금 테스트(`char_shape_default_matches_spec_except_base_size`)가 이 경계를 계속 고정한다.

### 기각한 대안

라이터 가드(증상 은폐) / 파서별 폴백 4곳+(재발 면적) / 폭 경로 가드 제거(이득 없는 회귀면) —
계획서 §3.

## 5. 회귀 고정 — `tests/cases/issue_4161_ratio_default_contract.rs` (신규 5건)

#4141 계약의 판형을 복사·파라미터화(오프셋 14..21, 범위 50..=200, `hh:ratio`/`RATIO`).
단언은 `==100` 이 아니라 **범위 소속** — ratios 는 HWP3 레코드 실데이터(95/90/97/100 편차)가
있어 relSz 와 강도가 다르다.

1. `hwp3_convert_emits_valid_ratios_for_every_sample` — 표본 전수 + 하한 가드
2. `so_sueop_convert_ratios_within_valid_range` — 재현 표본 고정 (개수 하한 >1000)
3. `public_document_core_export_also_emits_valid_ratios`
4. `hwp3_export_hwpx_emits_valid_hh_ratio`
5. `hml_roundtrip_without_ratio_child_emits_valid_ratio`

## 6. 검증

### TDD — 빨강을 먼저 확인했다

red 5/5 실패(stage1 §2 원문) → 기본값 1줄 수정 → green 5/5 (stage2 §2).

### 게이트 (`local_validation.md` 4.3 — model + renderer 두 lane 합집합)

| 게이트 | 결과 |
| --- | --- |
| rustfmt(변경 파일, LF 정규화) · 유닛 티어 · manifest 규칙 | 통과 (호스트 아티팩트 1건 기록 — stage2 §3) |
| `cargo clippy --all-targets -- -D warnings` | 통과, 경고 0 |
| lib 유닛 | 4,068건 통과, 실패 0 |
| release-test 전체 nextest | **6,892 / 6,892 통과** (8 slow, 38 skipped) |
| 골든 SVG | 8건 PASS, **갱신 0건** |
| IR field sweep | PASS (321s, baseline 무변동) |
| Native Skia 3종 | 58 + 2 + 4 전건 통과 |
| WASM (Docker 표준 경로) | 통과 — Done in 13m 52s |

### 시각 증적

exambank `export-svg` 전후 **byte-identical** · SO-SUEOP 46쪽 `export-pdf` 전후
**byte-identical** · `render-diff --via hwpx` PASS(변위 0.00px) · 왕복 HML `RATIO` 0→100.
after HWPX `hh:ratio` 분포 100×6,846 / 95×6,531 / 90×3,969 / 97×238, **0 = 0건** —
실데이터 편차는 보존되고 placeholder 만 정상화됐다.

## 7. 실물 확인 (한컴 판정 핸드오프 — 선택)

`output/task_m100_4161/` 에 before/after 번들과 판정 안내를 두었다(로컬 전용).
한컴 변환 PDF 를 같은 폴더에 넣으면 144DPI 판정을 걸 수 있다. rhwp 화면은 전후 동일한
것이 성공 기준(무회귀)이며, 한글 [글자 모양 → 장평]에서 실데이터가 그대로 읽히는지가
확인 포인트다.

## 8. 사용자 영향

- HWP3→HWP5/HWPX 변환본·RATIO 부재 HML 왕복본의 **재생성을 권장** — 기존 산출물에는
  스키마 불법값 0 이 실려 있다 (idx0 placeholder 는 참조 0건이라 화면 영향은 없음).
- rhwp 자체 렌더 산출은 변하지 않는다 (byte-identical 실측).

## 9. 후속

- **`base_size` 기본값 프로버넌스** (별도 이슈 제안) — §4 의 제외 근거 3축이 그대로
  요건이다: "기본값 유래 여부"를 IR 이 표현해야 sentinel·은닉 판정·레이아웃 계약을
  깨지 않고 스펙 기본값 1000 을 채울 수 있다. 계측 실측은 stage1 §5.
- OWPML 정합 관찰 노트 §17 에 본 사례 기록 완료.

## 10. 커밋

| 커밋 | 내용 |
| --- | --- |
| `docs(plan): #4161 수행계획과 Stage 1 재현·계측` | 계획서 + stage1 (red 원문·계측) |
| `08901bb6f fix(model): CharShape 기본 장평을 OWPML 기본값 100 으로 (#4161)` | 기본값 1줄 + 잠금 테스트 + 계약 5건 + 티어 기준선 |
| `docs(working): #4161 Stage 2 green 보고와 OWPML ratio 정합 관찰 §17` | stage2 + 관찰 노트 |
| (본 커밋) | stage3 + 최종 보고서 |
