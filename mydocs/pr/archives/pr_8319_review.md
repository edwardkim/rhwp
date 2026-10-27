# PR #8319 리뷰 — 편집·표 조판·차트 45개 PR 통합과 회귀 보정

## 최종 판정

**승인.** 원 PR 45개를 각각 검토하고 필요한 메인터너 보정을 포함한 통합 head의 로컬 검증과
정확한 code head CI·정책 성공을 확인했다. 최신 사용자 지시에 따라 self-review와 오늘할일을
**같은 PR의 문서-only trailing commit**으로 반영한다. 아직 병합하지 않았으며 trailing head의
CI·정책·병합 가능 상태 확인 뒤 일반 merge한다. 원 contributor head의 직접 merge와 구분한다.

| 항목 | 실제 값 |
| --- | --- |
| PR·작성자·base | [#8319](https://github.com/edwardkim/rhwp/pull/8319) / jangster77 / devel |
| 통합 브랜치 | review/semanticist21-20261019 |
| Full CI 검증 code head | `aee691ae596d288382369b9d5243f4cd06e7fa65` |
| 고정 검증 base | `faa885e8b54ffc7cdf4196cd43a3103f362499a2` |
| 제품·기존 회귀 검증 source | `04c92da78165ccbf63b467855591df183655a98d` |
| 병합 상태 | trailing head CI·정책 성공 뒤 일반 merge 예정 |
| 정확한 head CI | [40489874571](https://github.com/edwardkim/rhwp/actions/runs/40489874571) |
| 기록 브랜치 | `review/semanticist21-20261019` |

## 원 PR별 검토와 구현 주장

[개별 review·구현 검토 45쌍](../assets/s21_20261027/reviews.md)을 각각 작성했다.
원 head·체리픽 계보·보정·실제 계약 함수·독립 Print·전체 TSV와 주장에 맞는 대표 쪽을 구분했다.
통합 PR 본문에도 45개 개별 검토 표와 24개 Markdown 이미지를 넣었다. 런타임 계약인 6개는
관계없는 PNG를 결함 검출 증거로 붙이지 않았다. 캐럿·링크·IME 이력과 정적 출력의 검증을 구분했다.

현재 보정 범위와 리베이스로 들어온 기능만 검증했다. 추가 CI 보정은 기존 PDF 3개와 HWP 1개로 한정했다. 추가 PR 체리픽·다른 실제 HWP 확대는 하지 않았다.
원 이슈→해결 PR→통합·메인터너 보정→독립 기준의 분석은 각 review_impl과
[입력·source 계보](../assets/s21_20261027/source-lineage.json)에 있다.

## 로컬 검증과 CI 실패 보정

- 최종 제품 source의 default-feature 전체 nextest **11,832 PASS·45 skip·exit 0**,
  release lib **3,983 PASS·12 ignored**, 집중 73건 PASS, Skia lib **4,040 PASS·12 ignored**.
- Studio **2,597 PASS**와 fresh 글꼴 추적·TypeScript·production build·실제 브라우저 3종,
  스크립트 계약 39 PASS·1 skip을 [명령·함수별 증거](../assets/s21_20261027/execution-evidence.json)에 보존했다.
- 이전 제출 head f528dba6의 Native #7950 검사는 고정 `RGB <80`이 얇은 대체 글꼴의 보이는
  B1을 놓쳐 실패했다. 원 해결 PR #7953과 원본 PDF를 대조해 색상 관계 검사로 보정했다.
- 이 Native 전용 CLI 검사는 기존 default-feature 전체와 Skia lib 검증에 포함되지 않았다.
  누락을 숨기지 않고 정상·최소 글꼴 각각 4 PASS, 실제 PNG 글자 가림 음성 대조 FAIL(exit 101),
  CI와 같은 Native 명령 **15개 전체 PASS**, fmt·Native/WASM/workspace/Skia Clippy·workspace build·정책 통과로 보완했다.
- #2293의 실제 보이는 라벨을 놓친 하단 6% 밴드도 제거했다. 원본 텍스트 12개를 각각 제외한 대조와
  원 Native 출력을 비교하는 관계형 검사로 교체했다. 정상·최소 환경 2 PASS씩, 글꼴 부재 음성 대조 FAIL,
  필수 Rust lint가 통과했다. 같은 원본의 새 MCP Print 한 쪽은 Native 97.45943% / WASM 96.57949%다.
  양쪽 98% 미만이므로 px·고정 영역으로 회귀를 걸지 않았다.
- 이 추가 보정은 테스트·문서·증적·독립 Print PDF만 바꿨다. 제품 source와 baseline·golden은 같아 기존
  제품·Studio·전체 시각 결과를 재사용했으며 새 제품 실행으로 합산하지 않았다.
  [실패 원인·독립 기대값·12개 전쪽 대조](../../working/task_semanticist21_20261027_ci_pdf_paint.md)를 연결한다.

Cargo 8 jobs·테스트 12 threads, 공유 `target/pr-review`를 순차 사용했다.
원문 로그·중간 실행 JSON·생성 suite는 커밋하지 않는다. 로컬 통과와 원격 CI의 성공을 구분했다.

## 정확한 head의 원격 CI

[CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)의 builder 예상 수와 worker 실행 수를 실제 artifact에서 각각 읽어 대조했다.

| Archive | 예상 | 실제 PASS |
| --- | ---: | ---: |
| A | 3,801 | 3,801 |
| B | 1,895 | 1,895 |
| C | 1,163 | 1,163 |
| D | 2,323 | 2,323 |
| E | 2,456 | 2,456 |
| 합계 | 11,638 | 11,638 |

Native Skia는 비어 있지 않은 15개 명령 묶음의 **124 PASS**를 실제 job 로그로 확인했다.
최신 head의 총 43개 check가 success/skipped/neutral로 끝났고 CI Impact Policy도 성공했다.
Frontend의 fresh WASM·Lint·CodeQL·Render Diff·Adapter·Proptest 등 실제 결과와 artifact SHA는
[최종 CI 증거](../assets/s21_20261027/final-ci.json)에 있다. code head의 MERGEABLE/CLEAN을 확인했다.
trailing head에서도 CI·정책·병합 가능 상태를 다시 확인하고 검증한 최종 head를 지정해 일반 merge한다.
현재 병합 전이며 admin 우회는 사용하지 않는다.

## 조판 원칙·입력·시각 증적

공통 로직의 생산→측정→배치 소비, 원본 줄·개체 소속과 경계 반례는 각 원 PR별 기록에 연결했다.
독립 Print의 입력·PDF SHA·제품·인쇄 설정과 실제 코드 provenance를 구분하고,
새 회귀의 90% 선행 조건 및 완료 쪽의 양쪽 98%·독립 ±2px 조건과 관계 검사를 함께 적용했다.
문서별 수치·ID로 출력을 맞추는 보정을 완료 근거로 쓰지 않았다.

Native/fresh WASM 각 **197조건·256쪽**, 후보 최저 **93.22230% / 93.52322%**,
90% 미만·누락·측정 불가·쪽수 불일치 0쪽이다. 같은 조건 base 공통 512쪽에서 사용자 지정
**0.1%p 초과 하락 0쪽**을 확인했다. #8188 지정 네 입력은 총 512쪽 양쪽 raw 100%·정수 미일치 0/0을 유지했다.
#8116의 기존 네 입력 15쪽과 기존 15개 검사도 보존했다. sample16·편람의 일반 회귀 삭제·범위 제외는 유지했다.

[전쪽 TSV·입력/Print·대표 PNG](../assets/s21_20261027/README.md),
[주장별 쪽 선택과 직접 판독](../assets/s21_20261027/claim-evidence.json),
[편집 중 실제 상태](../assets/s21_20261027/editing-session-evidence.json)를 각각 연결했다.
전체 TSV 796개와 CI 보정의 12개 TSV는 요약값으로 대체하지 않았다.
수식·미세 간격·표 외곽·후속 본문을 review와 standalone overlay에서 직접 확인했다.
raw100을 RGB·획 농도·글꼴 모양의 완전 일치로 선언하지 않았다.

## Merge 후 후속 처리

이 PR의 trailing head CI·일반 merge와 devel 동기화 뒤 원 PR 45개의 실제 최신 head·상태를 다시 확인한다.
같은 merge·증적 comment는 중복 게시하지 않고 각 PR별 한국어 존댓말 문안에 원 기여·보정·실제
CI·merge SHA·그 SHA로 고정한 대표 review/overlay를 남긴 뒤 체리픽 통합 완료로 종료한다.
contributor fork·branch는 변경하거나 삭제하지 않는다. 원 PR을 직접 merge했다고 표시하지 않는다.
#7721·#7722의 기존 완료·부모 #7445의 열린 상태를 유지하며 새 issue close를 추정하지 않는다.

코드 merge 후 duration metadata 갱신만 관찰하며 제품 CI는 재실행하지 않는다. 별도 기록 PR은
만들지 않고 같은 PR에서 검토 기록과 오늘할일을 함께 반영한다. comment·원 PR 종료와 전용 branch/worktree 정리,
실제 사용 종료 뒤 root output 전체·공유 target의 debug 삭제 및 외부 정리 로그까지 완료한다.
