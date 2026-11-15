# Native SVG의 공급 프로그램 준비 누락 조사

현재 생산 소스 `2749e491e843ec851de12467751ec200ab551014`의 각주 원본 전50쪽을
같은 독립 한컴오피스 2020 Print PDF와 대조했다. Native/fresh WASM PNG는 모두
최저99.29%다. Native SVG는 7쪽94.37%, 28쪽97.19%로 이 브랜치의98% 목표에 미달했다.
[실행 결과](before.json)는 미해결 기록이며 최종 수용이 아니다.

7쪽의 문단78에서 실제 `「」` 전진은 PNG16px, SVG8px다. SVG는 줄 양끝을 유지하면서
남은 공백 폭을 재분배하여 중간 한글의 위치도 달라졌다. PNG가 보고한 실제 fontEnvironment를
같이 명시한 SVG 집중 재실행에서도 두 쪽은 같은 값이었다. 따라서 환경 JSON의 유무만으로
해결됐다고 판단하지 않는다.

공통 PNG exporter의 `prepare_document`는 실제 공급 renderer의 `prepare_native_otf_fonts`로
세션 전진을 준비한다. Native SVG CLI는 환경을 적용하고 곧바로 페이지 트리를 만들어 이
준비 단계를 호출하지 않는다. WASM 호스트의 원본 프로그램 등록은 별도 경로다.
공개 합성 TTF를 사용하는 CLI/API 관계형 검사를 `tests/cases`에 추가해 이 차이를 검증 중이다.
제품 수정의 수용은 반례와 대상 원본 재측정 뒤 판단하며 현재 보호 PNG 통과를 SVG 통과로
대체하지 않는다. 독립 PDF·입력 문자·98% 목표를 바꾸지 않는다.

## 보정과 합성 반례

Native Skia 지원 CLI의 layer SVG에서 공통 `prepare_native_otf_fonts`를 페이지 수·대상 쪽
열거 전에 호출한다. PNG의 기존 프로그램 선택·원본 전진 준비를 그대로 재사용하며 문서별
예외나 글꼴 이름·수치를 추가하지 않는다. legacy SVG와 Skia를 포함하지 않은 빌드의 경로는
이번 보정 대상이 아니다. 공통 조판·WASM·PNG 구현에는 변경이 없다.

올바르게 수집한 합성 CLI/API 관계 검사는 보정 전1FAIL(2.596초) → 보정 후1PASS(2.439초)다.
최초 파일명 가정 오류와 suite 재생성 누락 실행은 제품 실패·성공 증거로 사용하지 않는다.
실물 Native SVG 전50쪽을 새 바이너리로 다시 산출해 최저99.08%·98% 미만0쪽을 확인했다.
7쪽94.37%→100.00%, 28쪽97.19%→99.23%이며, 같은 쪽의 review·overlay에서 책괄호 뒤
한글의 전진·줄 끝과 다음 문단, 표·각주의 내용 소유를 직접 대조했다.
[전쪽 TSV·대표 이미지·생산 기록](after-native-svg/validation.json)을 따른다.
전체 최소 글꼴 회귀13,617PASS·45SKIP과 필수 Native/WASM lint도
[현재 검증](current-validation/results.json) 및 [lint 기록](current-validation/push-lints.json)에 고정했다.
