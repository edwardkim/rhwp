---
kind: pr-review-implementation
status: completed
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-10
---

# PR #4274 Windows 시나리오 이식성 메인터너 보정 및 검증 기록

## 병합 확정

- [PR #4274](https://github.com/edwardkim/rhwp/pull/4274)는 2026-08-10에 merge commit
  `92100edbf6bb1dade47af1342a49b089a6ed2e1c`으로 `devel`에 병합됐다.
- 최신 head `d69bf7dc1`의 GitHub Full CI, CodeQL, Canvas visual diff, Native Skia와
  Build & Test가 모두 성공했다.
- 메인터너 보정은 다른 Windows 작업자에서도 Oracle gate를 재현할 수 있게 경로를
  `{repo}`·`{out}` 토큰으로 고정하고, Hancom 종료 대기와 저장 질문 차단을 추가한 것이다.
- 최종 Windows Hancom 2022 검증에서 82개 시나리오 3,519/3,519 호출이 일치했고, 실행 뒤
  `Hwp.exe`·`HwpFrame.exe` 잔류와 저장 확인 대화상자는 없었다.

## 근거

최신 contributor head `27161a67b`의 `pF-insertpicture`, `pG-pageimage`,
`pH-insertrepeat`은 `paths.win`에 `C:\\Users\\planet\\...`를 고정했다. `runner_ocx.py`는
Windows에서 이 값을 그대로 COM Oracle에 전달하므로, 다른 Windows 작업자에서는 그림 입력과
쪽 그림 출력이 실패한다. 실제 `win10-ted`에는 contributor 경로가 없고, review worktree의
`samples/s1.jpg`만 존재함을 확인했다.

## 보정 순서

1. 세 시나리오의 Windows 갈래를 `{repo}` 또는 `{out}` 토큰으로 전환한다. Windows에서만
   역슬래시 경로가 되도록 작성하되, POSIX 갈래와 호출 계약·기대 반환값은 바꾸지 않는다.
2. `scenario_spec.py`의 예시와 `test_harness_contract.py`를 함께 고친다. Windows 경로를
   `PureWindowsPath`, POSIX 경로를 `PurePosixPath`로 각각 검증하고, tracked scenario에
   사용자 홈 절대경로가 다시 들어오지 못하게 한다.
3. Linux 계약·원장 검사를 다시 실행한다. Windows 별도 worktree에서 fresh WASM을 만든 뒤
   `pF`, `pG`, `pH` Oracle 대조와 80개 전체 gate를 순차 실행한다.

## 1차 결과 (Linux, 2026-08-09)

- `test_harness_contract.py`: 20건 통과. Windows·POSIX 토큰 해석과 contributor 홈 경로 금지
  계약을 포함한다.
- `npm --prefix npm/hwpctrl-ocx run ledger:check`: `308/484 완료`로 기존 원장 수를 유지했다.
- `npm --prefix npm/hwpctrl-ocx run test:contract`, 문서 메타데이터·상대 링크, JSON 문법,
  `git diff --check`: 모두 통과했다.
- Windows `win10-ted` review worktree에서 fresh WASM 빌드를 시작했다. 완료 후 실제 Hancom
  Oracle의 `pF`, `pG`, `pH`와 전체 gate 결과를 이 문서에 추가한다.

## 2차 결과 및 보정 (Windows Hancom 2022, 2026-08-09)

- fresh WASM(`pkg/rhwp_bg.wasm`, 9,195,123 bytes)과 active RDP 세션의 Hancom 2022 Oracle로
  portable path 시나리오를 재실행했다. `pF-insertpicture` 14/14, `pG-pageimage` 10/10,
  `pH-insertrepeat` 20/20 호출이 모두 `MATCH`였다.
- 기본 `run_gate.py`는 `Quit()` 뒤 10초 동안 남은 새 `Hwp.exe`를 안전상 자동 종료하지 않아,
  live Windows에서 첫 시나리오가 `OK/LEFTOVER`가 되고 뒤 시나리오가 `OCCUPIED`가 됐다.
  이는 시나리오 계약 불일치가 아니라 npm 공개 gate의 실행 경로 결함이다.
- `run_package_gate.py`를 추가해 Windows package gate에만 기존의 명시적
  `--cleanup-spawned`를 전달한다. baseline에 있던 사용자 Hancom 프로세스는 기존처럼
  `OCCUPIED`로 거부하며 종료하지 않는다. Linux/macOS는 이전과 동일한 WASM self-check다.

## 3차 결과 및 보정 (Windows package gate, 2026-08-09)

- 실제 `npm --prefix npm/hwpctrl-ocx run gate`에서 `p4-setmutate` 뒤 Hwp 종료가 비동기여서,
  `taskkill` 직후의 즉시 PID 재조회가 `OK/LEFTOVER`를 만들고 다음 일곱 시나리오를
  `OCCUPIED`로 건너뛰는 것을 재현했다.
- 정리 경로도 `wait_for_hwp_exit`로 재확인하도록 고친다. 정리 대상은 실행 전 baseline에 없던
  PID로 한정하며, 새 단위 계약은 `taskkill` 뒤 대기 호출을 고정한다.

## 4차 결과 및 보정 (CI shard 2, 2026-08-09)

- 최신 contributor head의 CI shard 2는 `issue_2027_picture_wrap_toggle_loss::
  tac_roundtrip_preserves_anchor_line_segs`에서 실패했다. 새 본문 그림 경로는 `char_offsets`만
  8-unit 이동시키고 기존 `line_segs.text_start`는 그대로 뒀다. floating 상태의 저장 줄 좌표는
  83인데 TAC off 재리플로우는 이동된 문자 좌표로 91을 산출해 왕복 정합이 깨졌다.
- 컨트롤은 문단 스트림에서 8칸을 차지하므로 `char_count` 증분은 유지한다. 대신 공용
  `shift_for_inline_control_insert`가 글자모양·범위 태그와 같이 `line_segs.text_start`도
  이동하도록 보정한다. 새 단위 테스트가 이 공용 계약을 고정하며, 그림 앵커 회귀는 기존 갭과
  control position 검증으로 계속 보호한다.
- 보정 후 `issue_2027_picture_wrap_toggle_loss::tac_roundtrip_preserves_anchor_line_segs`,
  `issue_4347_insert_leaves_coordinate_trace`의 본문·글상자 2건, 그리고
  `test_inline_control_insert_line_segs_shift`가 모두 통과했다.

## 5차 계획 (Windows Hancom 종료 대화상자, 2026-08-09)

- 실제 full gate에서 수정된 `2026_oss_rst.hwp`를 직접 `Quit()`해 “저장할까요?” 대화상자가
  활성 RDP 세션을 막는 것을 확인했다. 이 실행은 중단했고 결과를 사용하지 않는다.
- 설치된 `pyhwpx` API 문서에 따르면 `clear(option=1)`은 변경을 버리고, `option=0`만 저장
  질문을 띄운다. Oracle runner가 모든 종료 갈래에서 명시적으로 discard한 뒤 `Quit()`하도록
  helper를 추가한다.
- mock 계약은 discard와 Quit의 순서 및 종료 예외 격리를 고정한다. Windows에서는 먼저 수정
  시나리오 한 건을 prompt 없이 끝내는지 확인하고, 그 뒤 전체 package gate를 새 로그로 재실행한다.

## 5차 결과 (Windows Hancom 종료 대화상자, 2026-08-10)

- `runner_ocx.py`가 종료할 때 `pyhwpx.Hwp.clear(option=1)`로 활성 문서 변경을 버린 뒤
  `HwpObject.Quit()`하도록 공용 helper를 적용했다. discard가 예외를 내더라도 Quit은 계속
  시도한다.
- Linux mock 계약 24건, npm 공개 패키지 계약 1건, 원장 검사 308/484가 모두 통과했다.
- Windows 한컴 2022 `12, 0, 0, 535`에서 원본을 실제 수정하는 `p3-action-autonum` 138개 COM
  호출이 exit 0으로 끝났다. 실행 후 `Hwp.exe`는 0개였고 저장 확인 대화상자도 나타나지 않았으며,
  원본 표본의 변경 감지 가드도 통과했다.
- 같은 보정으로 당시 81개 live Oracle/rhwp 시나리오가 끝까지 실행됐다. 마지막 L3 비교만
  `target/release/rhwp.exe` 부재로 `WinError 2`가 나서 전체 exit 1이었으며, 이는 한컴 종료
  모달과 무관한 검증 선행조건 누락이다. 이후 Windows release CLI를 생성했고, 최신 contributor
  head의 확장 시나리오까지 반영한 full gate에서 최종 판정을 다시 고정한다.

## 6차 계획 (동결 source 재정렬과 업데이트 한컴 재검증, 2026-08-10)

- contributor가 반복 검증 요청을 수용해 PR source를 `4276d5dea`로 되돌리고 동결했다.
  `89312fdbc`의 저장본 차분 관측과 `4e2db92d8`의 z-order 6개 액션은 새 PR #4418로
  분리됐으므로, 해당 두 commit과 그 위의 메인터너 보정은 #4274 최종 판정에서 제외한다.
- 같은 가시성 branch를 동결 source 위로 다시 정렬하고, 이 PR에 필요한 경로 이식성·Windows
  package gate·종료 대기·line segment 왕복·한컴 종료 보정 5개만 single-parent commit으로
  유지한다. `upstream/pr4274-head`가 local `HEAD`의 조상인지와 diff 범위를 다시 확인한다.
- Linux에서는 하니스 계약, package 계약, 원장, 좌표 집중 회귀, fmt·clippy를 현재 head에서
  다시 실행한다. source가 바뀌었으므로 `89312fdbc` 이후 실행 결과는 역사 기록으로도 이
  단계의 통과 근거에 사용하지 않는다.
- Windows에서는 기존 dirty worktree를 덮어쓰지 않고 `4276d5dea` 전용 새 worktree를 쓴다.
  메인터너 보정 파일만 적용한 뒤 fresh release CLI와 WASM을 만들고, 업데이트된 Hancom 2022
  버전을 기록한다. `pF`·`pG`·`pH`, 수정 문서 종료, 공식 전체 gate를 순차 실행해 반환값,
  저장본 판정, 잔류 `Hwp.exe`와 저장 질문 대화상자를 함께 확인한다.

## 6차 결과 (동결 source와 업데이트 한컴 재검증, 2026-08-10)

- GitHub의 #4274 head는 `4276d5dea`로 동결됐고 local review branch는 그 위에 경로 이식성,
  package gate 정리, 종료 대기, 저장 질문 차단 네 commit만 둔 상태다. line segment 왕복 보정은
  contributor source의 `1ae455345`에 포함돼 있다. `89312fdbc`부터 시작하는 저장본 차분 작업은
  별도 PR #4418로 이동했으며 #4274의 현재 commit 목록에는 없다.
- Linux에서 하니스 mock 계약 24건, npm 공개 패키지 계약 6건, 원장 `312/484`, line segment와
  그림 좌표 집중 회귀가 모두 통과했다. `cargo fmt --all --check`와
  `cargo clippy --all-targets -- -D warnings`도 통과했다.
- Windows 전용 clean worktree에서 release CLI(19,473,920 bytes,
  SHA-256 `83A9EC8E7D4BD16473B4E84A39572AA12970C250BEDAC5A0A95582B027621D2E`)와
  WASM(7,853,478 bytes,
  SHA-256 `318229ABDBD1B628D0F36823A00008EFF997EA3763F642178040F7D3B935DED1`)을
  새로 만들었다. 업데이트된 한컴 COM 버전은 `12, 0, 0, 4605`다.
- 실제 한컴에서 `pF` 14/14, `pG` 10/10, `pH` 20/20, `p4-setmutate` 20/20이 모두
  일치했다. 공식 82개 시나리오도 중단·점유 없이 끝까지 실행됐지만 20건이 달랐다:
  `GetTextFile` 계열 11건, `SetTextFile` 6건, `ViewProperties` 2건, undo 뒤 텍스트 1건이다.
- Windows ACP가 65001인 환경에서 한컴의 `GetTextFile("TEXT", "")` 자체가 한글을 U+FFFD와
  깨진 문자열로 반환하고, 한글 `SetTextFile(..., "TEXT", "")`는 세 글자의 캐럿을 3이 아니라
  6만큼 옮겼다. 같은 호출을 ASCII로 실행하면 3·2만큼 정확히 이동했다. 반면 공식
  `UNICODE` 형식은 한글 4,030자를 손실 없이 반환하고 세 글자·두 글자 삽입도 각각 3·2만큼
  이동했다. 이는 rhwp 구현 차이가 아니라 `TEXT`의 시스템 ANSI code page 의존성이다.
- `ViewProperties`는 새 process에서도 `ZoomType=5`, `ZoomRatio=125`, `OptionFlag=0`으로
  반복됐고 `ViewZoomNormal` 뒤에는 확대율만 100으로 정규화됐다. `OptionFlag`는 이전 실측
  8192와도 달라 문서 계약이 아니라 사용자·버전별 창 상태임이 확인됐다.

## 7차 계획 (코드페이지·보기 상태 독립 Oracle 계약, 2026-08-10)

- Rust의 문서 글 조립은 한 번만 수행하고, 기존 `TEXT`용 CP949 수치 참조 JSON과 새
  `UNICODE`용 원문 JSON을 각각 노출한다. npm bridge는 `GetTextFile`의 format이
  `UNICODE`일 때만 원문 경로를 사용하고 기존 `TEXT` 동작은 유지한다.
- 한글 live gate의 본문·편집 검증은 시스템 ACP에 영향을 받지 않는 `UNICODE` 형식으로 바꾼다.
  `TEXT`의 CP949 밖 문자 `&#N;` 규칙은 Rust와 npm 공개 계약 테스트로 별도 고정해, Oracle
  이식성을 얻기 위해 기존 형식 계약을 없애지 않는다.
- `p8-props`는 먼저 `ViewZoomNormal`을 실행해 확대율을 100으로 고정한다. 버전·사용자 상태에
  따라 0과 8192가 모두 관측된 `OptionFlag`는 검증 시나리오와 rhwp의 알려진 값 목록에서
  제거하되, 한글의 실측 `Count=12`는 유지한다.
- Linux 집중 회귀와 계약·원장 검사를 통과시킨 뒤 Windows WASM을 다시 만들고, 같은 한컴
  `12, 0, 0, 4605`에서 82개 live gate를 재실행한다. 최종 로그에서 호출 불일치, 원본 변경,
  저장 질문, 잔류 한컴 process를 각각 확인한다.

## 7차 결과 (코드페이지·보기 상태 독립 Oracle 계약, 2026-08-10)

- 문서 글 조립을 공용 `text_file_content`로 모으고, `TEXT`의 CP949 수치 참조 경로와
  `UNICODE` 원문 경로를 별도 WASM 메서드로 노출했다. npm bridge는 format이
  `UNICODE`일 때만 새 메서드를 사용하며 기존 `TEXT` dispatch와 escape 규칙은 유지한다.
- 본문·편집 live 시나리오는 `UNICODE`로 전환했다. `p8-props`는
  `ViewZoomNormal`로 확대율을 100에 고정하고, 버전·사용자 상태에 종속된 `OptionFlag`를
  알려진 값과 비교 호출에서 제거했다. `ViewProperties.Count=12`와 안정적인
  `ZoomType=5`·`ZoomRatio=100` 계약은 유지했다.
- Linux에서 Rust 형식별 단위 테스트 1건, npm 공개 계약 8건, 하니스 mock 계약 24건,
  원장 `312/484`, 공식 82개 WASM self gate가 모두 통과했다.
  `wasm-pack build --target web --out-dir pkg`, `cargo fmt --all --check`,
  `cargo clippy --all-targets -- -D warnings`, `git diff --check`도 통과했다.
- Windows 검증 환경은 16 logical CPU·16GB RAM이며 한컴 COM은 `12, 0, 0, 4605`다.
  fresh WASM은 7,853,949 bytes, SHA-256
  `41972C136E10A701EB983BE7ECD0164CF3D8118A33D7DFC3DAAE0BFD6D4BE0A3`이고,
  생성된 JS·TypeScript binding에서 `getTextFileUnicode`를 확인했다.
- 첫 live full run은 82개 시나리오를 끝까지 실행해 3,519개 중 3,518개가 일치했다.
  유일한 차이는 `p4-setmutate`의 `ParameterArray.Copy`에서 한컴 COM이 한 차례 낸
  `RPC 호출 실패`였다. 같은 head·새 한컴 process의 단독 재실행은 20/20이 일치해 재현되지
  않았고, 갱신된 p4 결과와 첫 run의 나머지 81개 결과를 다시 비교해 최종
  **82개 시나리오·3,519/3,519 MATCH**를 확인했다.
- 최종 결과에서 `p2-textfile-*`, `p5-undo`, `p6-deleteline`,
  `p8-props`, `p8-settextfile`를 포함한 모든 호출과 저장본 L3가 일치했다. 실행 뒤
  `Hwp.exe`·`HwpFrame.exe`는 0개였고, 추적된 sample 변경이나 저장 질문 대화상자는 없었다.

## 경계와 rollback

- HwpCtrl API 구현·원장 완료 수·Oracle 반환 계약은 수정하지 않는다.
- 보정이 Windows Oracle과 다르면 scenario path만 contributor 경로로 되돌리지 않고, 반환값에
  경로 의존성이 있는지 probe로 확인해 portable fixture 규약을 다시 정한다.
- 보정 commit과 검증 결과는 같은 review branch에 고정한다. remote push·GitHub comment는
  작업지시자의 다음 승인 전에는 수행하지 않는다.
