# HWPX Roundtrip Baseline 가이드 (Task #1315, #1378·#1379 게이트 강화)

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
**문단별 인라인 슬롯 컨트롤 타입 시퀀스** (#1379 — 셀·글상자·각주/미주 재귀 동승).

> **중요**: baseline 통과 = 시각 충실도 보장이 **아니다**. 컨트롤 내부 속성, lineseg
> 값, 표 캡션(`hp:caption`) subList, secPr 페이지 여백은 비교하지 않으므로 캡션
> 소실·여백 변형·페이지 수 변화도 baseline을 통과할 수 있다 (#1315 4단계·#1379
> 4단계 보고서 실증). 시각 판정 권위는 작업지시자(한컴에디터)에게 있다.

## 2. 등급 체계

| 등급 | 의미 | 코드 위치 |
|------|------|----------|
| **A (baseline)** | 위 4개 기준 전부 통과. 신규 샘플 자동 포함 | `tests/hwpx_roundtrip_baseline.rs` 기본 대상 |
| **B (xfail)** | 식별된 결함/미지원으로 baseline 제외. 사유 필수 | `XFAIL` 상수 |
| **제외** | 샘플 자체가 HWPX가 아님 (serializer 결함 아님) | `EXCLUDED` 상수 |
| **C (oracle 부적합)** | A이지만 full visual fidelity oracle 금지 (복합 실문서) | `ORACLE_UNFIT` 상수 |

현황 (2026-06-12, .hwpx 54건): A=48, B=5, 제외=1(`hwpx-01.hwpx`), C=13.

- B 5건 내역: 파서 autoNum 폭 비일관 1건(`143E433F503322BD33.hwpx`, #1382) +
  borderFillIDRef 미등록 4건(`exam_kor`/`exam_social`/`exam_social-p1`/`issue_1133`, #1384)
- #1379 해소로 임시 xfail 목록(`XFAIL_1378_RECURSIVE` 13건 + `XFAIL_1379_CONTROLS`
  16건, 중복 4건)이 제거되고 25건이 baseline으로 승격됨 (task_m100_1379_stage3.md 측정)
- C(`ORACLE_UNFIT`)는 xfail과 중복 가능 — xfail 해소 후 승격 시에도 시각 oracle
  부적합 표시는 유지된다 (예: `exam_kor` #1384 해소 후에도 C 유지)

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
| lineseg 원본 보존 불완전 | 페이지 수 변화 (form-002 10→17쪽, 보도자료 9→13쪽), 1~2px 시프트 | #1380 |
| borderFillIDRef 미등록 참조 | SERIALIZE_FAIL 4건 (exam_kor/exam_social/exam_social-p1/issue_1133) — xfail 등록됨 | #1384 (#1381 포함 계열) |
| 파서 autoNum 폭 비일관 | char_shapes 축 1 유닛 vs offsets 축 8 유닛 — 재파싱 경계 1 유닛 시프트 (143E433F503322BD33 각주) | #1382 |
| 표 캡션(`hp:caption`) 미직렬화 | 캡션 subList 문단 소실 (ta-pic-001-r "&lt;그림 1&gt;…" 18자) — `diff_documents` 미비교라 baseline 통과 | #1387 (#1379 4단계 발견) |
| secPr 페이지 여백 템플릿 하드코딩 | `hp:margin` left/right=8504 고정 방출 — 원본 여백 변형 (ta-pic-001-r 4252→8504) | #1388 (#1166은 landscape/width/height만 동적화) |
| hp:pic 크기 요소 IR 미반영 | curSz=sz 값·imgRect=common 합성·imgDim=0×0 방출 — 한컴에서 셀 내 그림 크기 미반영 (ta-pic-001-r 판정) | #1389 (#1379 한컴 판정 발견) |

시각 검증 자료: `output/poc/task1315/svg/` (대표 8건 원본·rt SVG쌍),
`output/poc/task1378/svg/` (#1378 대표 8건),
`output/poc/task1379/svg/` (#1379 대표 2건 — tbox-v-flow-01 SVG 동일, ta-pic-001-r 캡션·여백 차이),
rhwp-studio 로드 검증: `rhwp-studio/e2e/task1315-load.check.mjs` (호스트 Chrome CDP).

## 7. 관련 문서

- 수행/구현 계획: `mydocs/plans/task_m100_1315.md`, `task_m100_1315_impl.md`
- 단계별 보고서: `mydocs/working/task_m100_1315_stage{1..4}.md`
- 최종 보고서: `mydocs/report/task_m100_1315_report.md`
- run 분할 보존 (#1378): `mydocs/plans/task_m100_1378{,_impl}.md`,
  `mydocs/working/task_m100_1378_stage{1..3}.md`, `mydocs/report/task_m100_1378_report.md`
- subList 컨트롤 보존 (#1379): `mydocs/plans/task_m100_1379{,_impl}.md`,
  `mydocs/working/task_m100_1379_stage{1..3}.md`, `mydocs/report/task_m100_1379_report.md`
- 트러블슈팅: `mydocs/troubleshootings/hwpx_lineseg_reflow_trap.md` (2-round 검사의 배경)
