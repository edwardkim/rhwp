# 최신 devel 통합 뒤의 검증 결과

후보 `696240b004707a6686864f94130befe3dd7f213e`, 통합 devel `1924d371d6757392d063b8dc2e9bbe9a298be817`다. 승인된 CI 시간 예산 보정 `4052e0bab1`과 최신 devel의 Native 웹폰트 공급을 원 기여 이력과 함께 병합했다. 제품 보정의 실제 조판 출력 producer는 `ef6d3029371262204f7b5c3ecf44820c66e54345`이며, 이후 `63532b1ee5`는 공통 JSON 출처 선언만 보정했다. 후보는 소비 fixture·문서 동기화도 포함한다. 이들 사이의 렌더링·글꼴 선택·paint source는 같다.

## 실행 게이트

- stock 전체 nextest: **Summary [ 542.785s] 12475 tests run: 12475 passed (1 slow), 45 skipped**. 기존 실패 검사를 완화하지 않고 출처 선언과 소비 fixture를 보정했다. [원 계약·실패·보정](ci-provenance-followup.md)을 구분한다.
- Rust fmt, root/WASM32/workspace all-target Clippy, workspace build, 일반 release build/lib, Native Skia 필수 3종, 기본/Native 실제 출처 계약, doc test, 최적화 fresh WASM을 통과했다. 일반 release lib는 ef6에서 3,983 PASS / 12 ignored이며, 뒤의 출처 메타데이터·소비 문서 변경으로 조판 검증을 재실행했다고 표시하지 않는다.
- Studio TypeScript exit 0, 단위 검사 **2,717 PASS / 2 skip**. Python 계약 89건·Node 운영 계약 186건: **275 PASS**. 실행 source는 ef6이며 이후 해당 source는 바뀌지 않았다.
- 회귀 검사는 private 글꼴 경로와 시스템 글꼴을 가리고 허용된 저장소 글꼴 5개만 노출한 최소 환경에서 실행했다. 독립 Print 시각 검증의 명시 글꼴 공급과 구분한다.

| 단계 | 실제 실행 head | 종료 코드 | 컴파일 포함 초 |
| --- | --- | ---: | ---: |
| clippy-root | 63532b1ee5 (제품 코드 동일 재사용) | 0 | 47.58 |
| clippy-workspace | 63532b1ee5 (제품 코드 동일 재사용) | 0 | 64.60 |
| clippy-wasm | 63532b1ee5 (제품 코드 동일 재사용) | 0 | 47.75 |
| workspace-build | 63532b1ee5 (제품 코드 동일 재사용) | 0 | 37.67 |
| provenance-default | 63532b1ee5 (제품 코드 동일 재사용) | 0 | 348.79 |
| full-nextest | 696240b004 | 0 | 543.70 |
| native-build | 696240b004 | 0 | 342.34 |
| native-skia-lib | 696240b004 | 0 | 383.87 |
| native-placeholder | 696240b004 | 0 | 327.53 |
| native-direct-pdf | 696240b004 | 0 | 19.01 |
| native-provenance | 696240b004 | 0 | 30.14 |
| doc-tests | 696240b004 | 0 | 109.74 |
| wasm-build | 696240b004 | 0 | 879.02 |

빌드·테스트의 경합을 포함한 경과 시간이며 성능 벤치마크로 해석하지 않는다. Rust 1.93.1·Node 24.15.0·Chromium 1660786·Pillow 10.2.0·pypdfium2 5.13.0의 WSL 호스트 경로다. nextest 실행 버전은 0.9.137이며 권고 버전 0.9.140과 구분한다. 검사는 동일 stock 명령·범위로 완료했다.

## 전쪽 Native/fresh WASM

각 **32개 입력·72쪽**, 독립 Print와 쪽수 일치·누락 0·raw 90% 미만 0쪽이다. Native 최저 **92.89641%**, fresh WASM 최저 **92.88059%**다. 96 DPI, 색상 경계 보정 전 2px 내용 실루엣이다. 원 PR의 Form01 HWPX도 별도 1쪽으로 실행했다.

