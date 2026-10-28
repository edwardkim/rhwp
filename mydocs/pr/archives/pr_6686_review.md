---
kind: pr_review
status: approved
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-03
pr: 6686
issue: 6669
author: edwardkim
---

# PR #6686 self-review — Gym JSON 증적의 결정론적 HTML 시각화

## 결론

**승인.** PR #6686은 저장된 Gym 전수 실행 증적을 다시 채점하지 않고, 검증된 입력 집합을 사람이
판독할 수 있는 단일 self-contained HTML로 변환한다. JSON 봉투는 유일한 기계 판독 정본이고,
manifest는 입력·실행 신원의 영수증이며, HTML은 비권위 파생 뷰라는 경계를 코드·문서·회귀 자산에
일관되게 고정했다.

최종 code candidate `c5efd5240e79a8c862df859dd6a505c55031e8ab`에서 로컬 focused 계약
20건과 GitHub의 실행 대상 check가 모두 통과했다. 최신 `upstream/devel`이 이후
`bd72886c02d301ff796b6b5c55a452a870cf317a`로 전진했지만 변경 경로 중첩은 0개이고 현재 base와의
merge-tree도 충돌 없이 생성됐다. 이 review·오늘할일·부모 보고서만 추가하는 후행 head의
review-only checks와 mergeability를 다시 확인한 뒤 정상 merge할 수 있다.

이 문서의 `승인`은 작성자 self-review 판정이다. 자기 PR이므로 reviewer 지정이나 GitHub approve
review event를 만들지 않으며, remote push와 merge는 각각 별도 사용자 승인 게이트다. PR 병합 뒤
#6669 자동 종료와 최신 `devel` 포함을 확인한 다음 부모 #6628의 완료 조건을 최종 정산한다.

## 라우팅과 메타데이터

