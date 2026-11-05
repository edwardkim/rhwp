# Task #8578 — 묶음 글상자 클릭과 Esc 편집 복귀

현재 상태: 최신 base `6e1de406209ba9a58032f6ca6de45b18aa687d1c` 리베이스 후 필수 로컬 검증과 사용자 정상 동작 확인 완료.
실제 producer는 `2c0432cc0646cf2ad0b277b50e4c685ab7092a80`다. 작업 위치를 `/Users/tsjang/rhwp`로 옮겼고 이전 작업트리를 제거했다.
사용자가 원격 push·PR 생성을 승인했다. 최종 범위·실행 결과·증적은 [최종 PR 검토](../report/task_m100_8578_report.md)와
[검증 결과](../pr/assets/issue_8578/20261105/verification.json)를 따른다. 아래 초기 단계는 진행 이력이며 새 후보 결과로 재인용하지 않는다.

## 독립 근거와 원인 경로

#8583에서 인쇄·글꼴 공급 보정과 분리한 [#8578](https://github.com/edwardkim/rhwp/issues/8578)을 처리한다.
한컴 편집기 화면에서 개체 선택 해제 후 같은 쪽의 소유 본문으로 복귀하는 동작이 기대값이다.
원본 HWPX `Contents/section0.xml`의 본문 문단 4에는 묶음 안 제목과 안내 본문 글상자가 각각
있다. 문단 6의 묶음은 `BEHIND_TEXT` 장식이며 목차 본문은 그 밖의 문단 8부터 시작한다.

- 생산: `shape_layout.rs`가 묶음 자식의 글상자에 바깥 Group 컨트롤 번호와 동일한 셀 번호 0을 부여한다.
- 조회·편집: `get_textbox_from_shape`는 독립 도형의 글상자만 찾으므로 Group 주소를 해석하지 못한다.
- 클릭: `cursor_rect.rs`는 텍스트 유무와 무관하게 Rectangle/Ellipse/Path의 외곽을 글상자로 수집한다.
  이 잘못된 후보가 목차 본문 클릭보다 먼저 빈 글상자 진입으로 선택된다.
- Esc: `moveOutOfSelectedPicture`가 삭제·붙여넣기용 인접 문단 이동을 그대로 사용한다.

## 수정 방향과 대조군

묶음 안 글상자도 기존 셀 편집 경로에서 서로 구분 가능한 원문 소유 주소를 생산하고,
조회·변경·리플로우·저장과 커서 조회가 같은 주소를 소비하게 한다. 묶음을 풀거나 원문을 변환하지 않는다.
실제 글상자만 클릭 후보로 등록하며 배경 장식이 본문을 가로채지 않게 한다.
Esc 복귀와 삭제·붙여넣기 착지는 분리한다. 독립 글상자, 중첩 묶음, 표 안 글상자와 개체 삭제의
기존 계약을 대조군으로 검증한다. 좌표·폰트·인쇄 출력은 이번 수정의 목표값으로 변경하지 않는다.

## 초기 단계의 검증 기록

- Esc 수정 전 행위 검사: 소유 문단 4 대신 문단 5로 이동해 FAIL.
- 최신 base fresh WASM 빌드 완료(호스트 locked wrapper `--no-opt`, Docker 결과 아님).
  실제 브라우저에서 3쪽 제목·본문이 같은 주소로 합쳐지고 조회 오류가 나며,
  4쪽 목차가 문단 6의 장식 주소로 반환되는 것을 재현했다.
- Esc를 `returnToSelectedObjectOwner`로 분리했다. 인접 문단 착지는 삭제·붙여넣기에 그대로 둔다.
  실제 브라우저 Escape 키 입력 후 문단 5 대신 소유 문단 4로 복귀했다.
  본문·중첩 셀 소유 복귀 행위 검사 및 관련 Studio 검사 10건 PASS, TypeScript 검사 PASS.
  이 단계는 묶음 자식 글상자의 클릭·입력 해결 완료를 뜻하지 않는다.
- 묶음 글상자의 `cellIndex`를 원본 자식 순서의 깊이 우선 글상자 번호로 정의했다.
  도형 자체를 복제하지 않으며 글상자가 없는 장식·중간 묶음은 번호를 소비하지 않는다.
  `textbox_shape_at` 공통 조회를 커서·텍스트 편집·리플로우·내부 붙여넣기·중첩 개체 주소에 적용했다.
  RenderNode의 실제 글상자 소유 메타데이터를 클릭 후보에 사용한다. 좌표·clip·paint는 바꾸지 않는다.
  자식 글상자의 입력 폭은 Group 외곽이나 비어 있는 child common.sz가 아닌 원본 orgSz·합성 변환을 쓴다.
