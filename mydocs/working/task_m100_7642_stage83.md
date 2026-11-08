# #7642 Stage 83 — 수학 중괄호와 NOT의 공통 표시 기하

Issue: #7642

## 독립 기준과 원인

Stage 82의 NOT 인자 소비 보정은 Native의 여섯 진단 입력에서
98.99% 이상이지만 확대 판독에서는 표시 중괄호의 높이가 다르다.
독립 Print에 포함된 HYhwpEQ subset을 메모리에서 조회한 결과
수학용 U+E04B의 잉크 범위는 (113,-203,446,821), 전진폭 594,
UPM 1024다. 설치 글꼴에서도 같은 메트릭이다. 일반 ASCII `{`는
y=-146..745로 더 작다. 같은 문자 모양이라고 같은 프레임으로
취급한 가정을 제거하며 사설 글꼴 윤곽·바이트를 복제하지 않는다.

동봉된 공개 Latin Modern Math의 `braceleft`는 UPM 1000,
잉크 범위 (75,-250,425,750)이다. WOFF2와 로컬 설치 OTF에서
추출한 윤곽 명령의 해시가 같음을 확인했다. 이 공개 윤곽을
수학 글리프의 잉크 범위로 변환하고 기존 측정 전진폭을 보존한다.
원 저작자와 GUST Font License를 코드 및 THIRD_PARTY_LICENSES에
유지한다. 한컴에서는 메트릭만 참조한다.

독립 Print의 6/9/12/18pt 중괄호와 뒤 변수의 원점 차이는
600dpi 인쇄 단위에서 3/4/5/8이다. 명목 em의 5%를 그 인쇄
격자에서 반올림한 값과 같다. 문서 위치나 고정 화면 px가 아닌
글자 크기·출력 DPI·독립 인쇄 메트릭으로 표시 원점을 계산한다.
사선의 장평도 문자 전진폭과 인쇄 글꼴 전진폭을 대조한다.
추가 20개 입력의 작은/큰 문자·숫자·중괄호·정상 등호를
한컴 저장/Print로 출력해 적용 범위를 확인한다.

## 생산과 실제 소비 경로

`layout/decoration_geometry.rs`가 측정한 body에서 실제 NOT
끝점과 굵기를 발행하고 `LayoutKind::Decoration`에 보존한다.
SVG, Canvas와 Native Skia는 이 결과를 소비한다. 출력 backend가
DPI나 글자 종류를 다시 추측하지 않는다. 다른 face와 다른 장식은
기존 계약을 보존하며 HYhwpEQ의 독립 기하와 구별한다.

추가로 `paint/json.rs` → Studio의 `LayerEquationLayoutKind` →
CanvasKit의 decoration 분기를 실제 대조했다. 현재 CanvasKit은
strikeThrough를 bar/overline과 함께 수평선으로 처리한다. Native와
fresh WASM의 SVG 점수가 이 화면 경로의 정확성을 입증하지 않는다.
같은 NOT 기하를 paint JSON으로 전달하고 CanvasKit이 그대로 그린다.
측정 기하가 없는 과거 JSON은 SVG 대체 경로로 보내며 수평선을
NOT의 대용으로 쓰지 않는다. 공통 굵기를 고정 px 최솟값으로
덮어쓰지 않는다. 중괄호의 공개 윤곽도 기존 FontGlyph 공통
표시 경로를 사용한다.

## 검증 범위와 남은 작업

Stage 82 fresh WASM 빌드의 exit 0 및 package 고정을 확인한 뒤
소스를 변경한다. 문자·숫자·그룹과 정상 등호·부등호, LaTeX
인자, 다른 장식·다른 face를 대조한다. DPI가 바뀌어도 같은 물리
메트릭인지 확인하고 실제 CanvasKit에서도 사선과 공개 윤곽을
확인한다. Native/fresh WASM 전체 페이지 TSV와 같은 영역의
확대 판독을 연결한다. API 필드의 기계적인 match 수정은 기존
assertion을 유지하며 기대값·baseline을 변경하지 않는다.

