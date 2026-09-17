---
kind: investigation
status: active
---

# #7195 2단계 — 실패 3건의 계약·조판 경계 조사

- 계획: [수행계획서](../plans/task_m100_7195.md)
- 사용자 승인: 2026-09-16, 1단계 이후 정밀 조사 진행 승인.
- 제품 기준: `8d45f242baa1a565357aaa38e9f459595b1e756c`.
- 작업 HEAD: `ce47261159e67ab1cfa765f64550f52905402c42` — 위 기준에 1단계 테스트 2건 복귀만 추가.
- 상태: 3건 재현·첫 실패 이후 assertion 조사·원인 계층 분류 완료. **최종 시각 판정과 제품 수정은 미완료**.
- 이번 단계는 제품 코드, 기존 assertion, ignore, baseline, 입력 문서를 변경하지 않았다.

## 1. 결론과 다음 처리

| 대상 | 확인한 사실 | 판정 후보 / 다음 처리 |
| --- | --- | --- |
| #2279 | r27 근거설명은 rhwp p27, 참고 PDF p28. 그 앞 산식 표부터 이미 rhwp p26 / PDF p27로 갈린다. 해당 산식 4행은 측정과 paint 모두 4줄·85.7px다. | **실제 페이지 배치 차이 + 상류 원인 미확정**. 이 핀을 맞추려고 4줄을 5줄 높이로 되돌리지 않는다. 첫 차이 발생점과 표/빈 host 흐름을 추가 조사하고 메인테이너 판정을 받는다. |
| #2308 | 줄 구성 폭 509.933px와 최종 child Cell clip 폭 503.133px가 다르다. `를`는 p81 줄에 들어가지만 셀 오른쪽에서 잘리는 영역에 걸린다. | **측정/줄 구성과 paint 경계 불일치**. 단순 PDF 한 글자 핀이나 폰트 폭 잔차만의 문제가 아니다. 공통 content box 소유권을 정리하는 구현계획이 필요하다. |
| #3798 | 문단 잉크는 본문 안에 있다. 40px 말미 공백의 일부만 흐름 소비 높이상 본문 밖이며, 다음 문단은 다음 쪽에 간다. 테스트는 폐기한 상수 trim-cap 가설을 요구한다. | **실험적 테스트 계약 + 독립 근거 부족**. 의무적 조판 실패와 구분한다. 정상 실물 출력 또는 규칙 근거 없이 cap을 추가하거나 PASS로 바꾸지 않는다. |

담당은 #7195의 메인테이너/작업 에이전트다. #2279는 추가 조사, #2308은 시각 검토 후 구현계획,
#3798은 실험 보존 계약으로의 정리 여부를 결정한다. 어느 것도 이 문서만으로 새 ignore 해제나
이슈 해결 종료를 뜻하지 않는다. 기존 CLOSED 이슈도 재개하지 않았다.

## 2. 실행과 증적 범위

전용 `/home/edward/mygithub/rhwp-review-7195`에서 원래 테스트 helper를 그대로 호출하는
진단 테스트 3개를 임시 추가했다. 실패 assertion은 바꾸지 않고 모든 피연산자를 별도로 수집했다.
추가 진단에는 #2308의 동일 원문·스타일, 서로 다른 content box 폭의 인메모리 비교를 포함했다.
진단은 로그 수집 성공만 뜻하며, 출력된 `false`를 PASS로 승격하지 않는다.

```sh
CARGO_BUILD_JOBS=4 cargo build --locked --profile release-test \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review -p rhwp --bin rhwp

CARGO_BUILD_JOBS=4 cargo nextest run --locked --cargo-profile release-test \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review -p rhwp \
  --test regression_suite_015 --test regression_suite_019 --test regression_suite_026 \
  --run-ignored all --test-threads 1 --no-fail-fast \
  --success-output immediate --failure-output immediate \
  -E 'test(issue_2279_layout_oracles::) | test(issue_2308_render_normalized_derived_state::) | test(issue_3798_page_end_trailing_spill::)'
```

