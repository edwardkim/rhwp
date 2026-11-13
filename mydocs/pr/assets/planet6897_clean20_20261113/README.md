# planet6897 무충돌 PR 20개 통합 검토

`review/integration-20261113-planet6897-clean20`에서 원 PR 20개의 고유 커밋 53개을 누적 적용했다. 중복9개는 다시 적용하지 않았고, 충돌 PR #8937·#8972·#8976은 제외했다. 원 contributor branch는 변경하지 않았다.

- 고정 base `31a422575722f634cfc2d9eb97e708da58aea188`, 최종 검증 제품 code `d0cc46573c7505633721c19062e504faff017d59`.
- 최소 글꼴 macOS 전체 nextest **13,420 PASS / 0 FAIL / 45 skip**, 이후 새 회귀1건 포함 관련19건 및 lint·manifest PASS. 중복 수를 합산하지 않는다.
- Rust fmt·Native/WASM Clippy·workspace build·all-target Clippy PASS. Native Skia lib workspace4,034 PASS/12 ignore, PNG2 PASS, PDF36 PASS.
- 주장22조건380쪽을 Native/fresh WASM 각각 전쪽 검증했다. 최저91.17003%/91.05707%, 90% 미만0·devel 대비2%p 초과 하락0.
- 정본 보호18조건1,771쪽을 양쪽 각각 검증해 공식 두 자리 표시값 하락0. 별도 Native `--handbook-gate`383쪽 PASS.
- 실제 일반 browser48건과 문제 페이지 추가12건을 확인했다. 실제 CK 지원 사례와 글꼴/기존 preflight 한도로 대체된 Canvas2D를 구분한다. 일반·추가 observer 각각 종료1·기존 미지원 CK 요청의8개 hard violation을 보존하며 전체 observer 통과/CK60건 통과로 주장하지 않는다. 실제 WASM 수신66건과 독립 JS 초기화6건의 해시 출처를 확인했다.
- fresh WASM은 Mac locked wrapper `--target web --out-dir pkg --no-opt` 로컬 대체 빌드이며 Docker 최적화 통과가 아니다.

- 첫 CI의 옛 실제 Canvas clip 기대값을 보정한 [원인·FAIL→PASS](final-d0cc4657/canvas-print-clip-contract-repair.json): 제품 source 변경 없이 동일 브라우저 entry19 PASS. #8999는 이 검사 보정까지 포함해 수용하며 새 head CI를 재확인한다.

- [통합 PR #9033의 정확한 code head CI](final-d0cc4657/pr9033-code-ci.json): `257b6fabee` Full CI·CodeQL·Render Diff·Adapter·Proptest·최종 정책 감사 성공. CI Archive A–E 합계13,236 PASS/0 FAIL/45 skip, Canvas browser19 PASS. 최신 문서-only trailing 검사와 실제 병합은 별도로 확인한다.

## 최종 검증과 개별 리뷰

- [고유 원 PR·source SHA](source-plan.json) · [최종 live head20개](final-d0cc4657/source-head-final-prepush.json)
- [현재 Rust 검증](final-d0cc4657/rust-validation.json) · [PR별 실제 회귀](final-d0cc4657/individual-test-execution.json)
- [Native/fresh WASM 전쪽·정본 보호](final-d0cc4657/whole-visual-qualification.json)
- [주장별44개 base/후보 비교·원본90쌍 증적](final-d0cc4657/main-claims-qualification.json) · [실제 직접 판독43쌍](final-d0cc4657/direct-reading.json)
- [실제 backend·54개 WASM 수신](final-d0cc4657/actual-backend-qualification.json) · [추가 주장 페이지12건](final-d0cc4657/actual-claim-pages-qualification.json)
- [실행 입력85개와 commit 바이트 확인](final-d0cc4657/committed-input-audit.json)
- [새 sample23개 보안·injection 검사](final-d0cc4657/new-sample-security.json) · [공식 oracle 선택 감사](final-d0cc4657/new-sample-oracle-registration.json)

| 원 PR | 개별 리뷰 |
| --- | --- |
| #8932 | [검토](../../archives/pr_8932_review.md) |
| #8936 | [검토](../../archives/pr_8936_review.md) |
| #8938 | [검토](../../archives/pr_8938_review.md) |
| #8949 | [검토](../../archives/pr_8949_review.md) |
| #8955 | [검토](../../archives/pr_8955_review.md) |
| #8959 | [검토](../../archives/pr_8959_review.md) |
| #8974 | [검토](../../archives/pr_8974_review.md) |
| #8980 | [검토](../../archives/pr_8980_review.md) |
| #8987 | [검토](../../archives/pr_8987_review.md) |
| #8990 | [검토](../../archives/pr_8990_review.md) |
| #8991 | [검토](../../archives/pr_8991_review.md) |
| #8994 | [검토](../../archives/pr_8994_review.md) |
| #8999 | [검토](../../archives/pr_8999_review.md) |
| #9001 | [검토](../../archives/pr_9001_review.md) |
| #9003 | [검토](../../archives/pr_9003_review.md) |
| #9004 | [검토](../../archives/pr_9004_review.md) |
| #9006 | [검토](../../archives/pr_9006_review.md) |
| #9013 | [검토](../../archives/pr_9013_review.md) |
| #9015 | [검토](../../archives/pr_9015_review.md) |
| #9023 | [검토](../../archives/pr_9023_review.md) |

## 반례 보정과 기존 잔여

- [공통 원인·보정](maintainer-repair/cause.md): health의 실제 새 쪽 앞 간격/줄 전진과 food의 아직 배치하지 않은 표 각주 예산 오판을 수정했다.
- [health5·6쪽의 시각 선행 조건·좌표/관계 회귀](maintainer-repair/health-visual-precondition.json): 6쪽 Native99.98623%/WASM99.99723%. 전체7쪽의 기존1쪽 미달을 숨기거나 정상 golden으로 고정하지 않는다.
- [food 공통141쪽의 base/후보 대조](maintainer-repair/food-common-qualification.json): 새144쪽 악화를143쪽으로 복원했다. 독립 Print141쪽과의 기존 차이는 #8997이며 이 통합으로 close하지 않는다.
- #8998은 receipt의 run 경계와 issue2439 저장 줄 정보 두 범위다. 현재 receipt 개선만 검증했으므로 전체 close 대상이 아니다.
- 기준 PDF 제품·빌드는 [실제 Creator/Producer](final-d0cc4657/reference-pdf-products.json)로 구분했다. 유효한 데스크톱 Print를 재사용하며 파일명만으로 출력 제품 연도를 단정하지 않았다. 추가 반례2개는 실제 한컴2020 MCP Print를 보존했다.

초기 code676e의 [이전 Rust 기록](local-rust-gates.json)·[이전 canonical 시각 결과](canonical-visual/)는 이력이며 현재 d0cc의 결과와 구분한다. 비공개 글꼴 바이너리와 `.log`는 커밋하지 않는다. PR의 최신 head CI·실제 병합·후속 close는 각각 완료 뒤 기록한다.
