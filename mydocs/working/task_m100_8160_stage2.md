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

## HWPX 저장 줄 없는 긴 어절의 끝 자간 진단

22쪽 원본 문단 446은 HWPX에 저장 LineSeg가 없다. 원본 2024 HWP는 시작
`[0,54,108,160]`, HWPX의 재구성은 `[0,54,109,161]`이다. 독립 Print는 앞의 줄 소속과
일치한다. 배치의 자연 메트릭·문자 모양은 같지만 HWPX가 한 글자씩 더 담아,
단어 공간의 정렬 배분과 뒤 두 줄을 바꾼다. 2024 HWP의 같은 문단은 이미 raw 100%다.

줄 나눔의 긴 어절 fallback은 글자 전진폭을 바로 fit 폭으로 사용한다. 일반 Text 경로의
`FitWidthHwp::trimmed`는 끝 글자의 실제 자간을 제외하므로, 자간 -20%인 이 입력에서
두 경로의 수용 판정이 다르다. 다음 후보는 fit 폭 생산을 공유해 fallback도 같은
끝 자간 계약을 소비하도록 수정하고 실제 출력에서 확인한다. 펜 전진폭·저장 줄·원본은
바꾸지 않는다. 작성 줄 끝의 압축 자간을 보존하는 기존 경계도 같은 helper로 유지한다.

## 2022 본문의 추가 원인: 전각 기호의 측정 축소

새 MCP에서 원본 2022를 2020 HWP로 저장한 진단 job
`55bdb191-47b1-4883-b54b-0e40c8c1d9c5`도 별도로 읽었다. 독립 CFB/레코드 읽기에서
최상위 문단 수는 양쪽 1,058개, 같은 인덱스의 텍스트 1,056개가 같고,
LineSeg가 바뀐 문단은 1개뿐이다. 40쪽 문단 633의 원시 LineSeg·텍스트는 완전히 같다.
따라서 이 본문을 글꼴 변경에 따른 재조판으로 설명하지 않고 저장 줄 계약을 보존한다.

문단 633의 U+2024는 독립 Print에서 전각(자간 -12% 적용 뒤 약 15.2px)이다.
Native는 TTF 선언의 메트릭 hit에 `U+2018..U+2027` 일괄 반각을 적용해 약 7.6px로 잰다.
이 부족 폭을 양쪽 정렬의 공백에 재배분해 앞뒤 본문까지 밀린다. 실제 관측은
Native 기호 x=498.9·뒤 본문 x=506.5, Print 기호 x=494.4·뒤 본문 x=509.76이다.
새 후보는 한점 리더를 그 일괄 반각 집합에서 제외해 원 프로그램 폭을 측정·paint가
같이 소비하게 한다. 폭이 작은 글꼴의 본래 U+2024 전진은 그대로 사용한다.
이 조치는 Gulim 글리프 윤곽이나 원본 저장 줄·입력·PDF를 변경하지 않는다.

## 후보 6 WASM·후보 7 기각·후보 9의 추가 검증

후보 6의 격리된 `328ac4d4420f47969f7fff12586072f33baf4982` 소스에서
최적화 WASM 빌드가 exit 0(32분 43초)으로 끝났다. 원본 글꼴 공급을 고정한 전 192쪽에서
HWPX는 35쪽, 2024 HWP는 36쪽이 raw 100%이고 2022 HWP는 아직 없다.
WASM SHA는 `270285da60271a100d2a15d40b496e5a81c8d642006d559deeb8db186a0cf6e8`이며
격리 worktree의 Studio public 사본도 같다. 최종 후보의 WASM 검증으로 대체하지 않는다.

후보 7은 U+2024 반각 범위와 긴 어절 fallback의 마지막 자간을 수정해 빌드했지만,
선택한 대상 페이지의 출력이 후보 6과 같았다. 이 두 비인과적 변경은 원래 상태로 되돌렸고
기각 patch·소스·실행 파일·TSV는 로컬 후보 7에 보존했다. 회귀 검사는 바꾸지 않았다.
첫 no-native-skia 빌드는 용도를 확인하고 중단했고, 첫 native-skia 실행은 세션 중단으로
최종 결과가 없었다. 실제 Cargo/Rust 프로세스와 handle이 없음을 확인한 뒤 재실행한
native-skia 빌드가 exit 0(6분 12초)이다. 해당 성공 산출물만 후보 7로 고정했다.

2022의 점 문자는 원본의 TTF 선언 때문에 `metric_font_family=None`,
`hft_fullwidth_dot=false`이고 반각 범위 코드까지 도달하지 않는다. 별도 빈 문서의
General Punctuation 전체 그래픽 문자를 같은 한양신명조 **TTF 선언**으로 바꾸고 새 MCP
2020 Print 0 job `acb1ceee-ba4c-4b47-98c2-9cf359b916ff`를 실행했다.
PDF SHA는 `ba7eeb2524efdd321acc281c7791f49e9d2546797d6ecf071a1c31c105fe2725`이고
U+2024 Type3 프로그램은 앞서 독립 HFT 선언 2024 출력과 완전히 같다.
원 프로그램 SHA도 양쪽 모두 `69952c4a7d4d1969235a4e2e68fadaa67c1d8b01a668413db32bda011c92b5d7`이다.
명시적 환경 치환으로 확인한 원 HFT face를 공급하면 기존 HFT 폭 표와 전각 점 속성을
공통 style resolver에서 함께 선택하도록 수정했다. 원본 TTF/HFT 선언은 변경하지 않는다.

반각 범위 코드의 선행 `950de31563667b8ca5729ab7165461f88b183528`은
실제 PR #5122 merge `0bc05ef81107ac61ec38d622f71b44a44d1b4821`에 포함된다.
셀 압축 코드는 최초 저장소 commit부터 있었고 추출 `052ef6295b`는 PR #2176 merge
`93862a4e16df59834ebce46d91e948cd739208e9`다. soft-wrap 공백 보정 `d9d17780aa`는
PR #8064 merge `9b01d5d1447220b3a7345703e79ad42cf3f502f4`, 가운데 점유 보정
`88abe494f7`은 PR #7832 merge `bfa28a77a2714482f647ee6a7b9aa7f7876d8404`다.

## 셀 말미 공백과 옛 수식의 공통 생산 결과

60쪽의 본문은 실제 잉크가 셀 안에 들어가지만 말미 공백을 포함한 전진이 셀 폭을 넘었다.
기존 압축은 이 공백까지 맞추며 앞 글자에 음수 자간을 배분한다. 빈 표에서 평문 말미 공백
0·1·3개와 문단 끝/강제 개행을 교차한 독립 6개 셀을 새 MCP 2024 Print 0
`3747ad17-b004-44dd-9cde-18a095697f5f`로 확인했다. 모든 본문 글자 원점이 동일하다.
강제 개행 앞 공백은 추가 빈 줄도 만들 수 있으므로 그 실제 전진·줄 소유는 보존한다.
새 공통 판단은 가시 본문이 맞는 경우 평문 말미 공백만으로 앞 글자를 압축하지 않는다.
밑줄·취소선·필드·겹침 글자·개체가 소유한 공백을 평문으로 취급하지 않는다.

옛 수식은 빈 version과 알려진 두 face의 한글·숫자·일반 연산자·단순 분수·괄호를
독립 빈 문서로 확인했다. 한글 줄은 1.2em/84%, 단순 분수는 2.34em/67% 프레임이며
현재 Equation Version 60과 다르다. 로컬에만 둔 완전한 로만 알파벳·숫자·연산자 표본에서
75개 원 프로그램을 복원했고, 대상 수식의 숫자·연산자·괄호 14개와 프로그램을 대조했다.
영문 전진·그리스 문자·첨자 등 미확인 구조나 한쪽만 띄어 쓴 연산자 조합은 옛 프레임으로
추정하지 않는다. 글리프 표시·흐름 폭·높이·기준선은 공통 layout 생산 결과를 소비한다.
수식 구조 보정 `38cb1ec05ea9`도 위 PR #8064 merge로 실제 통합됐다.

후보 8에서 HWPX 6쪽은 98.29575% → 99.88790%, 60쪽은 99.67990% → 99.99839%다.
60쪽의 실제 미대응은 Native 0/PDF 1개다. 괄호의 남은 차이는 PDF 추출 span 크기를
균일 배율로 읽은 원인이다. 독립 원점 0·대소문자/공백 대조 출력의 원시 Type3 명령은
12pt 괄호를 `Tf=203`, `Tz=49%`로 그린다. span의 17.052pt는 기하평균이므로 세로·가로
확대를 따로 소비해야 한다. 이 변환을 FontGlyph의 공통 paint 결과와 Native·SVG·Canvas2D·
CanvasKit에 전달한 후보 10을 빌드/검증 중이다. 후보 9까지는 이 변환이 없다.