- CLI build 성공, 2분 52초.
- 최종 nextest ID: `90a9c8b8-1478-47bd-8248-ef173d77009a`, exit **100**.
- 기존 테스트 **9 PASS / 3 FAIL**, 추가 진단 **3개 실행 성공**. 합계 15개 중 12 PASS / 3 FAIL.
- 필터로 선택하지 않은 596개는 ignored 수가 아니다. 전체 회귀 실행이 아니다.
- nextest 0.9.137 / 권장 0.9.140 경고는 1단계와 같다.
- 추가 진단 작성 중 `i16` padding을 `u32::from`으로 변환한 컴파일 오류가 한 번 있었다.
  진단 코드의 checked conversion으로 수정 후 위 최종 실행을 완료했다. 제품 오류가 아니다.

로컬 산출 루트: `output/7195/stage2/`.

- `run.mjs`, `focused.result.json`, `focused.stderr`: 실제 인자·cwd·시간·결과.
- `diagnostics.patch`: 검증 worktree에만 적용한 진단 소스. 실행 후 그 추가분만 제거했다.
- `provenance.json`: 제품/검증 기준, CLI hash, 원본/PDF hash·커밋 bytes 대조.
- `analyze.py`, `analysis.json`: render tree의 행/줄/셀 경계 및 SVG clip 연결.
- `2279-metrics.stderr`, `2308-metrics.stderr`: 기존 `RHWP_DIAG_ROWH`,
  `RHWP_DIAG_ROWH_LINES`, `RHWP_DIAG_SPLITSCAN`을 켠 `dump-pages <원본> --json` 진단.
- `*-raw.stdout`, `*-pages.stdout`: IR 구조와 페이지별 항목/분할 cursor.
- `font-list.stdout`, `*-pdfinfo.stdout`, `*-pdffonts.stdout`: 현재 raster 글꼴 목록과 PDF 메타데이터.

집중 테스트 빌드 뒤 CLI hash가 달라진 것을 검출하여, 안정된 최종 바이너리로 `final/`에 전부
다시 내보냈다. 최초/최종 바이너리 hash는 각각의 `provenance.json`에 남겼다. 전체 SVG와 render tree
**296개가 최초 산출과 byte 단위로 동일**함을 확인했다. `closure.json`에는 이 재확인과 임시 진단
제거 후 3개 테스트가 커밋 원본과 동일하다는 검사를 기록했다. 아래 최초 증적 경로의 좌표/내용을
최종 바이너리에도 적용할 수 있는 근거는 이 재출력 비교다.

제품/샘플 변경, WASM 재빌드, Clippy 3종, 전체 회귀, push/PR/댓글은 수행하지 않았다.
단계 조사 결과를 PR 준비 완료나 한컴 일치 판정으로 사용하지 않는다.

## 3. 기준 자료·비교 환경

`bug-hunter`의 기준 자료 확인 절차를 적용하고, 프로젝트 `fidelity_compare`로 직접 대조했다.
실물 2건은 SVG/render tree 전체를 생성하되 PNG 시각 비교는 실패 지점 p27–29, p81–82에 한정했다.
PDF가 없는 합성 #3798은 source/IR/배치 내부 계약 조사로 제한했다.

| 자료 | 확인한 provenance | 한계 |
| --- | --- | --- |
| `samples/issue1891/86712_regulatory_analysis-2024.pdf` | 65쪽, Creator `Hwp 2024 13.0.0.3622`, Producer `Hancom PDF 1.3.0.550`, 2026-07-05 생성 | 원 테스트가 인용한 자료. export UI 경로·당시 설치 폰트 목록 미기록. |
| `samples/issue1891/76076_regulatory_analysis-2024.pdf` | 82쪽, 위와 같은 Creator/Producer, 2026-07-05 생성 | 위와 동일. |
| `pdf/86712_regulatory_analysis-hwp-2024.pdf` | 65쪽, Creator `Hwp 2024 0.0.0.0`, Producer `Hancom PDF 1.3.0.550`, 2026-08-29 생성 | 형식 구분 canonical 이름의 후속 PDF도 추가 대조. 출력 경로·폰트 목록은 여전히 미확인. |