| 입력 | 쪽수 | Native 최저 raw | fresh WASM 최저 raw | 전쪽 TSV |
| --- | ---: | ---: | ---: | --- |
| form | 1 | 99.60303% | 99.58079% | [Native](tsv/rebased/native/form.tsv) · [WASM](tsv/rebased/wasm/form.tsv) |
| form-original-hwpx | 1 | 99.60303% | 99.58079% | [Native](tsv/rebased/native/form-original-hwpx.tsv) · [WASM](tsv/rebased/wasm/form-original-hwpx.tsv) |
| form-slot-hwp | 1 | 100.00000% | 100.00000% | [Native](tsv/rebased/native/form-slot-hwp.tsv) · [WASM](tsv/rebased/wasm/form-slot-hwp.tsv) |
| form-slot-hwpx | 1 | 100.00000% | 100.00000% | [Native](tsv/rebased/native/form-slot-hwpx.tsv) · [WASM](tsv/rebased/wasm/form-slot-hwpx.tsv) |
| forms-before | 1 | 100.00000% | 100.00000% | [Native](tsv/rebased/native/forms-before.tsv) · [WASM](tsv/rebased/wasm/forms-before.tsv) |
| forms-edited | 1 | 100.00000% | 100.00000% | [Native](tsv/rebased/native/forms-edited.tsv) · [WASM](tsv/rebased/wasm/forms-edited.tsv) |
| grow-after | 1 | 100.00000% | 100.00000% | [Native](tsv/rebased/native/grow-after.tsv) · [WASM](tsv/rebased/wasm/grow-after.tsv) |
| grow-before | 1 | 100.00000% | 100.00000% | [Native](tsv/rebased/native/grow-before.tsv) · [WASM](tsv/rebased/wasm/grow-before.tsv) |
| grow-nested | 1 | 100.00000% | 100.00000% | [Native](tsv/rebased/native/grow-nested.tsv) · [WASM](tsv/rebased/wasm/grow-nested.tsv) |
| grow-nested-unchanged | 1 | 100.00000% | 100.00000% | [Native](tsv/rebased/native/grow-nested-unchanged.tsv) · [WASM](tsv/rebased/wasm/grow-nested-unchanged.tsv) |
| header-hwp | 1 | 100.00000% | 100.00000% | [Native](tsv/rebased/native/header-hwp.tsv) · [WASM](tsv/rebased/wasm/header-hwp.tsv) |
| header-hwpx | 1 | 100.00000% | 100.00000% | [Native](tsv/rebased/native/header-hwpx.tsv) · [WASM](tsv/rebased/wasm/header-hwpx.tsv) |
| nested-wide | 1 | 100.00000% | 100.00000% | [Native](tsv/rebased/native/nested-wide.tsv) · [WASM](tsv/rebased/wasm/nested-wide.tsv) |
| no-lineseg-empty | 1 | 100.00000% | 100.00000% | [Native](tsv/rebased/native/no-lineseg-empty.tsv) · [WASM](tsv/rebased/wasm/no-lineseg-empty.tsv) |
| no-lineseg-text | 1 | 100.00000% | 100.00000% | [Native](tsv/rebased/native/no-lineseg-text.tsv) · [WASM](tsv/rebased/wasm/no-lineseg-text.tsv) |
| no-lineseg-wrap | 1 | 92.89641% | 96.55502% | [Native](tsv/rebased/native/no-lineseg-wrap.tsv) · [WASM](tsv/rebased/wasm/no-lineseg-wrap.tsv) |
| notice-reopened | 24 | 97.69193% | 97.45945% | [Native](tsv/rebased/native/notice-reopened.tsv) · [WASM](tsv/rebased/wasm/notice-reopened.tsv) |
| ownership-after-table-after | 1 | 100.00000% | 100.00000% | [Native](tsv/rebased/native/ownership-after-table-after.tsv) · [WASM](tsv/rebased/wasm/ownership-after-table-after.tsv) |
| ownership-empty-host-after | 1 | 100.00000% | 100.00000% | [Native](tsv/rebased/native/ownership-empty-host-after.tsv) · [WASM](tsv/rebased/wasm/ownership-empty-host-after.tsv) |
| ownership-fresh-reference | 1 | 100.00000% | 100.00000% | [Native](tsv/rebased/native/ownership-fresh-reference.tsv) · [WASM](tsv/rebased/wasm/ownership-fresh-reference.tsv) |
| ownership-loaded-before | 1 | 100.00000% | 100.00000% | [Native](tsv/rebased/native/ownership-loaded-before.tsv) · [WASM](tsv/rebased/wasm/ownership-loaded-before.tsv) |
| polygon-270 | 1 | 100.00000% | 100.00000% | [Native](tsv/rebased/native/polygon-270.tsv) · [WASM](tsv/rebased/wasm/polygon-270.tsv) |
| polygon-90 | 1 | 100.00000% | 100.00000% | [Native](tsv/rebased/native/polygon-90.tsv) · [WASM](tsv/rebased/wasm/polygon-90.tsv) |
| polygon-hflip | 1 | 100.00000% | 100.00000% | [Native](tsv/rebased/native/polygon-hflip.tsv) · [WASM](tsv/rebased/wasm/polygon-hflip.tsv) |
| polygon-resize | 1 | 100.00000% | 100.00000% | [Native](tsv/rebased/native/polygon-resize.tsv) · [WASM](tsv/rebased/wasm/polygon-resize.tsv) |
| polygon-vflip | 1 | 100.00000% | 100.00000% | [Native](tsv/rebased/native/polygon-vflip.tsv) · [WASM](tsv/rebased/wasm/polygon-vflip.tsv) |
| rotated-hwp | 1 | 100.00000% | 100.00000% | [Native](tsv/rebased/native/rotated-hwp.tsv) · [WASM](tsv/rebased/wasm/rotated-hwp.tsv) |
| rotated-hwpx | 1 | 100.00000% | 100.00000% | [Native](tsv/rebased/native/rotated-hwpx.tsv) · [WASM](tsv/rebased/wasm/rotated-hwpx.tsv) |
| spacing | 1 | 100.00000% | 100.00000% | [Native](tsv/rebased/native/spacing.tsv) · [WASM](tsv/rebased/wasm/spacing.tsv) |
| spacing-after | 1 | 100.00000% | 100.00000% | [Native](tsv/rebased/native/spacing-after.tsv) · [WASM](tsv/rebased/wasm/spacing-after.tsv) |
| spacing-before | 1 | 100.00000% | 100.00000% | [Native](tsv/rebased/native/spacing-before.tsv) · [WASM](tsv/rebased/wasm/spacing-before.tsv) |
| split-row | 18 | 92.93123% | 92.88059% | [Native](tsv/rebased/native/split-row.tsv) · [WASM](tsv/rebased/wasm/split-row.tsv) |

