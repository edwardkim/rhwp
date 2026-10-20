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

### 새 Docker WASM과 직접 시각 판정

동일 runtime source `8684913e0`를 Docker에서 빌드했다. WASM SHA-256은
`c44975b3639fb05c3a37188da80af5bff7ea00c381f877a21bf81143168c38b7`이며 pkg/public이
일치한다. 실제 Studio 7797의 Network 응답 body가 이 hash와 일치하고, PDF 파일 선택 경로에서
4쪽 로드·browser 오류 없음도 확인했다. 해당 4쪽의 fresh WASM 일치율은
97.79679 / 99.77118 / 99.62293 / 99.03310%다. 사용자가 이 보정본을 직접 확인하여
**“시각 판정 통과”**로 판정했다. 이 판정은 이 문서의 배경·표제·괘선 보정에 연결한다.

생성 색상 대조군도 fresh Canvas 전쪽 100%이며 Native와 같은 세 쪽을 직접 판독했다.
그 뒤 `issue_7959_pdf_path_colour.rs`와 생성 PDF를 추가했다. q/Q·Device 전환·page/Form scope·
ICCBased Alternate와 stroke의 입력 색·객체 소유/순서, 최종 SVG까지의 색 보존을 보호한다.
수정 전 CLI의 최종 SVG에 같은 색 assertion을 적용한 결과는 FAIL, 보정 후 PASS다.
이 실행은 수정 전 Rust test binary 자체를 실행한 결과와 구분한다. 새 Rust 검사 실행 결과는
진행 중인 통합 검사 뒤 이어 기록한다. 테스트/자료 추가만으로 동일 Rust runtime의 WASM을
재빌드하지 않는다.

보존된 F5 debug 라이브러리에 같은 Rust 회귀 소스를 연결해 실제 실행한 결과는 2 FAIL이며,
원인은 이름 기반 색 값이 검정으로 남는 것과 최종 SVG의 회색/RGB paint 누락이다.
새 release-test 라이브러리에 연결한 실행은 2 PASS다. compiler·profile 차이·입력·원본 라이브러리·
test binary hash를 `f6/colour-tests-evidence.json`에 고정했다. 따라서 앞선 CLI 단독 증거와
별개로 Rust 회귀 자체의 수정 전 결함 검출도 실행했다. Cargo suite 편입 확인은 이어 진행한다.

새 Docker WASM의 실제 browser API/Worker 검사는 84 assertion·실제 Worker 5개·대조 문서
12쪽 render/save를 통과했다. 입력 크기 경계 등 F2/F3의 변경되지 않은 계약은 앞서 구분한
검증을 보존하고 반복하지 않았다.

전쪽 Native 313쪽 중 312쪽이 90% 이상이며 `content_operators.pdf` 12쪽만 88.96829%다.
글꼴·글자 간격 차이이며 색상 보정 전 `995be944b`의 WASM에서도 89.87994%로 관측됐다.
최초 devel 기준과의 동일 출력 여부는 이 관측만으로 확정하지 않는다. 다른 문서의 미달을 위
사용자 판정으로 면제하지 않는다. 대표 PNG·전후 비교는 `E:\tmp\issue7957-colour-review-20261017\index.html`에
복사했으며 모든 이미지 링크는 같은 폴더의 상대 경로다.

명령·binary/source hash·원본/전후 PNG·전쪽 TSV는 ignored
`output/pr-review/issue7957/f5/`와 `f6/`에 보존한다. `parserAllocationBoundsProven=false`를 유지한다.
기본 Studio 7700과 그 bundle·사용자 타입 변경은 보존하며 검증 서버는 7797을 사용한다.

### 색상 checkpoint 검증 완료와 다음 범위

runtime `8684913e0`의 workspace Clippy와 전체 회귀 10,595건이 모두 통과했다(45 skip).
새 색상 회귀 3건은 별도 실행에서 수정 전 3 FAIL / 수정 후 3 PASS이며, 마지막 검사는
색 공간 이름 순환의 sticky 한도와 다음 요청의 격리를 확인한다. 전체 회귀가 종료된 뒤
정식 Cargo suite에 편입한다. 이전 runtime의 Native Skia 추가 검사는 실행하지 않고
새 글꼴 보정 이후의 최종 검증에 합친다.

전쪽 fresh WASM도 313쪽 중 312쪽이 90% 이상이며 Native와 동일하게 operators 12쪽만
미달이다. 사용자가 이 쪽도 이번에 보정하도록 승인했으므로, 후속 백로그·시각 예외로
이관하지 않는다. 색상 보정에 대한 사용자의 통과 판정은 유지하고 다음 단계에서 PDF
글꼴 이름과 실제 선택·글자 간격을 추적한다.

Cargo suite 편입 뒤 `run-rust-test.mjs issue_7959_pdf_path_colour -- --cargo-profile release-test
--target-dir <공용 target>`도 정식 회귀 3건 PASS다. 위 10,595건과 구분하여 합계
10,598건의 서로 겹치지 않는 실행 결과를 확보했다. 다음 단계의 글꼴 runtime 변경에는
이 결과를 그대로 최종 검증으로 적용하지 않는다.