두 최초 PDF는 원본 HWP·HWP5-in-.hwpx와 함께 `1a9131b47e`에 추가됐다. 당시 기록에도 두 입력의
rhwp 쪽수가 다르므로, 파일명이 비슷한 `.hwpx`를 같은 bytes의 원본이라고 가정하지 않았다.
이번 실행 입력은 테스트가 읽는 **`samples/*.hwp`**이며, 현재 커밋과 bytes가 동일하다.

`pdf/issue3820/76076-regulatory-analysis-hwp2020-20260814.pdf`도 있지만 Producer가 Cairo이고,
이 단계에서는 출력 경로를 확인하지 못했다. 이름만 보고 추가 한컴 정답으로 쓰지 않았다.

기존 테스트/CLI 기본은 `compat 2022`다. 같은 선택 쪽 5개의 `--compat 2024 --font-style` SVG도
별도로 생성했으며, **2022 기본 SVG와 byte 단위로 동일**했다. 이번 지점의 차이를 모드 불일치만으로
설명할 수 없다. 전체 문서 모든 쪽의 compat 동등성까지 검사한 것은 아니다.

Chrome은 로컬 puppeteer cache의 `linux-152.0.7977.54`를 명시했다. 최초 PATH 자동 탐색은 실패하여
`CHROME_BIN`을 지정한 뒤 비교 5쪽을 모두 완료했다. 로컬 글꼴 fallback의 외형 차이는 남을 수 있다.
PDF 경고 `Illegal annotation destination`도 stderr에 보존했다. 페이지 수·추출·diff%는 후보 지표다.
**PNG 제목의 '한컴 기준 PDF'는 도구 기본 표기이며 위 provenance 한계를 없애지 않는다.**

## 4. #2279 — 뒤쪽 핀보다 앞에서 시작한 배치 차이

입력: `samples/86712_regulatory_analysis.hwp`.

기존 6개 assertion을 순서대로 독립 실행한 결과:

| 기대 | 결과 |
| --- | --- |
| p27에 `편익 수혜자` 없음 | FAIL |
| p28에 `편익 수혜자` 있음 | FAIL |
| p28에 `88.2` 없음 | FAIL |
| p29에 산식 `2891017` 없음 | PASS |
| p29에 `88.2` 있음 | FAIL |
| p29의 5×4 표에 `주민대표단 구성` 있음 | FAIL |

현재는 총 **64쪽**, 두 한컴2024 참고 PDF는 **65쪽**이다. pi172 28×4 표의 실제 분할은 다음과 같다.

- rhwp p26: rows `0..27`, 시작 y=120.0px. 가용 예산 921.2px, 분할 scan 소비 887.1px.
- rhwp p27: row27 앞부분, `endCut=[1,55]`, 소비 867.5px. 근거설명이 처음부터 여기에 있다.
- rhwp p28: row27의 나머지, `startCut=[1,55]`, 소비 500.7px. 3×12와 5×4 표가 여기에 있다.
- 참고 PDF p27: 앞 비용 표의 꼬리와 `□편익` 뒤 산식 표, p28 근거설명, p29 내부 표들.

따라서 첫 실패 메시지의 '남은 공간에 첫 유닛만 조기 paint'를 현재 현상 전체로 해석하면 안 된다.
**현재는 p26에 산식 표가 먼저 끝나고 p27을 근거설명이 차지한다.** p27만 강제로 비우는 수정으로
상류 차이를 해결할 수 없다.

