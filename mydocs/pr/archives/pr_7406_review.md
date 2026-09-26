---
kind: snapshot
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-26
---

# PR #7406 리뷰 — KoPub 라틴 전진폭

## 최종 판정

**메인터너 보정 후 수용 가능.** 원 contributor head는 직접 병합하지 않는다. 보정한 code head `9e7518243b251193ee91378e2d09e754d0223d64`에서 전체 nextest **10,260/10,260 PASS**, Native Skia lib **4,112 PASS**와 집중 **6/6 PASS**, fmt·세 Clippy·workspace build·base 비교 정책 검사·fresh Mac WASM을 완료했다. Native **140/140쪽**, fresh WASM 영향 **43/43쪽**과 분리 OLE HWP/HWPX/WASM 비교도 완료했다. 76쪽 표 하단·본문 하강과 33쪽 정상 저장 여섯 줄 변경은 각각 `90cfa99be`·`9e7518243`로 단계별 보정했다. 물리 1쪽에만 해시 고정 글꼴 예외를 적용한다. 31·66쪽 및 OLE 범례의 남은 차이는 아래에 공개하며, 문서 전체 일치나 #7390 해결을 주장하지 않는다. 통합 PR 최신 head의 CI·mergeability 확인이 병합 전 조건이다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| 원 PR·작성자·base | [#7406](https://github.com/edwardkim/rhwp/pull/7406), `planet6897`, `devel` |
| 고유 기여 commit / `-x` cherry-pick | `f49ebdefb9254d153e20a8cfd1424be398290eb4` / `c611aa5aff6e72ac02df722e07d4bda0d2b8b6e1` |
| 검토 base / branch | `c80a8370ab294259557c850c2e54495ecd0e79c0` / `review/planet6897-7406-20260925` |
| 관련 이슈 | [#7390](https://github.com/edwardkim/rhwp/issues/7390); 이 검토로 닫지 않음 |
| 원격 참고값 | 2026-09-26 재조회: 원 PR OPEN, Draft 아님, `MERGEABLE/CLEAN`, 원 head `298bd802ac3fc843e1d70ca6f3dbe95134b361c5`. 이 원 head를 직접 병합하지 않음 |
| 메인터너 보정 | KoPub 줄 구성·각주·표 분할·그림 뒤 흐름과 겹침 상한 15→1 보정. OLE 차트의 OOXML 값축·백분율 레이블·범주·미리보기 색상표 보정. 각 원인·회귀·시각 결과는 아래에 기록 |
| 최종 code candidate | `9e7518243b251193ee91378e2d09e754d0223d64` — 92쪽 저장 빈 줄·76쪽 완결 표·33쪽 정상 저장 어절 경계 보정 |

## 변경과 조판 원칙 검토

원 PR은 KoPub Dotum/Batang의 Basic Latin hmtx 폭을 `text_measurement.rs`에서 사용하고, 집중 검사와 p94 Native 이미지를 추가했다. 변경 폭은 줄 나눔과 표 안 텍스트 및 선행 본문 위치에 전파된다. `tests/fixtures/text_overlap_baseline.tsv`의 PrEP 상한도 14→15로 올렸다.

최초 검토에서 section 2 para 27의 비-TAC TopAndBottom RowBreak 표가 선행 본문·캡션 위로 올라오는 **미충족**을 확인했다. 메인터너 보정에서는 행 컷·예약 높이·페인트 점유를 함께 수정했고, 67~70쪽 국소 게이트와 정식 회귀가 통과했다. 각 보정의 전후 좌표와 정식 회귀는 아래에 기록했다. 실행 로그는 Git 제외 `output/pr-review/planet6897-7406-20260925/`에만 남기고, 병합 뒤 정리한다. 최종 head·명령·결과·시각 provenance는 [검증 요약](../assets/pr7406_20260925/validation_results.json)과 이 review에 보존한다.

OLE 차트는 HWPX의 `Chart/chart*.xml`과 HWP의 중첩 `OOXMLChartContents`가 편집 가능한 값·레이블을 담고, 중첩 EMF 미리보기가 문서 전용 색을 담는다. #7406은 OLE 안의 OOXML `chartSpace`이므로 [일반 OOXML 차트 경로](../../tech/chart_ole_v1_boundary.md)로 그린다. `colorIndex=-1`인 누적 막대에서 색만 미리보기로 복원하고 축·막대·레이블은 OOXML 데이터로 렌더한다. `samples/issue7406`의 값을 바꿔도 낡은 미리보기 막대를 재사용하지 않는 집중 검사와 한컴 PDF 비교를 수행했다. HWPX 구조 근거는 `mydocs/tech/hwp_ole_spec.md` 및 사용자가 제공한 `hwpx_complete_guide.pdf`의 4.2절이다.

최종 lint에서 OLE 색상표 source 파일의 `#[cfg(test)]` 모듈 위치 오류와 PR base 대비 source 단위 테스트 5개 증가가 확인됐다. 테스트 모듈을 파일 끝으로 정리하고, 축·범주·레이블·색상표 검사를 `tests/cases/issue_7406_ole_chart.rs`의 공개 API 회귀로 옮겼다. source 단위 테스트 정책은 기존 4205개 기준으로 통과하고, 옮긴 integration 테스트 **5/5 PASS**다. 제품 함수의 구현은 이 정리에서 바뀌지 않았다.

OLE 분리 샘플의 최종 Native 재캡처는 HWP **90.66968%**, HWPX **90.58460%**로 gate를 통과했다. 범례 색·기준선·글자 스타일 차이는 남아 있다. 기준 PDF의 첫 범례 `성생활 함`은 보라색 키지만 대응 막대 하단은 녹색으로 보인다. 편집 가능한 `Chart/chart1.xml`은 이 계열에 `accent5`를 지정하고 별도 `legendEntry` 색 재정의를 두지 않는다. 현재 일반 차트 출력은 값·막대와 같은 계열의 복원 색으로 범례를 그린다. 따라서 PDF의 범례 색을 그대로 복제한 출력이나 전체 픽셀 일치를 주장하지 않는다. 레이블·백분율·범주·막대 길이·캡션 위치와 계열 대응을 별도로 판독했으며, 기준 PDF와 OOXML 기반 출력의 차이는 이 보정의 남은 출력 차이로 기록한다.

## 검증 입력과 결과

| 항목 | 결과 |
| --- | --- |
| 원 HWPX | `samples/issue2006/1790387_prep_final_report.hwpx`, SHA-256 `c68baed24096386f9041930d24d39409b61ac99463bf04dfd242440dfdeb739f`, `hancom-office-2024` 저장 메타데이터, 140쪽 |
| 신규 한컴 PDF | [1790387_prep_final_report-2024.pdf](../../../pdf/issue2006/1790387_prep_final_report-2024.pdf), 원본을 HWP MCP `--engine 2024`로 변환, SHA-256 `04b95a6e41420fb45934ce2ee5abd8cf6dac4ce12fd47977dacbe7fca28018a8`, 140쪽, Hancom PDF 1.3.0.550 |
| KoPub 집중 검사 | 검토 source head `regression_suite_016` 2/2 PASS |
| 겹침 baseline | base 14건, 원 PR 단독 15건, 메인터너 보정 후 1건. 상한을 15→1로 축소했고 최종 head 전체 nextest에서 PASS |
| 최종 Native Visual Sweep | code `9e7518243`: **140/140쪽 완료**, PDF·rhwp 140쪽, flagged=0/140, exit 0. 1쪽 **88.10703%**만 해시 고정 글꼴 예외. 2·4·10·138쪽은 양쪽 잉크 0·차이 픽셀 0인 동일 빈 페이지로 실루엣 분모가 없고, 글꼴 예외 대상이 아니다. 나머지 최저 **91.85231%**(27쪽), 33·76·90·92쪽 **97.75050/99.92066/100/96.95925%**. 점수와 별도로 줄 경계·표 외곽·캡션·뒤 본문을 직접 판독했다 |
| OLE 분리 입력 | HWP `a29d415a…`, HWPX `a64ee3e5…`, [한컴 PDF](../../../pdf/issue7406/7406_OLE__CHART.pdf) `bd3a5a0e…`. 한 페이지 Native HWP **90.67%**, HWPX **90.58%**, 두 gate 통과 |
| 최종 Rust 검증 | code `9e7518243`: 전체 nextest **10,260/10,260 PASS·50 skip**, 738.215초, exit 0. fmt, Native/WASM32/workspace all-target Clippy, workspace build, manifest·unit-tier base `c80a8370ab294259557c850c2e54495ecd0e79c0` 비교 PASS. Native Skia root lib **3,930 PASS·13 skip**와 나머지 workspace lib **182 PASS**를 합해 **4,112 PASS**; 그림 누락·직접 PDF 출력 집중 **6/6 PASS** |
| 최종 fresh WASM | root wrapper `--no-opt` PASS(로컬 대체, Docker 최적화 빌드 아님). pkg/Studio JS·WASM SHA 일치, WASM SHA-256 `3df341e2575f4da056cfe335591fb13466ad299079fac26fb2f3ca1c3479e73a`. 영향 **43/43쪽 완료**, 1쪽만 글꼴 예외 **87.66357%**, 다른 측정 쪽 모두 90% 이상, exit 0. 분리 OLE 한 쪽 **90.58460%**, gate passed·exit 0. Studio 브라우저 UI 별도 검증은 주장하지 않음 |
| 이전 code head 참고 기록 | code head `e049c490d`: 전체 nextest **10,258/10,258 PASS·50 skip**, 629.122초. fmt, Native/WASM32/workspace Clippy, workspace build, manifest·unit-tier base `c80a8370ab294259557c850c2e54495ecd0e79c0` 비교 PASS. Native Skia lib **4,112 PASS·13 skip**, 그림 누락·직접 PDF 출력 집중 **6/6 PASS**. fresh Mac WASM wrapper `--no-opt` PASS(로컬 대체, Docker 최적화 빌드 아님), pkg/Studio JS·WASM SHA 일치. WASM 영향 **38/38쪽 완료**, 1쪽만 글꼴 예외 **87.66%**, 나머지 90% 이상. OLE 한 쪽 WASM **90.58460%**, gate passed. 이 결과를 이후 코드 보정 head에 재사용하지 않는다 |

기존 `...-hwp2020-20260814.pdf`의 절차적 생성 출처는 [#7399 검토](pr_7399_review.md)에서 미검증이었다. 이번에는 원본에 대응하는 새 한컴 2024 PDF를 생성해 판정에 썼다. PDF의 KoPubDotum subset과 Mac 글꼴 공급을 확인했고 94·108쪽은 99% 이상이다. 68~70쪽의 표·본문 배치 차이를 글꼴 예외로 분류하지 않는다.

## 최종 검증 재현 및 영구 증적

모든 검증은 code head `9e7518243b251193ee91378e2d09e754d0223d64`, base `c80a8370ab294259557c850c2e54495ecd0e79c0`, Mac 로컬, 공유 `target/pr-review`에서 실행했다. 로그는 커밋하지 않았다.

```sh
cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 8 --no-fail-fast
cargo nextest run --locked -p rhwp --cargo-profile release-test --target-dir target/pr-review --features native-skia --lib --test-threads 8 --no-fail-fast
cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --features native-skia --lib --test-threads 8 --no-fail-fast -E 'package(=rhwp-contracts) | package(=rhwp-ooxml-chart) | package(=rhwp-password-crypto)'
CARGO_TARGET_DIR=target/pr-review scripts/wasm-pack-locked.sh --target web --out-dir pkg --no-opt
python3 scripts/visual_sweep.py --key prep7406 --hwp samples/issue2006/1790387_prep_final_report.hwpx --pdf pdf/issue2006/1790387_prep_final_report-2024.pdf --rhwp-bin target/pr-review/debug/rhwp --pages 1-140 --font-mismatch-evidence mydocs/pr/assets/pr7406_20260925/font_mismatch_001.md --out output/pr-review/planet6897-7406-20260925/visual/full-native-9e7518243
python3 scripts/visual_sweep.py --key prep7406 --hwp samples/issue2006/1790387_prep_final_report.hwpx --pdf pdf/issue2006/1790387_prep_final_report-2024.pdf --rhwp-bin target/pr-review/debug/rhwp --wasm-pkg pkg --pages 1,13-15,26-27,32-35,39-41,50-52,56-58,60-62,67-70,76,78-83,89-94,104-106,108 --font-mismatch-evidence mydocs/pr/assets/pr7406_20260925/font_mismatch_001.md --out output/pr-review/planet6897-7406-20260925/visual/final-wasm-9e7518243
```

- [Native 완료 summary](../assets/pr7406_20260925/native_summary.json), [Native source/input/font manifest](../assets/pr7406_20260925/native_run_manifest.json), [Native 쪽별 수치](../assets/pr7406_20260925/native_overlay_metrics.json)
- [fresh WASM 완료 summary](../assets/pr7406_20260925/wasm_summary.json), [fresh WASM source/input/font manifest](../assets/pr7406_20260925/wasm_run_manifest.json), [fresh WASM 쪽별 수치](../assets/pr7406_20260925/wasm_overlay_metrics.json)
- 140쪽 전체 review PNG를 재표본화·손실 압축 없이 PDF에 내장했다. 네 PDF 모두 35쪽·50MB 미만이며, pypdf/Pillow로 **140개 내장 이미지의 픽셀이 원 PNG와 동일**함을 확인했다. PDF 재렌더 33·76·90·140쪽의 잘림·배열도 직접 확인했다. [PDF/PNG SHA-256 색인](../assets/pr7406_20260925/native_review_pdf_index.json)
- 전체 비교 PDF: [물리 1–35쪽](../../../pdf/issue7406/pr7406-native-review-001-035-9e7518243.pdf), [36–70쪽](../../../pdf/issue7406/pr7406-native-review-036-070-9e7518243.pdf), [71–105쪽](../../../pdf/issue7406/pr7406-native-review-071-105-9e7518243.pdf), [106–140쪽](../../../pdf/issue7406/pr7406-native-review-106-140-9e7518243.pdf). 한 PDF 쪽에 rhwp·한컴·overlay가 함께 있고 물리 쪽 bookmark를 제공한다.

### 공통 결과의 실제 소비와 적용 경계

| 보정 | 생산 → 요구/예약 → 실제 배치 | 독립 기준과 경계 검사 |
| --- | --- | --- |
| 34→35쪽 저장 1×1 첫 프레임 | `table_layout.rs:13854/13925`의 저장 프레임 높이/소유 컷 → `scan/runner/row_step.rs:515/639/955`의 fit·painted 예약·이월 → `table_partial.rs:4473`이 같은 시작/끝 컷의 프레임 높이로 실제 행을 그린다. 다른 조각은 기존 row-cut 높이를 유지 | 저장 첫 프레임·뒤 문단 vpos=0과 한컴 마지막 줄/35쪽 첫 줄을 독립 기준으로 정식 `prep_intra_paragraph_cell_split_keeps_last_line_on_page_34`가 텍스트 소유·경계·본문 하한을 검사한다. 39→40·80168·RowBreak HWP/HWPX 대조군도 최종 전체 PASS |
| 76쪽 완결 TAC 프레임 | `height_measurer.rs:767`의 저장 끝+패딩=외곽과 마지막 간격 중복 판정 → `typeset/table.rs:174/190`의 공통 fitted `MeasuredTable` → `table_layout.rs:4142/4166/4451`의 공통 fit 소비와 실제 행/뒤 본문 원점 | 원본 37100HU·한컴 150.976/659.424px와 수정 전 FAIL/후 PASS를 연결했다. 되감김/내용 성장/편집/불완전 프레임은 이 규칙 비적용. 페이지 분할은 이 보정 비해당 |
| 33쪽 정상 저장 줄 보존 | `composer.rs`의 마지막 어절 조각 수선과 정상 저장 경계 구분 → 같은 composed lines를 측정·paint에서 소비 | 원본 여섯 LineSeg와 한컴 여섯 줄·마지막 y=574.464px. 정식 `prep_page_33_keeps_complete_final_word_on_saved_sixth_line` 수정 전 FAIL/후 PASS; 67쪽 고아 음절 수선은 유지 |

전체 호출 경로의 추가 반례는 원본 정식 17개 PrEP 회귀와 전체 nextest에서 검증했다. 명시한 실제 쪽/좌표 검사를 넘어 모든 편집 조합 또는 모든 한컴 출력의 일치를 주장하지 않는다.

## 시각 증적과 남은 차이

물리 69쪽의 표 보정과 94쪽 KoPub 대조군은 기존 이미지에 남겼다. OLE 분리 샘플의 백분율 축·막대 길이·색·레이블을 직접 확인했다. 원본 27쪽은 도형 줄 뒤에 저장된 720HU 줄간격을 복원해 다음 본문·그림 위치를 기준 PDF에 맞췄다. 39→40쪽 1×1 셀 분할은 원본 위·아래 안 여백 각 850HU가 작은 셀 높이 282HU에 의해 축소된 문제를 고쳤다. 측정/컷 높이 차이 20.8px을 해소하고 저장 경계의 마지막 줄을 39쪽에 보존했다. 빈 문단의 시작 좌표가 표 조각 끝과 일치하면 문단 앞 간격도 한 번만 소비한다. 50→51쪽은 표 앞 한 줄 캡션의 저장 LineSeg가 지정한 표 시작 49275HU(본문 상대 657.0px)보다 분할 스캔의 흐름 시작이 26.7px 늦었던 문제다. 같은 저장 좌표를 첫 조각 예산에 적용해 한컴 PDF의 `치과의사`·`약사` 행을 50쪽에 남기고, 51쪽은 `간호사` 행부터 재개한다. 정식 회귀는 수정 전 FAIL/후 PASS, PrEP 관련 7건 PASS이며 새 이미지에서 표 외곽·뒤 문단을 직접 대조했다. 앞선 head의 전체 140쪽 review·overlay는 Git 제외 `output/pr-review/planet6897-7406-20260925/visual/full-native-final/prep7406/`, 국소 재캡처는 `visual/p40-padding-impact/prep7406/`와 `visual/p51-caption-anchor/prep7406/`에 있다. 대표 이미지:

57쪽은 가운데 정렬 TAC 표의 host 문단 왼쪽 여백 1000HU를 대체 배치 경로가 정렬 폭에 반영하지 않아 표 괘선이 한컴 PDF x=206px보다 6px 왼쪽이었다. 문단의 좌우 여백을 적용한 가용 줄 폭에서 표를 가운데 정렬했다. 정식 회귀는 수정 전 x=200.19px FAIL/후 PASS, PrEP 관련 8건 PASS. 56~58쪽 국소 sweep은 통과했고 표의 좌우 괘선과 57쪽 뒤 내용을 직접 확인했다. 재캡처는 Git 제외 `output/pr-review/planet6897-7406-20260925/visual/p57-centered-margin/prep7406/`에 있다.

61쪽의 분포도는 OLE 차트가 아니라 원본 `BinData/image16.tiff`(676×539) 그림이다. Para 기준 그림의 paint 상자만 host 왼쪽 여백 1000HU를 빠뜨려 PDF보다 13.3px 왼쪽에 놓였고, 그 여백은 typeset 쪽 공통 배치 상자에는 이미 포함돼 있었다. 두 경로의 상자를 맞췄다. 그림 설명의 끝과 다음 저장 LineSeg 시작이 같은 43579HU인데 문단 앞 간격 500HU를 다시 더하던 것도 그 경계에서만 제거했다. 그림 원점·뒤 두 문단의 정식 회귀는 수정 전 FAIL/후 PASS, PrEP 관련 9건 PASS. 60~62쪽 국소 sweep은 **100/99.95/99.55%**로 통과했고 그래프 본체·캡션·뒤 본문을 직접 판독했다. 재캡처는 Git 제외 `output/pr-review/planet6897-7406-20260925/visual/p61-caption-flow/prep7406/`에 있다.

81→82쪽의 1×1 저장 RowBreak 표는 한컴 PDF에서 ‘대상자 7’ 라벨·응답이 81쪽 끝, `vpos=0`으로 재개하는 ‘대상자 8’이 82쪽 처음이다. 기존 컷은 빈 종료 줄의 600HU 후행 간격까지 81쪽 예산으로 더해 약 4px 초과하자 대상자 7까지 되돌렸다. HWPX 저장 줄의 양수→0 되감김과 빈 종료 줄이 함께 확인될 때 그 후행 간격만 컷·페인트 공통 높이에서 제외했다. 정식 회귀는 수정 전 FAIL/후 PASS, PrEP 관련 10건 PASS(140쪽 수 고정 포함). 최종 바이너리의 81~83쪽 sweep은 **99.44/99.88/97.65%**로 통과했고 표 외곽·대상자 7/8·뒤 본문을 직접 대조했다. 재캡처는 Git 제외 `output/pr-review/planet6897-7406-20260925/visual/p82-tail-final/prep7406/`에 있다.

사용자 지적에 따라 물리 78쪽(인쇄 쪽 66)을 현재 head에서 다시 대조했다. 77/78/79쪽의 2px 관용 내용 실루엣은 **99.91/98.73/95.65%**로 국소 gate를 통과한다. 78쪽 엄격 내용 픽셀 일치율은 22.57%라 review가 붉지만, ‘사회적 장벽’ 시작 y=94.5/95.1px, ‘4. 소결’ y=437.9/438.8px, 마지막 본문 y=766.5/767.1px(rhwp/PDF)로 배치 차이는 1px 이내다. 줄바꿈과 누락도 직접 대조했다. PDF는 KoPubDotum Light/Bold subset, rhwp SVG와 Mac 폰트 매핑은 KoPub돋움체 Light/Bold이다. 두 바이너리가 동일한지는 확인하지 않았으며, 이 페이지를 `font_mismatch_exception`으로 판정하지 않는다. 재캡처는 Git 제외 `output/pr-review/planet6897-7406-20260925/visual/p78-user-check/prep7406/`에 있다.

사용자가 지적한 **인쇄 78쪽은 물리 90쪽**이다. 첫 문단의 저장 `vpos=500HU`가 문단 앞 간격 자체인데 이를 페이지 기준점으로 다시 빼서 두 번째 문단부터 제목·표까지 약 7px 위로 밀렸다. HWPX의 첫 저장 LineSeg가 앞 간격과 같고 다음 저장 문단도 자신의 앞 간격을 포함한다는 조건을 확인한 경우에만 페이지 기준점을 0으로 사용한다. 한컴 2024 PDF의 첫 문단/뒤 소제목/절 제목/표 캡션 y=101.8/161.2/346.5/456.9px을 독립 기대값으로 삼은 정식 회귀는 수정 전 뒤 소제목 y=153.9px로 실패했고, 수정 후 통과했다. 89~91쪽 국소 Native sweep은 **99.96/100.00/98.13%**로 통과하며, 물리 90쪽의 표 행과 뒤 본문도 직접 대조했다. 전체 140쪽 sweep에서도 물리 90쪽의 2px 이웃 관용 내용 실루엣은 **100%**였다. 엄격 내용 픽셀 일치율은 **32.60%**라 문자 획에 붉은 overlay가 남는다. 1쪽에 한정한 글꼴 예외를 이 페이지에 적용하지 않으며, 표의 18행·괘선·제목·앞뒤 본문 위치와 누락 여부를 별도로 판독한 결과로 배치 수정을 판단한다.

전체 sweep을 직접 판독하면서 발견한 물리 34→35쪽의 회색 1×1 표 분할도 보정했다. 한컴 PDF p34 마지막 줄 `는 경향을 보임`의 상단은 993.696px, p35 첫 `(4) Jenness`는 106.496px인데, 직전 메인터너 보정은 그 마지막 줄을 p35로 넘겼다. 원본 HWPX의 첫 저장 프레임은 마지막 줄 `vpos=29800HU`+줄 높이 1000HU+위·아래 안 여백 각 850HU=표 선언 높이 32500HU이고 다음 문단은 `vpos=0`으로 재개한다. 이 저장 근거가 일치하고 첫 프레임이 남은 페이지에 들어갈 때만 마지막 한 줄을 첫 컷에 붙이며, **스캔 예약과 partial-table 그리기가 같은 433.3px 프레임 높이를 사용**한다. 첫 시도에서 그리기만 440px로 재계산해 표가 본문 바닥을 3.9px 넘은 사실을 확인하고 공통 높이 소비로 고쳤다. 정식 회귀는 수정 전 FAIL, 수정 후 마지막 줄·다음 쪽 첫 항목·표 본문 경계 **3/3 PASS**다. 새 Native 11쪽 선택 sweep은 gate passed, p34 **99.95610%**, p35 **100%**, p39~41 및 인쇄 78쪽(물리 90쪽)도 **99.99% 이상/100%**다. [34쪽 review](../assets/pr7406_20260925/native_review_034.png)와 [35쪽 review](../assets/pr7406_20260925/native_review_035.png)를 최종 head 캡처로 포함한다.

물리 79→80쪽의 1×1 회색 표는 원본 HWPX `pageBreak="CELL"`이 내부 `RowBreak`로 정규화된다. 셀의 ‘필로폰이 제게…’ 마지막 저장 줄 `vpos=4800HU` 다음 문단은 `vpos=0`으로 되감긴다. 한컴 PDF는 이 응답을 79쪽 끝에 두는데, rhwp는 마지막 줄의 600HU 후행 줄간격까지 첫 조각에 예약해 약 1.1px 초과로 판단하고 80쪽으로 넘겼다. 원본 저장 줄·비편집 HWPX·가시 마지막 줄·다음 문단 reset이 함께 확인된 경우에만 그 후행 간격을 컷과 페인트 공통 높이에서 제외했다. 정식 회귀는 수정 전 FAIL/후 PASS, PrEP 관련 nextest **12/12 PASS**(140쪽 수 고정 포함). 국소 sweep은 79쪽 **95.65→99.94%**, 80쪽 **89.03→99.98%**이며, 79쪽 회색 상자 끝과 80쪽 시작 응답·뒤 문단을 직접 대조했다. 39~41·78·81~83·89~91쪽도 90% 이상으로 통과했다. 재캡처는 Git 제외 `output/pr-review/planet6897-7406-20260925/visual/p80-cell-reset-final/prep7406/`에 있다.

물리 93쪽은 `BinData` 그림 본체의 위치가 기준 PDF와 거의 같지만, 그림 뒤 빈 문단·[그림 15] 캡션·후속 본문이 모두 약 15.5px 아래에 있었다. 원본 HWPX의 그림 host 다음 빈 문단 저장 `vpos=34725HU`는 실제 그림 바닥과 일치하고, 캡션 `vpos=35775HU`는 빈 줄의 1050HU 높이만큼 뒤에 있다. 그림 host 줄 높이를 이 경계에서 다시 더하지 않도록 실제 ImageNode 바닥과 저장 좌표가 1px 이내로 일치하는 비편집 HWPX에 한해 빈 문단 원점을 맞췄다. 한컴 2024 PDF의 캡션/첫 후속 문단/CDC 문단 y=572.1/613.0/685.8px을 검사한 정식 회귀는 수정 전 캡션 y=587.6px로 FAIL, 수정 후 PASS이며 PrEP 관련 nextest **13/13 PASS**다. 93쪽 국소 sweep은 **75.61→99.42%**, 60~62·78~83·89~94쪽 15개 선택 페이지가 모두 90% 이상이었다. 그림 끝·캡션·두 후속 문단을 직접 대조했다. 재캡처는 Git 제외 `output/pr-review/planet6897-7406-20260925/visual/p93-picture-bottom-final/prep7406/`에 있다.

물리 92쪽(인쇄 80쪽)은 자동 실루엣 **94.89%**에도 `[그림 14]` 캡션이 PDF보다 **12.064px** 낮아 직접 판독에서 미충족으로 판정했다. 저장 그림 뒤 빈 줄의 `vpos=66533HU`, 높이1050HU, 줄간격−420HU와 다음 캡션 `vpos=67163HU`가 **630HU=8.4px**의 물리 전진을 증명한다. 기존 그림 바닥 보정은 다음 줄 차이를 줄 높이와만 대조해 이 음수 간격을 누락했고, hidden 빈 글자 경로도 실제 간격을 0으로 만들었다. 실제 그림 바닥과 저장 빈 줄 원점이 일치하는 비편집 HWPX에서 원점·다음 줄 흐름 끝을 같은 저장 결과로 사용한다. 내부 캡션이 있는 그림·TAC·편집본·합성 줄·불일치 좌표에는 적용하지 않는다. 정식 회귀 `prep_page_92_caption_keeps_saved_negative_empty_line_advance`는 수정 전 **FAIL(y1002.560px)**, 수정 후 **PASS(독립 PDF990.496px±1.5px)**이며 PrEP **15/15 PASS**다. 진단 Native 92·93쪽은 **96.96/99.42%**, review·standalone overlay에서 그림 본체·캡션·앞 본문을 직접 대조했다. 최종 140쪽 render tree를 이전 head와 대조하면 바뀐 페이지는 **92쪽 하나**이고 그림 본체 원점·크기는 그대로다. 최종 head 시각 결과는 아래 검증 표에 연결한다.

물리 105쪽(인쇄 93쪽)은 6열 20행 연구비 표의 저장 행 높이 `cellSz`와 한컴 PDF 괘선이 일치하지만, rhwp 측정값은 행마다 약 0.5px 작아 중간 행부터 수평선이 누적 이동했다. 마지막 행에서 표 전체 높이를 맞추며 약 10px이 몰린 것이 원인이었다. 저장 HWPX `noAdjust=1`, 비편집 TopAndBottom 표, 모든 단일행 선언과 병합 셀의 선언 합이 일치하고 각 행 측정 오차가 1.5px 이하인 경우에 한해 첫 19행을 저장 높이로 정하고, 마지막 행은 저장된 표 전체 높이의 잔여로 닫았다. 마지막 행의 저장 글줄과 여백이 그 높이에 들어가는지도 검사한다. 조판과 페인트가 같은 보정된 `MeasuredTable`을 사용한다. 한컴 PDF 10행·19행 괘선을 독립 기대값으로 둔 회귀는 수정 전 10행 y=629.9px 대 PDF 634px로 FAIL, 수정 후 PASS. 105쪽 국소 sweep은 **77.50→99.90%**, 인접 104·106쪽은 **99.79/99.89%**이고, 전체 표의 행 경계·마지막 총액 행·하단을 직접 대조했다. 재캡처는 Git 제외 `output/pr-review/planet6897-7406-20260925/visual/p105-row-heights-final2/prep7406/`에 있다.

전체 nextest에서 `issue_6632_cell_tac_shape_line_height`의 글줄 기준선 검사가 기존 x 필터 때문에 FAIL했다. `samples/hwpspec.hwp`의 한컴 2024 PDF 물리 106쪽을 글자 단위로 직접 추출하면 `(x2, y2)`의 숫자 `2`는 **(515.50, 540.64)px**, `(x3, y3)`의 숫자 `3`은 **(271.58, 582.88)px**이다. 현재 renderer의 두 글자 좌표는 이 독립 기준과 0.2px 이내로 맞는데 테스트는 과거 x=505.2/261.5px만 찾고 있었다. 테스트의 x 선택 기준을 PDF 값으로 고쳤다. 위치를 이전 x로 되돌린 임시 코드에서는 해당 회귀가 통과했지만, 직접 시각 비교에서 곡선·라벨이 PDF보다 10px 왼쪽으로 이동하고 p106 실루엣이 **87.74→86.21%**로 악화해 그 임시 코드를 폐기했다. 최종 테스트는 수정 전 FAIL/수정 후 **2/2 PASS**, PrEP 57쪽 가운데 정렬 대조군도 PASS다. p106의 남은 글꼴·픽셀 차이는 #7406 승인 근거로 사용하지 않는다. 분석 자료는 Git 제외 `output/pr-review/planet6897-7406-20260925/diag-6632/`에 보존한다.

전체 nextest의 본문 넘침 원장에서는 `issue1853_caption_precedes_body_split.hwpx` 물리 10쪽이 3.84px 아래로 넘쳤다. 이 쪽 첫 문단은 원본 HWPX의 명시적 `pageBreak="1"`이고, PrEP 물리 90쪽(인쇄 78쪽)의 첫 문단은 자연스러운 흐름이다. 두 문서 모두 첫 저장 `vpos=500HU`와 뒤 문단의 앞 간격 사다리가 있어, 기존 90쪽 보정이 명시적 쪽나눔까지 페이지 원점 0으로 분류했다. 명시적 쪽나눔은 첫 `vpos`를 기준점으로 유지하도록 보정했다. 원장 검사 수정 전 FAIL(1건 증가)/후 **PASS**, 90쪽 회귀 **PASS**, CLI `layout-anomaly`의 해당 쪽 넘침 1→0건, 원본 물리 90쪽 직접 Visual Sweep **100%**다. `issue1853` 물리 10쪽의 기준 PDF 대비 실루엣은 **53.31%**로 별도 배치·글꼴 차이가 남으므로 이 문서 자체의 시각 일치를 주장하지 않는다. 증적은 Git 제외 `output/pr-review/planet6897-7406-20260925/visual/issue1853-p10-after/`와 `visual/p90-after-1853/`에 있다.

전체 nextest의 80168 문서 157→158쪽 증가 4건은 1×1 표 안 여백 보정의 적용 범위가 넓었던 결과다. HWPX 저장 XML에서 바깥 높이가 셀 높이보다 큰 1×1 표 53개 중 32개는 위·아래 안 여백 합이 셀 높이와 **같다**. PrEP 39→40쪽은 저장 셀 282HU보다 위·아래 여백 합 1700HU가 커서 작은 셀 높이만으로 여백을 폐기할 수 없다는 근거가 있다. 바깥 높이를 여백 판정에 쓰는 조건을 이처럼 **초과**하는 경우로 좁혔다. 80168 HWP/HWPX는 각각 한컴 기준 **157쪽**으로 복귀했고, 관련 nextest **11/11 PASS** 및 PrEP 39→41쪽 직접 Native sweep **99.99% 이상**이다. RowBreak 이어진 줄 검사 HWP/HWPX **2/2 PASS**도 확인했다. 증적은 Git 제외 `output/pr-review/planet6897-7406-20260925/visual/p40-after-80168/`과 `logs/issue80168-focused-after.log`에 있다.

`issue2004` HWPX 물리 4쪽 첫 그림은 기준 124.9px보다 6.68px 높은 118.22px에 그려졌다. 앞 제목의 저장 줄 끝 3060HU와 표 호스트 첫 `vpos=3560HU`의 차이는 표 호스트의 문단 앞 간격 **500HU**와 정확히 같다. 기존 `HeightCursor`는 TopAndBottom 표 호스트의 첫 vpos를 항상 버리고 이전 줄 끝에서 앞 간격을 다시 빼서 첫 표 조각을 올렸다. 저장 사다리가 이처럼 앞 간격만 인코딩한 경우에는 표 높이가 포함되지 않았으므로 해당 첫 vpos를 사용한다. 수정 전 그림 좌표 회귀 FAIL, 수정 후 **6/6 PASS**(HWP/HWPX 그림, PrEP 39→40·90쪽, 80168 쪽수, RowBreak HWP/HWPX). 새 Native 그림 상단은 **124.9px**, PrEP 39·40·41·90쪽은 모두 **99.99% 이상**으로 gate PASS다. `issue2004` 물리 4쪽 전체 실루엣은 **57.45→81.87%**로 개선됐으나 이미지 내부 픽셀 차이가 남아 이 대조 문서의 시각 일치를 주장하지 않는다. 증적은 Git 제외 `output/pr-review/planet6897-7406-20260925/visual/issue2004-p4-after/`와 `visual/p40-p90-after-2004/`에 있다.

그 뒤 전체 nextest **10,255 PASS/1 FAIL/50 skip**에서 `issue1853` 물리 10쪽 넘침이 다시 검출됐다. 같은 숫자상 앞 간격 사다리를 가진 3×2 텍스트 표까지 그림 표 규칙이 적용된 반례다. 저장 1×1 표의 셀에 실제 Picture 컨트롤이 있는 경우로 규칙을 한정했다. 수정 전 본문 넘침 원장 FAIL/수정 후 **PASS**, `issue2004` HWP/HWPX 그림과 PrEP·80168 대조군 포함 집중 회귀 **5/5 PASS**다. 새 CLI의 `issue1853` 물리 10쪽 넘침 **0건**, `issue2004` 그림 상단 **124.9px**, PrEP 39·40·41·90쪽 국소 sweep 최저 **99.99354%**다. `issue2004` 전체 실루엣은 **81.87%**로 남아 대조 문서의 전체 시각 일치를 주장하지 않는다. 이 단계 뒤 전체 검증을 최종 `9e7518243`에서 재실행해 통과했다(위 최종 결과 표).

물리 1쪽의 기준 PDF는 `H2hdrM`과 `Dotum` 내장 subset을 사용하지만, 같은 제목 SVG의 Chrome 실제 적용 글꼴은 `HCR Dotum`이다. 두 출력의 글꼴이 완전히 다른 것을 [해시 고정 증거](../assets/pr7406_20260925/font_mismatch_001.md)에 기록했다. 이 예외는 물리 1쪽에만 적용하며 표·그림 경계와 글줄 수·순서를 review/overlay에서 직접 확인했다. 최종 전체 Visual Sweep에는 이 증거 파일을 `--font-mismatch-evidence`로 전달하고 다른 쪽의 점수와 배치 차이는 별도로 판정한다.

![#7406 원본 물리 1쪽 Native review](../assets/pr7406_20260925/native_review_001.png)

![#7406 원본 물리 1쪽 Native overlay](../assets/pr7406_20260925/native_overlay_001.png)

![#7406 OLE 차트 분리 샘플 Native review](../assets/pr7406_20260925/ole_chart_review_001.png)

![#7406 OLE 차트 분리 샘플 Native overlay](../assets/pr7406_20260925/ole_chart_overlay_001.png)

![#7406 OLE 차트 분리 샘플 fresh WASM review](../assets/pr7406_20260925/wasm_ole_review_001.png)

![#7406 OLE 차트 분리 샘플 fresh WASM overlay](../assets/pr7406_20260925/wasm_ole_overlay_001.png)

![#7406 원본 27쪽 Native review](../assets/pr7406_20260925/native_review_027.png)

![#7406 원본 27쪽 Native overlay](../assets/pr7406_20260925/native_overlay_027.png)

![#7406 원본 34쪽 Native review](../assets/pr7406_20260925/native_review_034.png)

![#7406 원본 34쪽 Native overlay](../assets/pr7406_20260925/native_overlay_034.png)

![#7406 원본 34쪽 fresh WASM review](../assets/pr7406_20260925/wasm_review_034.png)

![#7406 원본 34쪽 fresh WASM overlay](../assets/pr7406_20260925/wasm_overlay_034.png)

![#7406 원본 35쪽 Native review](../assets/pr7406_20260925/native_review_035.png)

![#7406 원본 35쪽 Native overlay](../assets/pr7406_20260925/native_overlay_035.png)

![#7406 원본 35쪽 fresh WASM review](../assets/pr7406_20260925/wasm_review_035.png)

![#7406 원본 35쪽 fresh WASM overlay](../assets/pr7406_20260925/wasm_overlay_035.png)

![#7406 원본 40쪽 Native review](../assets/pr7406_20260925/native_review_040.png)

![#7406 원본 40쪽 Native overlay](../assets/pr7406_20260925/native_overlay_040.png)

![#7406 원본 50쪽 Native review](../assets/pr7406_20260925/native_review_050.png)

![#7406 원본 50쪽 Native overlay](../assets/pr7406_20260925/native_overlay_050.png)

![#7406 원본 51쪽 Native review](../assets/pr7406_20260925/native_review_051.png)

![#7406 원본 51쪽 Native overlay](../assets/pr7406_20260925/native_overlay_051.png)

![#7406 원본 57쪽 Native review](../assets/pr7406_20260925/native_review_057.png)

![#7406 원본 57쪽 Native overlay](../assets/pr7406_20260925/native_overlay_057.png)

![#7406 원본 61쪽 Native review](../assets/pr7406_20260925/native_review_061.png)

![#7406 원본 61쪽 Native overlay](../assets/pr7406_20260925/native_overlay_061.png)

![#7406 원본 78쪽 Native review](../assets/pr7406_20260925/native_review_078.png)

![#7406 원본 78쪽 Native overlay](../assets/pr7406_20260925/native_overlay_078.png)

![#7406 원본 물리 79쪽 Native review](../assets/pr7406_20260925/native_review_079.png)

![#7406 원본 물리 79쪽 Native overlay](../assets/pr7406_20260925/native_overlay_079.png)

![#7406 원본 물리 80쪽 Native review](../assets/pr7406_20260925/native_review_080.png)

![#7406 원본 물리 80쪽 Native overlay](../assets/pr7406_20260925/native_overlay_080.png)

![#7406 원본 물리 90쪽(인쇄 78쪽) Native review](../assets/pr7406_20260925/native_review_090.png)

![#7406 원본 물리 90쪽(인쇄 78쪽) Native overlay](../assets/pr7406_20260925/native_overlay_090.png)

![#7406 원본 물리 90쪽(인쇄 78쪽) fresh WASM review](../assets/pr7406_20260925/wasm_review_090.png)

![#7406 원본 물리 90쪽(인쇄 78쪽) fresh WASM overlay](../assets/pr7406_20260925/wasm_overlay_090.png)

![#7406 원본 물리 93쪽 Native review](../assets/pr7406_20260925/native_review_093.png)

![#7406 원본 물리 93쪽 Native overlay](../assets/pr7406_20260925/native_overlay_093.png)

![#7406 원본 물리 105쪽 Native review](../assets/pr7406_20260925/native_review_105.png)

![#7406 원본 물리 105쪽 Native overlay](../assets/pr7406_20260925/native_overlay_105.png)

![#7406 원본 81쪽 Native review](../assets/pr7406_20260925/native_review_081.png)

![#7406 원본 81쪽 Native overlay](../assets/pr7406_20260925/native_overlay_081.png)

![#7406 원본 82쪽 Native review](../assets/pr7406_20260925/native_review_082.png)

![#7406 원본 82쪽 Native overlay](../assets/pr7406_20260925/native_overlay_082.png)

![#7406 69쪽 Native review](../assets/pr7406_20260925/native_review_069.png)

![#7406 69쪽 Native overlay](../assets/pr7406_20260925/native_overlay_069.png)

![#7406 94쪽 Native review](../assets/pr7406_20260925/native_review_094.png)

![#7406 94쪽 Native overlay](../assets/pr7406_20260925/native_overlay_094.png)

물리 76쪽의 TAC 1×1 표는 저장 높이 37100HU가 마지막 글줄 끝(34400+1000HU)과 위·아래 여백(850+850HU)을 이미 포함한다. 기존 측정에서 마지막 줄간격 600HU를 다시 더해 표가 8px 커지고 가운데 정렬 글자와 뒤 본문이 내려갔다. HWPX XML `CELL`의 내부 `RowBreak` 매핑을 확인하고, 저장 끝·여백이 전체 프레임을 정확히 닫고 저장 줄이 되감기지 않는 완결 1×1 표에 공통 trailing-spacing 보정을 적용했다. 조판과 실제 그리기가 같은 `MeasuredTable`을 소비한다. 한컴 PDF 좌표 기반 회귀는 수정 전 FAIL/후 PrEP **16/16 PASS**다. 표 높이는 **502.7→494.7px**, 첫 줄 **154.333→150.333px**(PDF 150.976), 뒤 본문 **666.7→658.7px**(PDF 659.424). 75·76·77쪽 Native sweep **100/99.92066/99.90731%**이며 review·standalone overlay에서 표 외곽과 후속 본문을 직접 확인했다. 140쪽 render tree에서 바뀐 것은 76쪽뿐이다. 물리 33쪽은 다음 단계에서 별도 보정했다.

물리 33쪽 마지막 문단은 원본 저장 LineSeg와 한컴 PDF 모두 여섯 줄이며 마지막 줄은 완전한 어절 `왜곡될 수 있음.`이다. 앞선 메인터너 보정의 “새 fill의 줄 수가 더 적으면 저장 분할이 무효”라는 가정이 이를 다섯 줄로 줄였다. 이 일반 가정을 제거했다. 현재 frame으로 합칠 수 있는 마지막 한글 어절 조각만 남은 소프트 분할의 수선과 정상 저장 어절 경계 보존을 구분한다. 저장 위치는 공통 HWP5 축 변환을 통해 해석하며 측정·배치는 같은 composed lines를 소비한다. PDF의 여섯 줄·첫 줄 끝·마지막 줄 내용·최종 y=574.464px 기반 정식 회귀는 수정 전 FAIL/후 PrEP **17/17 PASS**다. 67쪽 마지막 음절과 76쪽 표 등 기존 대조군도 통과했다. 140쪽 tree에서 영향은 14·33·91쪽뿐이며, 인접 쪽 포함 9쪽 Native sweep 모두 gate passed다. 직접 review·overlay에서 33쪽 **92.40→97.75050%**, 14쪽 **100%**, 91쪽 **99.90666%**와 줄 경계·뒤 본문을 대조했다. 최종 전체 검증은 새 code head `9e7518243`에서 완료했으며 위 표에 기록했다.

최종 code head `9e7518243`의 Native CLI `layout-anomaly --json`은 140쪽에서 텍스트 겹침·저장 줄 이탈·용지 밖 개체·일반 겹침 각각 **0건**이다. 저장 표 선언의 본문 경계 신호 8건은 남아 있다: 우측 1.1867~5.7333px, 물리 37쪽 하단 2.3867px. 이는 겹침 baseline 상한 1과 다른 Native 실행 지표이며, 모두 0이라고 보고하지 않는다. 37쪽 출력의 표 바닥과 뒤 내용은 직접 비교했고 전체 픽셀 일치를 주장하지 않는다.

![#7406 물리 33쪽 native review](../assets/pr7406_20260925/native_review_033.png)

![#7406 물리 33쪽 native overlay](../assets/pr7406_20260925/native_overlay_033.png)

![#7406 물리 33쪽 wasm review](../assets/pr7406_20260925/wasm_review_033.png)

![#7406 물리 33쪽 wasm overlay](../assets/pr7406_20260925/wasm_overlay_033.png)

![#7406 물리 76쪽 native review](../assets/pr7406_20260925/native_review_076.png)

![#7406 물리 76쪽 native overlay](../assets/pr7406_20260925/native_overlay_076.png)

![#7406 물리 76쪽 wasm review](../assets/pr7406_20260925/wasm_review_076.png)

![#7406 물리 76쪽 wasm overlay](../assets/pr7406_20260925/wasm_overlay_076.png)

![#7406 물리 92쪽 native review](../assets/pr7406_20260925/native_review_092.png)

![#7406 물리 92쪽 native overlay](../assets/pr7406_20260925/native_overlay_092.png)

![#7406 물리 92쪽 wasm review](../assets/pr7406_20260925/wasm_review_092.png)

![#7406 물리 92쪽 wasm overlay](../assets/pr7406_20260925/wasm_overlay_092.png)

### 전체 판독에서 확인한 미해결 범위

이번 통합은 KoPub 라틴 전진폭·표 분할/높이·그림 캡션·편집 가능한 OOXML 차트의 명시된 보정 범위다. 문서 전체의 한컴 출력 일치를 해결했다고 주장하지 않는다. 물리 31쪽 수식의 대문자 `SIGMA`가 기호 `Σ` 대신 문자열로 남는 수식 파서 결함과 물리 66쪽 구역 첫 문단의 저장 줄 경계 차이는 **미해결**이다. 두 쪽 render tree는 초기 보정 전 후보와 최종 `9e7518243`에서 동일하며, 해당 수식 parser/symbols와 `Paragraph::line_seg_text_start_of`는 base `c80a8370ab294259557c850c2e54495ecd0e79c0`와 동일하다. 이는 수정 범위의 구분 근거이며 두 결함의 시각 통과 근거가 아니다. 66쪽 저장 `textpos=85/149/214/274/344`와 첫 스타일 경계85에 대한 HWPX/HWP5 축 해석은 추가 독립 입력·기준 출력 검증이 필요하다. 글꼴 예외를 적용하지 않는다. 전체 gate 점수가 90% 이상이어도 이 두 쪽을 한컴 출력과 일치한다고 판정하지 않는다. 원 이슈 #7390은 열린 상태로 유지하고, 통합 PR 본문과 contributor 안내에도 이 범위를 명시한다.

## 보류 해제와 contributor 안내

`e049c490d`의 자동 gate 통과 뒤 직접 판독에서 검출한 76쪽과 33쪽 결함을 단계별로 분석·보정·보고·커밋했다. 각 수정 전 FAIL/후 정식 회귀, 최종 `9e7518243`의 전체 검증 및 fresh WASM 직접 비교까지 완료해 **명시된 보정 범위의 통합 후보**를 수용 가능으로 갱신했다. 물리 1쪽 예외는 다른 페이지의 배치나 줄바꿈 차이를 면제하지 않는다. 현재 원 contributor head 자체는 직접 승인하지 않으며 통합 PR의 최신 CI를 확인한 뒤 병합한다.

## Merge 후 contributor PR comment 계획

통합 PR이 실제로 병합되고 asset이 `devel`에 들어간 뒤 원 PR #7406에 한국어 존댓말로 원 기여와 메인터너 보정을 구분해 알린다. 통합 PR 번호·실제 merge SHA·최신 CI URL·전체 nextest 및 Native/fresh WASM Visual Sweep의 최종 결과를 적는다. 인쇄 78쪽(물리 90쪽), 34→35쪽 표 분할, 물리 92쪽 음수 간격 캡션, 물리 76쪽 완결 표, 33쪽 정상 저장 어절 경계 보정을 설명하고, merge SHA로 고정한 `mydocs/pr/assets/pr7406_20260925/native_review_090.png`, `native_overlay_090.png`, `native_review_034.png`, `native_overlay_034.png`, `wasm_review_090.png`, `wasm_overlay_090.png`, `native_review_092.png`, `native_overlay_092.png`, `wasm_review_092.png`, `wasm_overlay_092.png`, `native_review_033.png`, `native_overlay_033.png`, `native_review_076.png`, `native_overlay_076.png`의 raw URL을 Markdown 이미지로 표시한다. #7390은 남은 범위가 있어 자동 종료하지 않고 상태를 별도 확인한다. 게시 후 원 PR을 통합 PR 링크와 함께 닫고 comment URL을 기록한다.
