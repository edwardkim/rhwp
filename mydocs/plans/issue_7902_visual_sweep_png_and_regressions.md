# #7902 실행 계획 — Visual Sweep 직접 PNG 경로와 통합 후 회귀 보정

Issue: #7902. 작업지시: 먼저 SVG 출력을 최소화하는 Visual Sweep을 구현·검증한 뒤,
#7877의 문서 정확도 유지 실패를 보정하고 관계형 회귀 검사를 보강한다.
2026-10-14 후속 지시에 따라 Visual Sweep 개선만 먼저 별도 PR로 제출하고, #7902는 다음 단계에서 처리한다.

## 기준과 순서

- 작업 브랜치: `perf/visual-sweep-direct-png-20261014`.
- 최신 base: `b3eaf7b10b15a2d40810d42acdcf4de6c1ba026b`.
- 기존 출력 진단은 고정 실행 코드 `6cbebc7457ea8899c3949e48ed1255d43f40bcb2`의
  immutable CLI/WASM으로 완료했다. 현재 base를 검증한 결과로 재사용하지 않는다.
- 원본·독립 Print는 #7877 제출 manifest와 SHA가 같다. 이전 출력은 재실행하지 않고
  원 PR의 38 TSV를 읽었다.
- 먼저 직접 PNG 도구를 완성·실물 검증·커밋·별도 PR로 제출한다. 그다음 #7902 보정 단계로 진행한다.

## 1단계 — SVG를 생성하지 않는 기본 검증 경로

1. Native는 `native-skia`의 `export-png --profile print`로 출력한다. 같은 DPI·compat·글꼴
   환경을 전달하고 전체 쪽수와 실제 산출물 목록을 기계적으로 검증한다. JSON manifest를
   추가하고 원문 경로·PNG bytes·누락·부분 내보내기를 구분한다.
2. fresh WASM은 Print profile의 Canvas API로 직접 PNG를 만든다. Chrome/탭/문서를
   재사용하고 글꼴과 비동기 그림의 준비 상태를 확인한 뒤 캡처한다. 문서 내장 글꼴과 외부
   공급 face를 보존한다. 페이지별 SVG와 Native 글꼴 정책용 전체 SVG를 생성하지 않는다.
3. 원래 SVG 경로는 명시적인 진단 옵션으로 유지한다. backend를 provenance·checkpoint에
   기록해 서로 다른 렌더링 경로의 결과와 캐시를 섞지 않는다.
4. 현재의 TSV/compare/overlay/review·90% gate·전체 쪽수·내용/구조 분석·resume·페이지
   작업자 옵션을 유지한다. PNG 경로는 벡터 파일 생성이나 웹글꼴 임베딩 검사에 의존하지 않는다.
5. 실물 입력에서 전후 실행 시간·중간 산출물 용량·실루엣 값과 대표 PNG를 확인한다. Native
   Skia와 WASM Canvas 및 SVG/Chrome의 서로 다른 paint 경로를 구분한다.
6. 문서/CLI 안내를 갱신하고 관련 Python/Node 검사, 변경된 Rust CLI의 focused 검사와
   필수 lint를 확인한다. Cargo는 공용 `target/pr-review`, jobs8, 한 번에 하나만 실행한다.

## 2단계 — #7902 출력 보정과 재발 방지

기존 고정 소스의 Native/fresh SVG 비교는 각각 19문서 중 13통과/6미달이었다. 측정 가능한
18문서는 각 787쪽이며 9쪽이 90% 미만이다. overfill은 49쪽/Print 48쪽으로 측정 불가다.

- 화학제품 원본과 #6761 세 대조본: 쪽번호만 있는29쪽76.10619%, 말미86%대.
- 행 나눔HWPX15쪽87.20887%: 그림/본문/표 경계를 직접 판독한다.
- giant overfill: 종료 조각과 후속 표의 이월을 전체 내용 대응으로 확인한다.

1. 값의 생산→측정→실제 배치→최종 원점/높이 소비를 대조한다. 그림/표의 선언 상자와
   실제 점유·빈 띠·종료 여백·뒤 문단을 함께 분석하고 일반 규칙에 따라 원인을 보정한다.
2. #7877의 같은 19문서 전체를 새 Native/fresh WASM 코드로 재검증한다. 한컴 Print 쪽수
   일치, 전쪽 90% 이상과 확대 판독을 확인한다. SVG 경로에서 확인된 결함은 해당 경로도
   직접 확인한다. 점수와 원인 분류를 분리하며 backend 변경으로 남은 결함을 숨기지 않는다.
3. 출력이 먼저 통과하면 관계형 회귀를 추가/보강한다. 쪽번호의 스타일·소속, 그림 뒤 내용
   순서·간격 소유, 표/셀 내용의 보존·포함·누락/중복 및 이어받기 종료를 보호한다. 절대px
   핀이나 허용치 완화로 현재 출력을 고정하지 않는다. 수정전 의도한 원인의FAIL과 수정후
   PASS·정상 대조군을 기록한다.
4. 집중 검사와 필수 lint, 필요한 최종CI를 확인하고 PR별 review·오늘할일·본문 증적을 갖춘다.
   기존 전체 회귀를 기계적으로 반복하지 않는다. 범위가 모두 충족된 뒤 #7902 종료를 주장한다.

## 증거와 디스크

ignored `output/pr-review/pr7877-after-rebase-20261014/`와
`output/pr-review/visual-sweep-direct-png-20261014/`에 원시 기록을 보존한다.
완료된 중간 SVG만 정리하며 원본 HWP/HWPX/HML/PDF·PNG·TSV·로그·글꼴·공용 target은 보존한다.

## 페이지별 좌표 회귀 지침

완료된 페이지의 Native/fresh WASM 2px 관용 실루엣이 각각 98% 이상이고 독립 PDF와
확대 판독을 마치면 ±2px 좌표 회귀를 허용한다. 해당 쪽은 이후에도 양쪽 98% 이상을 유지하고
100%로 개선은 허용한다. 다른 쪽까지 98%일 필요는 없으며 전체 검증의 90% gate는 유지한다.
정식 조건은 `mydocs/manual/pr_review/visual_fixture_evidence.md`를 따른다.