산식 r23–26/c2의 current 측정은 각각 `content=79.7 + padding=5.9 = required 85.7px`이고,
render tree도 각각 4줄·셀 높이85.7px다. 줄은 `17.3px` 높이와 약 `20.8px` pitch를 사용한다.
참고 PDF의 같은 산식은 4줄이며 마지막 행만 p27–28에 걸친다. 과거의 5줄·106.5px 측정으로
되돌리는 것은 이 독립 관측에 맞지 않는다.

역사: `989d01fef3`의 Frame 이관에서 기존 5줄 측정이 틀렸고 그 보상 오차가 페이지 핀을 지탱했다고
기록하며 ignore했다. 이 과거 설명과 **현재 p26부터 달라진 배치**를 구분한다. 저장 LineSeg가 없는
표 내용과 상류 pi161 분할/뒤 빈 host 흐름 등 어디서 첫 격차가 생기는지는 아직 확정하지 않았다.
열린 #7184의 중첩 행 높이/예산 변경과 범위가 교차하지만 해당 PR은 가져오지 않았다.

시각 자료:

- `output/7195/stage2/2279/cmp-p026.png` — PDF/rhwp 물리27쪽.
- 같은 폴더 `cmp-p027.png`, `cmp-p028.png` — 물리28·29쪽.
- `output/7195/stage2/2279-canonical/` — 후속 canonical PDF로 같은 세 쪽 추가 대조.
- `output/7195/stage2/2279/debug/` — 디버깅 SVG.

**제안:** 이 단계에서 페이지 핀을 1쪽 앞당기거나 예외 등록하지 않는다. 동일 내용 대응으로 앞쪽
첫 배치 차이를 좁힌 뒤, 유효한 분할 차이인지 규칙 위반인지 메인테이너 시각 트리아지한다.

## 5. #2308 — content box와 최종 clip의 불일치

입력: `samples/76076_regulatory_analysis.hwp`, s0:pi842/ci0, 마지막 r4/c1의 1×1 child.

- 원문 첫 문단은 LineSeg가 **0개**다. 저장된 줄 소속을 옮긴 문제가 아니라 Frame 재조판 결과다.
- child 원본 폭 36,572HU, parent 셀 폭 38,245HU, parent/child 좌우 padding 각각510HU.
- `RenderNormalizationOverlay::collect_nested_tables`가 short RowBreak child를 parent 폭으로
  투영한다. `use_owner_content_box` 경로가 줄 구성 폭을38,245HU(509.933px)로 사용한다.
- 최종 child Cell은 x207.440px, 폭503.133px, 오른쪽710.573px로 clip한다.
  TextLine은 같은 x에서 폭509.933px(오른쪽717.373px)다. **510HU=6.8px 불일치**다.
- `extend_clipped_cell_horizontal_clip_to_nested_table_borders`의 후처리는 nested Cell bbox를
  parent grid 오른쪽으로 제한하지만 그 자식 TextLine을 다시 조판하지 않는다.
  해당 제한 코드의 이력은 `e7b799ae1b`(2026-08-10, jangster77)다. 이것만을 단독 원인 커밋으로
  확정한 것은 아니며 폭 투영/Frame 이관과의 상호작용이다.

실제 SVG에서 마지막 `를`의 x=700.059px, baseline y=1025.113px, 글자 크기17.333px다.
상위 `cell-clip-211`의 오른쪽이710.573px여서 해당 글자 영역이 잘린다. p82 여러 줄도 같은
오른쪽 경계 문제를 보인다. `cmp-p080.png`, `cmp-p081.png`와 SVG clip 연결을 함께 확인했다.
단순 자연 폭 후보만으로 확정한 것이 아니라 실제 출력과 최종 clip을 대조한 결과다.

원문과 글자모양을 고정한 재조판 진단:

| 제공 content box | 첫 줄 끝 | 둘째 줄 시작 |
| --- | --- | --- |
| 38,245HU (현재 owner 폭) | `… 등의 사고를` | `예방함으로써 …` |
| 37,735HU (현재 최종 clip에 해당) | `… 등의 사고` | `를 예방함으로써 …` |
| 37,225HU (parent 좌우 padding 차감) | `… 등의 사고` | `를 예방함으로써 …` |

