# #8160 단계 2: 원본 글꼴 메트릭과 실제 배치 보정

## 첫 수정의 근거와 범위

독립 printable ASCII 전체 표본을 새 2024 MCP Print로 출력했다. 일반체와 굵은체의
Type3 `/Widths` 95개는 모두 같고 `<`·`>`는 605/1000em이다. job·입력/PDF SHA와
전체 값은 `../pr/assets/issue_8160/poppy-ascii-metrics.json`에 기록한다.
표본은 대상 문서의 내용·좌표를 사용하지 않는다.

현재 `font_metric_hci_poppy.rs`는 원본 일반체의 부분 표만 제공한다.
`font_metrics_data::find_metric_decision_for_character`는 굵은체를 제외해 Palatino
대체 메트릭을 선택한다. HWPX 41쪽의 `<` 전진은 10.6667px이고 독립 Print는
12.96px여서 뒤 한글 원점이 어긋난다. 독립 표본으로 확인한 일반체·굵은체 ASCII를
원래 em과 함께 공통 선택 경로에 제공한다. 공백 정책과 미확인 italic은 유지한다.
다만 `/Widths`를 실제 굵은 문자 전진으로 일반화하지 않는다. 아래 후속 표본의
검증 결과에 따라 굵은체 제공 범위를 정했다.

생산은 독립 원본 메트릭 표, 측정 소비는 `text_measurement.rs`의
`measure_char_width_embedded_decision_for_font` → `char_width_decision`이다.
줄 구성과 실제 글자 배치는 같은 측정 결과를 사용한다. 선택 쪽의 실제 render tree·PNG와
독립 Print 원점으로 최종 전진을 다시 확인하고, 개선을 확인한 후 전쪽을 재측정한다.
문서 ID·쪽수 조건이나 backend별 좌표 보정은 추가하지 않는다.

## 순서와 검증

현재 fresh WASM base 빌드는 최적화 중이며 종료를 확인한 뒤 공용 Cargo target을 사용한다.
Native 선택 경계와 대조군을 먼저 측정한다. 새 회귀 테스트는 세 문서 Native/fresh WASM
모든 쪽 raw 100%·미대응 0을 확인할 때까지 추가하지 않는다.

## 독립 표본의 실제 전진과 첫 보정 결과

base WASM 빌드·전쪽 측정이 완료된 후 Native 후보를 순차 빌드했다.
첫 후보는 굵은 ASCII 전체에 일반체 `/Widths`를 적용해 41쪽 raw 100%·미대응 0이 됐지만,
18쪽 96.07707%·38쪽 97.55109%로 하락했다. 이 후보를 최종 보정으로 채택하지 않았다.

문단 양쪽 정렬을 제거한 별도 전체 ASCII 표본을 12·16·20·32·52pt의 일반/굵은체로
출력했다. 각 스타일은 ASCII 전체를 세 번 순회하고 kerning은 꺼져 있다.
새 MCP job `5ea451f8-4b75-4816-84ac-b51474acaf75`는 실제 2024 Print 0·9쪽으로 성공했다.
입력/PDF SHA와 2,697개 실제 전진 관측을 묶은 927개 통계는
`poppy-ascii-neutral-advance-evidence.json`·`poppy-ascii-neutral-advances.tsv`에 기록했다.
굵은 영문은 Type3 `/Widths`와 실제 전진이 다르다. 기존 굵은 스타일 표를 보존하고,
원본 폭과 실제 전진의 일치를 확인한 기호만 부분 표로 제공했다.

후보 2 Native build는 exit 0, 6분 8초다. 실제 CLI SHA-256은
`b0c61492b111045efc0bb5ed9d81a38f77ca3e269e60ddeafd7ffb1a82d36025`다.
선택 쪽 2·6·18·38·56은 base raw와 같고, 41쪽만 97.73925%에서 100%로 개선됐다.
첫 후보의 41쪽은 임계값 232·반경 2px로 별도 계산한 미대응도 정확히 0이며,
실제 compare PNG를 열어 제목·목차 본문을 판독했다. 전체/후보 WASM 검증은 아직 미완료다.

## 목차 탭 목표의 공통 보정