- Native 주소 검사 수정 전 2 FAIL → 수정 후 2 PASS. 추가 입력·삭제·Enter·병합·HWP/HWPX
  저장 왕복과 제목/본문/후속 문단 보존 검사도 PASS: 최종 집중 검사 총 3 PASS, 22.312초.
- Studio 전체 단위 검사: 2,721건 중 2,719 PASS, 0 FAIL, 2 skip.
- 구현 중 suite manifest가 바뀐 초기 실행은 0 tests이므로 무효로 분리했다.
  최종 실행은 `--prepare` 후 실제 `regression_suite_011`의 3건을 실행한 결과다.
- 후보 fresh WASM, 실제 브라우저 편집·undo/redo와 필수 범위 게이트: 아직 완료하지 않음.
- 임시 로그·진단 스크립트: 기본 작업장의 ignored `output/issue8578/`.

이 초기 단계에서는 해결 완료 또는 PR 제출 가능 상태로 판정하지 않았다.

## 실제 편집 경로에서 찾은 추가 원인

`ec217c275b`의 fresh WASM 초기화 응답 SHA-256은
`efbd2cce1c78b7583c06f3f7940749bd769d7666d2e5fd5735c9641712aeafe2`로 확인했다.
실제 클릭·입력·드래그·삭제·undo/redo·Enter/Backspace는 진행됐지만, Esc의 논리 주소를
소유 문단 4로 고친 것만으로는 캐럿이 생성되지 않았다. 자식 글상자의 TextRun은 셀 주소이며,
비어 있는 바깥 본문 문단의 TextRun이 아니므로 `get_cursor_rect_native(0, 4, 0)`이 실패했다.

묶음을 가진 빈 non-TAC 본문 문단에는 기존 숨은 문단 표시기를 재사용한다. 개체의 외곽이나
자식 글상자 좌표를 캐럿으로 추측하지 않고 `layout_shape`가 받은 확정 본문 흐름 원점에
편집 전용 표시를 남긴다. 같은 소유자의 본문 캐럿이 이미 있으면 중복하지 않는다.
기존 OLE의 전용 문단 끝 앵커와 독립 도형 배치는 변경하지 않는다.

추가한 Native 관계 검사에서 소유 문단 4·6의 캐럿은 각각 물리 3·4쪽에 남았다.
최종 집중 검사 4 PASS, 0 FAIL(9.007초). 이 검사는 위치 수치를 고정하지 않는다.
새 WASM 재빌드·실제 파일 열기부터의 브라우저 재검증과 출력 보존·전체 회귀는 계속 진행한다.

머리말 배치기도 같은 `layout_shape`를 호출하는 것을 추가 추적했다. 새 본문 캐럿은
Column/Body 컨테이너에만 생산하여 머리말·꼬리말·각주의 전용 주소를 침범하지 않는다.
실제 머리말 묶음을 생성·저장해 본문 문단 0의 캐럿 주소가 누출되지 않는 관계 대조군을 추가했다.
이 대조군을 포함한 최신 전체 회귀는 아직 완료하지 않았다.

고정 base Native의 편람 전383쪽 필수 gate는 PASS(335.752초): 최저 표시 99.96%, raw 완전일치 376쪽.
이는 base `22947f6a90`의 증거이며 후보의 통과 결과로 재인용하지 않는다.

## 실제 파일 열기부터의 브라우저 확인

보완 WASM(`13d3a784d7a6ed27d52f48fc677651844e3ff0f5ae1d373c4c7dd675e338732f`)으로
파일 입력 채널에서 원본을 열어 Canvas2D 383쪽을 확인했다. headless의 로컬 글꼴 감지 창에서는
지원되는 `대체 글꼴로 보기`를 선택했다. 실제 초기화 glue JS 응답과 서버 산출물의 해시를 대조하고,
전용 CDP buffer의 WASM 초기화 응답도 pkg/public의 프로그램 해시와 일치했다.

3쪽 본문은 `parentParaIndex=4, cellIndex=1`로 편집되고 제목 주소와 구분됐다.
4쪽 목차는 장식 주소 대신 본문 `paragraphIndex=8`을 반환했다. 실제 마우스·키보드로
입력·드래그·삭제·undo/redo·Enter/Backspace·그룹 선택·Esc·본문 입력을 수행해 PASS,
잘못된 셀 주소 및 캐럿 경고 0건을 확인했다. Esc 복귀 주소는 본문 문단 4다.

