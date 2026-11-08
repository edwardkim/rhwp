# R52 CI 변경 리베이스와 충돌 처리 보고서

- 이전 head: `ada4fdffaac0b9a7491a3f2cfb09994303380c40`.
- 이전 base: `1b78dc23fd8add8d2bc61958be5533cbeafdbd20`.
- 최신 `upstream/devel`: `2a9a88a0ddc59eb6757ee73e03eee42b4e3ac497` ([PR #8776](https://github.com/edwardkim/rhwp/pull/8776), [PR #8780](https://github.com/edwardkim/rhwp/pull/8780)).
- 리베이스 반영 head: `a5099f8ed24f1a2f85d3b1de6f21fda491a948dc`.
- 실제 replay와 기본 작업 트리 반영의 종료 코드는 모두 **0**이다. 충돌은 **0건**이다.
- 자체 157개 커밋의 순서·저자·이메일·저자 시각·전체 메시지와 원 contributor 27 PR/56개 고유 커밋을 보존했다.

[전체 157개 대응표](rebase-r52-all-commit-map.tsv) · [원 기여 56개 대응표](contributor-commit-map-r52.tsv).

## 제품 입력 보존과 검증 재사용

새 base의 변경은 CI workflow·renderer baseline 검증 도구·관련 문서 24개 경로다. Rust 제품·Cargo·글꼴 자원·corpus·테스트·Studio 제품 입력은 [Git 객체 동일성 증거](rebase-r52-product-input-identity.json)와 같이 리베이스 전후 바이트/트리가 같다.
진행 중인 고정 producer와 입력이 중간 checkout으로 바뀌지 않도록 새 sparse detached 작업 트리에서 replay하고, 검증된 결과만 기본 작업 트리에 `git reset --keep`으로 반영했다. 사용자 변경을 되돌리거나 contributor source에 push하지 않았다.

작업지시자는 CI 관련 head 변경 때문에 전체 회귀를 반복할 필요가 없다고 지정했고, 대기 중인 추가 전쪽 시각 검증은 계속하라고 확인했다. 이에 따라 R51 제품 source `ada4fdffaac0b9a7491a3f2cfb09994303380c40`에서 완료한 집중 회귀 11/11과 실제 Native 하락 27행의 0.1%p 초과 하락 0건을 보존한다. 그 source의 Native/fresh WASM producer는 위 제품 입력 동일성을 근거로 현재 리베이스 결과에도 사용한다. 실제 producer source와 리베이스 후 제출 head를 혼동하지 않는다.

추가 Native/fresh WASM 전쪽 검증과 최신 head CI·개별 PR 최종 리뷰·제출·병합은 아직 완료로 표시하지 않는다. 전체 회귀·Studio의 기존 실행 결과는 원 source·범위와 함께 기록하며 새 실행으로 재명명하지 않는다.

## 실제 로그

- `output/pr-review/planet6897-integration-20261106/logs/rebase-r52-session.log`: 새 작업 트리 준비·157개 replay·종료0.
- `output/pr-review/planet6897-integration-20261106/logs/rebase-r52-adopt.log`: 기본 작업 트리 반영·종료0.
- `output/pr-review/planet6897-integration-20261106/rebase-r52-before.json`: 실제 이전157개 메타데이터.
- `output/pr-review/planet6897-integration-20261106/rebase-r52-progress.json`: 실제 결과·입력 동일성·충돌0건·반영 완료.

임시 실행 로그는 Git에 넣지 않는다. 원 contributor PR별 최종 판단과 주장 페이지 증적은 해당 리뷰에서 확정한다.