- 기본 경로: `collaborator_self_merge.md`
- 보조 경로: `intake_and_review.md`, `local_validation.md`, `review_only_fast_pass.md`,
  `rework_and_exceptions.md`
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`와 위 자식 문서
- 시각 검증은 최종 보고서의 host Chrome desktop·mobile·offline·접근성 증적을 사용했다.
- `review_impl`은 추가하지 않는다. 승인된 계획서·Stage 보고서·최종 보고서가 구현과 검증 계보를
  이미 고정한다.

| 항목 | 값 |
| --- | --- |
| PR / 작성자 | [#6686](https://github.com/edwardkim/rhwp/pull/6686) / @edwardkim |
| 관련 이슈 | [#6669](https://github.com/edwardkim/rhwp/issues/6669) (`Closes #6669`) |
| 부모 이슈 | [#6628](https://github.com/edwardkim/rhwp/issues/6628) |
| 최초 검증 base | `devel@d770ef80ed5ccc82a834558355b6786213ca2e05` |
| self-review 시점 최신 base | `devel@bd72886c02d301ff796b6b5c55a452a870cf317a` |
| code candidate | `c5efd5240e79a8c862df859dd6a505c55031e8ab` |
| 규모 | 61 files, `+4,085/-13`, 7 commits |
| 작성 시점 GitHub 상태 | Open, 비 Draft, `MERGEABLE`, `CLEAN`; 성공 29, skip 4, 실패·대기 0 |
| reviewer | self PR이므로 지정하지 않음 |

1,000줄을 넘으므로 대형 PR 경로를 적용했다. 변경 수가 큰 이유는 loader·seal·renderer와 41개 입력의
공개 합성 fixture, byte-identical 예시 HTML, 계약 시험, 인간·AI 운영 문서를 함께 제출하기 때문이다.
fixture와 tracked HTML을 분리하면 결정성·무결성 주장을 재생성으로 검증할 수 없고, loader와 renderer를
분리 제출하면 중간에 권위 경계가 성립하지 않으므로 논리적으로 원자적이다. 기능 구현은
`gym/tools/evidence_report.py`, 표현은 `gym/core/evidence_html.py`로 책임을 분리했다.

## 코드 검토와 보호 불변식

### 입력·seal·실행 신원

- metadata 10개, 일곱 JSON과 process sidecar 28개, unit sidecar 3개의 정확히 41개 입력을 요구한다.
  누락·symlink·비 UTF-8·BOM·중복 JSON key·비유한 숫자·파일 상한 초과를 성공으로 접지 않는다.
- 기존 Gym validator를 exact 저장소 경로에서 불러와 역할별 schema와 집계를 다시 검증한다. audit,
  oracle structural/selftest, authority ledger, positive, discrimination, trajectory 일곱 역할을 서로
  바꾸거나 섞을 수 없다.
- positive·discrimination·trajectory의 제품 binary 경로와 실행 metadata를 교차 확인하도록
  `discriminate.py`에 `binPath` provenance를 추가했다. producer의 채점 연산이나 통과 기준은 바꾸지
  않았다.
- manifest는 모든 입력의 byte length와 SHA-256, generator version, 실행 identity와 역할별 상태를
  canonical JSON으로 기록한다. 검증과 원자적 교체 사이, seal 검증과 HTML 기록 사이의 입력 변경도
  다시 읽어 거부한다.

### 판정 의미와 종료 코드

- 역할 및 전체 결과는 `PASS`, `FAIL`, `INCOMPLETE`를 보존한다. 정직한 FAIL/INCOMPLETE 증적도 seal할
  수 있지만 HTML 생성 명령은 종료 1을 반환하므로 녹색 성공으로 오인되지 않는다.
- 구조·schema·집계·실행 신원·seal 오류는 종료 2와 기계 판독 오류 봉투로 닫힌다. 기존 manifest나
  HTML은 원자적 쓰기 전 검증이 실패하면 덮어쓰지 않는다.
- discrimination의 의도된 scorer 거부와 설명되지 않은 score error를 분리한다. false-pass는 FAIL,
  설명되지 않은 오류와 `trajectory.trusted=false`는 INCOMPLETE로 남는다.
- `trajectory.ok`, `trajectory.trusted`, load-bearing, single-step N/A를 별도 값으로 표시해 경로 필요성
  감사를 일반 성공 수치로 축약하지 않는다.

### 비권위 HTML·보안·개인정보

- renderer는 검증된 정규화 bundle만 받고 I/O나 재채점을 하지 않는다. HTML의 PASS가 새로운 제품
  정답이나 한컴 조판 동등성으로 승격되지 않도록 authority boundary를 본문에 표시한다.
- raw JSON·stderr·binary 절대경로·hostname·문서명을 복제하지 않는다. 자유 문자열은 HTML escape,
  경로·문서 토큰 redaction, 400자 상한과 원문 길이·hash 표기로 처리한다.
- CSP `default-src 'none'`을 포함하고 JavaScript, 이미지, 외부 stylesheet, CDN, 웹폰트를 사용하지
  않는다. 생성 시각이나 난수를 새로 넣지 않아 같은 sealed fixture는 byte-identical이다.
- 출력이 필수 입력이나 manifest를 덮어쓸 수 없고, `.html`이 아닌 출력·symlink·디렉터리는 거부된다.

코드 diff에서 제품 검증기 완화, 일반 CI·release·게시 gate 연결, Rust 제품 변경, 사설 코퍼스나
식별 가능한 실제 문서 증적은 발견되지 않았다. 새 blocker도 발견되지 않았다.

## 로컬·구조·시각 검증

[최종 보고서](../../report/task_m100_6669_report.md)와
[도구 규약](../../../gym/docs/evidence_report.md)에 다음 결과를 고정했다.

| 검증 | 결과 |
| --- | --- |
| evidence report focused suite | self-review 재실행 포함 20/20 통과 |
| 전체 Gym Python discover | 3,171건 통과, 정책상 skip 1건 |
| 공개 fixture 재생성 | manifest와 HTML byte-identical |
| Gym audit | 21 pack, 1,035 task/reference, issue 0 |
| oracle structural / selftest | issue 0 / 14 check, failure 0 |
| authority ledger | 1,035 task/reference/entry, issue 0 |
| Markdown 링크·`git diff --check` | code candidate에서 통과 |

결정론적 공개 자산은 manifest SHA-256
`b6171710dfdff64b5a2db45a3ddceb6d96a9f2edfadfd994a257b836a1d1075a`, HTML SHA-256
`e438908b9661fdbe8eb9ef694205896b5efa347f1b859c6b2169510a14c1f7f9`, identity fingerprint
`b6818ef1147689a04869d315468a1874582176e3f4f6011a3c3d15e49c1339cb`로 고정됐다.

host Chrome 151, CDP protocol 1.3에서 desktop 1440×900과 mobile 390×844를 검사했다. page-level
가로 overflow, console/page error, 외부 resource 요청은 0이고, mobile 표는 wrapper 내부에서만
수평 스크롤한다. heading 계층, native summary keyboard focus, 표 caption, 상태 aria-label과 최소
대비율 6.01:1도 확인했다. 설치 확장 프로그램의 최초 요청은 격리 context에서 재측정해 보고서
의존성이 아님을 분리했다.

## GitHub Full CI

code candidate `c5efd5240e79a8c862df859dd6a505c55031e8ab`에서 이름이 있는 check 29개가 성공했고,
조건상 비대상 4개가 skip됐으며 실패·대기는 0개다.

- [CI 33729326973](https://github.com/edwardkim/rhwp/actions/runs/33729326973): preflight, Lint,
  Native Skia, frontend package, 네 archive build/test shard와 Build & Test 성공
- [CodeQL 33729326940](https://github.com/edwardkim/rhwp/actions/runs/33729326940): Rust, Python,
  JavaScript/TypeScript 분석과 최종 CodeQL 집계 성공
- [Gym Benchmark Validation 33729326720](https://github.com/edwardkim/rhwp/actions/runs/33729326720):
  Gym benchmark contracts 성공, 수동 전수 job은 조건상 skip
- [Proptest 33729326895](https://github.com/edwardkim/rhwp/actions/runs/33729326895): preflight와
  prop roundtrip 성공
- [Adapter inter-diff 33729326844](https://github.com/edwardkim/rhwp/actions/runs/33729326844):
  preflight와 본 검사 성공
- [CI Impact Policy Controller 33729326904](https://github.com/edwardkim/rhwp/actions/runs/33729326904):
  trusted CI impact policy 성공

## 최신 devel 호환성

Full CI 뒤 `upstream/devel`은 `bd72886c02d301ff796b6b5c55a452a870cf317a`까지 4커밋 전진했다.
추가 변경은 PR #6591의 renderer near-fit TAC 표 수정과 integration source·두 golden/baseline이며,
PR #6686의 61개 경로와 중첩이 없다. exact current-base `git merge-tree --write-tree HEAD
upstream/devel`은 성공했고 GitHub도 `MERGEABLE`, `CLEAN`을 보고한다.

`review_only_fast_pass.md`에 따라 base 전진만으로 검증된 code candidate에 merge/rebase를 추가하지
않는다. 이 review를 포함한 후행 커밋은 `mydocs/` 세 파일만 바꾸며, 최신 head에서 review-only
classifier와 필수 check를 다시 확인한다.

## 잔여 위험과 후속 경계

- HTML은 비권위 파생 뷰다. 실제 1,035-task 결과의 machine decision은 원본 JSON이며 공개 합성
  fixture의 PASS를 실제 전수 성공으로 인용할 수 없다.
- authority ledger의 external oracle은 0개다. 이번 기능은 Gym 자체 정합성과 API 사용 증적을 읽기
  쉽게 만들 뿐 한컴 결과와의 독립 동등성을 만들지 않는다.
- 실제 대형 전수 증적의 HTML 생성 성능은 이번 범위의 gate로 고정하지 않았다. 제품 runtime에는
  연결되지 않으며 현재 측정은 기초 자료다.
- [#6684](https://github.com/edwardkim/rhwp/issues/6684)의 AI 온보딩 문서 품질 개선은 독립 후속이며
  #6628의 완료 범위를 자동 확장하거나 이 PR의 병합을 막지 않는다.
- 대형 PR이므로 후행 review-only head에서도 trusted 판정, 최신 base 관계와 mergeability를 다시
  확인한다. 자동 merge나 admin 우회는 사용하지 않는다.

## 최종 판정과 다음 조건

- 판정: **승인**
- 판정 대상: code candidate `c5efd5240e79a8c862df859dd6a505c55031e8ab`
- trailing 조건: 이 review·오늘할일·부모 보고서만 추가한 최신 head에서 review-only checks 성공,
  `MERGEABLE`·`CLEAN` 및 최신 base 재확인
- merge 조건: 최신 head SHA 고정, 사용자 merge 승인, `--admin` 우회 없는 정상 2-parent merge commit
- GitHub review: self PR이므로 approve event와 reviewer 지정 없음
- merge 후: merge SHA·최신 `devel` 포함·post-merge CI와 #6669 자동 close를 확인하고, 부모 #6628의
  두 sub-issue 완료 및 최종 보고서 정합성을 감사한 뒤 별도 승인으로 부모를 close한다.
