# Kang-T 3건 메인테이너 검증 증거

대상은 #8566(#8555), #8595(#8556), #8632(#8557)이다. 원 기여 SHA와 입력/PDF는 [입력 원장](input-provenance.json)에 고정했다. 원본 HWP 3개와 Mac 한컴 Print PDF 3개는 원 기여 commit을 재작성하지 않고 보존했다. 원 기여자의 과거 측정과 아래 메인테이너 실제 실행은 서로 다른 실행이다.

## 실행 출처

최초 기준은 `45684640be5eddcf5972b063260250345b34d928`, 경계 보정의 Rust source는 `5b1d43f3cf321478a2a8dfb1b26be48acfc579d6`다. 직렬화 검사 보정 뒤 `ad64d93156ee8ab9d74cc853932c9dda86ea300d`에서 최적화 WASM을 완료했다. Native source digest와 WASM source digest는 같고, 이후 public binding 동기화 `fd6f6d3a38`은 제품 Rust를 바꾸지 않았다.

검토 중 원격 devel에 #8696(`a7feb849e11642b1438b7e9ee460ff4c8a7c946e`)의 글꼴 선택 수정이 들어왔다. `before-devel-8696/`는 **그 통합 전 실제 측정**이다. 이 자료를 새 head에서 재출력한 것으로 표시하지 않는다. 최신 통합의 Rust 빌드 source는 `226f0d487cc8d72305979ceb8ab46832017634d0`, 뒤이은 Studio 포인터 보정은 `63236b9cc5`다. 최신 기준과 후보를 별도 worktree에서 같은 글꼴·입력·profile로 빌드하고 전쪽을 대조했다. [최종 검증 집계](latest/validation-summary.json)와 [실제 base 빌드·실행](latest/latest-base-results.json)을 보존한다. 이후 검토 문서와 생성 선언 동기화는 검증한 제품 코드를 바꾸지 않는다.

Native는 공유 절대 target의 `release-test --features native-skia`로 빌드했다. WASM은 `scripts/wasm-pack-locked.sh --target web --out-dir pkg`, release·CGU 16·정식 wasm-opt를 사용했다. 성공 exit 0과 실제 생성물 해시를 보존한다. 오케스트레이터가 143으로 끝난 첫 최종 WASM 시도는 최적화 로그가 있어도 성공으로 세지 않았고, 정상 종료한 재실행만 사용했다. 중간 `--no-opt` 산출물은 검증에 사용하지 않았다.

## 전쪽 시각 비교와 글꼴

96 DPI·Print·compat 2022·raw 2px 실루엣이다. 원 입력의 전 4/5/4쪽을 모두 비교했고 마스크·경계 보정 점수·평균값으로 통과를 대신하지 않는다.

| 문서 | 기본 글꼴 후보 Native / WASM 최저 | Print 대체 글꼴 명시 후보 Native / WASM 최저 |
| --- | --- | --- |
| #8555 | 96.17274% / 98.69132% | 99.13472% / 99.09683% |
| #8556 | 96.53824% / 98.78853% | 99.31803% / 99.26090% |
| #8557 | 96.24179% / 98.68436% | 99.23183% / 99.19892% |

정확한 각 쪽 수치는 [최신 원본 TSV](latest/tsv/)와 [최신 base/후보 전쪽 비교](latest/latest-base-candidate-primary-comparison.json)에 있다. #8556의 기준 devel은 중간 쪽의 행이 어긋나 Print 글꼴 명시에서 Native 66.84323%·WASM 66.45711%였으며 후보는 각각 99.31803%·99.26090%로 개선됐다. 후보 90% 미만은 0쪽이다. #8555·#8557 저장본의 정상 출력은 유지됐다. #8555는 실제 글자 삽입→삭제→HWP 저장으로 재조판을 거친 같은 내용의 전 4쪽도 비교했다. 렌더러의 기준선 제한 때문에 화면이 같아지는 것과 저장 LINE_SEG가 바뀌는 것을 구분한다. 실제 셀 5의 글자 삽입→삭제 뒤 저장 기준선은 원본 600/1200 HWPUNIT에 대해 최신 devel 1020, 후보 600이며, 4,442자의 원문은 양쪽 모두 보존됐다. [저장 계약](latest/center-edit-contract.json)·[본문 보존](latest/center-edit-text-preservation.json)과 정렬 0/1/2/3의 별도 계약 검사를 연결한다.