최종 소비점 감사에서 하이퍼링크 문단 조회에도 독립 글상자의 `cellIndex=0` 제한이 남아 있었다.
텍스트 편집과 동일한 indexed helper로 바꾸고 제목과 본문의 링크 필드 소유를 구분하는 관계 검사를
추가했다. 이 마지막 보완을 포함한 최종 검증은 별도로 완료한 뒤 기록한다.

최소 글꼴 회귀는 ubuntu-ted3의 별도 작업트리에서 수행한다. 기존 기본 작업트리는 clean이며,
시스템·사용자 글꼴 경로를 파일 시스템에서 차단하고 프로젝트/fixture의 64개 프로그램만 노출했다.
첫 metadata 실행의 DNS 실패는 격리 환경의 resolv.conf symlink 대상 누락으로 확인하여 해당 DNS
파일만 읽기 전용 노출했다. 이는 테스트 FAIL이 아니라 실행 전 인프라 오류이며 통과 결과로 세지 않는다.

## 1차 프로그램의 편집·출력 보존 확인

Rust/Studio 구현 head `124e02bf06805f65d792aab939b42aa2b56b2475`에서 Native 집중 6건 PASS,
fmt·Native/WASM Clippy·workspace build·all-target Clippy·manifest·unit tiers·E2E manifest·
TypeScript 검사까지 Mac 순차 게이트 13개를 통과했다. 새 회귀는 자동 수집되는 관계 검사이며,
인쇄 위치를 픽셀로 고정하거나 기존 문서의 기대값을 완화하지 않았다.

같은 입력·독립 Print PDF·실제 설치 글꼴·현재 정본 Visual Sweep으로 base와 후보를 각각 출력했다.
Native/fresh WASM 모두 전383쪽, 최저 raw 99.96%, 90% 미만 0쪽이다. 각 backend의 base/후보
TSV 행과 383개 출력 PNG SHA-256이 모두 동일하다. Native 필수 편람 gate도 통과했다.
Native raw 100%는 376쪽, WASM raw 100%는 380쪽이며 기존 미세 차이를 새 개선으로 주장하지 않는다.
3·4쪽의 canonical review/overlay를 직접 판독해 제목·안내문·목차·장식의 내용과 배치 보존을 확인했다.

최종 fresh WASM은 `9c3f92982f0cce3ee16af2860712afcdb5946f5c82971668db2bca205d6f6b61`이다.
pkg/public과 실제 Canvas2D 초기화 응답이 일치했다. 실제 파일 열기부터 입력·드래그·삭제·Enter·
undo/redo·목차 클릭·개체 선택·Esc가 통과했고 관련 경고는 0건이다. 추가한 E2E assertion으로
목차 본문과 Esc 복귀 본문이 실제 문자열로 변경되고 undo 후 복원되는 것까지 재실행해 확인했다.
이 assertion 보강은 Rust/Studio 구현과 프로그램을 바꾸지 않았다.

전쪽 TSV와 3·4쪽 raw/review/overlay, 실제 Studio 화면·주소 JSON·프로그램 출처는
[증적 디렉터리](../pr/assets/issue_8578/20261105/)에 보존한다. `.log`, 빌드 바이너리, 글꼴은 커밋하지 않는다.
최소 글꼴 Linux 집중 6건도 PASS이며 전체 nextest·Native Skia 3종은 진행 중이다.
전체 회귀의 최종 summary와 exit code를 확인하기 전에는 PR 준비 완료로 판정하지 않는다.

## 전체 회귀에서 발견한 호환성 보정

위 프로그램의 최소 글꼴 전체 nextest는 12,487건 실행, 12,485 PASS·2 FAIL·45 skip이었다
(실행 1,082.523초). 따라서 위 시각·편집 통과만으로 해결 완료를 주장하지 않는다.

