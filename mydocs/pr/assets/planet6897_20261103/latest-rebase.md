# 문서 전용 devel 갱신 뒤의 증거 재사용

현재 PR base는 `ed057276c1e0e07e743218d95641e015b46b6715`이며 직전 기준은 `f529dc92d4d172bf32390d7749743a45d9e41bfc`다.
현재 브랜치의 183개 패치를 충돌 0건으로 리베이스했고 range-diff의 183행이 모두 `=`다.
리베이스 전후 Git tree의 차이는 새 base의 후속 처리 문서 14개뿐이다.
제품·테스트·fixture·검증 도구와 글꼴 프로그램 내용은 리베이스 전과 같다.

- 이전 head: `19ef3335db458f8381540897256f27df4c7417d7`.
- 리베이스 결과 head: `267ea7873dc07ca11b81d3445e78292ef18b1790`.
- 실제 렌더·최종 회귀 source: `ba2ebfd34a4ebf402bce8e002254b672a09ef764`.
- 실제 base 제품 source: `f529dc92d4d172bf32390d7749743a45d9e41bfc`.

해당 source의 완료한 필수 실행 41개·최적화 제품·실제 API·전쪽 TSV·직접 판독을 재사용한다.
새 head에서 다시 빌드·전체 회귀·전쪽 렌더를 수행했다고 표시하지 않는다.
회귀 측정 주석 1개는 전체 검증 종료 후 갱신했으며 assertion·helper 본문이 바이트 단위로 같음을 별도로 확인했다.
리베이스 뒤 `cargo fmt --all -- --check`와 새 base SHA를 지정한 manifest·unit-tier 정책은 모두 exit 0이다.
파생 harness는 제출하지 않는다. 문서·링크·입력 바이트 감사는 제출할 문서 내용에 다시 적용한다.

새 base에서 반영한 조판 기능은 실제 f529 기준 제품과 ba2 후보 제품을 같은 조건에서 대조한다.
ed057은 그 기준의 제품 내용과 같으므로 해당 실제 대조는 현재 base의 기능 보존 증거이기도 하다.
기존 전쪽 비교를 반복하지 않는 작업 지시를 따르며 새 base 대비 정밀 raw 0.1%p 초과 하락은 허용하지 않는다.
[전체 검증 목록](README.md), [실제 필수 41개 실행](rebase8609/rust-regression-validation.md),
[원 PR의 현재 적용 SHA](integration-commits.md)를 함께 확인한다.

## PR 제출 직전의 이슈 라벨링 base 갱신

최신 base는 `d4a72efd7c91d1e25092ed30b9d44e43cbebb9f0`이며 PR #8618의 workflow·Node 계약 검사·운영 문서 3개만 추가됐다.
위 문서 전용 리베이스 이후 184개 패치를 다시 적용했고 모두 `=`·충돌 0건이다.
이전 head `15cc3424fecdb9813b341e3f6a613452d5d3c2ce`, 적용 직후 head `750e3c7aef83d8dd61b1b82ad1190e00d0f95f7d`를 비교한 tree 차이도 이 3개뿐이다.
렌더·Rust 테스트·fixture·글꼴은 그대로이므로 전체 회귀·전쪽 렌더를 반복하지 않았다.
`node --test scripts/tests/issue-form-labels.test.mjs`는 실제 반영된 workflow를 실행해 32 PASS·0 FAIL이다.
새 base SHA의 manifest·unit-tier 정책과 fmt도 통과했다.

제출 diff 공백 검사에서 Native 표시값 대조표의 CRLF만 LF로 정리했다.
비교 수치·행 순서와 별도 보존한 전쪽 원 TSV 바이트는 바꾸지 않았다.
