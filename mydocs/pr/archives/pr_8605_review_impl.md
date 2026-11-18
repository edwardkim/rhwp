# PR #8605 통합 적용·메인터너 보정 기록

## 통합 PR의 exact code head CI

- [통합 PR #8857](https://github.com/edwardkim/rhwp/pull/8857), code head `29c74b69851bc6578f10a2c78e23592a05b69a5d`.
- [CI 실행](https://github.com/edwardkim/rhwp/actions/runs/42621490405)과 같은 code head의 CodeQL·Render Diff·Adapter·Proptest·정책 검사를 실제 확인했습니다. 기록 확인 시각: `2026-11-09T22:58:24.689803+09:00`.
- 기본 기능 worker A–E의 실제 nextest는 **12,934 PASS / 0 FAIL / 45 skip**입니다. 기존 로컬 13,105건과 실행 범위·HEAD가 다르므로 합산하지 않습니다.
- source·test·workflow를 추가로 변경하지 않고 이 검토·오늘할일만 단일 부모 trailing commit으로 보완합니다. trailing head의 실제 CI/fast-pass와 mergeability는 push 뒤 별도로 확인하며, 문서만 바뀌었다는 이유로 통과를 미리 가정하지 않습니다.
- [잔여 이슈 #8856](https://github.com/edwardkim/rhwp/issues/8856)은 9문서군의 글리프·간격·도형 및 clip5587 Native 4쪽의 엄격 0.1%p 미충족을 계속 추적합니다. 부분 해결 #8635·#8636과 부모 #7445는 일괄 종료하지 않습니다.


## 최종 source와 재사용 범위

제출 code head `2e262d0d4b2885b130e2db917b86e894710425e4`, base `e00f9b7e26e581896fa17eaff3c8eb3f4889f9d4`입니다. 아래 전체 회귀·전쪽 수치는 원 실측 HEAD의 증거를 재사용한 기록입니다. 최신 집중 검사와 페이지별 producer, 엄격 0.1%p 잔여 한 건은 [재사용·최종 판정](../assets/planet6897_20261106/final-b768bee1-20261109/validation-reuse.md)을 따릅니다. 보호 TSV 표시값 하락은 0입니다.

## 최종 구현·검증 결과

검증 code head `eb876d1416fb558c874f9efec65c5722a6769751`, base `b768bee1c4c0023087687f79a0a95f12108743e9`에서 원 기여와 메인터너 보정을 함께 검증했습니다. 최소 글꼴 전체 회귀 13,105 PASS / 0 FAIL / 45 skip, Native 전용 73 PASS이며, Native/fresh WASM 각각 94조건·2,995쪽의 미달·누락·보호 표시값 하락은 없습니다. 원 PR의 개별 입력·페이지 판독과 보정 내용은 [최종 review](pr_8605_review.md)에 연결합니다. 원격 통합 PR의 최종 CI와 병합은 별도 단계입니다.

## 단계별 기록의 시점

아래 R 단계와 과거 실행의 진행 중·실패·추가 검증 표현은 해당 시점의 원인 추적 기록입니다. 과거 결과를 현재 code head의 통과로 합산하지 않으며, 현재 상태는 위 최종 결과와 연결된 review를 기준으로 판단합니다.

## 대상과 단계

[원 PR #8605](https://github.com/edwardkim/rhwp/pull/8605)의 원 head는 `5d37286267136c9978d7ac2f804913730ab6becf`다.
[공통 계획](../assets/planet6897_20261106/integration-plan.md)의 27건 누적 순서를 따른다.
원 PR별 입력·주장·회귀와 공통 보정을 분리하고, 원 기여 SHA와 적용 SHA를 아래에 기록한다.

| 원 commit | 적용/보존 결과 | 제목 |
| --- | --- | --- |
| `4f8570309e925468552036f4c20d099011d55a35` | `45c2e2a27034ab415e728d4d3601f1fb2eac9bd4` | fix(render): WASM Canvas 합성 굵기를 실제 face 윤곽의 fill+stroke로 그린다 (#8430) |
| `3d0b4420dbee220253189fc149c48125cc0e2dc4` | `62ce18ddf8d751c3c40f41706cb9c3c54aee361b` | docs(#8430): 30396 권고문 Native/fresh WASM 전쪽 TSV·확대 판독·글꼴 예외 증적 |
| `5d37286267136c9978d7ac2f804913730ab6becf` | `56a863b98538cf8938e97be401e7e4964881de35` | docs(#8430): 최신 devel rebase head 재측정과 글꼴 예외 source SHA 갱신 |

## 최종 리베이스 후 적용 상태

검증 base `b768bee1c4c0023087687f79a0a95f12108743e9`, 검증 제품 head
`eb876d1416fb558c874f9efec65c5722a6769751`의 실제 Git 이력을 대조해 위 적용 SHA를 갱신했다.
[고유 56개 최종 적용표](../assets/planet6897_20261106/contributor-commit-map-final.tsv)는 원 기여 commit의
cherry-picked footer와 현재 적용 commit을 일대일로 확인한 결과다. 중복 patch·bridge의 기존 처리는 유지한다.
이번 b768bee1 리베이스에는 실제 파일 충돌이 없었습니다. index.lock 경합이 끝나고 lock이 없는 상태에서 계속했으며 lock을 삭제하거나 todo를 바꾸지 않았습니다. 제품 src/crates/tests/scripts와 지침의 바이트는 이전 후보와 같고 의존성 변경에 대한 새 검증을 수행합니다.
이력 확인과 현재 제품의 기능·회귀·시각 검증 판정은 구분하며, 필수 검증 완료 전에는 승인하지 않는다.

## 완료 조건

충돌·컴파일 보정 → 원인별 전체/집중 회귀 → 같은 원본·독립 Print Native/fresh WASM 전쪽 TSV와 직접 확대 판독 → 필수 lint/실제 backend → 개별 최종 리뷰 순서다.
실행하지 않은 항목은 통과로 쓰지 않고, 보정 코드는 통합 브랜치의 일반 커밋으로 남긴다.


## R34 실제 검증 완료 범위

원 HFT 대응 OTF의 CFF 굵기 경로를 실제 최적화 WASM에서 확인했다. 원래 TTF 합성 굵기와
새 CFF 경로, Screen·옛한글 대조군은 3개 집중 브라우저 검사에서 통과했고, 기존 CI 집계에
연결한 실제 Canvas 글꼴 검사 16건도 통과했다. 새로운 원 CFF 검사는 소유 fixture를 명시적인
source family에 연결하고 실제 공개 편집 API로 굵기를 설정한다. SVG의 해당 글자 stroke로
합성 굵기 경계에 진입했는지 확인하며, 페이지 테두리의 stroke를 근거로 삼지 않는다.

추가 보호 대조군인 sample16 HWPX는 같은 독립 2024 Print와 Native/fresh WASM 모두 64쪽이
일치한다. 전쪽 최저는 Native 99.99% / WASM 99.99%이고, 기존 기준 64행의 표시값 하락은 0행이다.
지적한 40·51쪽은 양쪽 100.00%다. 이는 원 PR의 다른 문서 주장에 대한 증거와 구분한다.
[전체 TSV·같은 쪽 review/overlay·재현 조건](../assets/planet6897_20261106/protected-r34/sample16-hwpx/README.md).

제품 source는 99f1340628이며 이후 검사 assertion과 문서를 보완한 commit은 실제 제품 실행과
구분한다. 전체 27개 PR 최종 판정은 남은 전체 WASM·Rust·lint 완료 후 갱신한다.


## R35·R36 원본 유형과 실제 CFF 프로그램의 불일치

2022 HWP의 51쪽 하락을 같은 공개 API와 독립 PDF로 재현했다. HWPX 저장 타입만으로
원본을 TTF 프로그램이라고 판단하지 않는다. 기존 `f70a8c937d3`은 HWP5 미정의 비트로
TTF를 확정하지 않는 정상 정책이며 유지했다. 실제 등록된 CFF를 소비하고 원 TTF 별칭의
글리프 소유를 우선 보존하도록 공통 paint 경계를 보정했다.
[원 PR·통합 커밋·원본 바이트·독립 PDF 윤곽·수정 전 실패와 반대 대조군](../assets/planet6897_20261106/cff-program-selection-analysis.md).

R36 제품 `3f2a7c96ee`의 실제 2022 HWP 51쪽은 **100.00%**이며 이전 99.90% 하락을
복구했다. [같은 쪽의 TSV·review/overlay·직접 판독](../assets/planet6897_20261106/protected-r36/sample16-hwp2022/README.md).
실제 HFT/TTF/미확정 CFF 소비, 실제 TTF 별칭, Screen·옛한글을 포함한 집중 브라우저
3건이 통과했다. 원 PR의 30396 권고문과는 구분한 추가 보호 대조군이다.
전체 27개 PR의 전쪽 검증·최종 Rust·lint와 2024 대조군은 실제 완료 뒤 갱신한다.

## R36 세 sample16 보호 입력의 전쪽 확인

2022 HWP·2024 HWP·HWPX의 각 64쪽을 실제 R36 fresh WASM으로 완료했다. 전쪽 최저값은
100.00%·99.99%·99.99%, 기존 기준 192행의 표시값 하락은 0행이다.
[입력별 Native/fresh WASM 전쪽 TSV·해시](../assets/planet6897_20261106/protected-r36/sample16-full-provenance.tsv)를
따른다. 원 PR 주장 문서의 검증과 통합 최종 회귀·lint는 여전히 별도 완료 조건이다.
