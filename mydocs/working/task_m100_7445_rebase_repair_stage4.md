# Issue #7445 — 로드맵 마지막 저장 프레임 보정

## 직전 커밋 검증과 범위 구분

base는 `86a3c88fc89de9a2306a10bdda9a31023f0e24a5`, 직전 커밋은
`fadc06e828abbc062708f62e2e75e0c59b9c1659`입니다. `final-v17`의 전체 nextest는
**10290 PASS / 2 FAIL / 45 skip**, 541.045초입니다. 실패는 #4138의 저장/재열기
197쪽 핀 두 건이며, 그 외 선행 13건 실패는 구현 보정 후 통과했습니다. 핀 이관에 대한
GitHub 신규 등록 승인을 요청했고, 답변 전에는 검사·기대값을 유지합니다.

fmt, Native·WASM·전체 target Clippy, workspace build, Skia lib(root3859 PASS /12ignore,
추가 lib182 PASS), PNG2·PDF4, manifest·unit-tier는 통과했습니다. Studio1835 PASS /2skip,
Gym3170 실행 실패0 /1skip, 글꼴 계약11 PASS /1skip입니다. Mac fresh WASM은
`--no-opt` 로컬 대체 빌드이며 Docker 최적화 통과가 아닙니다.

Native **50조건1576쪽**을 끝까지 산출했습니다. 기존 완료43조건에서는 로드맵295쪽만
87.68818%로 미달했습니다. 리베이스 전 같은 쪽은98.24314%였습니다. 나머지 추가7조건은
같은 완료 범위로 간주하지 않습니다. KPS77쪽, 가맹사업142쪽, 가스31쪽, 각주242쪽과
규제분석56345에 미달이 있어, 최신 고정base에서의 정확한 등록 자격을 전쪽 비교합니다.
56345는 동일 범위가 이미 열린 #7806에 등록되어 있습니다. 별도의 현재 앵커 회귀는 고쳤고
정상 관계 검사는 유지했습니다. 기존 미달의 이관을 현재 회귀 수정 대신 사용하지 않습니다.

로드맵 실제 회귀의 보정을 위해 WASM은 **3조건192쪽**까지만 산출한 뒤 중단했습니다.
실제 소비36조건은 아직 실행하지 않았습니다. 그 결과를 전체 성공으로 재인용하지 않습니다.

## 원인과 실제 소비 경로

구역6·문단1304의 마지막 행33은 같은 문단 안의 저장 원점0에서 새 쪽을 시작합니다.
원본 마지막 줄은 `vpos36960 + LH1100`, 위·아래 안 여백은 각각141 HWPUNIT이며,
독립 Print295쪽의 물리 조각은 이 종료 줄과 여백을 포함합니다. 선언 cellSz282는
여백만 소유하는 작은 최소이며 전체 문단 전진량을 물리 상자로 선언하지 않습니다.

`stored_first_frame_row_remainder`는 첫 상자의 닫힘만 인증한 뒤 `measured_rows[row] - opening`을
뒤 상자의 최소로 넘겼습니다. 측정 행 합계의 마지막 줄 전진은 실제 쪽 프레임 공간과 다릅니다.
그 값이 `emit::next_start_row_height_override` → `RowScanQuery::whole_row_height` →
최종 partial Table/Cell 높이에 전달되어 마지막 상자가511.2267px에서516.2667px로 커지고,
Center 본문 전체가2.52px 내려갔습니다. 글줄 간격·순서·쪽수는 같아도 새 시각 회귀입니다.

보정은 **독립된 셀 선언 최소가 실제로 남은 경우에만** 빈 물리 공간을 전달하도록 합니다.
첫 저장 상자의 인증과 전체 측정 내용의 합계를 분리합니다. 내용 컷의 요구 높이는 기존 공통
query가 측정·scan·paint에서 계속 소비하며 현재 좌표·문서 ID·특정 행 모양으로 분기하지 않습니다.
진짜 선언 빈 밴드와 다른 저장 컷 계약은 그대로 대조합니다.

