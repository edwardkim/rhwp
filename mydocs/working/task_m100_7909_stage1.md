# Issue #7909: 표시 문자열 export 계약

기준: 최신 upstream/devel `def947c2149df5bc812d801e6e79be19a403b4d1`.
브랜치: `codex/issue-7909-display-text`. 다른 후보 PR의 미병합 코드는 사용하지 않는다.

## 분석과 범위

이슈 #7909와 댓글(0개), #7472 및 댓글 3개를 확인했다. #7909 검색 결과 기존 PR은 없었다.
선행 #3323의 모델/표시 분리와 #7053의 autoNum 치환 계약을 확인했다.
`get_page_text_layout_native`는 모델 text만 export한다. renderer가 확정한
`TextRunNode.display_text`를 export하지 않아 SVG에 그려지는 번호를 소비자가 받지 못한다.

표시의 생산은 기존 autoNum 치환 → TextRunNode.display_text,
그리기 소비는 display_or_text/replay_positions_for이며 수정은 읽기 전용 JSON export에 한정한다.
기존 text/charX/charStart 및 셀 좌표를 보존하고 optional displayText/displayCharX를 추가한다.
표시 좌표는 런 원점에 상대적이고 Unicode scalar N개에 N+1개 경계를 사용한다.
표시 문자열의 인덱스를 선택·커서·편집 API에 전달하면 안 된다.
#7910의 그림·사각형·꼬리말 배치, parser/model/serializer/paint는 범위 밖이다.
조판·렌더링을 바꾸지 않는 API export 계약 검사이며 시각 일치 개선은 주장하지 않는다.

## 검증 계획과 진행

- 공개 #7472 원본과 no-picture ZIP: SHA-256 앞자리 1c4415a9b550fc86 확인 후 사용.
- generated display와 모델 오프셋 분리, 일반 문구/자동번호 없는 문단, 형식별 검증.
- native와 fresh WASM 공개 API, 기존 선택·커서·편집 계약, 수정 전 FAIL/후 PASS.
- Rust lint 전체 묶음, 전체 release-test 회귀, git diff --check.
- 필수 자료가 없거나 게이트 실패 시 push/PR을 보류한다.

현재 공개 첨부의 objects.githubusercontent.com 리다이렉트는 클라우드 정책상 403이다.
허용 호스트 추가를 요청했으며 자료 접근 전에는 원본 검증을 완료로 기록하지 않는다.
CLI `gh api user`는 401 Bad credentials. connector 인증은 postmelee이며 열린 본인 PR은 0개다.

## 확보한 증거

공개 ZIP은 사용자 첨부로 확보했으며 SHA-256 전체는
`1c4415a9b550fc865d79a832424510e3d656d21ccbafc0856b1961c02ab25918`이다.
원본 두 HWP의 해시·출처는 `tests/fixtures/issue7909/README.md`에 보존했다.
#7909의 assignee를 postmelee로 지정했다. 연결된 GitHub 인증 계정의 기존 열린 PR은 없었다.
`getPageTextLayout` 관련 열린 PR 검색도 0건이다.

수정 전 최신 devel의 native JSON: 원본 69런, no-picture 70런이며 displayText는 둘 다 0개다.
수정 전 정식 회귀(`--no-fail-fast`): **6건 실행, 2 PASS / 4 FAIL**.
실패는 generated display 분리, HWPX/HWP 자동번호, 공개 원본, 공개 no-picture의 표시 문자열 누락이다.
일반 문구 및 선택·커서·삽입 오프셋 대조 2건은 PASS다. suite의 다른 검사 253건은 focused filter에서
제외된 것이며 실행·통과로 세지 않았다. 초기 fail-fast 실행과 준비가 오래된 suite 배정으로 0건이 된
실행은 정식 결과에서 제외하고 최신 `--prepare` 뒤 6건 전체를 다시 실행했다.

테스트 배정 계약은 sandbox 밖에서 23/23 PASS, manifest base 비교는 PASS다.
새 문서 링크·메타데이터 검사에서 오류가 없다. 저장소 전체 메타데이터 검사는 기존 문서의
누락 20건으로 실패했으며 전체 성공으로 기록하지 않는다. 해당 파일은 이번 변경에 포함하지 않았다.
최종 head 검증은 아래 결과가 확정된 뒤 별도로 기록한다.


## 대조본 기대값 재검토와 focused 결과

초기 첨부 no-picture 검사 1건의 기대값은 잘못됐다. parser 결과 controls=0이며,
원시 `dump-records`에도 해당 꼬리말 atno 헤더가 없다. 원본의 `6f 6e 74 61` 레코드와
대조했다. 따라서 초기 수정 전 4 FAIL 중 그 1건을 export 결함의 검출 증거로 세지 않는다.
원본 대조본은 그대로 보존하고, 번호가 없는 정상 대조 검사로 바꿨다.
자동번호를 유지한 별도 최소 변형을 만들었으며 생성 절차·해시는 fixture README에 기록했다.
이는 기대값을 낮춘 것이 아니라 원본 레코드에 맞춰 부적합한 기대값을 교정하고,
의도한 양성 입력을 별도로 보완한 것이다. 생산 코드의 조판·parser는 변경하지 않았다.

현재 focused 결과는 **7건 실행, 7 PASS / 0 FAIL**이다. 원본 및 rhwp 생성 HWPX,
자동번호 보존 no-picture HWPX/HWP, 원본 첨부 음성 대조, 일반 문구, 모델/표시 분리,
selection/cursor/edit 오프셋을 검사했다. 기존 API 필드는 69개/70개 전체 런에서
수정 전 JSON과 같았다(새 displayText/displayCharX 두 키만 제외한 전체 JSON 비교).

이 단계는 구현·focused 검증까지다. 최종 7개 검사 소스로 수정 전 export를 다시 실행하는
증거와, 별도 review worktree의 최종 lint·전체 회귀·fresh WASM은 다음 단계에서 확정한다.
필수 게이트 완료 전에는 push/PR을 하지 않는다.
