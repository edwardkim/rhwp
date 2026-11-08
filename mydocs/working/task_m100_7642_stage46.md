# 이슈 #7642 Stage 46 — 실제 미주 경계의 설정 간격과 그림 원점

Issue: #7642

## 시작점과 독립 근거

시작 source는 `981632ee9b8217d20a555bc90164ba8f4ec093b0`, 정책 base는
`79abd9e49675f4286eaed38197f7e837afb744e7`이다. 전체 요청 55문서·853쪽의
Native TSV를 이 source의 고정 바이너리로 산출 중이다. fresh WASM과 전체 회귀는
미완료이며 범위를 축소하지 않는다.

9/8/7 HWPX 10쪽의 미주 6 본문은 독립 Print와 같은 순서·거의 같은 위치다.
마지막 수식 줄의 Native 원점은 236.5, Print 잉크 상단은 236.48이다. 다음
미주 7 제목은 Native 254.5 / Print 278.664로 갈린다. 섹션 설정의 미주 사이
`betweenNotes=2267` HU는 30.2267(96 DPI)이며, 앞 수식 줄 상자 높이 12와 이
간격을 합하면 독립 제목 원점과 일치한다. 원본 마지막 줄의 간격은 452 HU다.
현재 layout은 앞줄 간격이 1000 HU를 넘는 삽입 마커에서만 미주 간격을 읽어
설정값을 잃는다. 이 진단 좌표는 원인 추적용이며 회귀 선택자·기대값으로 쓰지 않는다.

구분선 없는 HWPX 11쪽의 BMP 그림은 형상·물리 크기가 맞지만 실제 페이지에서
Native 상단 862.3 / 독립 Print 이미지 객체 상단 855.384로 갈린다. PDFium의
실제 이미지 객체 bounds로 확인했고 페이지 잉크 상단의 추정값으로 대체하지 않았다.
이전 단계의 100% 그림 프레임 보조값은 이 원점 차이를 판정하지 않는다.

## 소비 경로와 묶음 보정

- 섹션의 미주 사이 설정 → 실제 앞 미주 마지막 줄 소유 판정
  (`endnote_spacing::starts_note_after_complete_tail`) → layout의 실제 줄 점유 끝
  → `origin_after_gap` → HeightCursor의 후속 원점 이동을 연결한다. 완전한 미주
  경계에는 설정값을 사용하고, 첫 미주·같은 미주 내부·새 단 첫머리에 이를 덧붙이지 않는다.
  제목 문자열이나 삽입 마커의 크기로 실제 경계 소유를 대신하지 않는다.
- 저장 BMP 프레임 → `picture_frame::relative_frame` → 현재 단의 실제 paint
  `flow_bottom` → `picture_frame::at_origin` → float placement의 host·occupied·후속
  flow 소비를 연결한다. 실제 흐름 결과가 없는 경로의 기존 fallback은 명시하고
  미검증으로 남긴다. 그림 크기를 바꾸거나 고정 좌표로 원점을 맞추지 않는다.

## 검증 범위

관련 수정을 함께 빌드한 뒤 기존 다섯 쪽수 실패, 2022년 11월 정상 대조군,
9/8/7 HWPX 10·11쪽과 구분선 없는 HWPX 11쪽을 비교한다. 제목 간격과 그림
후속 수식까지 직접 판독한다. 개선된 범위를 확인한 뒤 같은 source의 전체
Native/fresh WASM TSV·확대 판독·관계형 회귀·전체 회귀를 이어간다. 모든
실행 로그·TSV·JSON·nextrun은 ignored output에 두며 Git에 포함하지 않는다.

## Native 실행 결과와 확대 판독

빌드 입력은 계획 head `70c8e1113`와 이 단계의 두 production 파일 변경이며
전체 1,453개 source 파일의 SHA-256을 빌드 전에 기록했다. 고정 review target의
`cargo build --locked --profile release-test --bin rhwp`는 exit 0(407.88초),
`cargo fmt --all -- --check`와 Native `cargo clippy --locked -- -D warnings`도
exit 0이다. 정확한 명령·바이너리 해시·입력 목록은 ignored
`endnote-owned-gap-native-provenance.json`, `endnote-owned-gap-native-lint.json`에 있다.

| 원본 / 검증 범위 | 이 단계의 실제 관측 |
| --- | --- |
| 기존 다섯 쪽수 실패 | 모두 독립 Print와 일치: 네 원본 22/22, 9/8/7 HWPX 21/21 |
| 9/8/7 HWPX 10·11쪽 | 96.43106% / 97.80799%, 선택 페이지 gate 통과 |
| 구분선 없는 HWPX 10·11쪽 | 97.62596% / 97.94297%, 선택 페이지 gate 통과 |
| 2022년 11월 HWPX 정상 대조군 4쪽 | 98.74440%, gate 통과 |
| 구분선 없는 HWPX 11쪽 BMP 원점 | 862.3 → 856.3; 독립 Print 이미지 객체 855.384 |

9/8/7 HWPX의 10쪽 미주 7 제목은 Native 278.7 / Print 278.664, 11쪽 미주 10
제목은 Native 182.2 / Print 182.184로 복구됐다. 그림 크기는 유지했다. 이 좌표는
원인 진단용 관측값이며 고정 px 회귀로 추가하지 않는다.

확대 판독에서는 새로운 미충족 항목을 확인했다. 9/8/7 HWPX 10쪽의 미주 7
합·근호·분수 여러 줄은 전체 페이지 96.43%와 달리 같은 영역의 2px 내용 실루엣
보조값이 **86.02059%**다. Native의 수식 너비와 기호·간격 차이가 남는다.
해당 영역은 문단 소유 483~486의 렌더 상자 합집합으로 선택했고, 독립 Print와
Native의 같은 페이지 영역을 같은 배율로 대조했다. 페이지 overlay에 붙은 제목
여백을 원본 좌표에 섞지 않고 두 원본 PNG로 차이 색상을 직접 다시 합성했다.

새 그림·입력 경로·영역 유도 과정은 ignored
`endnote-owned-gap-note7-equations-detail-review.png`와 같은 이름의 JSON에 있다.
대표 페이지 review는 `endnote-owned-gap-focused-review/all-31/all-31/review/`
아래에 보존했다. 경계 소유 분석은 `endnote-owned-gap-boundary-analysis/summary.json`,
세 문서 선택 Sweep은 `endnote-owned-gap-focused-review-results.json`에 있다.

**최종 판정: 미주 경계·그림 원점 부분 보정 완료, 수식 확대 판독 미충족.**
관련 회귀 기대값을 바꾸지 않았다. 전체 55문서·853쪽의 최신 Native/fresh WASM
및 전체 회귀는 미완료다. 페이지 90% 통과만으로 승인·PR·완료로 판정하지 않으며,
다음 단계에서 수식의 실제 글꼴 메트릭·선언 프레임·장평·기호 간격을 함께 분석한다.
