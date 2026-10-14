# #7899 — 편집 저장 후 수식 줄 소속 보정 진행

계획: [수행](../plans/task_m100_7899.md), [구현](../plans/task_m100_7899_impl.md). 이전 저장 검증: [단계 2](task_m100_7899_stage2.md). **진행 중이며 시각 gate·PR 준비 완료가 아니다.**

## 제출 범위 확정과 첫 전체 검증

첫 전체 nextest는 10,294건 중 10,293건 통과, 1건 실패, 45건 skipped였다. 실패는 `serializer::cfb_writer::tests::test_serialize_real_hwp_files`의 공개 ParameterSet 원본 저장이며, 직접 `parse_hwp` 경로에 tail 투영 seal이 없어서 미편집 원본도 UnsupportedInput으로 거절됐다. `DocumentCore`의 공통 load fixup 이후 봉인 경로만 확인한 기존 집중 검사로 놓친 호출 경계다.

직접 파서의 strict/lenient·password 공통 종료 지점에서 tail 투영을 봉인하게 했고, 기존 전체 body/raw 캐시 봉인과 같은 helper를 사용한다. 직접 파서의 다른 raw 캐시 수용 정책은 바꾸지 않는다. 새 순수 저장 검사에는 직접 파싱·미편집 저장·본문 변경의 tail 보존과 실제 master 모델 변경 거부를 포함한다. 기존 실패 검사의 기대값·원본·baseline은 그대로 유지한다. 수정 전 실패는 첫 전체 실행 로그로 보존하고, 수정 후 해당 검사와 저장 계약 10건의 집중 nextest 11건이 모두 통과했다. 새 코드의 전체 nextest도 10,295건 통과 / 0건 실패 / 45건 skipped, 334.106s로 완료했다. 이 소스 수정 때문에 새 검증이 필요한 범위와 기존 완료 증거를 구분한다.

추가로 원격 `devel`의 문서-only `c0d330737fabae77ca2bf1706cba8514d0e8f1df`를 병합했다. renderer/parser의 upstream 제품 코드는 이전 기준 `116b2b6ae`와 같으며, 병합 자체를 이유로 기존 코드 검사를 반복하지 않는다. 새 시각 거버넌스의 devel 대비 조건은 수치 통과 주장과 구분해 최종 기록에 남긴다.


작업지시자는 `{` 기호 크기 차이뿐 아니라 다중 행 수식의 약 2px 세로 위치 차이도 이번 PR의 예외로 수용하고 PR 등록·self-review 진행을 승인했다. 추가 수식 AST 또는 세로 배치 변경 없이 통합 source `97616c522`을 제출한다. 자동 수치 gate 미달과 Canvas2D 글리프 누락은 그대로 공개하고, 예외를 정상 캡처 또는 수치 통과로 바꾸지 않는다.

기존 완료 증거를 재사용한다. 저장 계약 9건, 통합 Native Skia 빌드, Docker WASM 빌드, 실제 저장·재열기, Native/Canvas2D/SVG 비교는 다시 실행하지 않는다. 이전 단계에 미실행으로 기록했던 세 Clippy·workspace build·정책 검사는 이번 PR 준비에서 처음 실행해 모두 통과했다. `release-test` 전체 nextest도 이 후보에서 첫 실행으로 진행 중이다. 완료된 전체 회귀를 반복하는 것이 아니며, 테스트·source 변경 없이 검증 로그만 새로 생성한 것으로 구분한다.

`.env.local`은 작업지시자의 지시로 `.gitignore`에 추가했다. 기본 작업공간에는 임시 local exclude도 적용해 즉시 제외했고 설정 내용은 읽거나 stage하지 않았다. 실행한 편집 저장본과 독립 Print는 `samples/issue7899/`에 원본 해시 그대로 보존했다. 대표 PNG만 `mydocs/pr/assets/issue7899-save-reopen/`에 제출하며 private 글꼴·WASM glue·로그·파생 suite는 제외한다. source/test tree가 검증한 통합 SHA와 같은지 제출 전 다시 확인한다.

## 최신 devel 병합 후 Visual Sweep 준비

작업지시자의 PR 생성 전 devel 병합 지시로 `upstream/devel`을 다시 fetch하고 `116b2b6ae0c1ec655c2db9878b1c8cf1bdd67cc4`를 병합했다. 통합 source head는 `97616c5223cde8985b431f60faba811f6d0c6b47`이며 충돌은 없었다. 기존 local commit 7개와 generated `rhwp-studio/public/rhwp.js` 변경을 보존했다. base는 HEAD의 조상이며 behind는 0이다. 원격 push·PR 생성은 수행하지 않았다.

통합본의 Native Skia 빌드, fmt check, 구역 끝·메모 보존 집중 검사 9건과 고정 base `116b2b6ae` 대비 integration manifest·unit tier 정책 검사는 통과했다. 전체 회귀·세 Clippy 단계의 실행 완료를 뜻하지 않는다. shared `target/pr-review`을 재사용했으며 새 파생 suite는 stage하지 않았다.