Native의 렌더링 source가 같은 ef6 실행은 재사용으로 명시한다. fresh WASM은 후보의 새 최적화 bundle을 실제 Chromium Canvas 경로에서 실행한 결과다. [증적 해시·producer·입력/쪽 대응](rebase/evidence-index.json)에 TSV와 대표 review/overlay를 연결한다. 원시 로그·전체 run JSON·글꼴 프로그램은 커밋하지 않는다.

## 실제 API와 저장 후 교차 재열기

Native 6개·WASM 7개 시나리오에서 실제 API를 호출했다. 표 삽입 53호출의 live/저장 재열기, 양식의 두 줄 캐럿·선택·복사·삭제, 다각형·간격·clip과 표 높이의 편집/무편집 저장을 확인했다. 아래 14개 actual/cross 역할 175쪽도 독립 Print와 전쪽 비교했다.

| 입력/실행 | 경로 | 쪽수 | 최저 raw | 전쪽 TSV |
| --- | --- | ---: | ---: | --- |
| native-grow-nested-live | actual-png-pair | 1 | 100.00000% | [TSV](tsv/rebased/actual/actual-png-pair-native-grow-nested-live.tsv) |
| native-grow-nested-reopened | actual-png-pair | 1 | 100.00000% | [TSV](tsv/rebased/actual/actual-png-pair-native-grow-nested-reopened.tsv) |
| native-notice-live | actual-png-pair | 24 | 97.71475% | [TSV](tsv/rebased/actual/actual-png-pair-native-notice-live.tsv) |
| native-notice-reopened | actual-png-pair | 24 | 97.71475% | [TSV](tsv/rebased/actual/actual-png-pair-native-notice-reopened.tsv) |
| wasm-grow-nested-live | actual-png-pair | 1 | 100.00000% | [TSV](tsv/rebased/actual/actual-png-pair-wasm-grow-nested-live.tsv) |
| wasm-notice-live | actual-png-pair | 24 | 97.45945% | [TSV](tsv/rebased/actual/actual-png-pair-wasm-notice-live.tsv) |
| grow-nested-native-saved | cross-native | 1 | 100.00000% | [TSV](tsv/rebased/actual/cross-native-grow-nested-native-saved.tsv) |
| grow-nested-wasm-saved | cross-native | 1 | 100.00000% | [TSV](tsv/rebased/actual/cross-native-grow-nested-wasm-saved.tsv) |
| notice-native-saved | cross-native | 24 | 97.69193% | [TSV](tsv/rebased/actual/cross-native-notice-native-saved.tsv) |
| notice-wasm-saved | cross-native | 24 | 97.69193% | [TSV](tsv/rebased/actual/cross-native-notice-wasm-saved.tsv) |
| grow-nested-native-saved | cross-wasm | 1 | 100.00000% | [TSV](tsv/rebased/actual/cross-wasm-grow-nested-native-saved.tsv) |
| grow-nested-wasm-saved | cross-wasm | 1 | 100.00000% | [TSV](tsv/rebased/actual/cross-wasm-grow-nested-wasm-saved.tsv) |
| notice-native-saved | cross-wasm | 24 | 97.45945% | [TSV](tsv/rebased/actual/cross-wasm-notice-native-saved.tsv) |
| notice-wasm-saved | cross-wasm | 24 | 97.45945% | [TSV](tsv/rebased/actual/cross-wasm-notice-wasm-saved.tsv) |