후보 9 Native 빌드는 exit 0(6분 44초)이고 전 192쪽 TSV가 생성됐다.
HWPX 최저 98.37964%(100% 28쪽), 2024 HWP 최저 98.54009%(100% 29쪽),
2022 HWP 최저 97.07654%(100% 0쪽)다. 2022의 40쪽은 96.12086% → 97.07654%이고
이제 점 문자를 포함한 위쪽 본문은 맞는다. 남은 차이는 저장 LineSeg가 없는 원본 문단의
U+119E다. 실제 Print는 HCRDotum에서 0 전진으로 그려 다음 `박`과 같은 원점을 쓰는데,
현재 Native의 Gulim 측정은 전각으로 전진해 뒤 글줄 경계가 한 글자씩 달라진다.
Gulim 원 프로그램에도 U+119E가 있지만 전각이고 HCRDotum의 같은 글자는 0 전진이다.
따라서 cmap 결측이나 일괄 글꼴 재조판으로 설명하지 않고 독립 자모 표본을 추가 조사한다.

원본 2022와 독립 2020 저장본의 원시 레코드는 1058개 문단 중 줄 레코드가 한 군데만
달라지고 그마저 줄 시작이 아니라 140번 문단의 두 번째 vpos다. 641번 문단은 양쪽 모두
LineSeg가 없다. HWPX 22쪽 문단 446의 원문/글자모양/가용 폭도 대응 HWP와 같지만
생성한 시작 `[0,54,109,161]`은 저장 `[0,54,108,160]`과 아직 다르다.
현재 단계에서 전쪽 100% 또는 회귀 검사 정리 완료를 선언하지 않는다.

## 원본 글꼴의 공급 범위·출력 세대와 정수 정밀도

후보 10에서 옛 괄호의 비균등 변환을 반영한 뒤 수식 영역의 실제 미대응은 0이다.
6쪽에 남은 2픽셀은 본문의 Poppy U+201D, 60쪽의 1픽셀은 한양신명조 `%`였다.
대상 PDF에서 글꼴을 추출하지 않고 빈 문서의 Basic Latin·Latin-1·General Punctuation
전체 그래픽 문자를 새 MCP 2024 Print 0으로 출력해 기존 독립 복원 글꼴에 공급했다.
작업 ID는 한양 `7da242dd-a55a-4d45-bd18-477cbf05a835`, Poppy
`25d54808-a367-452e-a6be-17b79daa8698`이다. 원본 글리프의 읽기 전용 대조를 거쳐
후보 11에서 HWPX·2024 HWP의 6·60쪽은 실제 양방향 미대응 0, raw 100%다.
새 글꼴 조건은 기준 Native·WASM 384쪽에도 동일하게 공급했다. 기준은 양쪽 모두
HWPX 11쪽·2024 HWP 11쪽·2022 HWP 0쪽이 raw 100%다.

자모를 일괄 0 전진으로 처리한 후보 11은 2024 문서 40쪽을 후퇴시켜 채택하지 않았다.
Gulim·휴먼명조·HCRDotum·한양신명조와 자모 8개 조합의 독립 32개 표본을 2020/2024
양쪽 Print로 출력했다. 2020은 단독 중성·종성을 HCRDotum의 0 전진 프로그램으로
선택하지만 2024는 기존 spacing 프로그램을 유지한다. 명시적 HCRDotum은 양쪽 모두
0 전진이다. 후보 12는 이 출력 세대·선택 프로그램 차이를 공통 style 결과로 전달하고,
문자 자체와 소유·오프셋은 보존한다. 2020/2024 작업 ID는 각각
`c87763e0-43e0-42ab-80f6-9f4b0f3e76e5`, `64b67728-cd47-40a0-8b50-bb011cdac8d7`이다.
후보 12 Native 전 192쪽은 HWPX 40쪽·2024 HWP 41쪽·2022 HWP 0쪽이 100%이며,
2024의 40쪽도 100%로 복구됐다. 아직 전쪽 완료가 아니다.

22쪽의 독립 혼합 문자 줄바꿈 경계 21개를 추가 출력한 결과, Poppy 대문자·소문자·
혼합 문자열의 폭 차이는 각각 0·12·8 HWPUNIT이었다. 설치된 원본 HFT를 읽기 전용으로
확인하니 em은 512이고, 현재 PDF의 em=1000 정수 표는 원래 폭을 내림해 손실한다.
소문자 b/e/j는 1800dpi 조판에서 각각 4 HWPUNIT이 달라 위 독립 경계와 일치한다.
일반·굵은 원본 HFT 폭은 기존 독립 Print 다섯 크기의 전진 관측 927개와 대조해
모두 600dpi 원점 양자화 한 단계 이내다. 수치·원본 SHA·독립 작업 ID는
`mydocs/pr/assets/issue_8160/poppy-original-widths.json`에 기록한다.
후보 13은 표시 윤곽과 별도로 이 원래 단위의 논리 전진을 사용한다.

2022의 쪽번호는 명시적으로 원 HFT 프로그램을 공급해 폭을 선택했지만 TTF 원본 선언만
보고 기준선 여유를 1/3em으로 적용했다. 동일 HFT를 쓴 2024의 기존 20%em 여유와 달라
1.7778px 위에 놓였다. 후보 13은 실제 선택된 메트릭 프로그램을 쪽번호에도 전달한다.
기존 근거는 PR #7941 `84764fa21c`·PR #8120 `80968497c2`와
PR #7505 merge `e5098bc91be44a49367a7f2895a14fcd4f4c2c7f`다.