devel의 #7903은 Visual Sweep 기본 출력을 Native Skia·WASM Canvas의 직접 PNG로 전환한다. 기존 SVG 결과를 이 방식의 증거로 재사용하지 않는다. 같은 `edited-source-baseline.hwp`와 정확한 독립 한컴 2020 Print로 실행한 새 Native 전쪽 TSV는 1쪽/1쪽, **77.01188% / re_review_required**다. 직접 review에서도 큰 수식의 조건 칸·첨자와 본문 글리프 차이가 남아 있어, 이전 메인터너 직접 판정과 자동 수치 미충족을 구분한다. Docker WASM 빌드는 8m05s에 완료했고 root pkg·Studio public의 WASM SHA-256은 `d158eca06fbe7912c7bb2edf57af0db2dfaa3169c309181affd89d29e1b83597`로 같다. fresh Node 편집→저장→재열기에서 5개 수식의 TextLine 소속·기하가 유지됐고 저장본 hash는 이전 보정본의 `e00edfc63c35d5753e1369bad72de5187a485e9a6d0e9406f5746cb7c7cebf52`와 같다. 직접 Canvas2D 비교의 기록값은 75.78392%이나, HyhwpEQ 공급·FontFace 적재 성공 뒤에도 수식 문자가 네모로 표시돼 정상 판정용 캡처로 인정하지 않는다. 이 실패를 조판 미달의 수치 증거 또는 글꼴 예외로 승격하지 않는다. 같은 fresh bundle의 명시적 SVG 진단은 정상 글자 표시를 확인했고 77.20043%였다. 이것도 미달이며 직접 PNG gate의 대체 증거가 아니다.

증거는 기본 작업공간의 ignored `output/pr-review/issue7899-20261014/devel-merge-116b/`에 보존한다. `merge.json`, `native-build.log`, `storage-tests.log`, `fmt-check.log`, 두 policy 로그와 `native-scores/math-edited/silhouette.tsv`, `native-review/math-edited/review/review_001.png`를 연결한다. source/policy base는 위 SHA에 고정했으며 새 head의 자동 gate 통과나 PR 준비 완료로 보고하지 않는다.

작업지시자의 Visual Sweep 직접 확인 요청에 따라 `visual-review.html`을 준비했다. 독립 PDF·Native Skia·fresh WASM SVG 진단, 확대/전체 쪽 전환과 overlay를 포함하고 Canvas2D 글리프 누락 캡처는 실패 증거로 별도 표시했다. PNG 12개를 HTML 안에 내장했으며 Chrome에서 전부 로딩되고 전환 버튼이 동작함을 확인했다. 사용자 열람본은 `/mnt/e/tmp/issue7899-visual-review-97616c5.html`이며 Windows에서는 `E:\tmp\issue7899-visual-review-97616c5.html`로 연다. Docker·Canvas·SVG 실행 로그, `fresh-save-reopen.json`, `visual-review-manifest.json`, `visual-review-browser-check.json`을 같은 증거 폴더에 보존한다.

## 메인터너 직접 시각 판정

통합본 Visual Sweep을 확인한 작업지시자는 단일 행 수식은 유사하지만 다중 행 수식은 독립 한컴 PDF보다 약 2px 아래에 조판된다고 관측했다. 사용자 직접 판독이며 자동 좌표 계측값이나 허용치 승인으로 바꾸지 않는다. `layout_cases` 및 `layout_matrix`는 현재 상자 높이의 절반을 내부 baseline으로 발행하지만, `layout_fraction`은 baseline과 수학 축을 구분해 axis 높이를 계상한다. 다중 행 수식의 공통 축 정렬을 원인 후보로 좁혔으며, source 개체 baseline·문단 줄 baseline·내부 AST baseline·최종 scale의 연결을 대조하기 전에는 원인을 확정하거나 일괄 -2px 보정을 추가하지 않는다. 단일 행 대조군의 보존도 함께 확인해야 한다.

2026-10-14 작업지시자는 code head `40f0ec7de`의 Docker WASM 편집 저장본에 **“시각 판정 : 통과”**로 응답했다. 판정 대상은 아래 Studio에서 재열기한 `line-ownership-native/studio-saved.hwp`이며 SHA-256은 `bb9d973f0cbaddfc3e510c957d7a85b936fe9efae99b35dab0cd6431c4332c57`이다. 첫 문장과 큰 수식의 겹침 해소·저장 전후 배치에 대한 직접 판정을 충족으로 기록한다. 새 code head의 직접 판정으로 재사용하지 않는다.

Native 비교 입력과 이 Studio 저장본의 BodyText 76개 레코드는 tag/level/payload가 전부 같고, 다른 CFB stream도 DocInfo 외에는 모두 같다. DocInfo의 차이는 DocumentProperties의 캐럿 글자 위치 17→16뿐이며 명세 표14의 해당 필드와 대조했다. 따라서 저장 레코드 차이로 자동 비교 결과를 배제하지 않는다. 직접 판정 통과와 **Native/fresh WASM 77.20043%의 자동 gate 미충족**을 별도로 유지한다. 이번 응답을 90% 수치 조건의 예외 승인이나 원격·전체 CI 승인으로 확대하지 않는다.

남은 수식 내부 칸 맞춤의 규칙 근거를 [한컴 공식 수식 명령어 설명](https://help.hancom.com/hoffice/multi/ko_kr/hwp/insert/equation/equation%28explanation%29.htm)에서 확인했다. `&`는 세로 칸 맞춤 표식인데 `parse_cases`는 고정 Tab 공백으로 발행하고 `layout_cases`는 각 행을 독립적으로 가로 나열한다. 독립 Print의 조건 문구 시작들이 같은 세로 축에 놓이는 관측과 대조해 추가 보정을 진행한다. 이 분석을 구현 완료나 새 시각 통과로 보고하지 않는다.

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
