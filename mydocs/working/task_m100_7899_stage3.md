# #7899 — 편집 저장 후 수식 줄 소속 보정 진행

계획: [수행](../plans/task_m100_7899.md), [구현](../plans/task_m100_7899_impl.md). 이전 저장 검증: [단계 2](task_m100_7899_stage2.md). **진행 중이며 시각 gate·PR 준비 완료가 아니다.**

## 사용자 판정과 원인 계층 정정

작업지시자는 원본이 한컴·Studio 모두 정상이고 한컴의 편집 저장본도 정상이나, rhwp의 저장 후 출력에서 조판이 틀어진다고 확인했다. 단계 2의 base/current 동일 SVG 관측은 같은 **편집 저장본**의 비교였으므로 tail 보정의 새 회귀가 아니라는 근거일 뿐이다. 원본도 틀렸다는 결론이나 사용자 판정의 대체 근거로 사용하지 않는다.

기존 Docker WASM의 실제 API에서 원본 → A 삽입 → export → 재열기 순서를 실행했다. 큰 수식은 원본의 TextLine 자식에서 편집 재조판 직후 Column 직속으로 바뀌고 export/reload에서도 유지됐다. 사용자에게 저장 뒤 관측된 증상의 생산 지점은 편집 재조판이며, API 결과와 화면 캐시 상태를 구분한다. Node API에 전체 검증 글꼴을 공급하지 않았으므로 그 좌표·글꼴은 PDF 일치의 증거로 사용하지 않는다. 개체 소속 변화와 파일의 원시 줄 경계는 독립적으로 확인했다.

## 보정한 생산·소비 경로

- 원본 큰 수식의 PARA_TEXT 시작과 LineSeg 시작은 UTF-16 41이다. A 삽입 후 실제 수식 시작은 42이나 기존 편집 재조판은 뒤 가시 개행 위치인 50을 LineSeg 시작으로 발행한다.
- `flow_inline_controls`에 실제 제어문자 시작을 전달하고, 일반 reflow·프레임 채움의 줄 범위가 같은 원시 개체 경계를 소비하게 했다. 첫 줄은 기존 문단 시작 0을 유지한다. 구조 마커·float·비인라인 개체를 앞 줄에 재배정하는 조건은 추가하지 않았다.
- `Paragraph.line_segs` → serializer → composer → paragraph layout에서 새 시작 42를 소비한다. Native 보정 저장본을 **기존 WASM**으로 다시 열었을 때도 큰 수식이 TextLine 자식으로 돌아왔다. 이것은 생산 경계 보정의 진단이며 fresh WASM editor 검증을 대신하지 않는다.
- 수식 자체의 원본 baseline 속성은 큰 수식 60%, 작은 수식 85%이다. 원본 큰 줄의 baseline 2457/height4095도 60%다. AST 내부 글리프 baseline과 개체의 줄 정렬 baseline을 구분하고, reflow와 frame이 동일 helper로 후자를 소비하도록 했다. 최신 Native 저장본은 시작 42, 높이 4136, baseline2481을 발행한다.
- 원본 문단의 attr1 bit29는 텍스트 기준 테두리 속성이다. resolved style이 누락하던 값을 전달해 paragraph border 범위 → 연속 문단 병합 → 공통 Rect로 소비한다. 기본 단 기준·wrap override 경로는 별도로 유지한다. 공식 도움말·독립 Print 근거는 구현 계획에 연결했다.

## 현재 검증과 남은 차이

Native Skia 빌드와 fmt를 실행했다. 구역 끝 목록·메모 보존 검사 9건은 줄 소속 보정, 테두리 보정 및 마지막 source baseline 보정 후보에서 각각 통과했다. 마지막 후보는 `storage-contract-tests-source-baseline.log`에 9 passed / 0 failed로 기록했다. 테스트 결과를 다른 source 후보나 렌더링 일치로 확대하지 않는다.

같은 A 편집 내용의 기존 독립 한컴 2020 Print를 진단 기준으로 비교했다. 줄 소속 보정 67.34039%, 테두리 보정 83.65798%, 한컴 설치 경로의 Haansoft Batang 공급 뒤 83.70927%였다. source baseline 계약까지 반영한 최신 후보는 **77.20043%**로 여전히 미달이다. 점수가 낮아진 사실을 숨기거나 기준값을 완화하지 않았다. 확대 대조에서는 큰 수식 중심의 변수와 cases 내부 행·괄호·첨자의 상대 배치가 별도 문제임을 확인했다. 아래 최신 같은 입력의 독립 Print와 fresh Docker WASM에서도 미달을 확인했다.