WMF의 source blit은 목적 사각형 전체에 늘려야 하지만 SVG image의 기본 meet로
원본 비율이 유지됐다. 원시 레코드와 그림 3의 복원 SVG에서 이를 확인했다.
[MS-WMF META_STRETCHDIB](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-wmf/7ebae08d-61ee-4d82-9aa5-9217ba2aa8c1)의
독립 목적 폭·높이와 [SVG preserveAspectRatio](https://www.w3.org/TR/SVG/coords.html#PreserveAspectRatioAttribute)를
따라 source blit의 공통 SVG 생산에 `none`을 지정한다. 음수 크기의 정규화·clip·transform은
기존 계약을 그대로 소비한다. 이 누락은 최초 commit부터 있었으며 음수 정규화
`7a8eb00656a9`(#6140), 미지원 ROP `b061a2a20877`(#6469), 후속 PR #8064의
`6c8bd31e2c7a`에서도 보존됐다. 빈 문서의 독립 GIF 테두리 12개와 비균등/음수 WMF
사각형을 만들어 새 MCP 출력과 대조 중이다. 전쪽 100% 확인 전 회귀 검사는 변경하지 않는다.

후보 12의 최적화 WASM은 exit 0으로 완료됐다. Rust release 빌드는 17분 43초이고,
후속 wasm-opt도 완료됐다. `rhwp_bg.wasm`의 SHA-256은
`fa86ed0772be3bbf37b3984856cda6f782b65f8980b11686f9886fc9c212c27f`, glue는
`da00f4937f352731a2d24a1ba4b54629009aa5edc9c929815aed2cd069cca3bc`다.
worktree의 pkg와 Studio public 일치를 확인하고 독립 후보 디렉터리에 동결했다.
후보 13의 세 변경은 포함하지 않는다. 이 WASM의 전 192쪽 검증을 시작했다.

독립 WMF Print `d15ae916-f983-4861-b369-186fe8f9448d`(2024, Print 0)의 PDF SHA는
`e2317f644c8db53059d5e66e54e3593e1be702e956fed42f3f3c8eddc5c3f514`다.
양/음수 목적 크기 4개의 실제 색 면은 모두 목적 사각형을 채운다. 기존 SVG는 가로로 긴
사각형에서 가운데 정사각형만 칠하고 세로 사각형에서도 같은 오차를 낸다. 공통 `none`
지정의 독립 래스터는 양쪽 축·반전 위치를 보존해 Print와 일치한다. 아직 이 수정의 대상
문서 전쪽 결과는 산출 전이며 후보 13 Native 빌드에 포함했다.

GIF도 위치 오차와 원본 색 전달을 나눠 독립 조사 중이다. 12개 크기 변화 Print
`2c290f48-0c6f-40f9-8be2-8dcf12e3c7cf`에서 실제 JPEG 양자화 표는 표시 폭의 96dpi
정수 장치 폭과 원본 폭으로 결정되는 품질(35~75)에 모두 대응했다. 단순 팔레트는 PNG로
나오는 반례도 확인했으므로 모든 그림을 일괄 JPEG로 바꾸지 않는다. 표시 사각형의
래스터 경계 처리도 차이가 있어 원본 로고의 남은 차이를 이 가설 하나로 완료 처리하지 않는다.
이 조사로 production bitmap 코드는 아직 바꾸지 않았다.

후보 12 fresh optimized WASM 전 192쪽도 exit 0으로 완료됐다. HWPX 40쪽·2024 HWP
41쪽·2022 HWP 0쪽이 100%이며 최저는 각각 98.39522·98.49132·98.51210%다.
후보 13 Native 빌드는 exit 0(9분 38초)이고 원래 Poppy 폭 단위로 HWPX 22쪽이 100%가
됐다. 전쪽 결과는 HWPX 41쪽·2024 HWP 41쪽·2022 HWP 31쪽이 100%이며 최저는
각각 99.07709·99.07709·99.19417%다. WMF 목적 비율 수정으로 18쪽 99.72425%,
35쪽 99.07709%까지 개선됐지만 내부 글자와 일부 raster 전달은 남았다.
CLI SHA는 `4d4758a2b725d1b1debfc059772f305e6801251c4c3adc1a1c7c206f3a9c6f1b`다.

목차의 원본 TAC 사각형이 있는 줄에서, TAC 앞/뒤로 나눈 TextRun 모두가 전체 run의
`tab_leaders`를 복제해 두 점선을 중복 출력했다. 27번 문단은 `추진목적` 조각과
` \t ` 조각이 서로 다른 원점으로 같은 범위를 칠했다. 최초 commit의 TAC 조각 생성부터
있던 누락이고 `052ef6295b`의 추출(PR #2176 merge
`93862a4e16df59834ebce46d91e948cd739208e9`) 이후에도 유지됐다. 후보 14에서는 각
TAC 조각의 리더를 비우고 탭이 있는 조각만 기존 공통 계산기로 다시 생산한다. 빈 꼬리
TextRun도 점선을 소유하지 않는다. Native 빌드 중이며 회귀 검사 변경은 아직 없다.

51쪽의 원 기호 두 개는 검증용 글꼴에 U+25CB가 없어 더 큰 대체 글꼴을 쓴 차이다.
Basic Latin·기호류 U+2100~27FF·CJK 구두점·한중일 호환/둘러싼 기호·전각 블록의
2801개 그래픽 scalar를 빈 문서에서 새 MCP Print 0으로 출력했다. 한양/Poppy는 각각
`8d9e3343-e38e-47b6-b011-84fd41bdcbda`, `a66978d7-64af-41bb-ae1d-95ba751fa681`이며
7쪽씩 성공했다. Type3 원 프로그램만 기존 독립 글꼴에 확장하고 Type0 fallback은
추출하지 않는다. 글꼴 조건이 달라지므로 기준 Native/WASM에도 같은 공급을 적용한다.

독립 전체 기호에서 한양 1404개·Poppy 1370개 Type3 프로그램을 보완했다. 대상 세 PDF의
기호 프로그램 33개씩을 읽기 전용 대조하니 모두 독립 윤곽과 일치한다. 굵은 11개는 같은
윤곽에 `20 w / B` stroke를 추가한 표현이므로 바이트 동일 주장 대신 윤곽·stroke를 나눠
기록했다. 후보 13의 새 글꼴 조건에서 13·51쪽이 실제 raw 100%다. 후보 14 Native는
exit 0(6분 32초), 이 조건의 전쪽 검증에서 HWPX·2024 HWP 44쪽씩, 2022 HWP 34쪽이
100%다. 아직 최저 99.07709·99.07709·99.19417%로 작업을 계속한다.

### packed DIB 색상표를 픽셀로 읽은 원인

그림 3의 605번 DIBSTRETCHBLT 원시 레코드를 rhwp 모델을 거치지 않고 추출했다.
180×145·8bpp·BI_RGB·ColorUsed=0의 packed DIB는 RGBQuad 256개 뒤에 픽셀이 온다.
Pillow로 원 DIB를 독립 디코드한 사진은 단일 서버인데, rhwp SVG의 PNG는 사진이
가로로 잘려 있었다. PNG의 여러 행을 원본 RGB 버퍼에서 찾으면 모두 1136픽셀 뒤의
데이터와 일치한다. 파서는 `DIB_PAL_INDICES`를 강제해 색상표를 생략하고 그 바이트를
픽셀 버퍼로 읽고 있었다. 이 누락은 최초 commit부터 DIBBITBLT·DIBSTRETCHBLT에 있었다.
후보 15는 packed DIB의 RGB 색상표를 먼저 소비한다. 원본의 좌표·크기를 움직이지 않는다.
[MS-WMF DIBSTRETCHBLT With Bitmap](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-wmf/3306ee6b-e7c1-4bef-af95-a75a040afadc)과
[BitCount](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-wmf/792153f4-1e99-4ec8-93cf-d171a5f33903)의
1·4·8bpp 색상표 설명을 따른다. 24bpp BI_RGB는 색상표가 없으므로 동일한 버퍼를 쓴다.
새 독립 packed DIB 표본과 Native 빌드로 확인 중이다. 테스트 기대값을 낮추지 않는다.

후보 15 Native는 exit 0(6분 05초)이다. 원 DIB의 독립 픽셀 SHA와 수정한 SVG PNG의
디코드 픽셀 SHA는 모두 `7ef3c56b06db2684afdd5791ee7a393f030c13fcdf425444dee41c43bb5a76f3`로
완전히 일치한다. 전쪽 검증에서 18쪽은 raw 100%, 35쪽은 99.97740%다. HWPX·2024 HWP
45쪽씩, 2022 HWP 34쪽이 100%이고 최저는 99.78091·99.78091·99.19417%다.

전각 물결표 U+FF5E는 CFF 공급 부족이 아니라 언어 슬롯 오분류였다. 빈 문서에 서로
구별되는 7개 슬롯 글꼴과 상대 크기(100/120/130/140/150/160/170%)를 지정해 출력했다.
전각 ASCII FF01~FF60·통화 기호 FFE0~FFE6의 103자 전부가 2020·2024 양쪽에서
기호 슬롯의 160% 크기를 사용했다. 작업은 각각
`71e4cb55-b33b-43db-9dfd-a7d5d2270ae4`, `ab67656d-f814-42db-84d9-46715123832d`다.
후보 16은 이 두 하위 범위의 분류를 기호 슬롯으로 고쳤다. 빌드 exit 0(6분 40초),
전 192쪽 검증 중이다. 이 근거로 반각 문자 블록 전체까지 추정해서 바꾸지는 않았다.

2022의 3쪽은 원본 83번 문단이 두 LineSeg를 보유하고 새 Print도 끝 글자 `립`을
둘째 줄에 출력한다. 그럼에도 `is_sample16_2022_bcp_orphan_tail_lineseg`가 문서의 특정
문자열을 조건으로 둘째 줄을 접었다. 최초 `8a26a1f88f0930fb7cac50e97d61e977cea6f8ba`는
PR #1120 merge `f7a68bc319ed67ace974722d324e49339954d121`에 통합됐고, 후속 추출
`0fec89c3cce2c8d0ee10a6ae02f9fdae01d6d603`도 PR #8064 merge
`9b01d5d1447220b3a7345703e79ad42cf3f502f4`에 들어갔다. 기존 회귀 검사도 한 줄을
기대하지만 이 기대는 원본과 새 Print에 어긋난다. 후보 17에서는 문자열 예외와 유효 줄
수 감소를 제거해 원본의 두 줄을 보존한다. 기존 잘못된 검사는 전쪽 100% 확인 후
두 줄의 내용·소유·다음 문단 관계를 보호하도록 수정/통합하며, 지금은 변경하지 않는다.

후보 17 Native 빌드는 exit 0(12분 03초)이다. CLI SHA는
`b3ac7e95de5b6e9b2a0e264ecb1d31707a69a1ad8c5ab11e49b3ad7ed58fe1ff`다.
사용자의 “예외 제거 먼저” 지시에 따라 BCP 영역을 우선 검증했다. 임시 독립 probe에서
제거 전 후보 16은 실제 줄 수 1로 실패(exit 101), 제거 후 후보 17은 두 줄의 전체 내용·
문단 소유 `(0, 83)`·다음 문단 `(0, 84)`의 내용과 비겹침을 모두 통과(exit 0)했다.
첫 줄은 `…Planning) 수`, 둘째 줄은 `립`이고 새 2020 Print와 같다. 둘째 줄 원점은
`(143.36, 916.10667)`, Print glyph bbox 원점은 `(143.36, 916.03890)` CSS px다.
다음 문단 원점 `y=947.62667`도 유지된다. 3쪽 raw는 99.19417%에서 99.91690%로
개선됐고 4쪽은 99.81493%로 유지됐다. 두 쪽의 실제 PNG를 확대 대조했다.
임시 probe·로그·비교 PNG는 `output/pr-review/issue8160-20261022/font-diagnostics/`의
`bcp_rows_probe.rs`, `bcp-rows-native16.txt`, `bcp-rows-native17.txt`,
`p3-native-pdf17-crop.png`이며 정식 회귀 검사 변경은 아직 없다.

후보 17의 전쪽 Native 검증은 HWPX·2024 HWP 51쪽씩, 2022 HWP 39쪽이 raw
100%다. 2022 최저는 BCP 복원으로 99.19417%에서 99.77773%가 됐다.

로마 숫자 U+2160~U+2188의 41자는 한글 슬롯으로 잘못 분류됐다. 새 독립 표본은
`57f34aef77ffaa04612b59da309177c9597a1fdf392c4cb601156ef12e787353`이며 새 MCP
작업 `83de5117-31d3-43bd-b4b5-72b3aa67051c`(2024),
`e68a1842-c3f4-4831-b393-0e500256eea4`(2020)의 서로 다른 슬롯 상대 크기에서
41자 전부 기호 슬롯 크기 16pt를 선택했다. 후보 18은 공통 분류를 바로잡는다.
같이 확인한 ×·÷는 영문 크기 12pt였으므로 이 둘의 슬롯은 바꾸지 않는다.

43쪽의 ×·÷는 슬롯 오류가 아니라 Poppy를 대신한 Palatino 폭의 오류다. 독립
8·10·13·20·26pt 일반/굵은 `A×A÷A` 표본을 작업
`02109100-49e8-461c-a3b2-a8bc03fbc880`으로 출력했다. fallback Type3의 /Widths는
em 1000에서 616·623이고, 독립 이웃 원점 20개도 600dpi 한 격자 이내로 대조했다.
후보 18은 일반체의 이 두 폭만 원래 em으로 선택한다. 영문 원본 em=512에 반올림해
섞지 않는다. 굵은체는 별도 추가 전진이 있으므로 이번 표 선택에는 포함하지 않았다.
수치 근거는 `poppy-original-widths.json`의 `independent_math_fallback`에 기록했다.

53쪽 끝의 평문 공백은 전체 글자에 음수 배분 자간 -0.03470px를 만들어 뒤쪽을
2~3px 당겼다. 새 빈 본문 표본(말미 공백 0·1·3개, 문단 끝/강제 줄바꿈)의 MCP
작업 `ce56c264-873e-4f2d-9c6e-5b8b3f000a26`은 모든 가시 글자 원점이 동일하다.
기존 후보 17은 공백 한 개가 있는 문단 끝에서 -0.20364px를 넣는다. 후보 18은
이미 독립 셀 표본으로 확인한 평문 말미 공백 점유 처리를 본문에도 공유한다.
자동 줄 나눔의 공백만 있는 후속 줄은 별개로 관측됐으므로, 실제 한컴 저장본의
LineSeg도 받아 같은 계약을 추가 대조한다. 정식 회귀 검사는 아직 변경하지 않았다.

후보 18 Native는 exit 0(6분 09초), CLI SHA
`d71f0e3af813eecbb5e9fd2a8a1c49d9e0188305c6bced21f0a535ec83102717`다.
전쪽 결과는 HWPX·2024 HWP 58쪽씩, 2022 HWP 43쪽이 raw 100%다.
본문 공백 독립 표본을 한컴 2024로 저장한 HWP도 받아 비교했다(작업
`cd38363e-d09f-48a4-a37d-f64629f428d9`). 원본 저장 LineSeg를 사용하는 문단 끝·강제
줄바꿈의 공백 한 개가 후보 17에서는 -0.20364px로 글자를 압축하고 후보 18은 원점을
보존한다. 공백 세 개의 저장 줄 구성은 그대로 두고 가시 글자의 전진도 보존한다.

후보 19는 기존 `NativePuaFonts`의 실제 공급 프로그램 지원에 F03C5를 추가했다.
원 프로그램이 없을 때는 종전 Unicode 표시 대체를 유지한다. 대상 2022 PDF의 PUA
outline은 설치 HBATANG.TTF의 F03C5와 완전히 일치(advance 1024/1024em)한다.
빈 독립 문서의 6개 원 face·3개 크기 출력은 2020/2024 모두 `휴먼명조`의 대체 상황에서
본문 Gulim과 별개로 Haansoft Batang을 선택했다. 명시적 Gulim은 Haansoft Dotum,
명시적 HCR Dotum은 자기 프로그램을 쓰는 반례도 확인했다. 작업은
`d45f4052-7b77-42bb-9d72-6d63ab39b2ee`, `d7f83c6d-f0b7-4d0f-8a33-d0b180dea8d5`다.
이 독립 근거의 프로그램 공급은 기존 FontEnvironment의 `glyphFallbacks`로 선언했다.
본문 치환이 Gulim인 이번 검증 환경에 Haansoft Batang을 명시하며 production에 face
이름이나 특정 문서 판정을 넣지 않는다. 이전 환경 JSON은 보존하고 `-pua.json`으로
분리했다. 새 동일 조건의 base Native/WASM 전 384쪽도 exit 0으로 완료했다.

후보 19 빌드는 exit 0(7분 02초), CLI SHA
`d960455bc00b710b83a62d195641e9befcd1d99ac8eaf65951b08bbab607bde2`다.
새 글꼴 환경의 전쪽 Native에서 세 문서 모두 58/64쪽이 raw 100%다. 2022는 PUA의
원 glyph와 전진 복원으로 15쪽이 추가 통과했다. HWPX·2024의 남은 쪽은 1·2·20·35·39·54,
2022는 1·2·18·20·35·39다. BCP 두 줄 복원도 유지된다. 아직 회귀 검사를 변경하지 않았다.

54쪽은 실제 PDF가 합성 기울임 Tm 계수 0.333을 쓰는데 Native는 0.25, CanvasKit은
0.2를 따랐다. Native의 해당 추출은 `59bfbd6e18136b51628abf8de3ca89c644fb48af`,
PR #8064 merge `9b01d5d1447220b3a7345703e79ad42cf3f502f4`로 추적했다.
원본 문서와 별도의 7개 face·3개 크기 일반/기울임 짝을 새 MCP로 출력했다. 2020/2024
모두 합성 기울임은 0.333이며 실제 Times/Arial Italic 프로그램은 추가 shear 0이다.
숫자·PDF SHA·작업 식별자는 `synthetic-italic-print-contract.json`에 보존했다.
후보 20은 실제 기울임 face가 없는 경우의 공통 paint 계수를 Native와 paint JSON에서
공유하고 CanvasKit이 그 값을 소비한다. 실제 italic face에는 추가 변환을 하지 않는다.

후보 20 Native 빌드는 exit 0(12분 06초)이며 전쪽 raw 100% 쪽수는 세 문서 모두
58쪽으로 유지됐다. 54쪽은 괄호 일부만 달라지고 한글 기울임은 달라지지 않았다.
원 HMKMM.TTF의 macStyle=0, fsSelection=64, italicAngle=0인데 시스템은 같은 프로그램을
Oblique로 반환했다. 따라서 기울임 face 보유 여부를 시스템 slant만으로 판정하면 안 된다.
후보 21은 SFNT·활성 ital/slnt 축을 확인해 가상 slant만 제거하고 공통 shear를 소비한다.
임시 Skia probe는 일반체가 그대로 남고 가상 Oblique가 Upright로 재구성됨을 확인했다.
빌드는 exit 0(6분 34초), CLI SHA는
`edbb3fc59de36056d11a3e36855a85473030e5b2071a0be12317edde2f9f3ce0`다.
우선 재검증한 54쪽은 세 문서 모두 실제 raw 100%다. 전쪽은 다음 후보에서 재산출한다.

목차 점선은 independent 54개 사례(두 원 HFT·9/13/18pt·세 접두 길이·세 자리 수)에서
폭이 있는 52개 모두 1/4em 셀 수와 일치했다. 남는 폭을 그 셀에 균등 배분하고 glyph
원점에서 0.192em만큼 이동한 점 중심은 독립 Print와 0.5px 이내다. 폭 0인 두 사례는
점을 만들지 않는다. 새 MCP 작업 `a764253c-6cfb-43d2-ac5a-a9ee6be03074`의 Print와
`5d6c441c-5bd8-4723-8fcd-62f58fe11eba`의 한컴 저장 LineSeg를 대조했다.
생성 HWPX의 width=0 탭은 직접 파서와 한컴 import의 정지가 달랐으므로 이 차이를
숨기거나 대상 문서로 보정하지 않고 한컴 저장본의 실제 탭 기하를 독립 비교에 사용했다.
수치 증거는 `tab-leader-print-contract.json`이다. 후보 22는 공통 점 중심을 Native·
SVG·Canvas2D에서 쓰고, paint JSON에 넣어 CanvasKit도 같은 결과를 소비한다.
이전 payload만 종전 근사 stroke를 사용한다. 별도 진단에서 대상 2쪽 점선만 새 기하로
그리면 양방향 미일치 0이지만, 이는 아직 후보 22의 실제 엔진 통과 증거가 아니다.

그림 6은 회색만 256색이라는 가정이 틀렸다. 실제로는 RGB 256색이고 검은색이 없어
흑백 예약색을 합치면 257색이다. 기존 `print_tone` 색 확장 경계에 포함된다. 이 사실을
재확인했으며 색 전달을 무조건 넓히는 변경은 하지 않았다. 독립 18개 BMP/PNG/GIF
17·18·24·31·32·256색 표본의 새 Print(`0a1daf77-7546-4ff5-b4be-6f94262b8f07`)는
31색까지 PNG, 32색 이상 JPEG였다. 32/256 회색 사례 6개는 원본을 libjpeg 품질 75,
4:2:0으로 압축한 디코드 픽셀과 완전히 같다. JPEG 전달과 축별 저역 통과는 아직
production에 적용하지 않았고 진단 중이다.


후보 22 Native 전쪽은 HWPX·2024 HWP 각각 60/64쪽, 2022 HWP 59/64쪽 raw 100%다.
CLI SHA는 `18d18a7e11f1dc6a1db73fbf24cfce8660bfbe52ebe2859da32d710e86e2a3fb`다.
목차 점선과 54쪽 기울임 개선이 실제 엔진에서도 유지됐다.

### 원본 raster의 Print 전달과 축별 표본

별도 18개 BMP/PNG/GIF 표본은 2020에서도 31색까지 무손실, 32색 이상 JPEG 4:2:0이었다.
가로 배율별 독립 Print의 양자화 표는 96dpi 배치 폭을 사용한 품질 식과 일치한다.
2024는 흑백 예약색을 합쳐 257색 이상일 때 RGB555 ordered dither를 거친다.
8×8 평탄 타일의 회색 전 범위와 RGB 순열은 독립 Print의 디코드 픽셀과 완전히 같다.
평탄 타일만으로는 위상이 충분히 구별되지 않아 픽셀마다 색이 바뀌는 자료를 추가했다.
원본 높이 125·126·127·128·129행의 결과는 `y-height` 위상을 따른다.
홀수 폭의 마지막 chroma 블록에는 아직 별도의 전달 차이가 있어 원인을 확인 중이다.
숫자 근거와 작업 식별자는 `print-raster-transfer-contract.json` 및 output 진단에 있다.

후보 23은 중립 효과·완전 불투명·전체 원본의 Print 그림에 한정해 JPEG 전달을 공유한다.
PDF importer의 명시적 표본 선언, crop, 투명도, 다른 효과와 JPEG 원본은 기존 경로다.
품질은 layout DPI에서 96dpi 기준 폭으로 환산하므로 출력 DPI를 고정하지 않는다.
배치에 따른 출력은 `OutputDevice` 신원이며 원본 BinData는 보존한다.
Native 빌드는 exit 0(10분 13초), CLI SHA는
`6861068b1d96c986f62de4be730c503a16ecd5b3ddfb0a4a2e250d2d00416989`다.
처음에는 잘못된 `native-cli` feature를 써 Cargo가 컴파일 전에 거부했으며
`native-skia`로 정정해 위 빌드를 완료했다. 소스 snapshot은 Cargo 파일도 포함한다.
5쪽씩 확인한 후보 23에서 2024의 20쪽은 PDF 누락 0, Native 잔여 3픽셀이다.

39쪽은 X 확대·Y 축소가 섞여 기존 등방 mipmap이 원본 가로선을 희석했다.
후보 24는 실제 CTM 위상에서 축별 삼각 저역 통과를 계산한다. premultiplied alpha로
누적한 뒤 마지막에만 알파를 되돌린다. crop·binary·일반 그림의 정책은 유지한다.
Native Skia·Rust Canvas2D·CanvasKit이 동일한 Rust 필터를 소비한다. CanvasKit에는
제한된 RGBA와 네 장치 좌표를 받는 WASM adapter를 두었다. 후보 24 Native 빌드는
exit 0(5분 36초), CLI SHA는
`2291d80749b1e7a0d20a606e0125159b487920193156a351ffdbcb81002a694f`다.
Studio tsc도 exit 0이다. 실제 39쪽은 2024 양쪽 raw 99.99898%, 2022 99.99694%로,
각각 PDF 잔여 1픽셀·복수 픽셀이 남아 있다. 전쪽 100%로 판정하지 않는다.

비교 TSV의 기준 rasterizer는 run manifest에 고정된 PDFium 153.0.7999.0,
`pdfium_smooth_rgb_v1`이다. PyMuPDF는 객체·원본 JPEG·벡터 연산 분석에만 쓴다.
MuPDF grid-fit을 따로 확인한 진단은 실제 TSV rasterizer의 계약 근거가 아니므로
production의 가장자리 보정이나 완료 증거로 사용하지 않는다.

새 pure Rust JPEG 인코더의 reciprocal 양자화는 품질 75, 두 표·64개 위치·
-2048..2048 입력의 1764개 정확한 반 양자 경계에서 표준 정수 반올림과 달랐다.
원본 `jpeg-encoder` 0.7.1을 라이선스·저작권과 함께 vendor에 보존하고 잔차가
반 양자 이상인 빠른 경로만 정수 나눗셈으로 재확인했다. 품질 35..75의 두 표·64개
위치·전체 i16 입력을 대조한 별도 진단은 미일치 0이다. Native/WASM 공통 JPEG
전달에서 같은 보정을 사용한다. 외부 원본 `encoder.rs`의 1000줄 이상 구조는
변경하지 않았으며 자체 수정은 `quantization.rs`의 경계 처리다.
후보 25 빌드는 exit 0(6분 50초)이고 전체 Native 192쪽 재검증과 WASM check 중이다.
회귀검사 파일은 아직 변경하지 않았다. 기존 BCP 한 줄 기대값은 전쪽 검증 뒤 정정한다.


후보 25 전쪽 재검증은 HWPX·2024 HWP 각각 60/64쪽, 2022 HWP 59/64쪽 raw 100%다.
남은 쪽은 2024의 1·20·35·39, 2022의 1·18·20·35·39다. WASM check도 exit 0이다.
JPEG 품질 식의 `round` 설명과 구현은 잘못됐다. 기존 독립 137px 원본의 80/120px
배치는 실제 품질이 43/65이며 round 식의 44/66이 아니다. 원본 bitmap4(859px)의
새 Print도 품질 54이나 후보 25는 55였다. 다음 후보에서 floor로 정정한다.
별도 GIF 211px 원본이 Print 자원에서는 210px로 전달되는 차이가 있어, 품질의
분모와 원본 전달 폭의 관계는 추가 조사 중이다. 이 차이를 숨기지 않는다.

A3·A4·A5 용지에 위치·크기를 바꾼 PNG 13개씩을 새 MCP로 출력했다. 실제 PDF는
landscape이며 입력의 `NARROWLY` 설정을 반영했다. 장치 용지 픽셀 수는 유효 용지의
HU 치수를 12HU(600dpi)로 나눠 반올림하고, 물리 PDF 용지는 정수 point로 내린 값이다.
따라서 장치 픽셀의 물리 크기는 용지 방향·크기에 따라 달라진다. 원본 배치와 같은
그림의 마지막 열이라도 PDFium의 정수 사각형·원본 재구성 후 clip에 따라 달라진다.
외부 진단에서 이 전달과 면적 표본을 쓰면 원본 1쪽의 두 그림은 각각 양방향 미일치 0이다.
다만 독립 39개 배치 중 일부 회색 하단 경계가 아직 달라서 생산 코드에는 적용하지 않았다.
한컴 저장 HWP의 실제 배치 필드와 교차 확인 중이며 단일 문서 보정으로 대신하지 않는다.


BCP 예외 제거와 독립 줄·소유 근거는 로컬 커밋
`f0f434b8f5f31386fa4f2c2ff881d0f150ceac45`로 먼저 고정했다. 회귀검사는 변경하지 않았다.
후보 26 Native 빌드는 exit 0(5분 49초), CLI SHA는
`06c5d4a3f592a752b313619b701d5963fb7ec4d9f6e581274da740aa25bd65b0`다.
품질 floor 정정 후 20쪽은 세 입력 모두 raw 99.99861%로, 가장자리 미일치가 남는다.
Native library Clippy(`native-skia`, `-D warnings`)는 exit 0(3분 24초), Studio tsc도 exit 0이다.
기존 필수 세 Clippy 전체 묶음·workspace 검증·fresh optimized WASM 전쪽 검증을
이 결과만으로 완료했다고 판정하지 않는다.

600dpi 전달의 용지 픽셀은 유효 HU 치수/12를 반올림한 값이다. 이전의 ceil 가설은
A3·A5 대조에서 기각했다. PDF 물리 용지는 정수 point이며 기록 좌표는 소수 셋째
자리로 직렬화된다. 이 반올림과 PDFium float/정수 사각형 재구성을 함께 쓰는 새 외부
진단은 A3·A4·A5 각 13개(총 39개) 배치에서 양방향 미일치 0이다. 아직 생산 적용 전이다.
추가 12개 원점 위상은 12HU 나머지를 모두 교차하며 그림의 정수 원점·크기와 clip
끝점이 다른 반올림을 소비하는 것을 확인했다. Wrap·flow·여백·참조 크기를 교차한
별도 표본으로 적용 범위를 확인 중이다.
한컴 저장 HWP는 생성 HWPX의 13개 원본 위치·크기 HU를 그대로 보존했다. 따라서
모델 값 자체를 바꾸지 않고 출력 장치의 공통 paint 결과로 전달해야 한다.

upstream을 다시 fetch해 현재 `upstream/devel`은
`def947c2149df5bc812d801e6e79be19a403b4d1`임을 확인했다. 기존 base 이후 네 커밋은
빌드·CI·절차 문서 변경이다. 아직 작업 브랜치를 rebase한 상태는 아니며, 통합 전에
표준 release WASM과 배포 산출물 신원 검증의 새 절차도 적용한다.

공통 그림 프레임의 원점을 확인했다. 별도 +10HU 보정은 필요하지 않다. 본문 참조 원점과
그 원점에 대한 그림 좌표를 각각 600dpi 장치 단위로 반올림하면 된다. 원본의 본문 원점은
(4252, 5672)HU, blank 표본은 (8504, 9920)HU다. 그림 크기는 별도로 반올림하고 clip의
끝점은 같은 참조 원점에서 따로 반올림한다. 따라서 clip 폭과 image 폭은 한 픽셀 다를 수 있다.
소스 bbox·여백·기준선·모델은 바꾸지 않으며 별도 소유 정보도 중복 생성하지 않는다.

생산 공통 sampler를 직접 포함한 외부 진단에서 A3/A4/A5 39개는 96dpi 양방향 미일치 0이다.
추가 12개 원점 위상과 16개 wrap/flow/여백/참조 크기 교차는 96/144/300dpi 모두 미일치 0이다.
총 123개 그림/배율 비교다. 교차 표본의 PDF 그리기 순서는 wrap plane에 따라 달라서 source와
단순 zip한 이전 중간 표는 잘못됐다. 서로 떨어진 원본 위치로 대응을 식별해 정정했으며
기대값을 맞추는 좌표 fitting은 하지 않았다. 원본 여백이 있는 그림의 ink 원점은 원본 위치에
여백을 더한 값이다. 이 공통 규칙이 16개 모두의 실제 image/clip 정수 좌표와 일치한다.

원본 39쪽의 마지막 1픽셀은 외부 진단의 Rust JPEG decoder 차이였다. source resolved JPEG와
Print JPEG를 libjpeg·PDFium으로 각각 decode한 RGB는 완전히 같다. 원본 source JPEG를
libjpeg으로 decode한 뒤 같은 생산 sampler에 넣으면 39쪽도 양방향 미일치 0이다.
PDF에서 추출한 pixels를 생산 입력으로 넣지 않았다. 실제 Native Skia·브라우저 decoder를
포함한 전쪽 실행은 후보 28에서 확인한다. 후보 27의 최초 빌드는 import 경로와 없는 bbox
끝점 메서드의 컴파일 오류 5개로 exit 101이었고 이를 정정했다. 후보 28을 별도로 동결했다.
Studio TypeScript는 exit 0이며 Native 빌드가 진행 중이다.

가상 oblique 제거의 정상 대조군도 실행했다. 실제 DejaVuSerif-BoldItalic.ttf를 SFNT에서
직접 연 결과 intrinsic Italic은 제거 대상이 아니며 typeface 신원을 보존했다(exit 0).
파일·진단 SHA는 기존 synthetic italic 숫자 근거에 추가했다. 회귀 검사 파일은 변경하지 않았다.

후보 28 Native 빌드 exit 0(524.37초), CLI SHA는
`add5ec734d1f042caa6491bf0beb2a35e0f55092956e0b13c901cc2fabfcd077`다.
세 입력의 전쪽 실행은 각각 exit 0, 151.39/154.13/166.21초다. HWPX·2024 HWP는 각각
63/64쪽, 2022 HWP는 62/64쪽 raw 100%다. 1·20·39쪽은 모두 양방향 미일치 0이며
남은 쪽은 WMF가 있는 2024 계열 35쪽(99.99244%), 2022 18쪽(99.99929%)·
35쪽(99.96986%)이다. 전쪽 완료로 판정하지 않는다. WASM target check exit 0(2분),
Native library Clippy(native-skia, -D warnings) exit 0(3분 44초)다.

원본 WMF3/5의 텍스트는 각각 554/445개가 모두 TA_BASELINE(24) 소유다. 이전 합성 표본의
기본 TA_TOP 보정(0.8em) 차이는 원본 잔여 쪽의 원인이 아니다. 같은 기준선 정렬로 새 표본을
만들어 MCP 두 engine으로 다시 출력했다. 입력 SHA는
`f5abc4f72e22b4128c7bfbb868e8a1b0b4b3dae5b93f88106fc26558baa5b24b`,
2020 job `ecb5fc1a-ed08-4794-9f1b-33c61ac2c531`,
2024 job `421a0641-4cda-4218-a619-2515a9ce5038`이다. 2020의 0.02em 획은 얇은 Bold의
누락을 줄이지만 실제 장치의 최소 1픽셀 획을 반영하지 않으면 충분하지 않다. 일부 Regular·
다른 font 프로그램 차이도 남으므로 단순 굵기 변경만으로 해결했다고 판정하지 않았다.
원본 35쪽 확대 판독에서도 Native의 글자가 Print보다 옅고 얇음을 확인했다.
기존 WMF mask 경로는 SVG 전체를 큰 원본 격자에서 렌더한 뒤 축소하므로 그 안의 벡터 글자까지
축소되는 소비 경로를 추가 조사한다. 회귀 파일·기대값은 아직 변경하지 않았다.

현재 단계의 소스와 숫자 근거는 로컬 커밋 `e098aa343`으로 보존했다. 아직 push·PR 제출은
하지 않았다. 이 커밋 전 diff check는 upstream vendor README의 두 줄 끝 공백을 보고했는데,
실행을 멈추지 않고 커밋한 명령 구성을 정정한다. README의 공백 두 개를 제거하고 vendor
패치 목록에 비기능 변경으로 명시했다. 라이선스 파일·문구는 변경하지 않는다. 다음 변경에
함께 보존하며 PR 제출 전에는 base 대비 diff check를 다시 확인한다.

WMF 마스크의 기존 근거도 다시 읽었다. `b77d0d649`·`912817f6c`와
`task_m100_7642_stage1978/1990.md`는 패턴 마스크를 축소 전에 합성해야 한다는 근거를 남기지만,
큰 SVG 전체를 축소하면 형제 벡터 글자도 함께 축소된다. 아직 production에 연결하지 않은
`metafile_mask.rs` 시안으로 원본 35쪽의 직접 마스크 개체 4개만 먼저 합성했다.
같은 Native producer의 그림만 재생하는 별도 진단은 원래 35쪽 그림 영역과 픽셀 완전 동일했다.
시안 적용 뒤 그림 영역 미일치는 Native 0/PDF 49 →2/41이며, 합성 Bold의 장치 획과 실제
장치 위상을 따로 확인하면 5/17까지 줄지만 아직 전쪽 완료가 아니다. 글자 농도는 독립 Print와
가까워졌으나 실제 font 프로그램 선택·다른 backend의 소비를 연결하기 전에는 생산 적용하지 않는다.

원본 2022 Print의 WMF 텍스트 행렬은 `0.073 0 0 -0.086 62.079 468.627`이다.
같은 600dpi 참조 프레임(550,3266,3898,2853), 용지(4961,7016) 장치 픽셀과 원본 창 원점
(53,223)으로 계산한 소수 3자리 행렬과 여섯 요소가 정확히 같다. 다만 새 독립 표본은 6자리다.
TextOut/ExtTextOut·Dx 유무·Placeable/일반 헤더·원점·굵기를 교차한 36개 표본은 6자리를
유지하므로 API/헤더만으로 precision을 나누는 가설은 기각했다. 숫자·문서 식별자로 원본에
3자리 보정을 적용하지 않는다. 원본 반복 출력과 WMF 24개 바이트를 그대로 보존한 HWP 저장본을
다시 Print해 이 차이를 추적한다. 두 source의 실제 WMF bytes 보존은 24/24 SHA 일치로 확인했다.

Microsoft GDI의 Brush Origin 설명은 pattern bitmap의 장치 픽셀·기본 원점을 명시한다.
https://learn.microsoft.com/en-us/windows/win32/gdi/brush-origin
현재 WMF 변환기는 8×8 pattern을 논리 단위로 지정한다. MM_ANISOTROPIC에서 장치 크기와
위상을 분리해야 하는 근거이며, 네 창 크기·세 원점·서로 다른 페이지 위상의 새 독립 표본을
두 MCP engine으로 출력 중이다. 입력 SHA는
`5fff490c6fc3e0f601065e4cd0ded326809ded9ca6c3745ea34622e048aa78b9`다.
회귀 검사·기대값은 아직 수정하지 않았으며 후보 28 Native 188/192 raw100 판정도 그대로다.

투명 아이콘의 별도 원인을 바이트로 확인했다. 그림 3의 6쌍·그림 5의 2쌍은 연속된
SRCPAINT→SRCAND이며 두 source 창·목적 사각형이 같다. 모든 source 픽셀에서
`(D | M) & C`가 상수 또는 D 자체로 환원됨을 확인했다. 그림 5의 각 125×123 원본은
6134픽셀이 투명, 9241픽셀이 불투명이며 비정규 픽셀은 0이다. 종전 코드는 두 연산을 각각
SRCCOPY로 근사해 배경을 흰 사각형으로 덮었다. 원본 source만으로 합성한 별도 SVG를 같은
후보 28 Native에 넣으면 그림 영역의 미일치는 0/49→0/17로 줄었다. PDF 픽셀은 합성 입력이 아니다.

공통 변환기에 전체 source 창·같은 목적 프레임·원시 레코드 인접성이 확인된 반대 순서의
OR/AND 쌍을 합성하도록 연결했다. 각 RGB 채널의 상수/항등 비트를 증명하며 모든 D에 대해
RGBA로 표현되지 않는 경우는 합치지 않는다. 일반 SRCCOPY의 흰색은 바꾸지 않는다.
원본 WMF bytes·외부 그림 소유와 배치는 보존한다. 임시 `metafile_mask.rs`는 여전히 미연결이다.
후보 29로 이 변경을 동결해 Native 빌드를 시작했다. 새로운 회귀 검사는 추가하지 않았다.

원본 2022 HWP를 새 MCP로 반복 출력한 35쪽의 PDFium 픽셀 SHA는 이전과 완전히 같다
(`3f303c8f0da3b8fb033f633eb0d19463ec63d5a4297d23e61c9b964be1bb1de6`).
즉 기준을 바꿔 남은 차이를 없애는 문제는 아니다. HWP 저장한 독립 24개 WMF도 6자리 행렬을
유지했고, 선의 앞뒤 순서·pitch/weight를 교차한 새 12개도 6자리였다. 원본의 3자리 precision을
단순 컨테이너/API/선 순서 조건으로 적용하지 않는다.

### 후보 29 원본 전쪽 결과와 마스크 반례

Native29는 전쪽 실행이 모두 exit 0이고 HWPX·2024 HWP 각각 64/64쪽, 2022 HWP
62/64쪽으로 **190/192쪽 raw 100%**다. 남은 2022 18쪽은 99.99929%(참조 2픽셀),
35쪽은 99.98954%(참조 17픽셀)이며 후보 방향 unmatched는 두 쪽 모두 0이다.
WASM target check도 exit 0(38.31초)이다. 아직 최적화 WASM 전쪽 완료를 주장하지 않는다.
CLI SHA는 `5b0d3e7c054eae6b83eb53c79794207ef178788e88676476ea523122f2a249a9`다.

별도 36개 표본의 원본 연산은 2개 순서·3개 stretch mode·좌우 반전·두 비백색 배경과
일반 SRCCOPY 흰색 보존을 교차한다. Native는 두 compat에서 36개 모두 원본 논리의
색·소유 관계를 보존한다. 새 한컴 2024도 전부 일치한다. 2020은 OR_AND와 SRCCOPY는
일치하지만 역순 AND_OR 12개가 흰 사각형·검은 중심으로 출력된다. 이 엔진 차이는
독립 증거에 남기며 두 한컴이 모두 일치했다고 보고하지 않는다. 실제 대상의 8쌍은 모두
OR_AND다. 원본 수학적 진리표 33,554,432개 결과도 별도 진단에서 통과했다.

아직 연결하지 않은 마스크 분리 시안은 제품 source에서 output 진단 경로로 옮겼다.
광범위한 최소 stroke 확대는 18쪽에 6픽셀을 새로 만들어 채택하지 않았다. 요청 굵기를
일괄 stroke로 바꾼 시안도 실제 Bold face를 중복 강화하므로 그대로 적용하지 않는다.
WMF 내부 사진의 JPEG 전달은 일반 raster와 같은 폭 의존 품질을 사용한다는 근거를
확보 중이다. 원본 서버 사진을 표준 q57/420으로 압축하면 한컴 자원의 디코드 픽셀과
완전히 일치하지만, 이를 SVG에 넣기만 해서는 18쪽의 두 경계 픽셀이 남는다.
따라서 압축 일치만으로 장치 합성까지 해결됐다고 판정하지 않는다.

### 최신 devel 통합과 표준 WASM 검증 착수

확정한 연속 마스크 합성을 `409184576`에 커밋했다. 이어 최신 devel
`def947c2149df5bc812d801e6e79be19a403b4d1`을 일반 merge
`f023d6f5201a589057c5cdadc6bdef627e7761b6`로 통합했다. 새 devel의 4개 PR은
문서·빌드·CI 변경이며 기존 Native 렌더 source를 바꾸지 않는다. Native lib Clippy는
exit 0(1분 41초), WASM lib Clippy도 exit 0(4분 41초)이다. 광범위 fmt 검사에서
기존 파생 suite가 없는 source를 가리키는 실패를 확인했다. 이번 source의 모듈 순서를
수정했으며, 정식 PR 검증은 review worktree의 suite 준비 후 전체 게이트로 수행한다.
회귀 source나 기대값은 아직 수정하지 않았다.

동일 head의 별도 worktree `/home/tsjang/rhwp-8160-wasm-candidate29`에서
`CARGO_TARGET_DIR=/home/tsjang/rhwp/target/pr-review`, jobs=14, release CGU=16으로
`python3 scripts/measure_render_diff_wasm.py --profile release --build-mode standard`를
실행했다. 실제 wasm-opt `-O` 진입을 확인했고 최적화 종료를 기다린다. opt-in cdylib
경로는 쓰지 않았다. 첫 실행은 worktree checkout이 끝나기 전에 wrapper 경로를 읽어
즉시 실패했으며, checkout exit 0을 확인한 후 clean tracked source에서 다시 시작했다.

### 2020 WMF의 독립 전달·출력 상태 진단

새 12개 원본 표본(`wmf-nested-raster-unique-independent.hwpx`, SHA
`241722af26b96490606f8700be32090bfb2b95e551412fe29ec2821d1a5f5b4c`)에서
16색 타일은 무손실이고 256색 타일의 JPEG 420 품질은 표시 폭 48/80/120px에서
40/68/75다. 좌우 반전 모두 같다. 같은 bitmap을 반복한 표본은 첫 작은 배치의 q40을
공유하며, 배치 순서를 뒤집은 새 표본은 q75를 공유한다. 따라서 문서 출력 자원의
첫 사용을 보존해야 하고, 같은 사진을 모든 배치마다 다시 압축하면 한컴과 달라진다.
실제 문서의 18쪽 서버 사진과 35쪽 서버 사진도 동일 자원(xref 358, q57)을 공유한다.

제품 코드에 연결하지 않은 진단에서 원본 그림의 JPEG 전달, 원본 GDI 프레임, 기존
PDFium 면적/선형 표본, 비트맵 마스크의 개체별 원본 합성, 글꼴의 합성 굵기와 장치
위상을 함께 소비했다. 18쪽 그림 영역은 양방향 미대응 0을 확인하고 확대 판독했다.
35쪽은 2픽셀(648,796~797)이 남는다. 이는 별도 합성기 진단이며 Native 전쪽 결과나
제품의 완료로 합산하지 않는다. broad minimum stroke 확대 시안은 새 오차를 만들므로
채택하지 않았다. 합성 굵기도 실제 공급 face의 weight를 확인해야 한다.

일반 본문 추가·JPEG/무손실 bitmap 추가·crop reference 변경·비균등 배치·TAC 변경·
rotateimage 변경·CP949 ExtTextOut만으로는 원본의 소수 3자리 상태를 재현하지 못했다.
하지만 새 `wmf-polygon-order-independent.hwpx`(SHA
`9fc8b19be9ee857cca1c2196788fc0ed5891286eb50ac1c2501345b0550cb045`)은
다각형을 그리기 전의 6자리와 그리기 뒤의 3자리를 재현하며, 다음 WMF에도 상태가
유지된다. 기존 단순 LineTo 표본에서는 이 전환이 없었다. 이 출력 상태를 문서 ID나
원본 수치로 추측하지 않고 공통 생산 결과로 마련할 근거를 확보했다.

추가 16개 독립 Dx 표본은 ASCII/CP949·자연/가변 Dx·83/180 높이·일반/굵기를
교차한다. 실제 PDF의 전진은 원본 Dx를 그대로 더한 값과 일부 차이가 있으므로 실제
글꼴의 폭·합성 굵기 측정과 출력 상태를 함께 추적한다. 원본 GulimChe의 네 글자
윤곽·hmtx·hhea는 참조 PDF의 subset TTF와 같음을 읽기 전용으로 확인했다. 앞선
CIDFontType0/CFF 추측은 틀렸으며 실제 F10은 CIDFontType2/FontFile2다. 이를 근거로
참조 PDF에서 글꼴을 가져와 제품에 넣거나, 미확인 첫 글자 간격 상수를 추가하지 않는다.

### 후보 29 fresh 표준 WASM 전쪽과 원본 잉크 폭 보정

표준 release 빌드·wasm-opt가 36분 24초에 exit 0으로 완료됐다. manifest의
source는 clean `f023d6f5201a589057c5cdadc6bdef627e7761b6`이고 rlib+cdylib,
manifest LTO, standard 모드다. 패키지와 Studio public의 해시 검증도 통과했다.
JS SHA는 `75dfc2d83b6e133e7e0d1057faa4f560b3a04aed563058d6a7a46f1e9c79da0e`,
WASM SHA는 `89c4205c4a5fcdd74c45c3392dc751c7e833b2df50bd53df2d4a9ce718d2ff39`다.
세 WASM 전쪽 실행은 exit 0(109.17/95.86/65.21초)이다. HWPX·2024 HWP는 62/64쪽,
2022 HWP는 60/64쪽이 raw 100%다. Native 190 + WASM 184 = **374/384쪽**이며
전쪽 완료로 판정하지 않는다. WASM 공통 38쪽 9픽셀·39쪽 1픽셀이 추가로 남는다.

38쪽은 실제 공급 HCI Poppy CFF가 있는데도 Canvas가 글리프를 원본 전진폭 안에
축소한 원인이다. 독립 Type3 원 프로그램에서 `/`의 Width는 605/1000em이지만
굵은 글자의 실제 전진은 약 0.29~0.30em이며, 이 둘은 다른 값이다. 원본 일반/굵은
ASCII 95개 윤곽이 같고 굵기는 별도 stroke라는 기존 독립 증거도 재확인했다.
런타임 measureText는 12.903564px, paint scale은 0.4877205였다. Native는 이 일반
본문의 윤곽을 축소하지 않는다. 브라우저 진단에서 잉크 폭을 보존하면 38쪽 전역의
양방향 미대응이 0이 된다. 첫 진단 hook은 outline의 em 스케일까지 바꿔 잘못됐고,
단위 y 축과 글리프 fit 변환만 분리한 교정 hook에서 전역 0을 확인했다.

공통 HFT 프로그램 registry에서 `preserves_program_ink`를 생산하고 Canvas가 소비한다.
실제 등록 CFF 윤곽·단일 글자·HFT 원본·고정 폭 맞춤 미요청을 확인하며, 글꼴 대체와
옛한글 등 기존 전용 경로는 보존한다. `/`나 문서·문단·좌표 조건은 제품에 없다.
Native lib Clippy exit 0(3분), WASM lib Clippy exit 0(3분 6초)이다. 이 제품 변경의
fresh optimized WASM 전쪽 검증은 아직 하지 않았으므로 진단 결과로 대신하지 않는다.

39쪽은 Canvas가 Rust image/zune로 JPEG를 디코드하지만 Native는 Skia/libjpeg를
소비하는 차이까지 추적했다. 동일 payload의 decoded RGB/YUV를 독립 비교했으며
IDCT·chroma upsample 차이가 남는다. 다른 decoder를 단순 교체한 시안도 libjpeg와
완전히 같지 않아 채택하지 않았다. 새 decoder의 None transform 진단은 라이브러리에서
panic을 일으켜 그 경로를 중단했다. 제품 decoder나 기준 PDF를 바꾸지는 않았다.

### Print JPEG 원본 픽셀을 공통 재표본기에 전달

Native/참조와 같은 libjpeg RGB를 브라우저 codec에서도 확인했다. rhwp가 원본에서
만든 네 Print JPEG(133×40·586×141·859×542·556×1055)의 모든 RGB가 libjpeg와
완전히 같다. 참조 PDF 픽셀을 제품 입력으로 쓰지 않는다. typed Print JPEG에 한해
브라우저 decoder가 복원한 원본 canvas를 기존 공통 PrintRasterFrame에 전달한다.
일반 입력의 decoder·원본 ImageNode·crop·최종 프레임을 바꾸지 않는다.

기존 HTML 그림 cache와 prefetch가 로드를 소유하며 대기 중 None은 영구 실패로
캐시하지 않는다. 전체/자르기 그림이 중복했던 decoded cache 코드를 하나로 합쳤다.
source header의 decode 한계와 실제 자연 크기를 확인하고, 복원 canvas가 생긴 뒤
임시 HTML image는 해제한다. 초기 대기·다시 그리기·crop도 fresh WASM에서 확인한다.
WASM lib Clippy는 exit 0(1분 49초)이다. 이 제품 변경의 fresh 표준 package·전쪽
측정은 다음 빌드에서 확인하며, 독립 codec 비교만으로 39쪽 완료를 선언하지 않는다.

MCP 2020의 실제 backend가 32-bit managed direct DLL host·session0임을 재확인했다.
로컬 Linux HSP font query는 Windows Print의 측정을 입증하지 않는다. GUI/PDF를
만들지 않은 로컬 font 메타데이터 진단만 수행했고, Windows GDI 쪽을 별도 읽기 전용
확인한다. win10-ted의 Gulim TTC SHA는 공급 프로그램과 같은
`4b9ac63e8920ed1bae29c068025ed30493464c18ee18617887daa18e59189226`이다.
