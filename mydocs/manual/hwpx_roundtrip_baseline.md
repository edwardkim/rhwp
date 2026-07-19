---
kind: guide
status: active
canonical: mydocs/manual/hwpx_roundtrip_baseline.md
last_verified: 2026-07-17
---

# HWPX Roundtrip Baseline 가이드 (Task #1315, #1378·#1379·#1380 게이트 강화)

`samples/hwpx/` 전수에 대한 HWPX→IR→HWPX roundtrip 검증 체계의 사용·유지보수 매뉴얼.

## 1. 개요

HWPX serializer의 **구조(뼈대) 보존**을 회귀 게이트로 고정한다. 검증 기준:

1. parse → serialize → 재parse 성공
2. `diff_documents(doc1, doc2)` == 0 (IR 뼈대 비교)
3. `check_package()` 통과 — 패키지(ZIP) 구조 규약
4. 2-round 안정성 — 재직렬화 → 재파싱 후 `diff_documents(doc2, doc3)` == 0

`diff_documents` 비교 항목: 섹션 수·문단 수·DocInfo 리소스 수·BinData 수 +
**문단별 char_shapes 시퀀스** (#1378 — `(start_pos, char_shape_id)` 전체 비교,
본문 + 셀·글상자(Group 재귀)·각주/미주 내부 문단 재귀 포함) +
**문단별 인라인 슬롯 컨트롤 타입 시퀀스** (#1379 — 셀·글상자·각주/미주 재귀 동승) +
**문단별 line_segs 9필드 시퀀스** (#1380 — textpos/vertpos/vertsize/textheight/
baseline/spacing/horzpos/horzsize/flags, 동일 경로 재귀. 빈 ↔ 비어있지 않음
합성 비대칭도 개수 불일치로 검출) +
**섹션별 PageDef** (#1388 — 용지 width/height/landscape/binding + 여백 7필드.
secPr 템플릿 고정값 방출에 의한 여백·제본 변형을 검출) +
**표 캡션** (#1387 — 존재 비대칭/속성 5종(side/fullSz/width/gap/lastWidth)/문단 수
+ 캡션 내부 문단의 char_shapes·controls·linesegs 재귀, 경로 `tbl.caption.p[k]`) +
**객체 캡션** (#1403 — 그림/도형/묶음 `pic.caption`·`shape.caption`, 표 캡션과 동일 비교) +
**객체 설명** (#1392 — 그림/도형/수식/묶음 `hp:shapeComment`(`common.description`)
존재·내용 비교, 경로 `pic`/`shape`/`eq`) +
**필드 내용** (#1391 — fieldBegin `parameters` verbatim + MEMO 본문 문단 재귀,
경로 `field`·`field.memo.p[k]`) +
**그림 크기** (#1389 — curSz(shape_attr current)·imgRect(border_x/y)·imgDim, 경로 `pic`) +
**표 page_break** (#1393 — 표 분할 속성. 방출은 PR #1405 정정, 게이트는 회귀 봉인. 경로 `tbl`).

> **중요**: baseline 통과 = 시각 충실도 보장이 **아니다**. 컨트롤 내부 속성(표
> pageBreak 등)과 문단 텍스트 내용은 비교하지 않으므로 페이지 수 변화·텍스트 축
> 변위(autoNum #1382 등)도 baseline을 통과할 수 있다 (#1315 4단계·#1379 4단계·
> #1380 4단계 보고서 실증). 시각 판정 권위는 작업지시자(한컴에디터)에게 있다.

## 2. 등급 체계

| 등급 | 의미 | 코드 위치 |
|------|------|----------|
| **A (baseline)** | 위 4개 기준 전부 통과. 신규 샘플 자동 포함 | `tests/hwpx_roundtrip_baseline.rs` 기본 대상 |
| **B (xfail)** | 식별된 결함/미지원으로 baseline 제외. 사유 필수 | `XFAIL` 상수 |
| **제외** | 샘플 자체가 HWPX가 아님 (serializer 결함 아님) | `EXCLUDED` 상수 |
| **C (oracle 부적합)** | A이지만 full visual fidelity oracle 금지 (복합 실문서) | `ORACLE_UNFIT` 상수 |

현재 `XFAIL`은 0건이고 형식 불일치 제외는 `hwpx-01.hwpx` 1건, full visual fidelity oracle로
사용하지 않는 `ORACLE_UNFIT`은 13건이다. 전체 A등급 수는 샘플 추가에 따라 달라지므로 문서에
고정하지 않고 `tests/hwpx_roundtrip_baseline.rs`의 재귀 수집 결과를 권위로 삼는다.

- **B=0**: #1384 해소(borderFill 등록 1-based 정정)로 마지막 xfail 4건(`exam_kor`/
  `exam_social`/`exam_social-p1`/`issue_1133`)이 승격됨 (task_m100_1384_stage2.md).
  baseline xfail 0 — 제외 1건(비-HWPX) 외 전수 A등급.
- #1379 해소로 임시 xfail 목록(`XFAIL_1378_RECURSIVE` 13건 + `XFAIL_1379_CONTROLS`
  16건, 중복 4건)이 제거되고 25건이 baseline으로 승격됨 (task_m100_1379_stage3.md 측정)
- #1382 해소(autoNum 폭 축 일관화)로 `143E433F503322BD33.hwpx` 승격 (task_m100_1382_stage3.md)
- C(`ORACLE_UNFIT`)는 xfail과 별개 — A등급 승격 후에도 시각 oracle 부적합 표시는
  유지된다 (`exam_kor` 등 복합 실문서 — #1384 해소로 A등급이나 C 유지)

## 3. 통합 테스트 (`tests/hwpx_roundtrip_baseline.rs`)

```bash
cargo test --test hwpx_roundtrip_baseline    # debug 기준 약 1분
```

| 테스트 | 역할 |
|--------|------|
| `baseline_all_samples_roundtrip` | 전수 재귀 스캔 (XFAIL/EXCLUDED/LARGE 제외) — **신규 샘플 자동 포함** |
| `baseline_large_samples_roundtrip` | 대형 3건(`LARGE`) 분리 — 하네스 병렬 실행으로 wall time 단축 |
| `xfail_entries_still_fail` | xfail이 통과하게 되면 실패 → baseline 승격 강제 |
| `grade_lists_are_consistent` | EXCLUDED/ORACLE_UNFIT 실재 + ORACLE_UNFIT은 HWPX 샘플(EXCLUDED 금지) 가드 |

### 신규 샘플 추가 시

`samples/hwpx/`에 `.hwpx`를 추가하면 자동으로 baseline 게이트에 포함된다.

- 통과 → 끝 (A등급)
- 실패 → 결함을 수정하거나, **사유와 함께** `XFAIL`에 등록 (사유 없는 등록 금지)
- HWPX가 아닌 파일 → `EXCLUDED`에 등록
- 복합 실문서 → A등급이어도 `ORACLE_UNFIT`에 추가 검토 (시각 판정은 작업지시자)

### xfail 승격 절차

serializer 결함이 해소되면 `xfail_entries_still_fail`이 실패한다.
해당 항목을 `XFAIL`에서 제거하면 baseline으로 자동 승격된다.

## 4. 배치 CLI (`rhwp hwpx-roundtrip`)

```bash
rhwp hwpx-roundtrip sample.hwpx                          # 단일 파일 검사
rhwp hwpx-roundtrip --batch samples/hwpx                 # 폴더 전수 (재귀)
rhwp hwpx-roundtrip --batch samples/hwpx -o output/poc/task1315   # 산출물 지정
```

- 산출물: `{out}/inventory.tsv` (13컬럼) + `{out}/{stem}.rt.hwpx` (재조립 파일)
- 상태 우선순위: `PARSE_FAIL → SERIALIZE_FAIL → REPARSE_FAIL → IR_DIFF → PKG_FAIL → ROUND2_FAIL → ROUND2_DIFF → PASS`
- 하드 실패 존재 시 종료 코드 1 (CI 사용 가능)

## 5. 패키지 검사 (`check_package`)

`src/serializer/hwpx/package_check.rs` — 재조립 ZIP을 IR 기준으로 검사:

1. ZIP 아카이브로 열림
2. `mimetype` — 최초 엔트리 + STORED + `application/hwp+zip`
3. 필수 엔트리 9종 (version.xml, header.xml, content.hpf, Preview 2종, settings.xml, META-INF 3종)
4. `Contents/section{N}.xml` 수 = IR 섹션 수
5. `content.hpf` manifest href 전부 실재
6. `BinData/` 수·확장자 멀티셋 = IR `bin_data_content` (serializer가 href를 재명명하므로 이름 비교 금지)

## 6. Known limitations (후속 이슈)

| 한계 | 증상 | 이슈 |
|------|------|------|
| ~~문단 run 평탄화~~ | ~~다중 char_shapes 문단이 단일 run으로 출력~~ | **#1378 해소** — 본문·셀·글상자 다중 run 분할 출력 |
| ~~셀·글상자 subList 컨트롤 미출력~~ | ~~셀 내 그림/컨트롤 소실 + char_shapes 경계 8 유닛 배수 시프트~~ | **#1379 해소** — 셀·글상자 공유 경로 전환 + colPr 인라인 방출 |
| ~~lineseg 원본 보존 불완전~~ | ~~원본 무 → RT 합성 방출 (H1: 0→38·0→55), 페이지 수 변화 기여~~ | **#1380 해소** — 파서 주입 제거 + 방출 생략 + 9필드 게이트 동승 (전수 diff 0) |
| ~~표 pageBreak 속성 미보존~~ | ~~원본 CELL/NONE → RT 일괄 TABLE 방출 (form-002)~~ | **#1393 해소** — 방출은 PR #1405(ad55059f, 정확 역매핑)·게이트 동승은 본 타스크(TablePageBreak 회귀 봉인). 전수 보존 |
| ~~MEMO 필드 subList 미직렬화~~ | ~~fieldBegin parameters + 메모 본문 소실 (전 fieldBegin 13건 parameters 공통 소실 + MEMO 2건 본문)~~ | **#1391 해소** — parameters verbatim(raw_parameters_xml) + MEMO subList 방출 + FieldContent 게이트 동승 (RT 가능 parameters 7/7·MEMO 2/2 복원) |
| ~~shapeComment 미직렬화~~ | ~~`hp:shapeComment` 소실 (전수 229건/27파일 — pic·equation·container 결손, rect는 정상)~~ | **#1392 해소** — 4경로 방출(equation/container는 파서 적재도 신설) + ObjectComment 게이트 동승 (RT 가능 119/119 복원) |
| ~~borderFillIDRef 미등록 참조~~ | ~~SERIALIZE_FAIL 4건 (exam_kor/exam_social/exam_social-p1/issue_1133)~~ | **#1384 해소** (#1381 통합) — doc_info borderFill 등록 축 0-based→1-based 정정 (방출·참조와 통일). xfail 4건 승격 → B=0 |
| ~~numbering 등록 축 0/1-based 불일치~~ | ~~방출 1-based vs 등록 0-based (borderFill 동형, reference 없어 미표면화)~~ | **#1409 해소** — context.rs:128 등록 1-based 정정 + 회귀 가드 (borderFill·numbering 외 동형 결함 없음 확인) |
| ~~파서 autoNum 폭 비일관~~ | ~~char_shapes 축 1 유닛 vs offsets 축 8 유닛 + serializer 슬롯 placeholder 비인지 — 슬롯 끝 변위·경계 시프트 (autoNum 14문단 전수)~~ | **#1382 해소** — calc 8유닛 일관화 + placeholder 슬롯 방출, 143E baseline 승격 |
| ~~표 캡션(`hp:caption`) 미직렬화~~ | ~~캡션 subList 문단 소실 (전수 17건 중 표 6건)~~ | **#1387 해소** — write_caption + TableCaption 게이트 동승 (RT 가능 표 캡션 5/5 복원) |
| ~~그림/도형/묶음 캡션(`hp:caption`) 미직렬화~~ | ~~pic·container 캡션 subList 소실 (aift)~~ | **#1403 해소** (PR #1406, oksure) — write_caption 공유 + ObjectCaption 게이트 동승 + lineseg 재귀 후속 보완 |
| ~~캡션 내 autoNum 슬롯 변위~~ | ~~slot 추론 mismatch → 끝 방출 + 재파싱 끝 공백 1자 (ta-pic-001-r)~~ | **#1382 해소에 포함** — ctrl mid-text 원위치 방출 (한컴 원본 XML 동형) |
| ~~secPr 페이지 여백 템플릿 하드코딩~~ | ~~`hp:margin` left/right=8504 고정 방출 — 원본 여백 변형 (전수 51/74 섹션 영향)~~ | **#1388 해소** — margin 7필드 + gutterType 동적 치환 + PageDef 게이트 동승 (전수 변형 0) |
| ~~hp:pic 크기 요소 IR 미반영~~ | ~~curSz=sz·imgRect=common 합성·imgDim=0×0 — 한컴 셀 내 그림 크기 미반영~~ | **#1389 해소** — curSz→shape_attr, imgRect→border_x/y 스칼라 역매핑, imgDim verbatim 적재 + PictureSize 게이트 동승 (ta-pic 3요소 복원) |
| ~~newNum 슬롯 위치 변위~~ | ~~복합 컨트롤 문단(머리말+필드+newNum)에서 newNum 이 fieldEnd 갭 가로채 텍스트 앞 방출 — char_offsets +8 밀림 (143E 문단 0.14)~~ | **#1407 해소(①)** — post-char fieldEnd 방출에 expected+=8 (텍스트-끝 슬롯 가로채기 차단, ir-diff 0) |
| ~~본문 colPr 템플릿 하드코딩~~ | ~~본문 섹션 단 정의(colCount=2)가 템플릿 colCount=1 로 방출 — 2단→1단 손실로 페이지 넘침 (143E RT 1→2)~~ | **#1407 해소(②)** — write_section 에서 첫 문단 ColumnDef IR 을 colPr anchor 에 치환 (#1388 secPr 동형, RT 페이지 1→1) |

시각 검증 자료: `output/poc/task1315/svg/` (대표 8건 원본·rt SVG쌍),
`output/poc/task1378/svg/` (#1378 대표 8건),
`output/poc/task1379/svg/` (#1379 대표 2건 — tbox-v-flow-01 SVG 동일, ta-pic-001-r 캡션·여백 차이),
`output/poc/task1380/` (#1380 전수 rt + lineseg_diff.tsv — H1 2샘플 SVG md5 동일),
`output/poc/task1388/` (#1388 전수 rt + margin_inventory.tsv — 여백 변형 0),
`output/poc/task1387/` (#1387 전수 rt + caption_inventory.tsv + svg/ — 표 캡션 복원),
`output/poc/task1382/` (#1382 전수 rt + svg/ — autoNum 슬롯 원위치·캡션 행 시프트 해소),
`output/poc/task1392/` (#1392 전수 rt + shapecomment_dist.tsv — 객체 설명 119/119 복원),
`output/poc/task1391/` (#1391 전수 rt — fieldBegin parameters 7/7 + MEMO 본문 2/2 복원),
`output/poc/task1389/` (#1389 전수 rt — ta-pic curSz/imgRect/imgDim 3요소 복원),
`output/poc/task1393/` (#1393 전수 rt — 표 page_break 전수 보존),
`output/poc/task1407/` (#1407 143E rt + batch — newNum 슬롯 원위치·colPr 2단 복원·페이지 1→1),
rhwp-studio 로드 검증: `rhwp-studio/e2e/task1315-load.check.mjs` (호스트 Chrome CDP).

## 6.5 열거 토큰 표면 표기 정합 검사 (#1402)

`tests/issue_1402_enum_token_whitelist.rs` — serializer 가 방출하는 열거 속성 토큰이
**실물 관측(samples/hwpx) ∪ owpml/한컴 스펙** 화이트리스트에 속하는지 전수 검증한다.
#1387 numType "FIGURE"(한컴 비실재 → 캡션 번호 미출력) 재발을 봉인한다.

- `serializer_enum_tokens_within_whitelist`: 전수 parse→serialize 후 방출 토큰 ∈ 화이트리스트
- `numtype_figure_regression_guard`: numType="FIGURE" 방출 금지

**화이트리스트 갱신 절차**: 새 열거 속성/토큰을 방출하게 되면, 한컴 실물 또는
owpml 스펙으로 확인(추정 금지) 후 `whitelist()` 에 추가한다. circleType
`SHAPE_REVERSAL_TIRANGLE` 은 owpml 권위 자료 미접근으로 현행 철자 유지(파서·serializer
동일 → roundtrip 정합).

## 7. 관련 문서

- 수행/구현 계획: `mydocs/plans/task_m100_1315.md`, `task_m100_1315_impl.md`
- 단계별 보고서: `mydocs/working/task_m100_1315_stage{1..4}.md`
- 최종 보고서: `mydocs/report/task_m100_1315_report.md`
- run 분할 보존 (#1378): `mydocs/plans/task_m100_1378{,_impl}.md`,
  `mydocs/working/task_m100_1378_stage{1..3}.md`, `mydocs/report/task_m100_1378_report.md`
- subList 컨트롤 보존 (#1379): `mydocs/plans/task_m100_1379{,_impl}.md`,
  `mydocs/working/task_m100_1379_stage{1..3}.md`, `mydocs/report/task_m100_1379_report.md`
- lineseg 원본 보존 (#1380): `mydocs/plans/task_m100_1380{,_impl}.md`,
  `mydocs/working/task_m100_1380_stage{1..3}.md`, `mydocs/report/task_m100_1380_report.md`
- secPr 페이지 여백 보존 (#1388): `mydocs/plans/task_m100_1388{,_impl}.md`,
  `mydocs/working/task_m100_1388_stage{1..3}.md`, `mydocs/report/task_m100_1388_report.md`
- 표 캡션 직렬화 (#1387): `mydocs/plans/task_m100_1387{,_impl}.md`,
  `mydocs/working/task_m100_1387_stage{1..3}.md`, `mydocs/report/task_m100_1387_report.md`
- autoNum 폭 축 일관화 (#1382): `mydocs/plans/task_m100_1382{,_impl}.md`,
  `mydocs/working/task_m100_1382_stage{1..3}.md`, `mydocs/report/task_m100_1382_report.md`
- 객체 설명(shapeComment) 직렬화 (#1392): `mydocs/plans/task_m100_1392{,_impl}.md`,
  `mydocs/working/task_m100_1392_stage{1,2}.md`, `mydocs/report/task_m100_1392_report.md`
- MEMO 필드 parameters·subList 직렬화 (#1391): `mydocs/plans/task_m100_1391{,_impl}.md`,
  `mydocs/working/task_m100_1391_stage2.md`, `mydocs/report/task_m100_1391_report.md`
- borderFill 등록 축 정정 (#1384, #1381 통합): `mydocs/plans/task_m100_1384{,_impl}.md`,
  `mydocs/working/task_m100_1384_stage2.md`, `mydocs/report/task_m100_1384_report.md`
- 그림 크기 요소 IR 보존 (#1389): `mydocs/plans/task_m100_1389{,_impl}.md`,
  `mydocs/working/task_m100_1389_stage2.md`, `mydocs/report/task_m100_1389_report.md`
- 표 page_break 게이트 동승 (#1393): `mydocs/plans/task_m100_1393{,_impl}.md`,
  `mydocs/working/task_m100_1393_stage1.md`, `mydocs/report/task_m100_1393_report.md`
- 열거 토큰 표면 표기 정합 검사 (#1402): `mydocs/plans/task_m100_1402{,_impl}.md`,
  `mydocs/working/task_m100_1402_stage1.md`, `mydocs/report/task_m100_1402_report.md`
- numbering 등록 축 정정 (#1409): `mydocs/plans/task_m100_1409.md`,
  `mydocs/working/task_m100_1409_stage1.md`, `mydocs/report/task_m100_1409_report.md`
- 트러블슈팅: `mydocs/troubleshootings/hwpx_lineseg_reflow_trap.md` (2-round 검사의 배경)