독립 Mac Print 세 파일의 `pdffonts`는 HCRBatang만 사용한다. 기본 fallback의 고딕/명조 차이를 확인한 뒤 로컬 `hamchob-r.ttf`(HCRBatang, SHA-256 `ba2ac8f2b1b9f45c310637565ac46c2a39982241f9c78d4a2668678a4f88ee85`)와 `hamchod-r.ttf`(HCRDotum, `ddbf97ce7e4c19accd208354369d77d89947bf48446a61ed1cbb28a23c1c0f43`)를 명시했다. 누락된 원본 글꼴 이름은 관측된 Print 대체 HCRBatang으로 연결한 [검증 전용 환경](print-reference-font-environment.json)을 사용한다. 원본 글꼴을 확보했다고 주장하거나 제품 fallback 정책을 바꾸지 않는다. 글꼴 프로그램은 커밋·재배포하지 않는다. 기본 환경에서도 전쪽 90% 이상이므로 글꼴 예외는 사용하지 않았다.

확대 PNG는 같은 쪽·같은 영역·같은 배율의 Print, devel Native, 후보 Native, 후보 WASM 네 패널이다. #8555의 제목·세로 정렬·첫 표 괘선, #8556의 그림 옆 글줄 및 2쪽의 34→35번 행 이어받기, #8557의 3쪽 그림과 9/10번 글줄·셀 외곽을 직접 판독했다. 내용 누락·중복·겹침 악화는 없었고 #8556의 행 이어받기가 Print와 맞게 개선됐다. 해당 영역에는 수식·첨자가 없다. 기본 PNG와 별도로 실제 SVG 출력의 전 13쪽도 검사했다.

#8658의 정상 대조군 8개·전 10쪽(시작/이어받기/종료·선언 높이 경계·뒤 본문 포함)은 Native와 WASM 모두 독립 Print 대비 100%이며, 각 backend의 base/후보 PNG 10쌍이 각각 byte-identical이다. [최신 정상 대조군](latest/latest-control-preservation.json)은 단순 helper 결과가 아닌 실제 PNG 20쌍의 해시 대조다. 새 devel #8696의 원 #8687 입력 3종도 같은 Windows 글꼴 프로그램을 명시해 양쪽에서 fresh 빌드·재출력했으며, Native/WASM 총 6쌍의 PNG가 같다([보존 대조](latest/latest-issue8687-preservation.json)). 독립 Print 대비 최저는 91.09617%다. 편람 전 383쪽은 [canonical Native gate](latest/handbook-gate.json)에서 후퇴 0쪽으로 통과했다.

## 실제 경계 보정과 음성 대조

1. 마지막 셀 문단 999 요청이 오류 뒤 BinData를 추가하던 실패 원자성: 공통 삽입에서 실제 문단을 먼저 검증한다.
2. x만 또는 y만 전달하면 두 값 모두 0으로 되던 선택 인수: 각각 독립적인 기본값 0을 사용한다.
3. 지원하지 않는 그림 캡션을 도착점으로 지정하면 원 그림을 지우던 재배치: 제거 전에 컨테이너 종류까지 확인한다.
4. 같은 바깥 문단에서 원 그림을 제거해 표 번호가 1→0으로 바뀌는데 Studio가 옛 경로를 쓰던 문제: Native의 실제 도착 `cellPath`를 반환하고 속성·선택이 함께 소비한다.
5. 칸 안 미리보기의 위치 제한을 받은 좌표로 도착점을 추측해 칸→본문 그림이 포인터에서 멀어지던 문제: 마우스를 처음 잡은 점을 기록하고 도착 쪽의 포인터 좌표에서 빼서 계산한다. 새 컨테이너의 원점에서 실제 배치를 구한 뒤 같은 경로로 오프셋을 정한다.

보정 전 optimized WASM에서 위 실패를 실제 공개 API로 관측했고, 보정 뒤 같은 요청에서 저장 해시·인수·도착 경로를 확인했다. Native 관계형 검사와 Studio의 실제 종료 함수 검사도 추가했다. Studio의 음성 대조는 Vite에서 옛 hit 경로를 쓰도록 실제 제품 함수를 변형하고 assertion 실패를 확인한다. 단순 소스 문자열 검사로 기능 검증을 대신하지 않는다.

## CDP 19222와 편집 이력

Windows Chrome 155의 테스트 전용 context/tab을 사용하고 기존 사용자 탭·문서를 건드리지 않았다. 초기화 시 실제 받은 WASM의 SHA-256을 frozen producer와 대조했다. 직접 API 호출과 실제 mouse down/move/up을 구분했다.