실제 저장 HWPX의 부모·자식 높이는 편집 후 각각 11748 HU, 무편집은 각각 8502 HU다. 원본 cellSz·live IR의 보존과 거부 시 원자성도 실제 검사에서 확인했다. 재열기 입력은 미리 만든 fixture가 아니라 이번 Native/WASM API의 저장 산출물이다.

## 최신 devel의 필수 편람 gate

Native 편람 383쪽 / 독립 Print 383쪽, 최저 raw **99.96465%**, 기준 대비 하락 **0쪽**, `--handbook-gate` passed다. ef6에서 실행한 동일 렌더링 source 결과를 재사용하며 handbook WASM을 실행한 것으로 표시하지 않는다. [전쪽 TSV](tsv/rebased/handbook-native.tsv) · [130쪽 review](rebase/png/handbook-p130-native-review.png) · [overlay](rebase/png/handbook-p130-native-overlay.png) · [272쪽 review](rebase/png/handbook-p272-native-review.png) · [overlay](rebase/png/handbook-p272-native-overlay.png).

## 글꼴 공급과 판독 범위

rowbreak 5쪽은 charPr 26의 hangul font 8인 TTF 휴먼명조를 사용한다. 주 측정은 `/home/edward/mygithub/ttfs/hwp/HMKMM.TTF`를 포함하는 경로와 공통 글꼴 경로를 명시했다. HFT 변환본이나 이름이 다른 TTC로 대체하지 않았다. 초기 광범위 경로의 Native 최저 92.16858%와 같은 설정의 이전 후보 대비 −0.71082%p 대조는 진단 결과이며, 위 주 표는 실제 원본 글꼴을 공급한 전 18쪽 결과다. private 글꼴은 경로·해시만 기록하고 배포하지 않는다.

점수는 완전한 픽셀 동일성이나 모든 기존 차이의 해소를 뜻하지 않는다. 조판·확대 판독 기록은 [직접 판독](direct-review.md)의 최신 devel 절에 연결한다. 이번 silhouette-only 실행은 `pixel_match`·`visual_accuracy_proxy_percent`·일반 `flagged`를 산출하지 않았으며 통과로 표시하지 않는다.

원 PR #8211의 실제 live physical 21쪽·측정 소유 4개 상태, #8235의 실제 Form01 HWPX 1쪽에 review/overlay를 대응시킨다. #8292의 ID·행 격자 직렬화와 #8299의 순수 vpos 단위 계약에는 다른 문서 이미지를 원본 증거로 제시하지 않는다. 동일한 제출 head CI·self-review·merge 후 옵션 2의 별도 기록 PR과 승인된 원 PR별 후속 처리를 진행한다.

[렌더링 source 동일성과 실제 글꼴의 SHA-256](rebase/source-and-fonts.md)을 연결한다.

## 제출 head 223 이후의 최종 보정

CodeQL 좌표 진단 3곳과 Native CLI의 과거 false 고정을 code `5e374d01f641745d20bc9424ce233021a327ed16`에서 보정했다. stock 12,475 PASS / 45 skip, Native CI 22개 target 합계 192 PASS, 별도 Native lib 전체 4,040 PASS / 12 ignored, Native 출처 11 PASS와 실제 보호 문서 정상 SVG/진단 부재를 완료했다. 아래의 시각 producer ef6/696은 조판 계산 동일성을 확인한 재사용 결과이며 이번 보정의 새 raster로 표시하지 않는다. [실패 원인·원 해결 이력·정확한 전체 Native 실행·재사용과 검증 한계](ci-head223-followup.md)를 따른다.
