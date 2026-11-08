# 이슈 #7642 — 미주 보정의 전쪽 Native·fresh WASM 검증

생산 소스 커밋은 `94ebd9c0a43c66b48a6ba1f94327e82b86731f65`다.
현재 작업의 회귀 실패는 모두 현재 브랜치에서 보정한다.

## 고정 근거와 순서

1. Stage36 frozen Native `rhwp-endnote-owned-span`의 24개 소스 해시와
   전체 Rust diff 해시를 생산 커밋과 대조해 일치함을 확인했다.
   바이너리 SHA-256은 `e89ec2b139a33921b0617b0d57c4cd43b0d683c2ca5217f8f63eb13ab7926a1f`다.
   진행 중인 `endnote-owned-span-native-all`은 2022 HWPX와 독립 2024 Print의
   전체 21쪽 비교다. source가 같은 문서 계획 커밋을 새로운 생산 소스로 혼동하지 않는다.
2. 공용 `/home/tsjang/rhwp/target/pr-review`에서 root WASM wrapper를 실행한다.
   로그·빌드 출처·package 해시를 ignored output에 고정하고 Studio public과
   root pkg의 JS/WASM SHA-256 일치를 확인한다. Native Clippy는 끝났으므로
   Cargo 동시 실행 없이 진행한다.
3. 이 fresh package로 같은 HWPX·2024 Print의 전쪽 TSV를 산출한다.
   HWP/2020 정상 대조도 전쪽으로 확인한다. 영향 쪽의 compare·overlay·review와
   수식 확대 이미지로 단 소속·누락·중복·간격·기호 모양을 직접 판독한다.
4. 간격이 없는 HWPX 15쪽 75.79976%, 수식 재현 입력의 미달과 확대 판독에서
   남은 수식 중괄호·첨자·기호 주변 차이는 현재 브랜치에서 계속 수정한다.
   부분 페이지의 점수 통과를 전체 완료로 바꾸지 않는다.
5. 요청된 52개 원본(독립 Print 총 783쪽)과 관련 실패 입력을 같은 조판 세대에서
   검증한다. 출력 보정·전체 범위 Native/fresh WASM 90% 이상·확대 판독을
   먼저 충족한 뒤 필요한 회귀 검사를 고정 px에서 소유·순서·보존·포함·겹침
   관계로 변경한다. 기존 110건 실패 목록은 새 전체 실행으로 갱신한다.

## 진행 기록

명령·결과·TSV·JSON·nextrun은 ignored
`output/pr-review/issue7642/failed-inputs-20261008/stage15-current-validation/`에만
보존하며 Git에 stage하지 않는다. 기본 검토 target을 삭제하거나 새 target을 만들지 않는다.
현재 전체 Rust·WASM lint/정책·전체 회귀와 PR/CI/merge는 미완료다.
최신 code head CI 이후에 PR review·오늘할일 trailing 기록을 커밋하는 기존 순서를 유지한다.

## Native 전쪽 결과

`endnote-owned-span-native-all`은 exit 1, 1044.42초로 끝났다. 전체 21/21쪽의
측정은 완료됐고 누락 쪽은 없다. 변경 전 미달 6쪽 중 13·14·17·18쪽이 90% 이상으로
개선됐다. 19쪽 62.18357%, 20쪽 61.77429%가 남아 최저값은 61.77429%이고
PR gate는 `re_review_required`다. 전체 통과로 보고하지 않는다.

두 미달 쪽의 새 review PNG를 현재 전수 실행의 실제 PNG에서 생성해 직접 열고
표시했다. 19쪽은 문26 제목과 첫 본문이 왼쪽 단 하단에 들어가야 하는데 오른쪽 단
위로 통째로 이동해 뒤 그림·문항이 늦어진다. 이 차이가 20쪽 왼쪽 단의 시작까지
이어진다. 평균 점수나 글꼴 차이로 면제하지 않고 해당 문항 경계의 실제 fit·이월
호출 경로를 현재 브랜치에서 계속 추적한다. 자료는 ignored
`representative-native/nov2022-owned-span-p019/`와 `nov2022-owned-span-p020/`에 있다.

fresh WASM 빌드는 exit 0, 1236.40초로 종료됐다. 검증 소스는 동일한 `94ebd9c0a`이며,
고정한 24개 소스 해시가 빌드 전후 일치한다. frozen package의 JS SHA-256은
`a043bdc9047d3ff5d89f55ccc21846f017ac109dad6de25b2885e43c44ee08da`, WASM은
`178006490906ab412c905fa54b15fd9dc823c5e091bff9f5162c71ad79396a02`다.
루트 pkg와 Studio public도 같은 해시다. 이 package의 HWPX/2024 전쪽 대조와
동일 frozen Native의 HWP/2020 전쪽 대조를 시작했다. 실행 중 결과를 통과로 보고하지
않으며, 다음 소스 변경 뒤에는 해당 소스의 Native/fresh WASM을 다시 검증한다.

19쪽의 실제 호출 로그는 `prepare.rs`의 다음 미주 전체 선행 이월 분기로 연결된다.
원본 본문 문단 379의 미주 제목·첫 본문은 `keepWithNext=0`, `keepLines=0`이며
강제 쪽·단 나눔도 없다. 앞 미주의 저장 좌표 되감기만으로 다음 미주를 분할 불가
묶음으로 취급한 가정을 다음 단계에서 제거한다. 기준 Print는 제목과 첫 본문을
왼쪽 단 하단에 남기고 다음 문단을 오른쪽 단으로 넘긴다.

사용자가 제시한 목록은 검사 제거 내역이며 관계형 전환 완료 목록이 아니다.
현재 `tests/issue_1139_inline_picture_duplicate.rs`에는 폭 23.8·188.98과
하단 1066.86 등 고정 px 선택·기대값이 남아 있다. 전체 전환은 미완료다.
현재 작업에서는 실패 원본을 보정하고 90% 이상 및 확대 판독을 확인한 뒤 필요한
검사를 관계형으로 바꾼다. 검사 삭제·허용치 완화로 실패를 해소하지 않는다.
