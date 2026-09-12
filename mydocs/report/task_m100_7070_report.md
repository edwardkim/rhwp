# #7070 병합 후 검증 CI 제거와 duration 전용 갱신

- Issue: [#7070](https://github.com/edwardkim/rhwp/issues/7070)
- 연계: [#7069 API 증거 수렴 개선](task_m100_7069_report.md)
- 기준: `3f34869b9c4d15a27b181dd22c63cd0a3730d46a`

## 원인과 변경

기존 구조는 devel push로 CI·CodeQL·Adapter·Proptest를 시작한 뒤 재사용을 판정했다.
#7068 뒤 4개 run은 `reuse=false reason=pr-merge-tree-evidence-unavailable`로 전체 실행에
진입했다. CI에는 `attempt-bound B/C/D artifacts unavailable`도 기록됐다. duration 문제만으로
네 workflow의 모든 tree 판정 원인이 설명된다고 주장하지 않는다. GitHub 내부 지연 원인도 미확정이다.

재사용 verifier를 반복 수정해도 push trigger와 Full fallback이 남으면 중복 실행은 반복된다.
이에 네 workflow의 main/devel branch push를 제거했다. Oracle advisory의 devel bootstrap도
제거했다. 전체 workflow의 push 구독을 검사하는 계약 테스트로 새 우회 경로를 방지한다.
PR 검증, 명시 수동 실행, release tag, main Pages 배포, 독립 보안 schedule과 issue-close 자동화는 유지한다.

devel push는 독립 `Refresh nextest target duration data`에서 메타데이터만 갱신한다.
신뢰된 병합 commit의 수집 코드로 GitHub API와 ZIP 내부 JSON을 읽으며, PR artifact에서 코드를
실행하지 않는다. 성공 PR CI와 B/C/D 원본 worker의 run/head/repository/PR/attempt/시각,
실제 tested merge의 PR head 부모, 자료 크기·키·수치·소유 관계를 검증한다. 후보가 최종 head가
아니면 완전한 linear review-only diff를 요구한다. 이는 실행 시간 추정 자료이며 CI 승인 증거가 아니다.
증거가 없으면 갱신만 보류하며 Cargo/CodeQL/CI 재실행 fallback을 호출하지 않는다.

## 실제 부분 재실행 증거

[보존 API fixture](../../scripts/tests/fixtures/ci-impact-policy/issue7070-copied-worker-attempts.json)는
#7068 PR CI `34702678657`의 실제 응답이다. lint만 재실행한 최신 attempt 2의 Jobs API는
B/C/D까지 새 job ID와 attempt 2로 복사했지만 시작·완료 시각은 attempt 1과 동일했다.
artifact 이름과 JSON의 attempt는 1이었다. latest job의 attempt만 신뢰하는 초기 구현도
실제 자료에서 실패했으므로 attempt별 원본 Jobs API로 보정했다.

수집기는 원본과 latest의 name/run/head/status/conclusion/시작·완료 시각이 모두 같은 경우만
원본 측정 attempt를 사용한다. 실제로 재실행돼 시간이 달라진 worker, 실패·미완료 worker,
중복·만료 artifact 또는 다른 head의 과거 성공 자료는 대체 후보로 승인하지 않는다.

2026-09-13 read-only 실제 수집 결과:

- source run `34702678657`, latest attempt **2**, B/C/D measurement attempt **1/1/1**.
- source head `85f08bda8e1b394e398c404004c27fd4bcd37fca`.
- tested merge `b6c6d3e2e3f40c58286514c8dcdc45ecc9bf23e7`.
- artifact IDs B `10301251247`, C `10301655406`, D `10301346099`.
- 기존 refresh script로 로컬 정책 **42 target 갱신**. 원격 metrics branch 쓰기는 실행하지 않았다.

## 검증과 적용 경계

- Node classifier/policy/evidence/duration/reuse 회귀 **450 PASS**, exit 0.
- Python workflow 계약 **179 PASS**, exit 0.
- 변경 workflow actionlint 구조·expression 검사 통과. ShellCheck 포함 실행의 SC2016 두 건은
  기존 CI와 Oracle의 의도적인 single-quote 문자열에서 발생한 baseline이며 신규 workflow에는 없다.
- 원본/복사 attempt 동일 실행, 혼합 attempt, 실행 시각 변경, identity/자료 누락·실패·만료,
  run 중간 변경, review-only/code 변경 후보와 실제 #7068 응답을 검사했다.
- Rust·렌더러·fixture HWP/PDF 변경 없음. Cargo 및 시각 검증 비해당.

push trigger 제거와 duration workflow는 devel 병합부터 적용된다. #7069 policy 모듈은 live base에서
로드되지만 controller 수집 배선은 main의 정상 release 반영 후 활성화된다. 이 변경을 main에 직접
push하거나 release를 강제하지 않는다. 운영 적용 뒤 merge SHA에 검증 workflow가 생기지 않는지와
duration 결과를 확인해야 하며, 로컬 검사만으로 해당 운영 확인이 끝났다고 기록하지 않는다.
