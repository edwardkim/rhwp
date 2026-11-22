# PR #9280 검토에서 확인한 기호 표시와 인접 코드

## HWPX를 저장한 뒤의 문자

기여자가 제시한 기호 15개는 버전별로 올바르게 저장된다. Windows 한컴 2020과 2024가 만든 HWPX, 기여자의 macOS 한컴 HWPX와 문자 및 굵은 B의 UTF-16 시작 위치가 모두 일치한다. Windows 한컴 2020으로 원본 HWP와 후보가 저장한 HWPX를 각각 열어 인쇄했을 때 두 버전 모두 96 dpi 전쪽 RGB 픽셀이 같았다. [문자·서식 30행](assets/pr9280/symbols.tsv) · [한컴 실행 근거](assets/pr9280/hancom-jobs.tsv)

5.0.1.6의 저장본을 rhwp로 다시 열면 `export-text --json`에서 아래 네 문자가 다른 문자로 나온다. 메인테이너가 한컴에서 직접 저장한 HWPX를 기존 devel에서도 열어 같은 결과를 얻었다. 특히 `U+A832` 줄에서 제품은 이중 가로선을 그리며 Windows 한컴 Print와 모양이 다르다. 이번 PR이 새로 만든 회귀로 분류할 근거는 없다. [1쪽 비교 이미지](assets/pr9280/modern-native-review.png)

| HWPX 원문 | rhwp 추출 |
| --- | --- |
| U+A807 | U+252C ┬ |
| U+A80E | U+2518 ┘ |
| U+A810 | U+2502 │ |
| U+A832 | U+2550 ═ |

HWPX writer는 원본 포맷과 파일 버전을 확인한다. 반면 표시·추출이 호출하는 `hancom_symbol_display_key(ch)`는 문자 값만 입력받아 BMP를 다시 Plane 15 키로 바꾼다. 이 경로는 최신 devel에도 있다. 기존 구형 HWP와 실제 보충 PUA의 선 기호는 계속 보호해야 한다. [저장 조건](../../src/serializer/hwpx/body_symbols.rs) · [표시 키와 텍스트 투영](../../src/renderer/composer/pua_surface.inc.rs)

## 원 표본 주변에서 추가로 확인한 네 문자

공개 HWP 두 개의 첫 기호 문단에 일반 편집 API로 여덟 기호와 ASCII 표식을 넣고 HWP로 저장했다. 두 파일의 FileHeader는 각각 5.0.1.5와 5.0.1.6이다. 5.0.1.5에서는 U+A805·U+A809·U+A847·U+A849 네 문자를 Windows 한컴 2020과 2024 모두 U+F0805·U+F0809·U+F0847·U+F0849로 저장했다. 후보는 원래 BMP로 남겼고 `export-hwpx --verify --verify-pages`를 통과했다. 5.0.1.6은 여덟 문자 전부 두 한컴 버전과 같았다. 네 문자는 원 PR이 검증 대상으로 명시한 15종 밖의 기존 누락이다. [기호별 대조](assets/pr9280/neighbors.tsv) · [5.0.1.5 입력](assets/pr9280/neighbors-v5.hwp) · [5.0.1.6 입력](assets/pr9280/neighbors-v6.hwp)

## 메인테이너 보완안

1. 현대 HWP5·HWPX의 본문 BMP와 구형 HWP5의 선 기호를 구분하는 근거를 공통 표시·추출 경로에 전달한다. 실제 Plane 15 문자, 글자 겹침과 구형 문서의 표시 의미는 대조군으로 검사한다.
2. 인접 네 문자는 독립 저장본이 일치한 값만 구형 본문 변환표에 보완한다. 다른 코드의 의미 또는 BMP 구간 전체의 대응은 추정할 수 없다.
3. 원 15종과 인접 대조군으로 저장·재열기, 추출, 서식·줄 경계, Native·fresh WASM 직접 표시를 재검증한다. 표시 코드 변경 시 편람과 특별 보호 문서의 쪽별 하한 및 전체 필수 검증을 적용한다.

원 기여의 저장 수정은 독립 인쇄까지 확인했다. 기존 표시 문제와 추가 네 문자를 이 PR에서 메인테이너가 함께 맡는 것은 별도 작업 범위다.