- `issue_919_textbox_inner_empty_hit_returns_textbox_entry`: 원 해결 `14920220ac`와
  [#919 완료 근거](https://github.com/edwardkim/rhwp/issues/919#issuecomment-4504068810)는
  글상자 안 표의 셀도 바깥 글상자 소유를 유지하도록 한다. `CellContext.in_textbox`는 직접
  글상자 문단 여부이므로 안 표에서 false가 된다. 실제 바깥 Shape와 indexed 글상자 주소로
  `isTextBox`를 확인하여 중첩 표와 두 번째 묶음 글상자를 함께 지원한다.
- `issue_1892_hwp3_group_roundtrip_preserves_owned_content_and_object_frames`: 원 해결
  [#1905](https://github.com/edwardkim/rhwp/pull/1905)와 메인터너 관계 검사 정리 `6097253468`
  (#8191)를 확인했다. 시작 캐럿에 원문 공백을 복제하면 HWP 저장 후 생긴 공백 표현만 표시
  내용으로 추가된다. 기존 숨은 문단의 원문 오프셋 보존과 새 묶음의 시작 위치 캐럿을 구분해,
  공통 생산기에 사용할 marker 문자열을 명시한다. 시작 캐럿은 빈 문자열이며 원문과 인쇄 흐름을
  바꾸지 않는다. 기존 숨은 문단 호출은 계속 원문 공백을 전달한다.

두 기존 검사의 단언과 기대값을 유지한다. 두 번째 묶음 글상자 안 표의 소유 경로와 공백 host의
표시 내용 비생성 관계 검사를 추가했다. 기존 head의 추가 Skia 컴파일은 보정을 위해 중단했으며
통과로 세지 않는다. 보정 후 관련 packet부터 다시 실행하고 최종 전체 검증·출력을 갱신한다.

보정 직후 기존 #1892·#919 10건은 모두 PASS였다. 추가 합성 대조군 2건의 첫 실패는 테스트
작성 오류였다. 정상 본문 공백과 editor-only 시작 캐럿을 구분하도록 기대값을 바로잡았고,
중첩 표 대조군에는 구역/단 정의가 있는 최상위 문단 전체가 아닌 유효한 표 컨트롤만 공급했다.
저장 전 seed 문단의 컨트롤 번호를 새 컨테이너 주소로 잘못 재사용하지 않는다.
수정 후 Native 18건 packet은 18 PASS, 0 FAIL(9.700초)이며 기존 두 실패의 단언은 그대로다.
이는 관련 집중 검사의 결과이며 새 전체 nextest·Skia·WASM·전쪽 검증은 별도로 완료한다.

## 최종 검증과 문서 전용 리베이스

보정 후 코드 `e12096b9da2e7f00c03c5c175f76d198df284e56`를 고정해 다음 검증을 완료했다.
로그는 ignored `output/issue8578/final-v3/`에만 보존하고, 제출용 수치·명령·source 해시는
[최종 증적](../pr/assets/issue_8578/20261105/producer.json)에 고정했다.

| 검증 | 실제 결과 |
| --- | --- |
| 최소 글꼴 Linux 전체 nextest | 12,489 PASS·0 FAIL·45 skip, 실행 1,018.448초, exit 0 |
| 최소 글꼴 Native Skia lib | 본체 3,858 PASS·12 ignore, 다른 libs 182 PASS, 0 FAIL |
| 최소 글꼴 Native PNG·PDF | PNG 2 PASS, PDF 22 PASS |
| 집중 packet | 새 #8578 8건 + 기존 #1892 5건 + #919 5건 = 18 PASS; Native·최소 글꼴 Linux 모두 PASS |
| Mac 필수 게이트 | fmt·Native/WASM Clippy·workspace build·all-target Clippy·manifest·unit tiers·E2E manifest·TypeScript 포함 13단계 exit 0 |
| Studio 전체 단위 검사 | 2,719 PASS·0 FAIL·2 skip; 이 실행 이후 Studio production source 변경 없음 |
| 실제 브라우저 | Canvas2D 원본 파일 열기부터 3·4쪽 편집·선택·삭제·Enter·undo/redo·개체 선택·Esc·원 본문 실제 입력 PASS, 관련 경고 0 |
| 독립 Print 전쪽 비교 | Native/fresh WASM 각각 383쪽, 최저 raw 99.96%, 미달·누락·중복·base 대비 하락 0 |

위 항목은 서로 다른 실행 묶음이며 고유 테스트 총수로 합산하지 않는다. 최소 글꼴 회귀는
ubuntu-ted3의 bwrap 안에서 시스템·사용자 글꼴 경로 0개와 프로젝트/fixture 프로그램 64개만
노출했다. 시각 비교는 독립 Print와 같은 글꼴을 공급한 Mac 실행이며 최소 글꼴 시각 정합으로
바꾸어 주장하지 않는다. 새 글꼴·검사 생략·문턱 완화는 없다.

제출 전 새 upstream PR #8603을 확인했다. 추가분은 과거 PR 검토 문서 3개뿐이었다.
로컬 백업 `backup/8578-before-e96b500-20261105`를 만든 뒤 충돌 없이 리베이스했으며,
리베이스 후 코드 head는 `67c7814f6e2bc02cc6f69365f1c24f581a6826ca`다.
`src`·`tests`·`rhwp-studio`·`scripts`·`crates`·`vendor`의 Git tree는 전후 동일하고
새 검토 문서 3개도 upstream 원문과 같다. 이 PR의 기능은 기록 보존이므로 runtime 회귀 범위는
비해당이며, 실제 diff와 소비 경로가 달라지지 않았음을 [동일성 증거](../pr/assets/issue_8578/20261105/source-equivalence.json)로 남겼다.

리베이스 후에는 새 head에서 집중 18건(16.714초), fmt·새 base 기준 manifest/unit tier·TypeScript·
실제 브라우저를 다시 통과했다. Native `--handbook-gate`도 전383쪽 새로 출력하여 통과했다.
WASM 프로그램은 위 코드에서 빌드한 `a54403d36e2c54472e9daa0491e90c61359ade7dab0d8e154020d14a1404b149`이며
문서 전용 rebase 후에도 source·pkg/public·실제 초기화 응답이 일치한다. Mac locked wrapper
`--target web --out-dir pkg --no-opt`의 로컬 대체 빌드이고 Docker 최적화 빌드가 아니다.

WASM PNG는 재출력했다고 주장하지 않는다. 프로그램·입력·PDF·source 동일성과 전쪽 PNG 해시를
고정하고 정본 `--png-pair`로 base/후보 전쪽 TSV를 다시 산출했다. Native 전후 PNG도 전383쪽
SHA가 동일하며, 두 backend의 최종 TSV는 각각 base 및 리베이스 전과 같다. [전후 비교](../pr/assets/issue_8578/20261105/tsv-before-after.json)와
[전쪽 raster 해시](../pr/assets/issue_8578/20261105/raster-sha256.json)를 보존한다.
TSV의 두 자리 표시값은 130쪽 99.96%, 나머지 382쪽 100.00%다. 표시의 반올림과 정밀값의
정확한 100% 개수(Native 376쪽·WASM 380쪽)를 구분한다.

3·4쪽 raw/review/overlay는 해당 원본과 독립 Print를 직접 대조했다. Studio raw 화면에는
검사 중 입력 문자가 있고, 인쇄 PNG는 미편집 원본이다. 원본 HWPX·PDF·글꼴·조판 문턱은
변경하지 않았다. 이전 두 실패는 기존 검사를 유지한 구현 보정으로 해결됐다.
원격 push·PR 생성·이슈 종료는 아직 수행하지 않았으며, 사용자 승인 후 제출한다.

## 최신 코드 리베이스 후 최종 범위와 검증

- base `6e1de40620`, producer `2c0432cc06`. 충돌 없이 리베이스했지만 새 코드 통합이 있어 전체 회귀를 다시 실행했다.
- 전체 12,591 PASS·0 FAIL·45 skip, Native Skia lib 4,040 PASS·PNG 2 PASS·PDF 22 PASS.
- Studio 2,721 PASS·2 skip, 실제3·4쪽 편집·Esc 통과. 사용자도 정상 동작을 확인했다.
- 해당 3·4쪽 Native/fresh WASM 100.00%, 동일 편람 Native 383쪽 최저 99.96%·기준 하락 0·PNG 전쪽 동일.
- 조판 수치·분할·인쇄 좌표의 수정이 아니라 편집 소유 주소 보정이다. 넓게 시작한 다른 HWP 시각 비교는 사용자 지시 후 중단하고 최종 제출에서 제외했다.
- 초기 후보 WASM의 공유 target 혼입은 실제 API로 검출했다. byte 변경 없이 자신의 crate를 재컴파일하고 다른 hash의 최종 프로그램으로 API·브라우저를 다시 통과했다.
- 최신 증적의 Native TSV는383쪽, fresh WASM TSV는 해당 3·4쪽이다. 이전 후보의 전383쪽 WASM 검증을 새 source의 완료로 사용하지 않는다.
- 임시 로그·중간 문서·스크립트는 `output/issue8578/rebase-6e1de406/`에만 보존한다.