2쪽 원본 문단 27의 마지막 탭 정지는 43860HU(584.8px)다. 단 왼쪽 56.6933px와
합한 오른쪽 정렬 끝은 641.4933px이며, 독립 PDF의 숫자 원점은 632.8px다.
Native는 약 628.2px다. 기존 `right_leader_tab_target_rel`와 교차 run의 측정·배치가
리더 여백 0.25em을 정렬 목표에서 빼면서 내용까지 왼쪽으로 옮긴다.
여백은 리더 잉크 끝의 계약이고 정렬 내용의 오른쪽 변은 탭 정지의 계약이다.
공통 `leader_tab::right_aligned_target`가 목표를 생산하고 문자 측정·run 폭 측정·
실제 run 배치가 같은 목표를 소비하도록 보정한다. 자동 우단 경로는 같은 helper로 보존한다.
기존 대형 호출 파일은 helper 연결만 바꾸고 새 helper는 1,000줄 이하로 분리한다.

원 도입 commit `b4935a5b07828658b9bb6ea541021ca7e8077961`은 PR #1120의 실제 merge
`f7a68bc319ed67ace974722d324e49339954d121`로 반영됐다. GitHub PR의 merged 상태·SHA와
로컬 first-parent 통합 경로를 대조했다. run 방출 추출 `897b2d00d9`는 PR #2176의
`93862a4e16df59834ebce46d91e948cd739208e9`에 포함된다.

## HWPX 18쪽 흐름의 추가 진단

그림 원본 바이트·ImageNode의 crop·imgDim·bbox는 2024 HWP와 같다.
차이는 저장 줄 없는 문단 395 뒤에서 시작한다. 원시 HWP의 문단 396 첫 vpos는 57608이고,
HWPX도 같은 값을 저장했지만 로드 재구성이 57324로 바꾼다.
공통 HeightCursor 진단에서 측정과 paint 모두 이미 맞는 순차 위치를 재구성 좌표로
3.7867px 되당긴다. 원본 좌표 snapshot과 재구성 좌표의 축·후속 원점 소유를 추적하며,
단순히 최종 y에 상수를 더하지 않는다. 이 흐름 보정은 아직 구현·재검증 전이다.

실제 원점의 독립 대조도 확보했다. 396번 문단의 순차 입력 y는 이미 맞는
839.9467px이고 앞 간격 3.7867px를 더한 첫 줄은 원본 vpos 57608과 같다.
하지만 재구성 vpos 57324를 기존 단 기준 0에서 해석해 836.16px로 되당긴다.
새 보정은 합성 텍스트 뒤의 유효 원본 snapshot이 실제 수용한 첫 줄과 같은지 확인하고,
그때만 재구성 좌표축의 기준을 다시 연결한다. 측정·paint의 공통 HeightCursor에서 수행하며
원본·IR의 줄 좌표와 실제 흐름 y를 바꾸지 않는다. 후속 줄도 같은 새 기준을 이어받는다.
원본 첫 줄과 수용 위치가 다르거나 편집·개체·다른 원점 정책인 경우에는 적용하지 않는다.

후보 4 Native는 exit 0(8분 37초), CLI SHA는
`5355a7739dd57b763c3cc7ad8eebba6a6fbe93611a89b1ba2fb49c5b9947507d`다.
HWPX 전 64쪽에서 18쪽만 96.24353% → 99.08287%로 개선됐고 나머지는 후보 3과 같다.
아직 18쪽에 그림 내부와 기호 차이가 남아 있으므로 완료로 판정하지 않는다.

## 교차 run 목차와 TAC 표 앞 글자 폭

2쪽의 오른쪽 정렬 목표를 복원한 뒤에도 Native 미대응 288개가 남았다.
대부분 마지막 리더 점이 독립 Print의 끝보다 오른쪽에 더 그려진 것이다.
원인은 `"\t "`와 다음 숫자 run을 따로 재면서 리더 앞쪽 측정에 숫자 폭이 빠지는 것이다.
기존 `right_tab_block_width_override`를 공통 생산 결과로 사용해 분리된 뒤쪽 숫자까지
합산하고, 측정·문자 위치·실제 run 배치가 동일한 자연 폭을 소비하도록 수정한다.
자동 우단 탭의 기존 합산도 같은 생산자로 옮긴다.

38쪽 표 앞의 원본 일곱 공백은 13.3333px 크기의 반각 전진을 가진다.
기존 TAC 문단은 문자마다 `estimate_text_width`를 호출해 6.6667px를 7px로 반올림했고,
표 앞 폭을 49px로 만들었다. 독립 Print의 표 왼쪽은 약 105.064px인데 Native는 107.6px다.
새 `InlineTableTextMetrics`는 원래 언어 이어받기를 적용한 반올림 전 문자 폭을 한 번
생산하고, 정렬 합계·말미 공백 제외·실제 펜 전진이 같은 배열을 소비한다.
표 너비·바깥 여백·저장 줄 소속과 줄 높이의 기존 계약은 보존한다.