전체 원본 55개·853쪽 및 관련 회귀의 관계형 전환 범위를 유지한다.
새 fixture·회귀 기대값은 관련 모든 페이지 90% 이상과 확대
판독 뒤에만 추가한다. 현재 단계는 독립 진단과 후보 보정 중이다.

## 추가 Print와 후보 구현

20개 추가 입력의 정상 저장 job
`91ddf4a8-b2c5-4977-9b70-22387c4c27b7`와 Print job
`d0540921-2f3e-4cc4-a7ec-1a12cf3e313d`가 성공했다. 한컴 2024
13.0.0.3901, Print method 0, `frame_print_to_pdf_ex_one_up`이며
PDF SHA-256은 `1f3df53f0992c8e060d968cbf44892e60b52fe9e8c8126308688cb305edbfef5`다.
MCP 인쇄 입력은 정상 저장 HWP이고 원본 HWPX는 보존한다.

Print의 Tf/Tz를 직접 읽었다. 문자·숫자·중괄호 사선은 글자
상자의 전진을 인쇄 단위로 환산한 값의 1.2배를 반올림하고,
사선의 반각 자연 전진을 같은 인쇄 단위에서 반올림한 값으로
나눈 장평이다. 등호는 문자 전진 전체를 사용한다. 좁은 x, 넓은 W,
숫자 7, 중괄호, 등호의 네 크기 대조가 이 규칙과 일치한다.
예를 들어 9pt의 x/중괄호는 139.473694%, W는 236.842102%,
숫자는 118.421051%, 등호는 147.368423%다. 글자 크기와 같은
물리 전진에서 계산하며 특정 문자열별 장평 표를 추가하지 않는다.
각진 관계 기호·복합 인자·본문 대체 face는 기존 대조의 경로를 유지한다.

레이아웃이 DPI를 사용해 NOT 기하를 발행하고 각 backend는
보존된 값을 소비한다. CanvasKit은 없거나 무효인 기하를 먼저
확인해 SVG 대체 경로로 보내며, 측정 굵기를 0.5px로 덮어쓰지
않는다. 공개 중괄호 윤곽은 FontGlyph의 기존 공통 표시 경로를
사용한다. 기존 match의 필드 추가는 assertion을 그대로 유지한다.
현재 후보 소스의 빌드와 출력 확인은 아직 미완료다.

## 후보 소스의 실행 결과

검증 소스는 `cd1b2b029979e16573f533c017ddaec0f34447cc`다.
Native `cargo build --locked --profile release-test --target-dir
/home/tsjang/rhwp/target/pr-review --bin rhwp`는 exit 0,
433.23초다. 바이너리 SHA-256은
`485da26dbdca12a0d094d77abc3170bf06ad433a181b8ea470cdc051a5b32696`이며
완료 뒤 검토 출력에 고정했다. fmt check와 Native Clippy도 exit 0이다.
fresh WASM과 나머지 필수 Rust 게이트는 아직 완료하지 않았다.

Native의 독립 NOT 경계 입력 여덟 개가 모두 같은 Print PDF와
비교되어 gate를 통과했다. 페이지별 값은 다음과 같다.

| 입력 | 2px 이웃 관용 내용 실루엣 일치율 |
| --- | ---: |
| 글자 크기별 원본 HWPX | 99.52933% |
| 글자 크기별 한컴 저장 HWP | 99.50151% |
| 인자 소비 원본 HWPX | 99.29469% |
| 인자 소비 한컴 저장 HWP | 99.66142% |
| 부정 기호 원본 HWPX | 99.98512% |
| 부정 기호 원본 HWPX v2 | 99.97387% |
| 부정 기호 한컴 저장 HWP | 99.90938% |
| 부정 기호 한컴 저장 HWP v2 | 99.84032% |

