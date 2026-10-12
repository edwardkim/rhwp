---
kind: review
status: active
canonical: mydocs/manual/pr_review/collaborator_external_pr.md
last_verified: 2026-10-11
---

# PR #7698 — 메인터너 보정

## 범위와 승인

사용자가 메인터너 보정 후 기여자의 원 PR을 merge로 처리하도록 지시했고, 실제 문서는 공개 샘플 대조군을 사용하도록 선택했다. `maintainerCanModify=true`를 확인했으므로 원 contributor head 직접 보정 경로(9.3.1)를 사용한다. 원 commit을 rewrite하거나 별도 통합 PR로 대체하지 않는다.

원 head는 `1489ca1d95e3d6e6b888870d3b14cf240866ef00`, fork ref도 동일하다. 같은 가시성 branch `review/scari-pr7698-20261011`에서 미공개 reviewer 문서 commit만 원 head 위로 replay했다. 앞선 최신-base 진단 후보와 운영 기록은 `refs/review/pr7698/latest-base-diagnosis`에 보존했다. 원 PR에는 source head보다 최신인 devel 운영 기록을 복사하지 않았다.

## 원인과 보정 계약

기여자의 ViewText 선택·fallback·크기 제한·암호 검증 구현은 유지한다. 반환값이 암호나 파생 키가 아니라 문단 레코드인 helper를 `decode_viewtext_paragraph_stream`으로 명명하고 반환 계약을 설명한다. 실제 password 인자·암호 구현·CodeQL query/language/path 정책은 바꾸지 않는다. 실제 반환값과 기존 로그의 의미를 명확히 하는 보정이며, 새 SARIF에서 효과를 확인하기 전에는 CI 해결로 판정하지 않는다.

기여자의 `tests/cases` 집중 회귀 6건을 보존한다. 추가 검사는 실제 CLI의 strict/lenient 암호 ViewText 열기를 실행하고 stdout/stderr에 공개 시험용 credential이 나오지 않는지 확인한다. 이 검사는 로그 보존 계약의 대조군이며, helper 이름 변경 전 결함을 검출한다고 주장하지 않는다.

## 검증 계획과 진행

- 필수 fmt/native·WASM32·workspace Clippy, workspace build, manifest base 비교와 관련 집중 회귀를 순차로 실행한다. 공유 target은 `/home/edward/mygithub/rhwp/target/pr-review`다.
- Docker WASM을 빌드하고 실제 `openWithPassword`로 공개 암호 문서 및 ViewText 재현 구조를 확인한다.
- 공개 #5169 문서의 원 스트림·저장 LineSeg를 유지하는 암호 변형은 생성 방식과 한컴 수용/Print 출력, 원본 대조를 구분한다. 비공개 기업 원본이 확인된 것으로 쓰지 않는다.
- source push 전 fork/head를 재확인하고 LFS 사전 판독·dry-run을 수행한다. 새 head의 full CI/CodeQL을 확인한다. merge 직전 exact head·required checks·최신 base merge-tree를 재확인하며 merge 방법은 원 contributor commit을 보존하는 merge다.

실행 결과와 남은 범위는 완료 시 이 기록 및 [검토 기록](pr_7698_review.md)에 연결한다. 현재 로컬 수정 진행 중이며 원격 반영·CI 해소·merge 완료가 아니다.

## 메인터너 직접 판정 준비 (2026-10-11)

사용자는 시각 예외 승인 대신 직접 판정용 sample/Studio 준비를 지시했다. 따라서 시각 게이트 예외를 승인받은 것으로 기록하지 않는다. 보정 code SHA는 `f90c362511d46255c353bc8867401b769b12cdf4`이며 원격 push/merge 전이다.

필수 Rust lint 묶음·workspace build·base 고정 manifest 검사는 통과했고 집중 회귀는 7 PASS다. Docker `docker compose --env-file .env.docker run --rm wasm`도 통과했다. root `pkg/`와 Studio `public/`의 JS/WASM SHA-256이 일치하며 실제 CDP Chrome UI가 받은 WASM도 `0a34c4db42cab41126b4346dc991d3bd57514d7746994b35f4e44bffe51cd761`로 확인했다. 전체 nextest는 아직 실행 중이다.

공개 #5169 원본은 Native와 fresh WASM 모두 18쪽, 독립 한컴 2020 Print는 6쪽이다. 전체 Sweep 두 실행은 쪽수 불일치로 실패했고 전쪽 TSV·90% 통과를 주장하지 않는다. 공개 원본은 `samples/issue5169_viewtext_changetracking.hwp`, PDF는 `pdf/issue7674/issue5169_viewtext_changetracking-2020.pdf`다. 자체 암호화 변형은 한컴에서 거부돼 정상 fixture로 채택하지 않았다.

별도 암호 ViewText 대조군은 기존 공개 한컴 EncryptVersion 4 문서의 모든 stream을 byte-for-byte 유지하고 encrypted BodyText stream을 ViewText에 복제했다. 새 암호 연산·문단/LineSeg 변경은 없다. 생성기와 입력은 ignored `output/pr-review/pr7698-20261011/maintainer/clone-known-viewtext.rs`, `known_password_with_viewtext.hwp`다. 한컴 2024가 정상 열고 Print 64쪽을 생성했다(job `0253ae87-7eff-4e25-8ba1-7135ed03f8da`, build 13.0.0.3901, `frame_print_to_pdf_ex_one_up`, method 0). 이 대조군은 암호 ViewText 공존 수용 확인이며 비공개 기업 원본의 Section1 누락 재현으로 승격하지 않는다.

`http://localhost:7700/`에 Docker WASM을 반영한 Studio를 실행하고 CDP Chrome에 위 두 문서를 실제 file-input 이벤트로 각각 열어 두었다. 원본 18쪽·암호 대조군 64쪽 및 Canvas 표시를 확인했다. 실행·hash 증거는 ignored `maintainer/studio-preparation.json`, `studio-*.png`와 `browser-contract-result.json`이다. 브라우저 API 합성 8개 조합의 본문 선택·오답 거부는 통과했으며, 정상 암호 문서는 64쪽으로 열렸다. 합성/자체 비교 통과는 한컴 시각 통과가 아니다.

