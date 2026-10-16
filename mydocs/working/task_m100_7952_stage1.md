# #7952 — 조회 CLI 통합 구현

Issue: [#7952](https://github.com/edwardkim/rhwp/issues/7952)

## 구현과 범위

기준은 `upstream/devel`의 `552514f423ec91f1b7171f6e6b0882a4efe034f2`다.
`rhwp-q-*` 23개를 `rhwp-q` 하나의 하위 명령으로 통합하고 Cargo 루트 실행 타깃을 26개에서
4개로 줄였다. 단건 20개, kit 50개, more/pack 각각 51개의 조회 경로를 유지한다.
`rhwp`, `rhwp-agent`, `font-metric-gen`은 별도 실행 파일로 남는다.

- dispatcher가 인자를 adapter에 직접 전달한다. 기존 실행 파일 재실행·전역 argv 교체는 없다.
- 조회 API·기존 JSON의 tool·종료 코드·원본 처리 의미를 유지한다. 도움말은 사용자 요청으로 확장했다.
- 단건과 kit의 사용법은 실제 parser의 USAGE를 공유한다. kit의 누락된 필수 주소·선택 옵션을 공개한다.
- 172개 개별 명령의 도움말에 명령·그룹·사용법·인자·옵션·예시·상세 설명·출력·종료 코드를 제공한다.
  파일을 열지 않고 도움말을 볼 수 있으며 검색값 `--q --help`는 문자열로 전달한다.
- more/pack의 이름과 다른 현재 출력은 실제 필드·필터 제한으로 설명한다. 이 PR은 해당 조회 기능을 새로 구현하지 않는다.
- 기존 22개 q 계약 검사와 8개 Node/Python 호출 스크립트를 새 실행 경로로 이관했다.
  기능·검사를 feature로 비활성화하지 않았고 renderer golden·px·쪽수 기대값을 바꾸지 않았다.
- dev profile은 `debug=1`이며 release/release-test 최적화 설정은 유지한다.
  release 사용을 기본으로 안내하고 debug·Windows·JSON 자동화도 매뉴얼에 기록했다.

## 변경 전 관측과 선행 진단

최종 commit의 정식 검증과 구분한 구현 중 진단이다. 최종 결과는 다음 단계 기록에 남긴다.

| 항목 | 관측 |
| --- | --- |
| 기존 캐시의 q 실행 파일 | 23개, 논리 크기 합계 7,798,918,392 bytes. 새 baseline 빌드 결과가 아님 |
| 통합 dev 최초 빌드 | 3분 13초, exit 0. 상세 도움말을 추가하기 전 중간 소스 |
| 같은 중간 소스의 CLI focused | 123/123 PASS. 최종 도움말 변경의 검증으로 재사용하지 않음 |
| 상세 도움말 변경 전 | page-images는 필수 설명 절 8개, kit cursor-in-hf는 9개 누락. more para-empty / pack forms-all --help는 exit 2 |
| 루트 도움말의 닫힌 파이프 | 초기 구현 exit 101 재현. stdout 오류 처리로 보정했으며 최종 실행을 재확인 예정 |
| 호출 스크립트 | Node 18개, Python 44개 PASS |
| suite 정책 도구 | Node 23개 PASS, 기존 Rust unit tier 검사 PASS |

기존 바이너리 논리 합계와 새 바이너리 크기에는 통합 및 dev debug 정보 축소 효과가 함께 들어간다.
두 효과를 분리한 측정이나 실제 디스크 확보량·전체 빌드 시간 단축으로 보고하지 않는다.
공용 캐시와 과거 실행 파일은 삭제하지 않았다.

## 정식 검증 순서

사용자의 “pr 준비 완료되면 pr”에 따라 구현 commit을 먼저 만든 뒤 **별도 review worktree**에서
그 commit의 suite 준비·focused 검사·Rust lint 묶음·release-test 전체를 수행한다.
`CARGO_BUILD_JOBS=8`, 기본 작업공간의 `/home/tsjang/rhwp/target/pr-review`를 공유하고 Cargo는
순차 실행한다. 정식 검증이 끝나기 전에는 push·PR을 하지 않는다.

source checkout에서 수행한 앞선 진단은 별도 worktree 검증을 대체하지 않는다. user dirty인
`rhwp-studio/public/rhwp.js`는 source에 그대로 보존하고 commit에 넣지 않는다. clean한 검증
worktree에는 그 변경이 들어가지 않는다.

조판 원칙·Visual Sweep: **비해당**. 변경은 실행 타깃/명령 dispatch/도움말/호출 연결이며
DocumentCore·저장 모델·font 공급·renderer·paint·WASM 구현을 바꾸지 않는다. 새 렌더링 회귀와
fixture도 추가하지 않는다. 기존 CLI 검사 입력은 커밋된 `samples/form-01.hwp` 등을 재사용한다.

기존 1000줄 초과 파일은 이동 내용 또는 호출 인자 한 줄만 유지한 범위로 한정했다.
새 dispatcher·도움말·catalog·검사 파일은 각각 1000줄 이하다.

로그: ignored `output/pr-review/cli-build-scope-20261016/`.
PR review/오늘할일은 실제 code head의 CI 성공 뒤 작성하며, 이번 지시는 PR 생성 승인이다.