- 최신 devel `a7feb849e1`의 별도 optimized WASM에서도 같은 실제 mouse drag를 실행해 4쪽 그림이 2쪽으로 이동하지 않고 원 앵커에 남는 것을 재현했다([음성 대조](latest/cdp/base-latest-negative.json)). 추가 실행의 처음 시도는 화면 밖 테스트 창의 hidden 상태에서 RAF 대기로 멈춰 성공으로 세지 않았다. 테스트 전용 창만 화면 안으로 활성화하고 진단 harness의 대기를 timer로 바꾼 최종 실행을 사용했다. 제품 코드·mouse 처리·WASM은 바꾸지 않았다.
- 실제 앞/뒤 페이지 이동, 같은 쪽 이동, 칸→본문 이동, 도착 경로와 선택 일치, keyboard Undo/Redo, HWP 저장 재열기를 실행했다.
- 같은 쪽 Undo/Redo는 모델 속성·실제 배치·소유 관계를 대조한다. 재조판/직렬화 메타데이터까지 포함한 HWP 전체 바이트 동일성은 그 계약으로 사용하지 않는다.
- 칸 안 flow 그림의 기존 위치 제한·본문 회피·재조판은 유지한다. 초기 owner/history 검사는 통과했으나, 화면 재판독에서 칸→본문의 실제 그림 y=961.3과 놓은 위치 y≈110의 차이를 확인했다. 이 상태를 완성된 이동으로 판정하지 않는다. 처음 잡은 점을 기준으로 계산하는 보정을 추가했고 단위 검사 3건(옛 owner와 잘못된 preview 좌표의 음성 대조 포함)이 통과했다. 최종 fresh WASM에서 같은 동작을 재실행해 본문 도착 y=76.4와 포인터 y≈108의 실제 잡은 점을 보존했다. 칸 도착은 원 문단 제거 후 재조판으로 칸 자체가 이동할 수 있어 실제 render tree의 칸 내부 포함과 동일 owner를 검사하며, 본문 도착은 실제 포인터가 그림 안에 있는지도 검사한다. 앞/뒤 이동·같은 쪽·Undo/Redo·저장 재열기가 모두 통과했다. containment를 제거하거나 임의 좌표로 clamp해 수치를 맞추지 않았다.
- `insertPictureInCell`은 실제 snapshot 명령과 사전 가열한 레이아웃 캐시를 거쳐 소속·Para/Column·본문 영역 제한·Undo/Redo·저장 재열기를 확인했다. 기존 삽입 메뉴가 이 신규 API로 바뀌었다고 주장하지 않는다.

추가 실행한 `textbox-picture-insert-1171.test.mjs`는 base와 후보 모두 본문 sibling 증가 조건 2개가 실패한다. 이 기대는 #1322의 메인테이너 보정 `a29987793316d43a414f2f66147f401e00ace834` 이전 계약이며, [독립 선행 검토](../../archives/pr_1322_review.md#7-메인테이너-추가-확인-및-보강)는 글상자 내부 삽입을 결정했다. 이 오래된 검사를 통과 수에 넣거나 기대값을 이번에 바꾸지 않았다. `textbox-picture-ops-1273.test.mjs`의 resize/undo/rotate/offset은 base와 후보가 같고 양쪽에 동일한 커서 rect 경고 1개가 있어 전체 PASS로 세지 않는다. 이번 변경의 새 실패로 표시하지 않으며 기존 오류 경계를 숨기지 않는다. 기존 #6202 실제 화면 재흐름과 #2303 Undo 선택 해제는 별도로 성공했다.

## 최종 제출 상태

세 PR 모두 **메인테이너 보정 후 수용 가능**이다. 전체 회귀 12,778건, Native 공식 4,064건, 집중 회귀 34건, Studio 2,745건, fmt·Native/WASM/workspace Clippy·workspace build·suite 수집 검사·타입/build·관련 E2E·전쪽 Native/fresh WASM·SVG·편람 gate를 완료했다. 원 기여 SHA 3개와 새 devel ancestry를 보존한다.

편람의 초기 실행은 최소 글꼴 sandbox에서 Python 모듈/DNS 경로를 숨겨 실패했으며, 실행 환경만 보완하고 동일한 5개 공개 글꼴·동일한 gate로 전 383쪽을 통과했다. 최초 forward CDP의 포인터 포함 assertion은 원 문단 제거로 이동한 칸에도 본문 불변식을 적용한 것이어서 실제 칸 내부 포함·동일 소유를 검사하도록 진단을 바로잡았다. 이 최초 실패 기록은 지우지 않고 최종 재실행과 구분했다.

최종 로컬 결과는 [집계](latest/validation-summary.json)에 고정했다. 이후 작업지시자 승인으로 통합 PR #8725를 생성하고 정확한 head CI·self-review 뒤 merge했다. [실제 원격 수용 기록](remote/completion.md)에 원 3개 PR의 MERGED, 이슈 종료와 후속 처리 결과를 구분한다.