## #5169 1쪽의 삭제 이력 표시 결함 확인

메인터너는 원본의 1쪽부터 한컴에서 보이지 않는 텍스트가 rhwp에 나온다고 지적했다. 이를 단순 페이지네이션 차이로 설명한 앞선 해석을 정정한다. source `f90c362511d46255c353bc8867401b769b12cdf4`에서 원본 ViewText는 현재 내용과 삭제 이력을 함께 담고 있다. 독립 한컴 2020 변환 HWPX(job `51ee32e5-a33d-4936-b495-e8f55a69b0be`, version 11.0.0.9136)의 `deleteBegin/deleteEnd`는 HWP5 RangeTag type `0x11`·하위 24bit ID와 대응하며, header의 해당 `trackChange`는 `type="Delete" hide="1"`이다. 형식 추론만으로 삭제라고 단정하지 않고 실제 한컴 저장본의 독립 대응으로 확인했다.

| 1쪽 내용 | raw ViewText | 한컴 Print·독립 삭제 이력 대응 |
| --- | --- | --- |
| 표제 | 연구제안계획서 | `제안`: UTF-16 [2,4), RangeTag 0x11/ID 11 → Delete/hide=1. 한컴에는 연구계획서 |
| 이전 제목 | 무역구제제도분야 협상 관련 | 삭제 범위 [0,4), [4,6), [6,15) / IDs 17,1,17이 전체 문구·문단 끝을 포함. 한컴에는 표시되지 않음 |
| 날짜 | 20187. 436 | 삭제 [4,5), [7,9) / IDs 20,194. 한컴에는 2018. 6 |

실제 호출 경로는 `src/parser/mod.rs:796`의 ViewText 수용(압축 해제·첫 PARA_HEADER만 검사) → strict `:879`/lenient `:1008`의 본문 선택 → `src/parser/body_text.rs:316`에서 RangeTag 원값 보존 → `src/renderer/layout/paragraph_layout.rs:7976`에서 type=2 형광펜만 처리하며 삭제 범위의 표시·점유 의미를 적용하지 않는 것이다. 삭제 텍스트가 문단 IR·줄 구성에 그대로 들어가 페이지 공간도 점유한다. 단순 paint 은폐·좌표 clamp로 해결할 범위가 아니다.

이 비암호 경로와 ViewText 우선 정책은 검토 base `90fa2f8955c2dd21e9f0ea8fd0c6ee8dc5996a47`에도 있다. #7698은 이 기존 정책을 암호 경로로 확장했다. 기존 #5169 이슈/회귀는 전체 컨트롤 수(표 10개)로 ViewText 수용을 검사하지만, 변경 추적의 숨긴 컨트롤도 포함한 구조 개수는 현재 보이는 본문이라는 근거가 아니다. 따라서 `한글은 ViewText가 있으면 그것을 렌더한다`는 기존 문서·테스트의 표현을 표시 상태의 계약으로 재검토해야 한다. 기존 회귀·baseline은 임의 변경하지 않았다.

ViewText만 제외하고 다른 원 스트림을 모두 보존한 **진단 대조군**은 rhwp에서 6쪽이 되고 표제·제목·날짜가 독립 Print와 일치한다. 이 변형은 source 수정 또는 정상 한컴 생성 fixture가 아니며, 모든 문서에서 BodyText를 우선해야 한다는 일반 규칙/전쪽 시각 통과를 입증하지 않는다. ViewText의 변경 이력 보존과 최종본/검토 표시 정책을 구분해 보정 범위를 결정해야 한다. 특히 BodyText 없는 #7674의 경로를 없애는 방식은 허용되지 않는다.

