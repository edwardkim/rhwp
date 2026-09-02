---
kind: pr-review
status: self-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-26
---

# PR #6106 self-review — W8 rank 7 KoPubWorld돋움체 Light 교정 qualification

## 라우팅

- base route: `collaborator_self_merge.md`
- modifiers: `intake_and_review.md`, `local_validation.md`, `visual_fixture_evidence.md`,
  `review_only_fast_pass.md`, `rework_and_exceptions.md`의 대형 PR 경로
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`, 위 기본·보조 문서와
  `docs_and_git_workflow.md`
- 작성자 본인 self-review이므로 reviewer를 지정하지 않는다.
- code candidate: `3e705d1203cc349ca593a4fc79fe829ee5bf161a`

## 작성 시점 metadata

| 항목 | 값 |
| --- | --- |
| PR | [#6106](https://github.com/edwardkim/rhwp/pull/6106) |
| 작성자 | `edwardkim` |
| 관련 이슈 | [#4967](https://github.com/edwardkim/rhwp/issues/4967), parent [#4960](https://github.com/edwardkim/rhwp/issues/4960) |
| base / head | `devel` / `task_m100_4967_v3` |
| 규모 | 27 files, +5,770 / -56, 5 commits |
| 상태 | Open, non-draft, `MERGEABLE`, `mergeStateStatus=CLEAN` |
| base SHA | `upstream/devel@70ebacc4c9589e8c778907e179a6dab18cce8eb0` |

GitHub 상태는 변할 수 있으므로 merge 직전에 다시 확인한다. self-review 시점의 `upstream/devel`은 PR base와
같고 code candidate는 그보다 5커밋 앞서 있어 별도 merge·rebase가 필요하지 않다.

1,000줄을 넘는 변경의 대부분은 단계별 판정을 재현하는 Python·Node projector, 계약 테스트, canonical
JSON과 계획·보고서다. Q0 baseline이 Q1 runtime boundary와 Q2 metric hypothesis를 거쳐 Q3 실제 cohort
판정의 입력이 되므로 임의 분할하면 hash 계약과 판정 계보가 끊어진다. 대형 PR 규칙에 따라 즉시 merge하지
않고 self-review, review-only trailing head의 CI와 메인테이너 merge 판단을 독립 gate로 유지한다.

별도 `pr_6106_review_impl.md`는 만들지 않는다. 외부 PR 보정, 복수 PR 통합 또는 source 충돌 해결이 아니며,
self-review에서 발견한 정정은 아래의 review 시점 문서 현행화뿐이다.

## 목적과 변경 범위 정합성

#4967 rank 7 lane의 목적은 exact font를 찾았다는 이유만으로 제품 metric을 추가하는 것이 아니라, 실제
HWP 편집 습관의 장평·자간·fixed frame에서 교정 이득과 비악화를 함께 증명하는 것이다.

- 기존 W3 journal에서 10k 원문 재parse 없이 HWP 3개·HWPX 2개의 bounded cohort를 고정했다.
- 공개 HWPX와 결정적 HWP5 fixture의 각 1,556건에서 native·WASM trace parity와 첫 divergence
  `layout-metric`을 확정했다.
- exact TTF와 CDN OTF·WOFF2의 공통 cmap 25,973자 advance mismatch가 0임을 확인했지만 이를 font·paint
  identity나 재배포 승인으로 확대하지 않았다.
- 공개 fixture의 평균 advance는 줄었으나, 실제 HWPX table-cell의 admitted stored-row에서 current
  overflow 0px가 candidate 0.707px로 증가하는 modelled signature가 관찰됐다.
- 한 줄의 actual fixed-frame 회귀도 평균 개선으로 상쇄하지 않는 보호 불변식에 따라 일괄 exact metric
  후보를 `no-change`로 기각했다.
- 제품 font rule·metric DB·fallback·paint·supply와 Rust renderer source는 변경하지 않았다.

## self-review findings

### [P2][해결] 보고서와 오늘할일에 PR 생성 전 상태가 남았다

code candidate의 최종 보고서는 Q5 문서가 뒤이은 stage commit에서 고정될 예정이라고 기록했고,
오늘할일도 제출 전 검증을 미래 절차로 남겼다. PR #6106 생성과 같은 SHA의 전체 CI 성공 뒤에도 이 문구를
그대로 merge하면 현재 통합 상태와 모순된다.

review-only tail에서 기술 판정과 계측 수치는 바꾸지 않고 Q5 commit `3e705d120`, PR 번호, Full CI 성공과
남은 trailing fast-pass·merge·tracker 정산 gate를 계획·보고서·오늘할일에 현행화했다.

### [P2][해결] investigation README가 문서 manifest에 없는 `completed` 상태를 사용했다

전체 문서 metadata 검사에서 저장소 기존 오류 16건과 별개로 이번 변경의
`mydocs/tech/investigations/issue-4967/README.md`가 허용되지 않은 `status: completed`를 사용한 신규 오류
1건을 확인했다. 문서 manifest의 허용 상태는 `active`, `historical`, `superseded`이며, 완료된 조사 결과도
현재 판정 정본으로 계속 사용되고 evidence 변화 때 재개될 수 있으므로 `active`로 정정했다. 정정 뒤 전체
metadata 오류는 기존 16건으로 돌아갔다.

### 추가 blocker 없음

- Q0 projector는 bounded streaming journal, regular non-symlink 입력, 입력 크기·line·문서 수·중복 index,
  exact digest와 owner-only private output을 fail-closed로 검사한다.
- Q1은 `maxCharacters=4096`, 완전한 trace, 형식별 native/WASM canonical byte parity와 output symlink 거부를
  유지한다. HWPX substitution metadata를 layout fallback으로 승격하지 않는다.
- Q2는 exact font·CDN artifact·package·license·registry와 Q0·Q1 canonical hash를 검사하고, font supply와
  layout metric identity를 분리한다.
- Q3는 동결한 5문서만 same-snapshot으로 실행한다. modelled regression을 open style·cache gap보다 먼저
  기각하고, private detail과 공개 aggregate를 분리한다.
- 공개 JSON 4종에는 private 문서명·경로·본문·식별 hash·font bytes와 절대 로컬 경로가 없다.
- rank 8 공용 helper 일반화는 기존 rank 8 계약과 rank 7 신규 계약을 같은 실행에서 통과했다.
- 새 integration source, generated suite·manifest, Cargo target 파생물과 `src/**` 변경은 PR diff에 없다.

## 렌더·시각 증적 판정

PR이 공개 HWP/HWPX fixture와 fixed-frame 수치 판정을 포함하므로 `visual_fixture_evidence.md`를 적용했다.
그러나 fixture는 runtime trace와 metric hypothesis를 고정하는 조사 입력이며, 제품 renderer·layout·typeset·
paint·WASM 출력 경로는 바뀌지 않는다. 특정 페이지나 표의 before/after 개선도 주장하지 않는다.

제품-visible 후보가 Q3의 modelled regression에서 기각돼 Q4 backend·시각 판정에는 진입하지 않았다. 따라서
직접 visual sweep 필수 조합이 아니며 별도 PDF·PNG와 대표 review asset을 추가하지 않는다. 이 review는
“시각 검증 통과”를 주장하지 않는다.

## 완료한 로컬 검증

| 검증 | 결과 |
| --- | --- |
| rank 7·공유 rank 8 Python 계약 | 28/28 통과 |
| rank 1·7·8 Node runtime 계약 | 13/13 통과 |
| Rust Font Decision Trace integration | 8/8 통과 |
| changed Markdown 링크 | 611문서, 이상 없음 |
| Markdown checker 단위테스트 | 5/5 통과 |
| 공개 JSON canonical·privacy | 4/4 통과 |
| Python compile·Node syntax | 통과 |
| fmt·diff | 모두 통과 |
| 전체 문서 metadata | 이번 변경 신규 오류 0, 기존 debt 16건 |

Q1에서 current release native와 표준 Docker WASM을 새로 빌드하고 HWPX·HWP5 trace byte parity를 확인했다.
그 뒤 code candidate까지 Rust·Docker WASM 빌드 입력과 Rust 제품 source delta는 0이므로 동일 빌드를 반복하지
않았다. Q3 private projection은 동일 입력에서 private/public 결과를 두 번 실행해 byte-exact함을 확인했다.

## GitHub Actions

code candidate `3e705d120`의 [CI run 32918576624](https://github.com/edwardkim/rhwp/actions/runs/32918576624)은
preflight, Lint, Native Skia, Frontend package, archive A/B/C와 모든 test worker, Build & Test aggregate가
성공했다. [CodeQL 32918576374](https://github.com/edwardkim/rhwp/actions/runs/32918576374),
[Proptest 32918576393](https://github.com/edwardkim/rhwp/actions/runs/32918576393),
[Adapter inter-diff 32918576439](https://github.com/edwardkim/rhwp/actions/runs/32918576439)도 같은 SHA에서
성공했다. 정책상 WASM Build와 Frontend unit gates의 skip 외에 실패·대기 check는 없다.

현재 self-review·오늘할일·계획·보고서 상태 정정은 이 녹색 code candidate 뒤의 `mydocs/` 한정
single-parent trailing commit이다. push 뒤 review-only fast-pass가 정확한 candidate를 재사용하고 최신
required aggregate가 성공하는지 확인해야 한다.

## 최종 권고

exact source와 공급 artifact의 metric 호환성만으로 실제 fixed-frame 비악화를 보장할 수 없으며, 관찰한
신규 table-cell overflow를 근거로 제품 변경을 만들지 않은 `no-change` 결론은 보호 불변식과 일치한다.
증거 계보, 비공개 자료 경계, resource limit, native/WASM parity와 Full CI에서 추가 blocker는 발견하지
않았다.

self-review는 **완료 / 조건부 merge 권고**다. review-only trailing head의 fast-pass, 최신
`MERGEABLE/CLEAN`과 메인테이너의 별도 merge 승인을 확인하기 전에는 merge하지 않는다. 병합 뒤 #4967을
#4960의 실제 sub-issue로 연결하고 W8 checkbox·최종 reopen 조건을 정산한 다음 #4967을 수동 close한다.
