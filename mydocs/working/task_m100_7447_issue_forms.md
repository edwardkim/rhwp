# #7447 이슈 폼과 자동 라벨링 운영 변경 기록

- Issue: https://github.com/edwardkim/rhwp/issues/7447
- 기준: `upstream/devel` / `443844b593c62a722cf9cc3d9d0256e94ab88cb8`
- 작업 브랜치: `codex/7447-issue-forms`
- 승인 범위: 4개 폼, 기존 라벨 자동 연결, OS 정보 수집. 원격 push·PR 생성은 별도 승인.
- 운영 분류: O3(새 Actions event·issues 쓰기 권한). 제품 소스 변경 없음.

## 구현 결정

1. 문서 표시·조판 오류, 동작 오류·설치 문제, 기능·개선 요청, 문서 오류·설명 보완을 YAML 폼으로 제공한다.
2. 사용 경로의 Studio/CLI/브라우저 확장/라이브러리·API/MCP만 기존 라벨에 자동 연결한다.
   OS·문서 형식·증상은 본문 정보이며 새 라벨을 만들지 않는다.
3. 라벨은 **추가만** 한다. 응답을 Studio에서 CLI로 바꾸면 기존 Studio 라벨을 보존하고 CLI를 추가한다.
   GitHub 라벨에는 자동화별 소유권이 없으므로 삭제 동기화를 하지 않는다. 잘못된 기존 분류는 메인터너가 정리한다.
4. 제거 이력이 있는 라벨은 자동으로 다시 붙이지 않는다. 메인터너가 수동으로 다시 붙일 수 있다.
   이벤트 재실행·관계없는 본문 수정도 수동 제거를 되돌리지 않는다.
5. 최신 이슈 본문·라벨·제거 이력을 API로 읽고, 허용한 폼 구조와 정확한 선택값만 처리한다.
   본문은 실행·로그 출력하지 않는다. workflow는 checkout 없이 SHA 고정 github-script로 실행한다.
6. 이슈 opened/본문 edited만 처리한다. PR·댓글·라벨 이벤트는 트리거하지 않는다.
   이슈별 직렬 실행과 최신 본문 재조회로 오래된 payload를 그대로 쓰지 않는다.

## 검증·적용 계획

- YAML 파싱, actionlint, 실제 workflow script의 API 모의 실행, 폼 선택지·기본 라벨 계약을 검사한다.
- 생성·수정·재실행·수동 라벨 보존·제거 이력·없는 저장소 라벨·Blank issue·미지 선택값·API 실패를 검사한다.
- 기존 CI Lint job에 계약 테스트를 연결하며 required check 이름·조건은 변경하지 않는다.
- 기본 브랜치는 조회 결과 `main`이다. `devel` 병합만으로 활성화되지 않는다.
- `main` 반영 뒤 실제 선택 화면·폼·opened/edited Actions 실행은 별도 적용 검증으로 남긴다.
- 되돌리기: 해당 변경을 revert하여 Markdown 템플릿을 복원하고 workflow를 제거한다.
  이미 부여한 라벨은 자동 삭제하지 않는다.

## 결과

- 구현 source SHA: `caa0f95f1e7a53c34641becfc65a1d6195c412f7`
- 작성일: 2026-09-27. 이슈 담당자는 `postmelee`로 할당 후 재조회했다.
- 4개 YAML 폼과 config, 추가 전용 자동 라벨 workflow, 실제 script 실행 테스트를 구현했다.
- CONTRIBUTING의 삭제되는 Markdown 템플릿 링크를 chooser 링크로 교체했다.
- 운영 매뉴얼 2.3에 선택값 대응·수정/제거 정책·재실행·활성화·복구를 기록했다.

| 검증 | 명령·관측값 | 판정 |
| --- | --- | --- |
| 실제 workflow script 실행 | `node --test scripts/tests/issue-form-labels.test.mjs` — 16 tests PASS, 4개 폼 × 5개 경로 조합 포함 | 충족 |
| CI 계약 연결 | `python3 -m unittest scripts/tests/test_workflow_contract_wiring.py` — 3 tests PASS. 새 Node 테스트도 자체 검사에서 CI 호출 확인 | 충족 |
| 새 workflow lint | `actionlint .github/workflows/issue-form-labels.yml` — exit 0 | 충족 |
| 기존 CI lint | `actionlint .github/workflows/ci.yml` — 기존 794행 SC2016 1건. 기준 SHA의 동일 파일도 같은 1건. `actionlint -shellcheck='' .github/workflows/ci.yml` — exit 0 | 신규 오류 없음, 기존 경고 잔존 |
| YAML 파싱 | Ruby `YAML.load_file`로 폼 4개·config·새 workflow·CI 총 7개 파싱 성공 | 충족 |
| 폼 기본 구조 | 파싱한 4개 폼의 고유 name/id/label, 입력 타입·required·문자열 선택지 검사. 버그 폼 각 11항목, 기능/문서 폼 각 5항목 | 충족, 실제 GitHub 렌더 검증과 구분 |
| 라벨 존재 | `gh api repos/edwardkim/rhwp/labels --paginate` — 기본 라벨 3개와 사용 경로 라벨 5개 모두 존재 | 충족 |
| required check | `gh api repos/edwardkim/rhwp/branches/devel` — protected, `Build & Test` 유지. 기존 CI에는 테스트 실행 1줄만 추가 | 충족 |
| 공백·패치 | `git diff --check`, `git diff --cached --check` — exit 0 | 충족 |
| 제품 빌드·렌더 검증 | Rust·Studio 제품 소스 변경 없음 | 비해당 |
| GitHub 실제 폼·Actions | 기본 브랜치에 반영되지 않음. 원격 push·PR·테스트 이슈 생성·main 반영 없음 | 미검증 |

YAML 파싱 재현 명령:

```sh
ruby -e 'require "yaml"; ARGV.each { |f| YAML.load_file(f); puts f }' \
  .github/ISSUE_TEMPLATE/*.yml .github/workflows/issue-form-labels.yml .github/workflows/ci.yml
```

테스트는 실제 workflow의 github-script 본문을 추출해 API 모의 객체로 실행한다.
생성, Studio→CLI 변경 시 이전/수동 라벨 보존, 재실행 무변경, 제거 이력의 영구 수동 관리,
100건 이후 제거 이력, 최신 본문 우선, 처리 중 상태 변경, 없는 라벨, API 실패,
Blank/기존 Markdown/중복 제목/미지 선택값/명령 형태 입력을 검사했다.
GitHub UI는 공개 서버의 기본 브랜치에서만 최종 확인할 수 있으므로 로컬 검사를 실제 이벤트
성공으로 보고하지 않는다. 원격 적용 후 이 이슈의 완료 조건을 다시 확인해야 한다.

### 남은 적용 검증

- 기본 브랜치 반영 후 chooser에서 폼 4개, Blank issue, Discussions, 기존 보안 신고 경로 확인.
- 각 폼의 필수/선택 입력과 실제 생성된 Markdown이 분류기에 맞는지 확인.
- 테스트 이슈의 opened와 본문 edited에서 실행 결과 및 추가 라벨 확인.
- 기존 라벨 보존, 제거한 라벨의 재부여 억제, 제목만 수정할 때 job skip 확인.
- 실제 이벤트 검증 전에는 #7447을 전체 완료로 닫지 않는다. PR은 `Refs #7447`로 연결한다.