## 독립 관계 검사와 수정 전후

`closing_note_fragment_preserves_its_stored_frame_and_first_line_origin`을 기존 로드맵 회귀에
추가했습니다. 원본 LineSeg의 재시작/마지막 잉크 끝과 실제 안 여백으로 physical frame을
확인하고, 최종 첫 글줄의 원점과 출처 보존을 검사합니다. UI 좌표의 고정 핀을 추가하지 않습니다.

- 보정 전 실제 실행: **2 PASS /1 FAIL**, 8.472초. 새 관계 검사가
  원본511.2267 대신516.2667을 소유한 상태를 검출했습니다. 기존 국가 행 소속2건은 통과했습니다.
- 보정 후 같은3건: **3 PASS /0 FAIL**, 8.922초.
- 관련 대조군 두 묶음: **43 PASS /0 FAIL**, 14.087초와 **8 PASS /0 FAIL**, 10.842초.
  기존 안 여백/빈 밴드·컷·본문 소유 계약을 유지했습니다.
- Native294·295·296쪽 선택 비교: 최저**98.24314%**, 전체302쪽 유지.
  295쪽 review를 직접 열어 상자 바닥·본문 원점·마지막 출처 위치의 복구를 확인했습니다.

[보정 전 review](../pr/assets/issue7445/completed-20261009/rebase-7882-20261014/roadmap295-before-native-review.png)
와 [보정 후 review](../pr/assets/issue7445/completed-20261009/rebase-7882-20261014/roadmap295-after-native-review.png)를
함께 보존합니다. 이 선택 비교는 새 최종 커밋의 전302쪽·fresh WASM·전체 회귀를 대신하지 않습니다.

검사 수 변경 뒤 generated 배정이 달라진 초기0test 실행과 합성 검사 타입 변환의 컴파일 오류는
회귀 결과가 아닙니다. `--prepare`와 타입 정정을 거친 실제 FAIL→PASS만 위 증거로 사용합니다.
처음 대조 묶음에서 빠진 target은 현재 manifest로 다시 계산해 누락한3target만 후속 실행했습니다.

## 추가 대상 검증과 공간 관리

고정base KPS 전77쪽은 최저**78.50828% /미달19쪽**이며, 현재 후보는 같은 조건에서
최저87.41922% /미달3쪽입니다. 전체 문서의 완료를 주장하지 않습니다. 가맹사업·가스·각주의
base 전쪽 검증을 계속하여 기존 미달과 현재 새 회귀를 분리합니다. 현재 선택 페이지의 미달은
위 로드맵 보정만으로 해소되지 않았습니다. upstream의 과거93% 주장도 최신base 결과를 대용하지 않습니다.

가맹사업의 전체 SVG export는 디스크 부족으로 실패했습니다. 실패 출력은 검증 증거가 아닙니다.
중단된 자신의 WASM 작업의 orphan3개를 stdout/cwd/이전 소유PID로 확인해 종료하고,
정확한 output 하위 SVG·font policy·raw SVG만 SHA manifest를 남겨 정리해29.56GB를 확보했습니다.
원본·PDF·PNG·TSV·로그·다른 작업장·공유 target은 보존했습니다. 이후 같은8개 캡처 동시성과
같은 비교 공식을 유지하며 페이지 묶음별로 산출·중간 SVG를 정리하고 전쪽 TSV를 합칩니다.

실행 정본은 ignored `output/pr-review/7445-completed-20261009/rebase-7882-20261014/`의
`final-v17/`, `roadmap295-before-repair-compiled.log`, `roadmap295-after-repair.log`,
`v18-related-focused.log`, `v18-related-remaining-focused.log`, `v18-focused-native/`,
`v18-fixed-base-extra-qualification/` 및 cleanup manifest입니다. HYHWPEQ 공급 질문도 미응답입니다.
지침 변경·GitHub 게시·push·PR·merge는 없으며 두 사용자 초안은 이번 커밋에서 제외합니다.