증거는 ignored
`output/pr-review/issue7642/failed-inputs-20261008/stage15-current-validation/`
아래 `not-glyph-frame-focused-native/queue.json`과 각 문서의
review·overlay·render tree다. 새 확대 그림
`stage83-not-glyph-frame-selected-detail-review.png`는 12/18pt의
문자·넓은 W·중괄호·등호를 같은 물리 영역에서 대조한다.
overlay의 도구 제목 높이는 잘라낼 때만 제외하며 본문을 이동하거나
정렬하지 않는다. 이 진단용 crop 좌표를 정식 회귀 검사에 사용하지 않는다.
낮은 해상도에서 높이·기울기 개선을 확인했고, 고해상도 확대와
fresh WASM 및 실제 CanvasKit 화면의 대조는 계속 진행한다.

Studio `tsc --noEmit --pretty false`가 exit 0이고 기존
`canvaskit-document-preflight.test.ts`와 `canvaskit-font-plan.test.ts`는
10개 PASS, 실패·skip 0개다. 별도 ignored 진단
`diagnose-canvaskit-not.mjs`는 수정 전후의 실제 renderer 메서드를
실행하되 `drawLine`과 paint를 대역으로 관측했다. 수정 전 수평선,
수정 후 전달한 사선·굵기 보존, 기하 누락 시 SVG 대체 반환을
확인했다. `stage83-canvaskit-not-contract.json`에 PASS를 기록했다.
이는 실제 CanvasKit/WASM 화면 캡처나 새 정식 회귀 테스트가 아니다.

새 정식 회귀는 아직 추가하지 않았다. 기존 golden helper의 변경은
새 Decoration 필드를 허용하는 `..` match뿐이며 assertion과
기대값을 유지했다. 전체 원본 55개·853쪽의 Native TSV 산출을
같은 고정 바이너리로 시작했다. 관련 px 검사의 관계형 전환,
전체 fresh WASM TSV, 전체 회귀와 최종 PR 검증은 남아 있다.

### 300dpi 확대 판독의 잔여 차이

동일 원본·Print PDF를 300dpi로 rasterize한 진단은 94.57865%다.
2px 관용의 물리 크기가 96dpi와 다르므로 두 점수를 직접적인
개선율로 비교하지 않는다. 새 그림은
`stage83-not-glyph-frame-300dpi-selected-detail-review.png`이며
문단 라벨과 수식 노드로 선택한 동일 영역을 확대했다.

중괄호 높이와 NOT 사선의 큰 차이는 가까워졌지만 `W`의
글자 모양·기울기가 다르다. 해당 SVG는 Latin Modern Math의
U+1D44A를 표시하고 Print는 HYhwpEQ의 U+E016을 표시한다.
각 잉크 범위는 UPM 1000에서 (55,-22,1048,683), UPM 1024에서
(19,-20,1033,702)다. 논리 전진과 실제 표시 face의 모양을 구분해야
하며 같은 사선만으로 수식 전체를 완료 판정할 수 없다.
관측값은 ignored `stage83-micro-findings.json`에 보존했다.
이 차이의 보정과 직접 판독은 아직 미완료다. 점수 통과로
면제하거나 새 회귀 기대값으로 고정하지 않는다.

Stage 82 집중 검증 중 ENOSPC가 발생했다. 실제 프로세스가
모두 종료된 상태를 확인했고 완료된 Native 10문서와 WASM
7문서의 결과를 보존했다. 새 source의 검증 완료로 승격하지
않는다. 종료된 이전 출력의 중복 `font_policy/*.svg`는 해시
원장을 기록하고 정리하며 입력·PDF·PNG·TSV·로그·tree와 공용
`target/pr-review`를 보존한다. 다음 실행의 문서별 종료 정리에도
이 중복 SVG를 포함해 공간 소진을 반복하지 않는다.
