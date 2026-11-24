# F11 누름틀 입력과 기울인 도형 크기 편집의 보완 검증

2026년 11월 24일, semanticist21의 [#9288](https://github.com/edwardkim/rhwp/pull/9288)과 [#9314](https://github.com/edwardkim/rhwp/pull/9314)를 함께 검토했다. `devel`은 `9ee9a5b0b12115f06b31e07ec89b94e454c1a744`이다. 원 기여 두 건의 커밋 `3fc45e7057300b21c49ced254cb476fdab69ac39`, `0cb5fcd1c941ec24e83d8420597dbeb5586e4b05`를 모두 보존했다. 메인테이너 보완을 포함한 제품 및 테스트 소스는 `10f5bca362a34f3c3f27ac15e5d921d17d1d8097`이다.

## #9288 같은 자리에 있는 두 누름틀

본문과 표 셀에서 F11을 한 번 누르면 ‘나’, 두 번 누르면 ‘가’를 선택한다. 편집기는 선택한 컨트롤과 해당 문단·셀의 주소로 코어에서 현재 필드 범위를 다시 조회한다. 첫 입력 직후 화면 갱신을 기다리는 동안에도 선택한 필드의 입력 소속을 유지한다. 문서를 바꾸거나 선택 밖으로 이동하면 그 상태를 해제한다.

| 실제 Studio 동작 | 검증 결과 |
| --- | --- |
| 본문·셀에서 F11을 1번 또는 2번 누른 뒤 영문 키로 `xy`, 한글 조합으로 ‘한’과 ‘한글’ 입력 | 12개 조합에서 입력 직후, HWP 재열기, HWPX 재열기의 필드별 값 일치. 같은 HWPX 12개에서 XML 필드 시작·끝 사이의 실제 글자도 일치 |
| 첫 입력과 다음 입력 사이에 화면 갱신이 없는 경우 | 보완 전 ‘가’에 `x`, ‘나’에 `y`가 저장됐다. 보완 뒤 현재 문서와 두 저장 형식 모두 ‘가’에 `xy`가 남았다. 실제 키 입력 외에 두 입력 이벤트를 한 번의 브라우저 작업 안에 보내 도착 순서도 고정했다 |
| F11로 다음 필드 선택, 이미 채운 값 교체, End 뒤 본문 입력 | 선택 이동 뒤 ‘가’와 ‘나’의 값이 각각 보존됐다. 본문과 셀 양쪽에서 선택한 `xy`를 `ab`로 교체했다. End 뒤 입력한 느낌표는 문단 끝에 남았다 |
| 공개 WASM API | wasm-pack 산출물·Studio public·배포용 dist 각각에서 메서드 3개의 TypeScript 소비 예제를 컴파일하고 실제 JavaScript로 필드 조회·활성화를 실행했다 |

누름틀 E2E는 46개 확인 항목을 통과했다. 각주·머리말에서 선택한 필드로 입력하는 기여는 [#9354](https://github.com/edwardkim/rhwp/pull/9354)에 있다. [입력별·형식별 필드 값](f11-roundtrips.tsv) · [HWPX XML 독립 판독](field-xml.tsv) · [브라우저 검사](browser-results.json) · [배포 타입 소비](typescript-consumption.json) · [JavaScript 소비](wasm-public-consumption.json)

## #9314 기울기와 크기의 저장

한컴 12.30에서 만든 공개 도형 문서를 Windows 한컴에서 열어 오른쪽 위 평행사변형의 너비를 14,912에서 16,000 HWPUNIT로 넓히고 원래 너비로 되돌렸다. 외접 상자의 크기 변화가 원래 전단 행렬에 합성돼야 한다. 너비 복원 뒤에는 20° 기울기와 원래 좌표가 복원된다.

Windows 한컴 2020에서 직접 편집·저장한 HWP와 HWPX를 각각 대조본으로 사용했다. 한컴 2024에서 이 대조본과 rhwp가 저장한 파일을 다시 인쇄했다. 형식에 따라 저장 행렬의 정밀도가 달라 복원된 현재 너비는 HWP에서 12,000, HWPX에서 11,999다. 후보는 각 형식에서 해당 값을 보존했다. 허용하지 않는 기울기와 크기를 한 요청에 함께 넘길 때도 거절 후 기존 너비·회전·기울기가 보존된다. [저장 크기와 행렬 대조](resize-geometry.tsv) · [편집 16건과 형식별 결과](shape-edits.json) · [가로·세로 각도 조합](angle-changes.json)

| Windows 한컴 인쇄와 비교한 출력 | 보완 전 Native / WASM | 보완 후 Native / WASM |
| --- | ---: | ---: |
| 기울인 원본 문서 1쪽 | 99.97% / 99.77% | 99.97% / 99.77% |
| 너비를 넓혔다가 원래 너비로 되돌려 저장한 문서 1쪽 | 77.99% / 78.03% | 99.97% / 99.77% |

Native와 fresh WASM에서 원본, 너비 변경·복원, 저장 형식별 재열기 등 각 13쪽을 96 DPI의 독립 인쇄본에 대조했다. 수정 뒤 원본과 복원본 사이의 렌더링 차이는 두 경로 모두 0픽셀이다. 한컴 2024로 다시 인쇄한 HWP와 HWPX는 확대 상태와 복원 상태 각각에서 한컴이 직접 편집한 대조본과 0픽셀 차이였다. [전쪽 실루엣](shape-pages.tsv) · [Native 확대 비교](native-width-restored.png) · [WASM 확대 비교](wasm-width-restored.png) · [인쇄본 픽셀 대조](windows-and-backend-pixels.json)

Rust 소스 트리 `9169ca1526ea904c830c95bca13615aa4dc44014`에서 빌드한 Native 바이너리의 SHA-256은 `01cc4a87f25fd0e8e5fb237f7cc3fde8159a499ca722d2cedd705aa9cc825118`이다. Docker에서 최적화한 WASM의 SHA-256은 `c85e0de944ca5448feab4411a7ebeb3793b137fb98e91213e807f385797c05f5`이며 브라우저가 받은 파일의 해시를 대조했다. 공통 공개 대조 문서 4개의 Native·WASM 검사를 모두 통과했다. [공통 문서별 검증 결과](public-controls/results.json) · [전쪽 TSV와 대표 PNG](public-controls/)

#9376으로 들어온 세 기능도 현재 후보에서 다시 열어 확인했다. 첫 구역의 용지·번호가 있는 문서 4쪽, 반투명 타원 1쪽, 꺾임 1개인 연결선 1쪽은 양쪽 렌더러에서 한컴 인쇄 대비 100.00%였다. 12개 출력 PNG 모두 현재 devel과 같은 제품 소스로 검증한 결과와 바이트 단위로 같았다. [Native](devel-features-native.tsv) · [fresh WASM](devel-features-wasm.tsv)