이 실험은 **좁은 폭을 정답으로 하드코딩하자는 제안이 아니다.** paint 가능한 공간과 다른 폭을
제공하면 줄 소속이 달라진다는 원인 분리다. 두 좁은 폭이 모두 같은 줄바꿈을 내므로 PDF의 한 줄
핀만으로 올바른 padding 규칙을 역산할 수도 없다.

현재 파싱된 글꼴은 한양중고딕 계열, 모두13pt다. 구간별 자간은0%, -3%, -7%이며13pt에서
-3%는 -0.52px다. 과거 주석의 '맑은 고딕 15글자'를 이번 실측으로 재사용하지 않았다.
이 진단은 원문 `CharShapeRef`를 보존하며, 폰트/자간을 부풀려 핀을 맞추지 않는다.

기존 활성 검사는 p81의 `사고` 포함과 p82의 `고를…` 비포함을 통과한다. ignored 검사는
p82의 `를 예방…`을 찾지 못한다. 단일 TextRun뿐 아니라 TextLine의 run 전체를 이어 검색해도
없으므로 **문자 스타일 경계로 문자열이 나뉜 데 따른 거짓 실패는 아니다**.

**제안:** 최종 시각 확인 후, 측정·줄 구성·분할·paint가 동일한 유효 content box를 받는 방향의
구현계획을 작성한다. 단순 clip 해제/확장이나 `를` 앞 개행은 하지 않는다. 기존 p34/p70 중첩 표,
여백·들여쓰기·문자모양 혼합, source/derived-state 분리 검사를 반례로 유지해야 한다.

## 6. #3798 — 실험 기대값과 제품 결함의 구분

입력: `samples/issue3798/page_end_trailing_spill.hwpx`.
생성기는 `tools/make_issue3798_fixture.py`, 반증 기록은 [실험 보고서](../report/task3798/README.md)다.
이번에 생성기를 실행하거나 입력 파일을 덮어쓰지 않았다.

현재 `RHWP_DIAG_ADV`와 `dump-pages`가 실제로 확인한 값:

- 본문 높이933.573px, 앞28문단 소비896.0px.
- pi28: 줄16.0px + 말미 간격40.0px = 총56.0px.
- `paragraph_page_end_fit_height`는 이 일반 문단에16.0px를 사용한다.
  `896+16=912 <=933.573`이므로 현재 쪽에 놓는다.
- 소비 높이는952.0px이지만 다음 문단pi29는 다음 쪽에 배치된다. 문서 총2쪽.
- pi28 TextLine은 용지 좌표 y990.5..1006.5px, 본문 하단1028.04px보다 위다.
  **본문 밖 글자 출력/다음 문단 겹침을 이 사례에서 검출한 것은 아니다.**
- 4개 assertion 결과는 `[true, true, false, false]`: 2쪽 이상/앞채움28 있음은 PASS,
  경계 문단 p1 없음/p2 있음은 FAIL.

`75937d2db2`는 cap 보정이 순이득 음수여서 채택하지 않았다는 보고서·합성 샘플·ignored 테스트를
보존한 커밋이다. #3798 종료 댓글도 수복 완료가 아니라 실험 기록의 병합 보존을 설명한다.
그 테스트의 '다음 쪽으로 이동해야 함'은 한컴 실물 출력에서 도출한 확정 규칙이 아니다.

독립 비교 기준을 확보하지 못했다. 수행은 원본/IR·fit/advance·SVG/tree 내부 조사다.
한계: 자기 내부 계약만 확인했으며 한컴 공식 출력·법정 서식·제출 요건과 대조하지 않았다.
충실도 결함이나 올바른 한컴 배치로 판정하지 않는다.

**제안:** 폐기 실험으로 명시하여 보관할지, 정상 입력·독립 근거를 새로 확보해 의무 계약으로
재정의할지 메인테이너가 결정한다. 결정 전에는 ignore와 기대값을 유지한다.