PDF의 HyhwpEQ 및 Haansoft Batang을 대조했고, 실제 설치된 `/mnt/c/Windows/Fonts`와 한컴 2020 `Shared/TTF/All`을 공급해 다시 캡처했다. 공급 부족과 조판 차이를 구분하며 글꼴 예외를 신청하지 않았다. 저장소의 원본 math_8 PDF는 A4인 반면 입력 용지는 272×394mm여서 그 출력의 낮은 직접 점수를 원본 결함이나 gate 근거로 승격하지 않았다.

## code head 40f0ec7de의 실제 저장·출력 실행

- Docker locked wrapper는 공용 target에서 8m04s 성공했다. Rust compile 4m50s 이후 wasm-opt까지 완료했다. root pkg와 Studio public의 WASM SHA-256은 `6292d46f31fcf1593c1b21ed19d34fcf9a3f15025bc28ac6c41cf0aacc0e2822`, glue는 `f377c6be818c64ffd20752e29271000ff5053ea8c7c984f65abb8fe74f17eddd`로 각각 같다.
- 실제 Node fresh WASM의 원본→A 삽입→export→재열기에서 큰 수식은 계속 TextLine 자식이었다. Native와 Node 저장본의 전체 파일 SHA는 모두 `e00edfc63c35d5753e1369bad72de5187a485e9a6d0e9406f5746cb7c7cebf52`이다.
- Chrome CDP 새 탭 `http://localhost:7799/?review=issue7899-save-reopen-40f0ec7de`에서 실제 응답 WASM hash도 위 값임을 확인했다. 편집 직후와 저장·재열기 후 5개 수식의 개체 소속·기하가 같고 모두 TextLine에 속했다. 브라우저 저장본은 font registration 상태 때문에 Native/Node와 전체 바이트가 같다고 주장하지 않는다. Native/Node/브라우저 세 저장본 모두 원본 tail을 보존하고 줄 시작 `[0,42,51]`을 유지했다. 새 탭을 저장본을 다시 연 상태로 남기고 직접 판정을 요청했다.
- 같은 최신 Native 저장본을 한컴 2020 11.0.0.9136으로 새 one-up Print했다. job `b1dce92f-ae09-4af7-ba97-e415be16e951`, preprocessing none, 1쪽이며 PDF hash `ab34d0ffd3647f417d75657c1f75d3f5ac7c19aa1e98a4e4edc6e8ee44796173`이다. 이 **정확한 입력의 PDF**를 사용한 전쪽 Native/fresh WASM TSV는 둘 다 **77.20043% / re_review_required**이다.
- source baseline 보정은 원본의 재측정 frame 경로에도 적용된다. 같은 글꼴을 로드하지 않은 Node 진단에서 원본의 수식 원점 변화도 관측했다. 원본 무회귀·수식 내부 정확성을 완료로 판정하지 않으며 직접 비교가 남아 있다. cases의 `&`가 고정 공백으로 처리되는 경로와 수학 축 정렬을 다음 보정의 원인 후보로 추적한다. 숫자 pin이나 glyph 숨김으로 처리하지 않는다.
- Source unit tier policy는 고정 base `b3eaf7b10` 비교에서 4103 tests / 296 modules로 통과했다. 새 렌더링 회귀/golden은 추가하지 않았고 기존 저장 계약 검사·원본은 유지한다. 전체 Rust/Clippy·원격 작업은 수행하지 않았다. code head를 바꾸면 영향 검증과 Docker WASM을 다시 실행한다.

ignored 증거 루트: `/home/edward/mygithub/rhwp/output/pr-review/issue7899-20261014/line-ownership-native/`.

- `save-boundary-before.mjs`, `save-boundary-before.json`, `before-*.json/svg`, `native-corrected-reopened-in-old-wasm.json`.
- `dump-preliminary.txt`, `dump-source-baseline.txt`, 각 후보의 Native build·focused test·fmt 로그.
- `visual-final`, `visual-border`, `visual-border-fonts`, `visual-source-baseline`의 전쪽 TSV·SVG·PNG·font inventory.
- `pdf-glyph-trace.json`, `native-glyph-trace.json`은 독립 출력과 실제 paint 좌표의 진단이며 절대 px 회귀 기대값이 아니다.
- `save-boundary-after.json`, `browser-stage3.json`, `final-save-record-check.json`, `studio-*.png/hwp`, `docker-wasm-build.log`.
- `oracle-source-baseline/`의 job·다운로드 결과와 새 Print, `visual-exact-print-native` / `visual-exact-print-wasm`의 전체 TSV·실행 로그.