진단 자료는 ignored `output/pr-review/pr7698-20261011/maintainer/diagnosis-5169/`의 `raw-stream-summary.json`, `cover-range-examples.json`, `hancom-track-summary.json`, `hancom-track-examples.json`, 원본 대응 `issue5169_hancom2020.hwpx`, 변형 `bodytext-only-control.hwp` 및 `bodytext-only-info.json`이다. 한컴의 변경 내용 보기 계약은 [공식 도움말](https://help.hancom.com/hoffice110/ko-KR/Hwp/view/toolbar/menu_review.htm)을 함께 대조했다.

전체 nextest 첫 실행은 8,121 PASS/1 FAIL/50 SKIP, 미실행 2,423건이었다. 실패는 이번 세션에서 만든 한컴 재저장 진단본을 active `samples/`에 둔 상태에서 기존 표본 전수 스캔이 새 파일의 겹침을 검출한 것이다. 신규 실험 파일 2개를 ignored `maintainer/rejected-experiments/`로 보존 이관했고(원본·기존 표본·검사·baseline 유지), `--no-fail-fast`로 전체 회귀를 다시 실행 중이다. 이 환경 교정과 제품 결함 보정은 구분한다. source push/merge는 아직 수행하지 않았다.


## 범위 확장과 구현 진입

사용자가 #5169의 숨긴 변경 이력 표시 결함도 이번 PR에서 해결하도록 지시했다. 삭제 범위가 숨긴 문단 끝·컨트롤을 포함하는 경우를 본문 선택 계약과 함께 처리한다. 한컴이 저장한 표시 상태가 독립 근거이며 문서 ID나 표 개수로 표시 본문을 선택하지 않는다. 같은 visible IR을 측정·Canvas·SVG·PNG가 소비하고 원 이력 보존과 저장 경로도 확인한다. BodyText가 없는 암호 ViewText 경로는 유지한다.

앞선 f90c362511 보정의 전체 nextest 재실행은 10,545 PASS / 50 SKIP다(`maintainer/full-nextest-clean-samples.log`). 이는 숨긴 변경 이력 수정 전 대조군이며 새 구현의 통과 증거로 재사용하지 않는다. 기존 #5169 회귀의 표 개수 기대값은 현재 가시성 계약과 독립 Print를 검증한 후 재검토하며, 90% 미달 상태에서 baseline이나 기대값을 갱신하지 않는다.


## 가시성 구현과 중간 검증

`TRACK_CHANGE`(DocInfo tag 96)의 순번 ID·Insert/Delete 유형·hide 값은 독립 한컴 HWPX의 229개 항목 모두와 대응했고 불일치는 0건이다. 이 설정과 RangeTag의 UTF-16 범위를 사용해 parser에서 visible paragraph stream을 생성한다. 숨긴 확장 컨트롤은 소유 subtree와 함께 제거하고, 숨긴 문단 끝은 같은 목록의 다음 문단을 이어받는다. PARA_TEXT 없는 char_count=1 문단의 암묵적 문단 끝도 같은 규칙을 따른다. 일반 빈 문단은 남기며, 삭제된 문단 끝 앞의 문단 모양은 이어받기에 보존한다. 원본 section raw와 DocInfo는 유지한다. 실제 숨김으로 UTF-16 축이 바뀐 문단 및 삭제된 문단 끝으로 이어붙인 문단의 줄 정보만 재조판하고, 가시 내용이 바뀌지 않은 문단의 유효한 저장 LineSeg는 보존한다. strict/lenient 모두 같은 projection을 거쳐 측정·각 backend에 전달한다.

원본 raw를 입력으로 같은 production projection을 실행한 진단은 최상위 문단 99개·표 4개로, 독립 한컴 최종 내용과 대응했다. 원본 BodyText와 visible projection의 재귀 문단 수는 모두 228개이며 완전 삭제·이어받기 후 표제와 삭제 문구의 표시를 확인했다. 이는 독립 Print 전쪽 시각 통과를 대신하지 않는다.

초기 글꼴 대조군은 6쪽이지만 1/3/4쪽이 90% 미달이었다. 실제 설치 face를 공급한 경우에도 남는 배치 차이를 글꼴 예외로 분류하지 않았다. 1쪽 표제의 첫·둘째 줄은 중앙 정렬 폭이 soft-wrap 말미 공백을 포함해 반 칸 좌측에 있었고, 독립 Print는 비가시 말미 공백을 제외했다. 마지막 줄 여부 조건을 제거해 같은 줄의 TAC·밑줄·전부 공백인 줄의 기존 표시 계약을 유지하며 중앙 정렬을 보정한다. 최종 head의 전쪽 Native/fresh WASM Sweep 및 회귀 검증은 진행 중이다. 첫 Clippy의 is_multiple_of 지적을 수정했고 Native Clippy와 release-test CLI build는 통과했다.


3·4쪽의 미달은 초기 구현이 같은 section의 **바뀌지 않은 문단**까지 저장 LineSeg를 제거한 탓이었다. 원본 ViewText의 `따라서` 문단은 숨긴 범위가 없고 저장 줄 시작은 0/39, `고위관리` 문단은 0/56이다. 실제 한컴 Print의 줄 나눔과 대응하므로 이를 review-mode 무효 정보라고 단정한 가정을 제거했다. 가시 축이 바뀐 문단·이어붙인 문단만 재조판한 production helper 진단 결과는 6쪽 모두 90% 이상(99.44/92.34/91.69/97.06/99.87/100.00%)이다. 변형 입력 진단은 최종 원본의 Native/fresh WASM 증적을 대신하지 않으며, 원본 기준 재검증을 이어간다.


저장 경로도 대조 중이다. 종전 writer는 채택한 review stream을 BodyText에만 쓰고 ViewText를 버렸으며, Native 재파싱/쪽수 검사는 통과해도 한컴 상호운영을 입증하지 못한다. 수정하지 않은 section의 raw provenance가 유효할 때 동일 visibility projection을 BodyText로, 원본 review history를 ViewText로 각각 저장하도록 보정한다. 암호 저장은 두 stream에 같은 압축·암호 계약을 적용한다. 편집으로 seal이 무효화된 section에는 오래된 ViewText를 재삽입하지 않는다. 한컴 Print 재열기 확인은 아직 진행 중이며, 변경 추적 편집 기능 전체를 구현한 것으로 보고하지 않는다.


원본 기준 정식 Native Sweep(가시성 code `1f218aa8e37488979298f4f2723177740dbf2f61`)은 6/6쪽·최저 91.69413%로 통과했다. 대표 1쪽은 99.44182%이며 90% 미달·누락 쪽은 없다(`maintainer/visible-native-final/visible/silhouette.tsv`). 이후 저장 보정은 별도 code `422e09032`에 두었고 같은 원본의 최종 Native/fresh WASM 검증을 재실행한다.

종전 writer 저장본은 한컴 Print가 600초 timeout으로 실패했다(job `9a9a13eb-644b-4546-92b7-cba940961561`). 그 저장본에 원본 ViewText만 복원한 대조군은 17초 만에 한컴 Print 6쪽으로 성공했다(job `9a39a02f-0fcd-4616-a8c7-8b898471156c`, build 11.0.0.9136, method 0, `hancom2020_pdf_driver_one_up`). 이 대조군은 스트림 누락의 독립 진단이며 production writer 보정의 최종 상호운영 통과로 대신하지 않는다.


production writer code `422e09032`로 저장한 `visible-preserved-roundtrip.hwp`도 한컴 2020 Print가 17초에 정상 완료했다(job `0d9a455e-458a-4a6a-b5af-314a33e83a32`, build 11.0.0.9136, method 0, 6쪽). 원본과 이 저장본의 독립 Print PDF를 96dpi로 전쪽 raster 비교한 결과 6쪽 모두 동일하다(`maintainer/diagnosis-5169/preserved-print-comparison.json`). 원본 ViewText의 압축 해제 바이트도 저장본에 그대로 보존됐으며, Native 재열기 IR/쪽수는 6/6으로 일치했다. 수정하지 않은 변경 이력 문서의 읽기·저장 계약 확인이며 변경 추적 편집 UI 전체의 보존 증거로 넓히지 않는다.

최종 Rust lint는 fmt, Native Clippy, wasm32 lib Clippy, workspace build, workspace/all-targets Clippy, 고정 base `90fa2f8955c2dd21e9f0ea8fd0c6ee8dc5996a47` 대비 manifest를 순차 통과했다(`maintainer/tracked-*-final.log`). 이후 Docker `docker compose --env-file .env.docker run --rm wasm`으로 root web package를 재빌드했다. pkg와 Studio public의 JS 및 WASM 해시가 각각 일치하고, 브라우저가 실제 내려받은 WASM SHA-256은 `fc14e5272d5bbc3840c99461ee7d98e647620250fa438b8174803010ef1930ad`다. Studio `http://localhost:7700/`에 원본(6쪽)과 공개 암호 ViewText 대조군(64쪽)을 새 탭으로 열었다(`maintainer/studio-final-preparation.json`). 전쪽 fresh WASM Sweep 및 보정 후 정식 회귀는 이어서 검증한다.


최종 원본 code `422e09032`에서 전쪽 Native TSV 최저 91.69413%, fresh Docker WASM TSV 최저 91.69413%다. 둘 다 6/6쪽·누락 없음·90% 미달 없음이며 각 TSV는 `maintainer/visible-native-422e/visible/silhouette.tsv`, `maintainer/visible-wasm-422e/visible/silhouette.tsv`에 보존했다. 실루엣-only 실행의 `not_evaluated`를 최종 승인으로 바꾸어 해석하지 않는다. 별도 대표 1·3·4쪽 Native/fresh WASM review/overlay 실행은 gate `passed`이며 표제·날짜·표 괘선·문단 순서와 중앙 정렬 영역을 직접 대조했다. 1쪽 표제는 같은 영역 확대 대조도 남겼다(`maintainer/visible-wasm-review/visible/review/cover-enlarged.png`). 본문 글리프·말미 인용 부호와 미세 자간의 차이는 전체 픽셀 동일으로 주장하지 않으며 메인터너 직접 Studio 판정이 남아 있다.

fresh WASM API 재검증은 Section1 Body 누락/공존·압축 여부·strict/lenient 8조건, 잘못된/없는 비밀번호 거부, 공개 실제 암호 대조군/동일 cipher View 복제 대조군의 64쪽 텍스트·SVG 동등성을 통과했다. 원본 변경 이력 문서는 제목이 표시되고 두 삭제 문구는 추출에서도 사라졌으며, production 저장본과 visible text/6쪽도 일치했다(`maintainer/browser-contract-final-result.json`). 이는 실제 비공개 원본의 한컴 출력 검증을 대신하지 않는다.

전쪽 Native/fresh WASM 90% 조건과 영향 영역 직접 판독 후 기존 `issue_5169_prefer_viewtext.rs`의 잘못된 `tables >= 8`을 재검토했다. 독립 Print의 표시 표 4개, 부분/전체 삭제 문구 제외, 원본 변경 이력과 ViewText 보존, 숨김이 없는 문단의 저장 LineSeg 유지로 검사 의미를 바꾼다. 수정본 집중 검사는 3 PASS다(`maintainer/tracked-5169-focused-final.log`). 최초 suite 번호가 prepare 뒤 바뀌어 0건 실행된 로그는 통과 증거로 세지 않으며 실제 재실행 결과만 사용한다. 숨김 구현 전 `f90c362511`에서 같은 검사 실행의 red/green 대조와 최종 전체 회귀는 진행 중이다.


같은 정식 source 검사를 숨김 보정 전 code `f90c362511d46255c353bc8867401b769b12cdf4`에 연결해 1 PASS / 2 FAIL을 확인했다. 표시 검사는 실제 표 10/기대 4로, 저장 검사는 ViewText stream 누락으로 각각 실패했다. LineSeg 대조군은 수정 전에도 PASS이므로 결함 검출 주장 대신 정상 보존 검사로 구분한다(`maintainer/tracked-5169-before-final.log`, 검사 원본 `tracked-5169-before-case.rs`). 임시 detached worktree는 이 진단용으로만 만들었고 원본/검사/로그를 보존한 뒤 제거했다. 공용 target은 유지했다.


검사 code head `9908d6ec2f3cab28ed32af87d556682d58ae7318`에서 필수 lint 묶음과 고정 base manifest 재검증은 모두 PASS, #5169 집중 3 PASS 및 #7674 집중 7 PASS다. 포맷 변경으로 prepare 산출물의 weight/harness가 한 번 drift해 재생성한 뒤 동일 묶음을 다시 통과했다. 파생 harness/manifest는 PR source에 포함하지 않는다. 전체 nextest 및 Native Skia 3종은 이 code head에서 순차 실행 중이다. host는 논리 CPU 16개·사용 가능 메모리 약 22GiB이며 브라우저/개발 서버와의 공존을 고려해 test threads 8을 사용한다. 새 실행의 완료 결과를 이전 f90 전체 PASS로 대신하지 않는다.

원격 PR head는 재조회에서도 원 기여 SHA `1489ca1d95e3d6e6b888870d3b14cf240866ef00`이고 maintainer_can_modify=true다. 원격 source에 아직 보정 commit을 push하지 않았다. 최신 `upstream/devel` fetch는 고정 base와 같았고 현 code head와 merge-tree는 conflict 없이 생성됐다(`maintainer/tracked-final-merge-tree.txt`). Studio의 새 6쪽 원본 탭을 앞으로 가져왔으며 전쪽 PDF/WASM PNG의 상대 경로 비교 뷰는 `maintainer/visible-review.html`이다.

## 공식 기술 정의와 Studio 재확인

공개 [HWP 5.0 revision 1.3 명세](https://cdn.hancom.com/link/docs/한글문서파일형식_5.0_revision1.3.pdf)는 변경 추적 플래그·레코드 식별자와 범위 태그의 시작/끝·상위 8bit 종류/하위 24bit 데이터를 정의한다. 그러나 이번 구현이 사용하는 TRACK_CHANGE의 hide 필드 바이트 위치, 순번 ID 및 Insert/Delete 범위 종류의 전체 의미를 상세히 기술하지 않는다. 해당 바이너리 해석은 독립 한컴 저장 HWPX 229개 항목의 대응과 원본 Print 검증으로 뒷받침하며, 공개 바이너리 명세에 모두 명시된 사실로 보고하지 않는다.

한컴의 [공식 개발자 답변](https://forum.developer.hancom.com/t/hwp-binary-format/1963/5)은 ViewText에 변경 추적 정보, BodyText에 최종본을 저장한다고 설명한다. 공식 [OWPML TrackChange 모델](https://github.com/hancom-io/hwpx-owpml-model/blob/main/OWPML/Class/Head/trackchange.cpp)과 저장소의 Header/ParaList 스키마에는 type·hide·id와 변경 범위의 TcId·paraend가 있다. XML 필드의 존재와 HWP 바이너리 payload의 상세 정의는 구분한다. BodyText가 빠진 #7674 입력도 처리해야 하므로 이 설명만으로 무조건 BodyText를 선택하는 정책으로 되돌리지 않는다.

메인터너가 직접 판정을 '추가 수정 필요'로 답하고, 이어 해당 샘플을 Studio로 다시 열었지만 변화가 없었다고 알려왔다. 이는 수치 통과로 면제하지 않으며 원격 push/merge를 진행하지 않는다. CDP 탭 재조회에서는 원본이 6쪽이고 `연구계획서`가 남으며 `연구제안계획서`와 삭제된 옛 제목은 추출에서 제외됐다(`maintainer/studio-user-recheck.json`, 화면 `studio-user-recheck.png`). 따라서 아직 구체적인 추가 출력 결함이나 사용자 환경의 이전 WASM 로드를 확정하지 않는다.

구분 가능한 새 검증 URL `http://localhost:7700/?review=pr7698-9908d6e-visible`을 열어 캐시를 끄고 원본을 실제 file-input 이벤트로 로드했다. 초기화에 실제 사용된 `/@fs/home/edward/mygithub/rhwp/pkg/rhwp_bg.wasm` 응답의 SHA-256은 Docker 산출물과 동일한 `fc14e5272d5bbc3840c99461ee7d98e647620250fa438b8174803010ef1930ad`였고, 6쪽·제목 표시·두 삭제 문구 제외를 확인했다(`maintainer/studio-identified-recheck.json`, 화면 `studio-identified-recheck.png`). 서버 제공 파일을 별도 fetch한 해시 확인보다 실제 초기화 응답 확인을 강화한 것이다. 새 탭을 앞으로 가져왔으며 메인터너가 확인한 URL·전체 쪽수의 응답을 기다린다.

최종 head `9908d6ec2f3cab28ed32af87d556682d58ae7318` 전체 nextest는 10,545 PASS / 2 FAIL / 50 SKIP로 종료했다(493.884s). `text_overlaps_do_not_grow_partition_10`은 `issue3837/stored_vpos_rewind_form.hwp`의 겹침이 3→4로 늘었고, `ir_field_sweep_does_not_regress`는 #5169의 control_mask·has_para_text 및 #6778의 has_para_text 발산 증가를 검출했다. 연속 실행 스크립트는 첫 실패에서 중단되어 Native Skia 3종은 아직 미실행이다. baseline은 갱신하지 않는다.

왕복 메타데이터 집중 진단(`maintainer/inspect-rebuild-metadata.rs`, `.log`)에서 #5169 visible 문단 20의 control_mask는 제거한 표 비트가 남아 2099200→2097152이고, 문단 96의 mask는 2048→0이다. 또한 #5169 문단 60·62·96 및 #6778 문단 25는 내용 없는 명시적 PARA_TEXT가 재생성 저장 후 생략된다(has_para_text true→false). 해당 관측은 레코드 재생성 경로의 계약 확인이며, 이것만으로 visible 출력의 손상을 단정하거나 래칫 허용치를 올리지 않는다. 현재 표시·독립 Print와 별도로 projection의 마스크 유지 및 serializer의 빈 텍스트 레코드 보존을 보정해야 한다.

## HWPX 표시 상태 누락으로 추가 범위 구체화

메인터너는 HWP는 해결됐고 `/mnt/e/tmp/issue5169_viewtext_changetracking.hwpx`가 해결되지 않았다고 명확히 했다. 앞선 '추가 수정 필요'는 같은 HWP 출력의 거부로 해석하지 않는다. 제공 HWPX SHA-256은 `1c822a1c9962eadf9ae4a68206051c7d1474f811d5401074fbb05de309f36b26`이다. header.xml/section0.xml은 기존 독립 한컴 2020 변환 HWPX와 각각 byte-for-byte 같다. header에는 229개 trackChange, section에는 deleteBegin/End 182쌍과 insertBegin/End 328쌍이 있으며, hide=1인 삭제 정보가 실제 존재한다. 현재 HWPX header/section 파서는 이 표시 설정과 범위 태그를 처리하지 않아 이력 텍스트와 개체가 모두 visible IR에 들어간다.

메인터너는 한컴의 '원본', '원본 및 변경 내용', '최종본' 보기에 비추어 현재 rhwp가 변경 내용을 함께 보이는 상태라고 설명했다. 이번 보정은 저장된 trackChange type/hide 및 범위·문단 병합 정보를 반영해 같은 visible IR을 생성하는 범위다. 보기 전환 UI·승인/거부 전체를 구현한 것으로 확장하지 않는다. 독립 공식 모델/스키마에는 hide·TcId·paraend와 문단 merged·paraTcId가 있으며, 문단 목록의 소유 경계를 넘지 않는 병합 및 저장 LineSeg의 유효 축을 실제 한컴 출력으로 검증한다. 표시 불일치는 backend paint에서 글자만 지우는 방식으로 처리하지 않는다.


## 저장 보기 상태에 따른 HWPX visible IR 보정

제공 원본의 독립 Hancom 2020 Print는 job `9d83c530-a806-405a-8a10-216a591b8d26`(Hwp 11.0.0.9136, method 0, 6쪽)이며 `maintainer/hwpx-reference/issue5169-hwpx-2020.pdf`에 보존했다. HWPX header의 `type/hide`와 문단 내부 `Id/TcId/paraend`, 같은 소유 목록의 `merged`를 XML parsing 전에 투영한다. 숨김 범위가 실제로 없는 구역은 원래 parsing 경로를 유지한다. 텍스트뿐 아니라 범위 소유 개체도 같은 IR에서 제외하며, 실제 문자 축·병합이 변경된 문단의 LineSeg만 무효화한다. 원본 입력은 수정하지 않았다.

초기 생산 코드의 HWPX는 6쪽/99문단이 됐으나 2쪽 27.65746%로 실패했다. 원인은 저장 빈 줄의 쪽 귀속이다. `DocumentCore::from_bytes_inner`의 XML reflow 후에도 문단 19의 저장 vpos=0 및 앞 문단의 꼬리 끝은 보존됐지만, `typeset/paragraph.rs::prepare_fit_budget`가 `entry::stored_tail_fills_page`를 HWP5에만 허용해 빈 줄을 1쪽 하단에 남겼다. `paragraph/entry.rs`의 실제 저장 줄 출처·앞 줄 꼬리 공간·현재 흐름 일치 조건은 유지하고 같은 HWPX 저장 사다리에도 적용했다. 문단 19·20의 빈 줄 두 개가 2쪽을 점유한 뒤 제목 문단이 이어지는 것을 직접 PNG로 확인했다. 좌표/문서 ID로 출력 위치를 맞추지 않았다.

현재 Native 최종 원본 전쪽 TSV(`maintainer/tracking-native-final/{hwp,hwpx}/silhouette.tsv`)는 HWP 최저 91.69413%, HWPX 최저 90.87964%로 전쪽 90% 이상이다. 이 결과는 Git HEAD `9e6882188` 위 로컬 보정의 산출이며 원격 PR 결과가 아니다. 문단 사이 XML 개행만 추가한 의미 동등 진단 입력은 앞선 코드에서 12쪽/164문단이 되어 병합 누락을 검출했다. inter-element XML whitespace를 같은 문단 목록의 소유 경계로 보지 않도록 보정한 뒤 6쪽/99문단으로 복원됐다(`hwpx-whitespace-before.json`, `hwpx-whitespace-after.json`). 최종 fresh WASM은 진행 중이다. 원문 paragraph 내부 공백과 빈 줄은 별도로 보존한다.

앞선 전체 회귀 두 실패도 baseline을 바꾸지 않고 구현을 보정했다. `tracked_viewtext`에서 숨김 축 제거 뒤 남은 제어 슬롯으로 mask를 재계산하고, writer는 명시적 terminator-only PARA_TEXT를 보존한다(원본의 PARA_TEXT 부재와 구분). 실제 #5169/#6778의 재구성 전후 control_mask/has_para_text 차이가 0인 진단 로그는 `maintainer/inspect-rebuild-metadata-after.log`다. 중앙 정렬은 유효한 저장 중간 줄의 정렬 폭을 보존하며 재조판으로 새로 나눈 줄의 비가시 trailing whitespace를 제외하도록 수정했다. #3837 실제 anomaly는 4→3으로 돌아왔다(`overlap3837-after.json`). 최종 전수 회귀·lint·fresh WASM과 메인터너 HWPX 시각 판정은 별도로 확인해야 한다.

검토 보기는 원본/원본 및 변경 내용/최종본 및 변경 내용/최종본을 구분하는 한컴 계약이다([공식 도움말](https://help.hancom.com/hoffice110/ko-KR/Hwp/view/toolbar/menu_review.htm)). 이번 구현은 저장된 visibility를 열람 IR에 적용한다. 보기 전환 UI, 서식 변경의 원본 복원, 변경 수락·거부 편집, HWPX 변경 이력 전체의 저장·재열기는 완료로 주장하지 않는다. HWP의 미편집 원 이력 보존 증거를 HWPX 저장 증거로 대체하지 않는다.

최종 XML 공백 보정 뒤 Native 전쪽 TSV는 `maintainer/tracking-native-whitespace-final/{hwp,hwpx}/silhouette.tsv`이며 앞선 Native 최소값을 유지했다. HWPX 대표 2·3·4쪽 `maintainer/hwpx-native-review-final/hwpx/review/`의 동일 배율 직접 비교에서 숨김 내용 제외·선행 빈 줄·표 소속·앞뒤 본문을 확인했다. 2쪽의 한 줄 위 이동은 해소됐고 남은 글자 폭·자형 및 표 괘선의 모양 차이는 화면에 보존했다. 이는 메인터너의 HWPX 직접 판정과 fresh WASM 검증을 대신하지 않는다.

## HWPX 추가 범위의 직접 판정과 통합 전 검증

메인터너가 `http://localhost:7700/?review=pr7698-hwpx-tracking-final`의 제공 HWPX를 확인하고 **시각 판정 통과**로 답했다. 초기화에 실제 사용된 WASM 응답 SHA-256 `58a0e64473c5f57d624419055c71b874b3716b879390cb192f598f8b3598f5de`는 Docker 빌드의 pkg/public과 동일하다(`studio-hwpx-tracking-final.json`).

- 통합 전 code: `060f38ba5`; 원 기여 commit은 보존했다.
- 전쪽 Native: HWP 최저 91.69413%, HWPX 및 XML 개행 대조군 최저 90.87964%.
- 전쪽 fresh Docker WASM: HWP 최저 91.69413%, HWPX 및 XML 개행 대조군 최저 90.87964%. 전쪽 TSV는 `tracking-wasm-final/{hwp,hwpx,xml-spacing}/silhouette.tsv`에 있다.
- HWPX Native/fresh WASM 대표 2·3·4쪽 review gate는 passed다. 직접 확대 대조와 메인터너 판정은 자동 gate와 따로 기록했다.
- 전체 nextest: 10,547 PASS / 0 FAIL / 50 SKIP(597.697s). 기존 두 실패는 모두 구현 보정으로 통과했으며 baseline을 바꾸지 않았다.
- HWPX 정식 회귀 3건 추가: 가시 텍스트·개체, 같은 소유 목록의 XML 공백 병합, 실제 빈 줄·제목의 새 쪽 소속. 수정 전 `9e6882188` 라이브러리에 같은 최종 테스트를 연결하면 HWP 3 PASS / HWPX 3 FAIL(가시 표 10→4, 18→6쪽)이다. 수정본 전체 6 PASS(`issue5169-hwpx-{before,final}-tests.log`). XML 개행만의 초기 12쪽/164문단 실패와 빈 줄 한 줄의 2쪽 27.65746%는 위 진단 증거로 구분하며, 이 원인의 각각에 대한 분리된 정식 수정 전 실행을 주장하지 않는다.
- Native Skia lib는 rhwp 3,927 PASS / 13 IGNORE이며 workspace lib 15·165·2건도 PASS다. placeholder/direct-PDF 검사는 아직 진행 중이다.

검토 종료 전 fetch한 upstream/devel은 여전히 `90fa2f8955c2dd21e9f0ea8fd0c6ee8dc5996a47`이다. 원 기여 이력을 유지한 source branch는 이후 devel 27개 commit을 포함하지 않는다. 여기의 통과는 통합 전 source의 결과로 구분하며, devel을 통합한 뒤 바뀐 실제 소비 경로의 lint·회귀·fresh Docker WASM·시각 증적을 갱신한다. 통합 후 source로 이미 통과했다고 보고하지 않는다.

## 최신 devel 통합 및 재검증

`upstream/devel` `90fa2f895`를 원 PR 이력을 보존한 브랜치에 충돌 없이 merge했다(통합 code `b9905ceee06c85703f8500462beb7c8511bb0ccb`). 통합본의 fmt, Native Clippy, WASM32 lib Clippy, workspace build, workspace all-target Clippy, base 고정 manifest 검사가 통과했다. 변경 추적 정식 검사 6건, 암호 ViewText 검사 7건도 통과했다. Native 전쪽 출력은 `maintainer/integrated-native/{hwp,hwpx,xml-spacing}/silhouette.tsv`에 있으며 통합 전 최소값을 유지한다. 새로 생성한 세 입력의 총 18장 Native PNG는 통합 전 PNG와 각각 SHA-256이 동일하다. 대표 HWPX 2·3·4쪽은 통합 head에서 다시 생성한 `integrated-native-review`로 확인한다.

앞선 Skia placeholder 단계는 테스트 추가 뒤 파생 suite 목록을 다시 준비하지 않아 0건 실행/exit 4였고 통과로 세지 않았다. integration suite와 case resolver를 동일 source에서 다시 준비해 통합 head의 세 Skia 단계를 재실행한다. 최종 fresh Docker WASM·전수 nextest·Skia 결과는 진행 중이다.

통합 후 fresh WASM package `a212e003d920de5a4632386622678f5d83cdc5a2d2edceb217a184f5c15e3550`는 실제 브라우저에 로드됐지만, devel의 새 exporter가 요구하는 `setHangul2024Compat`가 없어 WASM Sweep이 명시 오류로 중단됐다. 이 결과는 측정 불가이며 통합 전 TSV로 대신하지 않는다. `DocumentCore::set_hangul2024_compat`는 이미 있고 `HwpDocument`의 WASM export가 없어, 같은 core 설정에 위임하는 6행 API를 code `c2f0bad49`로 추가했다. 조판 규칙·임계값·검사 guard를 바꾸지 않는다. 진행 중인 통합 nextest는 최종 코드에서 다시 실행하기 위해 중단했으며 완료로 집계하지 않는다. 최종 lint·전체 회귀·Docker WASM을 다시 시작한다.

## 최종 통합 code의 재검증 (c2f0bad499)

code SHA는 `c2f0bad49959b853a100458b3d74810c37d0b385`, base는 `90fa2f8955c2dd21e9f0ea8fd0c6ee8dc5996a47`이다. fmt, 세 Clippy, workspace build, base 고정 integration manifest 및 source unit-tier 정책 검사가 모두 통과했다. 집중 회귀는 #5169 HWP/HWPX 6 PASS, #7674 암호 ViewText 7 PASS다. 최종 전수 nextest 및 Native Skia 3종은 아래 종료 결과로 확정한다.

최종 root Docker WASM과 Studio public의 WASM 해시는 `3988720b9da0d2fac1a3a59d612a8c291950d0e2a6669aa008cdbd70166e3a4a`로 일치한다. 최신 JS에는 동일 core setter의 생성 binding도 포함했다. Studio `http://localhost:7700/?review=pr7698-c2f0bad49-final`의 실제 초기화 응답도 같은 해시이며 6쪽·표제 표시·숨긴 두 문구 제외를 확인했다(`studio-hwpx-compatible-final.json`). 메인터너가 승인한 앞선 Docker WASM과 최종 WASM의 세 입력 18장 raster는 각각 byte-identical하다. API 노출 보정 뒤 기존 시각 판정을 임의로 재요청하지 않는다.

전쪽 새 Native/fresh Docker WASM TSV는 `maintainer/compatible-{native,wasm}/{hwp,hwpx,xml-spacing}/silhouette.tsv`다. 세 입력 모두 각각 6쪽이고 누락·90% 미달은 0쪽이다. HWP 최소 91.69413%, HWPX 및 의미 동등 XML 공백 대조군 최소 90.87964%다. TSV 해시와 전후 PNG 비교는 `compatible-final-evidence.json`에 고정했다. 최종 대표 HWPX 2·3·4쪽의 Native/fresh WASM review·overlay는 `compatible-{native,wasm}-review/hwpx/`에서 새로 생성했고 gate passed 및 직접 확대 판독을 확인했다. 빈 줄의 새 쪽 소속, 제목·본문 순서, 표 괘선과 앞뒤 문단을 대조했다. 남은 자형·글자 폭·미세 간격 차이는 픽셀 동일로 표현하지 않는다. 공개 PNG는 [안정 경로](../assets/pr7698/README.md)에 보존했다.

최종 fresh WASM `openWithPassword` 검사는 압축 여부·BodyText Section1 유무·strict/lenient의 8조건, 비밀번호 누락/오답 거부를 통과했다. 공개 암호 대조군과 동일 cipher View 복제 대조군 모두 64쪽이고 64쪽 전체 text-layout/SVG가 동등하다. 공개 HWP 원본/미편집 저장본은 visible text·6쪽이 동일하다(`browser-contract-compatible-result.json`). 실제 비공개 기업 원본의 독립 Print 검증으로 확장하지 않는다.

새 WASM setter는 기존 #5524 공개 문서로 실제 호출했다. 기본/2022에서 문단 13은 2쪽, 2024에서 1쪽이며 기존 Native 계약과 같다(`api-compat-page-contract.json`, `api-{2022,2024}/manifest.json`). 서로 다른 Print engine의 독립 PDF도 확보했지만 별도 전쪽 일치율은 측정하지 않았으므로 이 API 경계 검사를 #5524 시각 완료로 승격하지 않는다. 이번 API는 설정 뒤 core의 재페이지네이션에 위임하며 exporter 검사를 우회하거나 조판 임계값을 바꾸지 않는다.

### 최종 로컬 검증 종료

`validate-compatible.sh`는 전체 단계 exit 0으로 종료했다. 전체 nextest는 10,565 PASS / 0 FAIL / 50 SKIP(513.110s). Native Skia lib는 rhwp 3,927 PASS / 13 IGNORE 및 내부 crate 15·165·2 PASS, placeholder 2 PASS, 직접 PDF 4 PASS다. 단계별 로그는 `compatible-final-*.log`, `compatible-nextest.log`, `compatible-native-skia-{lib,placeholder,direct-pdf}.log`다. 이전 0건 실행/중단 로그와 구분하며 테스트·baseline·허용치 변경 없이 기존 래칫까지 통과했다. 사용한 nextest 0.9.137의 권장 버전 경고는 기록하고 실제 실행 건수·exit 0을 확인했다.

원 fork head는 SSH/REST 모두 `1489ca1d95e3d6e6b888870d3b14cf240866ef00`, 기존 branch `fix/7674-encrypted-viewtext`이며 maintainer 수정 허용이다. upstream/devel은 REST 재조회에서도 고정 base와 동일하고 merge-tree는 충돌 없이 `dbcd5a61cd3d30e0d7cfdf77bb4d63d39a65cfde`다. 제출 diff의 신규 HWPX/PDF/PNG는 일반 Git 바이너리이고, 원 fork head 이후 553개 object의 사전 판독에서 신규 LFS pointer는 0개다(`lfs-range-preflight.json`). 전체 역사 대상으로 URL 직접 지정한 LFS dry-run은 기존 역사의 19개 객체를 열거했으며 신규 자료 업로드 범위로 취급하지 않는다. lock 확인의 URL 인증 오류도 제품·검증 실패로 세지 않는다. 원격 source push·본문 갱신·새 CI/CodeQL·merge는 별도 승인 범위에 따라 진행하며 아직 수행하지 않았다.


## 승인 후 원격 반영·병합 결과

사용자가 보정 검증 통과 뒤 다음 절차 진행을 승인했다. 원 fork `scari/rhwp`의 `fix/7674-encrypted-viewtext`에 원 기여 이력을 보존한 head `34e1276015eb892a199cfa576b1d90fdd194135b`를 fast-forward push하고 한국어 제목·본문을 갱신했다. 본문 일치·UTF-8과 정확한 head SHA의 대표 PNG 4개 raw URL을 확인했다. 신규 LFS pointer는 0개이며 fork lock 인증 실패에는 해당 push 한 번의 `lfs.locksverify=false`만 적용했다. hook을 우회하거나 영구 설정을 변경하지 않았다.

- [CI 38117884014](https://github.com/edwardkim/rhwp/actions/runs/38117884014) success. Lint·Native Skia·Archive A/B/C/D·필수 Build & Test가 성공했다. CI WASM Build는 skipped이며 실제 WASM 증거는 로컬 Docker 빌드·브라우저 검사다.
- [CodeQL 38117884066](https://github.com/edwardkim/rhwp/actions/runs/38117884066)와 별도 GHAS check success. CI merge SHA `96bc1e2471fe93b229af73e9c73c90dc832ef8cd`의 네 언어 SARIF 결과는 각각 0건, Rust 분석 ID `1930807311`이다. CodeQL 2.27.2 / rust-queries 0.1.44에서 `CleartextLogging.ql`의 로드·평가·해석을 실제 로그로 확인했다. 이름·반환 계약 보정의 새 분석 결과이며 문서 본문 일반의 비민감성을 주장하지 않는다.
- merge 직전 base `90fa2f8955c2dd21e9f0ea8fd0c6ee8dc5996a47`를 fetch하고 head·필수 checks·MERGEABLE/CLEAN 및 merge-tree `e7f587ea2d9f75a7012575136ca65b162d906470`을 재확인했다. 총 37개 check가 종료됐고 failure/pending은 없었다.
- [PR #7698](https://github.com/edwardkim/rhwp/pull/7698)을 `--merge --admin --match-head-commit`으로 병합했다. merge `4020c0428defba3459c22879f38facd8019da2d7`, 시각 `2026-10-11T06:42:52Z`. 원 기여 `1489ca1d95e3d6e6b888870d3b14cf240866ef00`과 제출 head가 upstream/devel의 ancestor인 것을 확인했다.
- [#7674](https://github.com/edwardkim/rhwp/issues/7674)는 [devel push 종료 작업](https://github.com/edwardkim/rhwp/actions/runs/38118957938)으로 `2026-10-11T06:43:20Z`에 completed 종료됐다. #5169 등 참조 이슈를 함께 닫지 않았다.
- [duration 갱신](https://github.com/edwardkim/rhwp/actions/runs/38118957946)은 success, `refreshed_targets=46`으로 metrics branch에 반영됐다. 병합 후 검증 CI를 재실행하지 않았다.

후속 운영 기록은 archive review 두 파일과 오늘할일만 devel에 직접 반영한다. PR 전용 clean worktree `/tmp/rhwp-pr7698-before-hwpx-20261011`와 local review branch는 clean·ancestor·활성 프로세스 부재 확인 뒤 제거 완료했다. contributor fork branch·다른 검토 worktree·공용 `target/pr-review`는 보존한다. 기본 작업공간의 Studio 7700은 메인터너가 확인 중인 환경이므로 유지한다. 초기 진단 전용 ref는 merged source에 없는 로컬 기록을 포함하므로 유지한다. ignored 원문 검증 자료도 댓글용 상세 검증 근거 보존을 위해 유지한다. 이번 승인 범위의 push·본문·merge·이슈 종료 확인은 완료했으며 별도 PR/issue 댓글은 게시하지 않았다.