이 경로의 문자별 정수 측정은 `ea564999e1`에서 도입됐으며 first-parent 통합은
`39d90d9d9997ed3114f2e41ca20886dfeeef8e36`(`Merge local/devel: task #955 rustfmt policy`)다.
저장 탭 추가 `82479c6463`은 PR #7832의 merge
`bfa28a77a2714482f647ee6a7b9aa7f7876d8404`로 통합됐다.
새 회귀 검사는 아직 추가·수정하지 않았다. 기존 검사의 잘못된 기대값과 중복은
전쪽 100%의 선행 조건을 충족한 뒤 독립 근거에 따라 정리한다.

진단 중 `cargo fmt --all`은 현재 작업과 무관한 기존 generated suite가 삭제된 case를
참조해 실패했다. 소스의 변경 범위는 확인했으며, 최종 검증에서는 canonical review
worktree의 suite prepare 후 전체 fmt·정책 검사를 다시 수행한다.

## 원본 기호 프로그램과 WMF 전진폭의 추가 원인

U+2024는 원본의 기호 슬롯 `한양신명조` HFT이고 전각으로 전진한다.
별도 빈 HWPX에 General Punctuation 블록의 모든 할당 그래픽 문자 71개를 넣어
새 MCP job `af88fa0d-88ca-4cc3-9056-f94f177df6b7`로 2024 Print 0 출력했다.
1쪽 PDF SHA는 `65292153af96d18b2feeab1d2192f9972604579097747e901589d02972dd31cf`다.
이 독립 표본의 Type3 45개 윤곽을 기존 독립 전체 한글 CFF에 원형대로 추가했고,
Type0로 대체된 26자는 추출하지 않았다. 대상 PDF는 생성 뒤 읽기 전용으로 대조했으며
U+2024 원본 프로그램이 완전히 같다. 글꼴/PDF는 로컬에만 보존한다.

같은 원본 기호를 공급해도 기존 공통 `forces_halfwidth_punct_paint`가
U+2018–U+2027 범위를 한꺼번에 절반으로 누르므로 U+2024 점이 왼쪽으로 좁혀진다.
측정에서 실제 반각/협폭 보정을 선택했는지에 따라 paint를 결정하도록 수정한다.
문자 범위만 보고 측정과 별도로 다시 좁히지 않는다. 이 helper는 Native·WASM이 공유한다.

WMF 그림은 원본의 explicit Dx가 있는데도 글리프 추정 폭을 빼고 음수를 0으로
자른다. 더구나 CP949의 한글 두 바이트에 대응한 `[117,0]` 등을 해독한 글자 순번으로
잘못 읽는다. 독립 [MS-WMF META_EXTTEXTOUT](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-wmf/7d07c44a-a828-4b82-9af0-e0a81cced5a8)는 Dx를 문자 셀 원점 사이의 부호 있는 논리 전진으로 정의한다.
새 공통 변환은 바이트 왕복 일치로 글자와 원본 셀을 연결하고 누적 Dx를 장치 좌표로
변환해 SVG 원점에 전달한다. ETO_PDY·glyph-index·해독 왕복 불일치의 기존 경로를
임의로 같은 계약으로 간주하지 않는다. 이 변환 결과는 Native·WASM에서 함께 소비한다.
원본 그림 3·5에는 WMFC enhanced payload가 없음을 원시 레코드로 확인했다.
수정 전 clamp 보정 `18a9fa85e9`는 PR #7263의 squash 통합이다.


## 후보 5·6 검증의 진행 상태

후보 5 build는 exit 0, 9분 13초다. 같은 기존 글꼴 환경의 Native 전 192쪽은
HWPX 최저 98.27803%, 2024 HWP 최저 98.36526%, 2022 HWP 최저 95.03337%다.
HWPX·2024 HWP의 41·56·58·63·64쪽은 raw 100%이고 2022는 아직 없다.
전쪽 원본 TSV·base 비교·CLI SHA는 `candidate5-native-summary.json`과
각 `*-candidate5-native-silhouette.tsv`로 보존한다. 2022의 33쪽은 0.09461%p 하락했으므로
원인을 이어서 추적한다. 이 결과를 100% 완료로 분류하지 않는다.

