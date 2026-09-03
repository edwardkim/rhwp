# Task M100 #6669 — Gym JSON 증적 시각화 최종 보고서

- **이슈**: [#6669](https://github.com/edwardkim/rhwp/issues/6669)
- **부모 이슈**: [#6628](https://github.com/edwardkim/rhwp/issues/6628)
- **브랜치**: `task_m100_6669`
- **최신 devel 기준**: `upstream/devel@d770ef80ed5ccc82a834558355b6786213ca2e05`
- **검증 구현 head**: `a13b53ab5d8076d48723657f9f1840f31410eb30`
- **검증 구현 tree**: `ddef6b607ed3621b5faabee0db3ae116552742cf`
- **보고일**: 2026-09-03 KST
- **판정**: `stage5-approved; ready-for-pr`

## 1. 결론

Gym의 JSON 증적을 다시 실행하지 않고 사람이 판독할 수 있는 단일 self-contained HTML로 변환하는
경로를 구현했다. JSON은 계속 유일한 기계 판독 정본이고, manifest는 입력 파일 집합의 영수증,
HTML은 판정을 새로 만들지 않는 비권위 파생 뷰다.

보고서 생성은 `--seal`과 `--out` 두 단계다. 첫 단계가 41개 입력의 형식·집계·종료 코드·실행 신원과
SHA-256을 고정하고, 둘째 단계가 seal 이후 입력 변경이나 혼합 실행을 다시 검산한 뒤 HTML을 원자적으로
쓴다. 입력이 깨졌거나 섞였으면 기존 출력도 덮어쓰지 않고 종료 2로 닫힌다.

공개 합성 fixture, 3,171건의 전체 Gym 빠른 계약, 현재 tree의 구조 감사, host Chrome의 desktop·mobile
검증을 모두 통과했다. 이 기능은 일반 PR·branch·tag·제품 release·게시 게이트에 연결하지 않았다.

## 2. 산출물과 책임 경계

| 구분 | 산출물 | 책임 |
| --- | --- | --- |
| 입력 검증·seal | `gym/tools/evidence_report.py` | 41개 입력, 신원·hash·집계 검증과 원자적 manifest 생성 |
| HTML renderer | `gym/core/evidence_html.py` | 검증된 정규화 모델을 결정론적 정적 HTML로 표현 |
| 도구 규약 | `gym/docs/evidence_report.md` | 입력·상태·종료 코드·redaction·manifest 계약 |
| 인간 운영 | `mydocs/manual/gym_benchmark_operations.md` | 격리 실행부터 입력 수집, seal, HTML, 정리까지의 순서 |
| AI 지침 | `gym/AGENTS.md` | JSON 권위, fail-closed, 민감정보, 역할별 행동 경계 |
| 공개 회귀 자산 | `scripts/tests/fixtures/gym-evidence-report/complete/` | 실제 코퍼스가 아닌 2-task 합성 증적 |
| 공개 예시 | `gym/examples/evidence-report.html` | fixture에서 byte-identical하게 재생성되는 정적 보고서 |

`discriminate.py`에는 세 실행 축의 binary 신원을 교차 확인할 수 있도록 `binPath` 기록만 추가했다.
producer의 채점 의미나 기존 Gym task/reference/oracle은 변경하지 않았다.

## 3. 입력·상태·종료 계약

필수 입력은 metadata 10개, 일곱 JSON과 각 `stderr`·`exit`·`seconds` sidecar 28개,
unit test의 `txt`·`exit`·`seconds` 3개로 정확히 41개다. 일곱 JSON 역할은 다음과 같다.

1. 구조 audit
2. oracle structural probe
3. oracle selftest
4. authority ledger
5. positive
6. discrimination
7. trajectory

역할과 전체 판정은 `PASS`, `FAIL`, `INCOMPLETE`로 보존한다. `trajectory.ok`와
`trajectory.trusted`는 다른 필드이며, 단일-step의 N/A는 별도 개수이지 성공 상태가 아니다.
정직한 실패 증적은 HTML로 볼 수 있지만, 네 핵심 집계가 모두 성공을 주장하면서 cardinality가
어긋나는 위조·손상 증적은 생성 단계에서 거부한다.

- `--seal` 종료 0: PASS뿐 아니라 정직한 FAIL/INCOMPLETE 입력 집합도 정상적으로 고정됨
- `--out` 종료 0: 유효한 PASS HTML 생성
- `--out` 종료 1: 유효한 FAIL/INCOMPLETE HTML 생성
- 두 명령의 종료 2: 필수 입력, schema, 집계, 신원 또는 seal 무결성 오류; 새 출력 없음

## 4. 보안·개인정보·결정성

- raw JSON, stderr, binary 절대경로, hostname, 문서 파일명을 HTML에 복제하지 않는다.
- 자유 문자열은 HTML escape하고 400자로 제한한다.
- 경로·문서명처럼 가려진 값은 원문 대신 hash와 source length만 표시한다.
- CSP를 포함하며 JavaScript, 외부 stylesheet, 이미지, CDN, 웹폰트, 네트워크 요청이 없다.
- 생성 시각을 새로 넣지 않고 입력에 고정된 실행 시각과 정렬 규칙만 사용한다.
- 같은 fixture의 반복 생성 결과는 byte-identical하다.

고정된 공개 자산의 SHA-256은 다음과 같다.

| 자산 | SHA-256 |
| --- | --- |
| `evidence-manifest.json` | `b6171710dfdff64b5a2db45a3ddceb6d96a9f2edfadfd994a257b836a1d1075a` |
| `gym/examples/evidence-report.html` | `e438908b9661fdbe8eb9ef694205896b5efa347f1b859c6b2169510a14c1f7f9` |
| fixture 실행 identity | `b6818ef1147689a04869d315468a1874582176e3f4f6011a3c3d15e49c1339cb` |

## 5. 자동 검증 결과

### 5.1 계약 시험

| 검증 | 결과 |
| --- | --- |
| evidence report focused suite | 20/20 통과 |
| `python3 -m unittest discover -s scripts/tests -p 'test_gym_*.py'` | 3,171건 통과, 1건 조건부 skip |
| fixture HTML byte-for-byte 회귀 대조 | 통과 |
| `git diff --check` | 통과 |

전체 빠른 계약의 `ResourceWarning`과 argparse 문구는 기존 음성 시험이 의도적으로 발생시키는
출력이며 실패가 아니다.

### 5.2 현재 tree 구조 도구

| 검증 | 결과 |
| --- | --- |
| audit | 21 pack, 1,035 task/reference, issue 0, tool failure 0 |
| oracle structural | `ok=true`, issue 0 |
| oracle selftest | 14 check, failure 0 |
| authority ledger | 1,035 task/reference/entry, issue 0 |

authority는 self-live 987, contract-constant 28, independent-fixture 20,
external-oracle 0이며, baseline source는 self-live 1,031, contract-constant 4다.

전체 저장소 metadata 검사는 이번 변경과 무관한 기존 네 파일의 16건을 계속 보고한다. 해당 파일이
`upstream/devel`과 동일함을 확인했고, 이번에 변경한 문서 다섯 개의 내부 상대 링크는 모두 통과했다.

## 6. 브라우저 시각·접근성 검증

host Chrome `151.0.7922.174`, CDP protocol 1.3에 연결해 정적 서버의 공개 샘플을 확인했다.

### Desktop 1440×900

- HTTP 200, title `Gym evidence report — PASS`, 전체 상태 `PASS`
- 상태 card 8개, 표 2개, failure details 1개가 정상 표시됨
- page-level horizontal overflow 없음
- script, image, external stylesheet, performance resource 요청 0
- raw fixture hostname과 `/opt/rhwp-fixture` 경로가 문서에 없음
- `trajectory.ok=true`와 `trajectory.trusted=true`를 별도 표시

### Mobile 390×844

- 상태 card가 1열로 배치되고 본문 가로 overflow가 없음
- 넓은 두 표는 문서 전체를 밀지 않고 각 wrapper 안에서만 수평 스크롤됨
- pack 분포와 provenance 표의 제목·열·상태가 읽을 수 있게 유지됨

### 접근성·격리 확인

- native `<summary>`가 keyboard focus 대상임
- 두 표에 caption이 있고 heading 계층은 `h1` 1개, `h2` 6개임
- 모든 상태 badge에 `aria-label=PASS`가 있음
- 상태 텍스트 최소 대비율 6.01:1, 전체 상태 6.45:1
- console error와 page error 없음

기본 Chrome profile에서는 설치된 확장 프로그램이
`chrome-extension://…/dev-tools-inject.js`를 요청하는 것이 한 번 관찰됐다. 보고서의 요청인지 분리하기
위해 새 browser context에서 다시 측정했고, document 이외 subresource 요청 0, performance resource 0,
외부 `src`·`href` 0을 확인했다. 따라서 최초 요청은 host browser 환경의 주입이며 HTML 의존성이 아니다.

시각 증적은 커밋하지 않는 `output/6669/stage5/` 아래에 보존했다.

- `evidence-report-desktop.png`
- `evidence-report-mobile.png`
- `evidence-report-mobile-top.png`
- `evidence-report-mobile-pack.png`
- `evidence-report-mobile-provenance.png`

## 7. 전수 재계측을 하지 않은 이유

#6628에서 exact runner·제품 후보와 binary SHA-256을 고정한 상태로 positive 1,035/1,035,
discrimination false-pass 0, trajectory 239/239를 이미 전수 확인했다. #6669는 그 판정 생산자나 의미를
바꾸지 않고 저장된 봉투를 검증·표현하는 소비자만 추가했다.

따라서 전수 세 축을 다시 실행하면 시각화 계약보다 제품 후보의 성능과 환경 변동을 재측정하는 별도
실험이 된다. 이번에는 합성 fixture의 정상·실패·skip·의도된 scorer 거부·trajectory load-bearing와
음성 변조 사례를 고정하고, 전체 Gym 빠른 계약과 현재 tree의 구조 도구를 실행하는 것으로 범위를
지켰다. 공개 fixture의 PASS는 실제 1,035-task 전수 결과로 인용해서는 안 된다.

## 8. 커밋 계보와 원격 기준

| 커밋 | 내용 |
| --- | --- |
| `f2d6ec3a0` | 수행계획과 단계·불변식 고정 |
| `86807506f` | 입력·상태·혼합 실행 거부 계약 보정 |
| `061eae7c2` | fail-closed loader와 deterministic seal |
| `824ef1406` | self-contained HTML renderer |
| `d6b785bb7` | 공개 합성 fixture와 byte-identical sample |
| `a13b53ab5` | 인간 운영 매뉴얼·도구 규약·AI 지침 연결 |

Stage 5 검증 직전 다시 fetch한 `upstream/devel`은 기준 SHA
`d770ef80ed5ccc82a834558355b6786213ca2e05` 그대로다. 검증 구현 head는 최신 devel보다 0개 뒤,
6개 앞이며 Rust source 변경은 없다.

## 9. 최종 승인·후속 경계

현재 구현·문서·샘플은 같은 41-input 계약을 가리키며 Stage 5 자동·시각 검증을 통과했다. 메인테이너가
시각 판정과 최종 결과를 승인했으며, 다음 순서는 승인된 commit·push·PR 생성 뒤 최신 PR head CI 확인,
별도 승인에 따른 self-review와 merge다. 승인되지 않은 후속 원격 변경은 수행하지 않는다.

#6669가 병합되기 전에는 이슈를 닫지 않는다. 부모 #6628도 #6669 병합과 최종 하위 이슈 감사를
마칠 때까지 열린 상태로 유지한다.

## 10. 승인 기록

- 2026-09-03: 메인테이너가 Stage 5 시각 판정과 최종 보고 결과를 승인했다.
- 2026-09-03: 메인테이너가 최종 보고서 commit, 작업 브랜치 push와 `devel` 대상 PR 생성을 승인했다.
  self-review와 merge는 이번 승인 범위에 포함하지 않는다.
