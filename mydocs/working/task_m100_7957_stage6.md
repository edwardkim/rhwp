# Task #7957 6단계 — F5 통합 검증과 이름 기반 도형 색상 보정

- 공동 Issue #7957–#7963. F4 checkpoint `995be944b` 이후 F5 진행 중이다.
- 실물 6개 전체 313쪽을 새 Docker WASM으로 열고 실제 page count를 대조했다.
  유효 PDF 크기 경계·늦은 원본 쪽 선택·실패 뒤 새 요청을 포함한 Worker 17건과
  실제 browser API/Worker 검사가 통과했다. Studio 실제 WASM 응답 body의 SHA도 확인했다.
- 변경 전 `995be944b`의 전체 회귀 10595 PASS, workspace build/Clippy·wasm32 Clippy,
  Studio 타입 검사·테스트 2024 PASS/2 skip를 보존했다. 아래 색상 보정 후 결과로 표시하지 않는다.
  전쪽 WASM TSV는 extraction/decoded 각 14쪽 전부 100%, pages 162쪽 최저 98.91134%다.
  image·paths·operators에는 미달이 남아 PR 완료 판정은 보류한다.

## 사용자 지적과 실제 원인

사용자가 `bounded_import_path.pdf`의 검정 텍스트 배경을 지적하여 이번 범위에 포함했다.
이 입력은 `/Cs1 [/ICCBased ...]`의 RGB Alternate를 쓰며 `cs`와 `sc`로 흰 배경·유색 괘선을
지정한다. 경로 추출기가 해당 연산을 무시해 이전 검정을 재사용했다. 최소 이름 기반 RGB PDF도
기존 제품 WASM과 Native에서 검정으로 출력되어 마스크·글꼴 문제와 구분했다.

독립 근거는 ISO 32000-1:2008 8.6.5.5, 8.6.8(Table 74)다. 원본 PDF의 Poppler 출력과 비교한다.
`path_color.rs`에서 현재 page/Form의 자원을 해석하고 q/Q 상태에 해석된 색 공간을 보존한다.
`cs/CS`, `sc/SC`, 패턴 이름 없는 `scn/SCN`, `g/G`, `rg/RG`, `k/K`를 페이지와 Form이 같은
함수로 소비한다. 직접 Device 연산도 색 공간을 변경하며, 공간 선택 시 규정된 초기값을 사용한다.
이름/프로필/Range의 참조는 기존 요청 load/copy/visit·깊이·sticky 한도 안에서 해석한다.

실제 소비 연결은 `document.rs`의 page/Form operator loop → `PathGraphicsState`의 공간·RGB →
`PathExtractor`의 `PathContent.fill_color/stroke_color` → frontend `path_paint.rs`의 shape fill/
벡터 paint → Native/WASM이다. PDF의 위치 기반 가져오기이므로 HWP/HWPX의 측정·줄 조판은
변경하지 않았다. legacy paths-only parser도 이름 기반 색 명령과 BT/ET 안 그래픽 상태를
보존한다. bounded 경로는 기존 공통 cached full operator 결과를 계속 사용한다.

ICCBased는 ICC CMM이 없는 제품 경로의 Alternate 또는 N에 따른 규정된 Device 기본 공간을
사용한다. 모든 ICC/CIE/Separation/DeviceN/Pattern 지원을 완료했다고 주장하지 않는다.
좌표 clamp·문서 ID 조건·출력 삭제로 검정 영역을 숨기지 않았다.

## 현재 검증 경계

- Native Skia 새 출력: 해당 4쪽 일치율 97.04914 / 99.45266 / 99.46265 / 98.61733%.
  검정 배경이 없어지고 원본의 흰 배경·붉은 표제·청록 괘선이 표시된다.
- 생성 대조군 3쪽: Device/이름 별칭, q/Q, 직접 Device 전환, 다른 이름 scope의 Form,
  BT/ET 상태, ICC Alternate와 fill/stroke의 Native 일치율 모두 100%.
- regular 글자 획의 차이는 확대 판독에서 별도로 관측했다. 배경 보정의 효과와 구분하며
  점수만으로 사용자 시각 통과를 선언하지 않는다.
- fresh Docker WASM·같은 7쪽 직접 Canvas 비교·정식 색상 회귀의 선행 gate와 최종 회귀는
  이 checkpoint 뒤 진행한다. 아직 원격 push·PR 생성·완료 처리는 하지 않았다.

명령·binary/source hash·원본/전후 PNG·전쪽 TSV는 ignored
`output/pr-review/issue7957/f5/`와 `f6/`에 보존한다. `parserAllocationBoundsProven=false`를 유지한다.
기본 Studio 7700과 그 bundle·사용자 타입 변경은 보존하며 검증 서버는 7797을 사용한다.