후보 5 진단용 rlib를 복사할 때 root alias가 오래된 2026-10-10 산출물인 것을 확인했다.
실제 CLI가 링크한 `target/pr-review/release-test/deps/librhwp.rlib`로 교체했고
producer 기록에 정정했다. CLI·페이지 측정에는 영향이 없다.
이후 rlib는 항상 실제 deps 경로에서 동결한다.

후보 6 첫 compile은 이동된 `record.rectangle` 뒤에서 record를 빌려 E0382로 실패했다.
독립 전진 결과 생산을 이동 전으로 옮겼고 재빌드는 exit 0, 6분 14초다.
CLI SHA는 `a487e09adb7f6a9adc185e8e909b5b1795045d64817fb6354893f36c8f61bcc0`이다.
그림 3의 원시 WMF 554개 ExtTextOut 레코드를 별도 Python으로 해석해,
CP949 바이트 묶음의 누적 Dx와 실제 방출 SVG의 모든 글자 원점을 대조했다.
554개 모두 일치하고 실패는 0개다. 실제 PNG 일치는 별도로 확인한다.

별도 원본 HFT 공급은 최우선 `--font-path`로 넣어야 한다. 후순위에 넣으면 같은 family의
기존 공급이 먼저 선택되므로 실제 원 프로그램 공급을 증명하지 못한다.
전체 ASCII 독립 표본의 일반·굵은 Type3 95개는 경로와 intrinsic Widths가 모두 같고,
굵은 프로그램만 20/1000em stroke+fill을 덧붙인다. 기존 합성 굵기 계약과 같다.
따라서 일반체 원본 CFF를 공급하고 기존 합성 굵기를 유지한다.
전체 굵은 실제 전진을 Type3 Widths로 바꾸지 않는 앞선 판정은 유지한다.

원본 한양 HFT(전체 현대 한글+독립 구두점 45개), 원본 Poppy ASCII 95개,
검증된 Windows Gulim과 Human TrueType을 공급한 후보 6의 선택 쪽에서
HWPX 3·38·41쪽은 raw 100%다. 3·38쪽은 별도 대칭 미대응 계산도 양방향 0이다.
글꼴/PDF 바이트는 로컬만 보존하고 생성 절차·SHA·MCP job을 공개 증적에 연결한다.
새 공급 환경의 전쪽 base/head 비교와 fresh WASM은 아직 미완료다.

원본 HFT 우선 공급의 후보 6 HWPX 전쪽 결과는 64쪽 중 28쪽 raw 100%,
최저 98.29575%다. 원본 TSV는
`sample16-hwpx-candidate6-original-font-native-silhouette.tsv`로 연결했다.
이 새 환경을 이전 기본 글꼴 TSV와 합치거나 기본값의 전쪽 완료로 주장하지 않는다.

수식은 별도 빈 HWPX의 48개(두 face·세 크기·CJK/Latin 일반 줄·분수·괄호 등)를
너비·높이·기준선 0에서 한컴 저장하고 다시 Print 했다.
`Equation Version 60` 입력 SHA는
`eede5c2dcacb3a11c0c14026ec6c7feecadc06ff7a8652a4a585a843bc0c7ac3`이며,
저장 job `e8ba4dc7-db43-497d-b99a-37bc6d2ff03c`, Print job
`6a8754eb-0df4-4c72-b930-bd1d4a3cdcd6`이다.
버전만 빈 문자열로 바꾼 독립 입력 SHA는
`f17bacbf5bda3df68658832f807241d6a9bfffdbcdc26bb6e2031bd394376e1e`,
저장 job `6f9ff782-9799-4453-a568-7c89da8f4f16`, Print job
`c833e7c1-7732-4379-bf65-4573c7f57ac1`이다.
후자의 Print SHA는 `63028647e49bea06253bcd20d5d35877a1075a4c8b5a2ba0967e54f327019a19`다.
한컴 2024 13.0.0.3901·Print 0의 6쪽 출력이며, 이 자료는 아직 진단용이고 회귀에 등록하지 않았다.

빈 옛 버전은 두 font 이름에 동일한 옛 Type3 프로그램과 프레임을 사용한다.
12pt 분수의 정상 저장은 높이 2808HU·기준선 67%이고 원본 sample16과 같다.
현대 버전은 2760HU·66%다. 원래의 비어 있던 캐시에서 한컴이 생성한 값이므로
임의 캐시나 rhwp의 계산값을 정답으로 재사용한 것이 아니다.
글리프 프로그램·구조 프레임·버전 소유를 함께 연결하는 공통 레거시 수식 경로를 다음으로 보정한다.
