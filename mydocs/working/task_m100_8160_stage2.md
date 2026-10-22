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
