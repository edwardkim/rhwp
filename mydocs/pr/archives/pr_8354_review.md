# PR #8354 리뷰 — fix(studio): 비보안 컨텍스트에서 crypto.randomUUID 없이 초기화한다 (#8353)

## 최종 판정

**메인터너 보정 후 수용 가능 — 비보안 컨텍스트 세션 초기화 검증 완료.** [통합 PR #8438](https://github.com/edwardkim/rhwp/pull/8438)에 반영됐으며, 검토한 원 PR head와 현재 head가 일치한다.

## 2026-11-02 통합 머지 사실과 적용 SHA 재확인

[통합 PR #8438](https://github.com/edwardkim/rhwp/pull/8438)은 `b8338ba38be288b6e9385f5372d184649323aa9f`로 머지됐다. 아래 체리픽 기록의 종전 통합 SHA는 리베이스 전 이력이다. 머지된 이력에서 원본과 커밋 제목이 일치하는 대응은 다음과 같다. 이는 변경 내용의 완전한 동등성이나 개별 시각 판정을 대신하지 않는다.

- 원본 → 머지 이력: `f36af4417f` → `9e39a7bd8e`, `0c3bbe9c1a` → `05548edfcc`.
- 제품 코드 `d536455b4e`의 최소 글꼴 전체 회귀는 12,368건 통과·45건 skipped였다. 이 로그에서 확인한 해당 모듈: 이 문서에서 지정한 Rust 모듈 없음(해당 실행 경로 별도 확인). 이 수치는 개별 PR의 독립 Print 검증과는 구분한다.
- 뒤의 중간 후보·검증 대기 문장은 당시 상태이며 최종 통합 결과가 아니다. 원 PR별 최종 시각·기능 검토와 최신 head 일치 확인은 계속한다.

## 최종 기능 검증

`command-error-contract.test.ts`는 `crypto.randomUUID`가 없는 상태에서 경계 객체가 `ready`가 되고 두 세션 ID가 서로 다른지 검사한다. 최종 제품 후보의 Studio Node 전체 실행에서 `boundary initializes without crypto.randomUUID in insecure contexts (#8353)`가 통과했다. 같은 실행에서 안전한 난수 경로의 EditorTransport 세션 테스트도 통과했다. 전체는 2,715건 통과이며, 이 변경은 조판 출력 변경이 아니므로 Visual Sweep 대상이 아니다. 코드 head의 [CI](https://github.com/edwardkim/rhwp/actions/runs/41322587728)와 통합 머지를 확인했다.

## 접수와 적용

- 원 PR: [#8354](https://github.com/edwardkim/rhwp/pull/8354), 작성자 `planet6897`, reviewer `jangster77`.
- 원 PR head 조회값: `0c3bbe9c1a8a7709a2a2e445338bdbce87c8f52f`. 조회 시 원 PR head와 마지막 체리픽 원본이 일치한다.
- 마지막 적용 원본: `0c3bbe9c1a8a7709a2a2e445338bdbce87c8f52f`. 검증 시작 당시 코드 후보: `abea5b028f8f135bdf000ca700e1993ebf0b1973`, 기준 `upstream/devel`: `bc9dfdd057258de59afebeb5121c4bee7398e07f`.
- 관련 이슈: [#8353](https://github.com/edwardkim/rhwp/issues/8353).
- 선행 PR: 별도 누적 선행 PR이 명시되지 않았다.

후속 `e1f03ffbb3`은 Python 테스트 준비, `af175315e7`과 `e920cba6b0`은 회귀 검사 주석, `a63bb719de`는 검증 경과를 기록했다. `5d1294811b`은 전체 그림의 잘림 오판을 보정했다. `e1b8ab0f54`, `d6fe30c3ad`, `0897582f6c`, `8c4eca265e`에 Native 진단, #8410 집중 6건 통과와 글꼴 대조를 기록했으며 새 코드의 전쪽 출력 검증이 필요하다.

## 변경 주장과 검토 관점

비보안 컨텍스트에서 randomUUID가 없어도 Studio 세션을 초기화한다.
브라우저 세션 식별자 생성과 오류 전달 경로를 확인한다.

## 체리픽과 보정

- 원본 `f36af4417feecc1fefc1f9e5a16a26adc140846f` → 통합 `a685b1dd8984febf1179b243759ee6797261b7d1`.
- 원본 `0c3bbe9c1a8a7709a2a2e445338bdbce87c8f52f` → 통합 `fcdcb794af685ca31581aa5b453a5519da988c22`.

이 PR의 원본 커밋 적용 중 별도로 기록된 충돌은 없다.

## 검증과 증거

이전 전체 Rust 로그에서 이 PR 고유 모듈의 실행 건수를 확인하지 못했다. 관련 경로를 새 후보에서 별도로 확인한다.
- 관련 검사: [`command-error-contract.test.ts`](../../../rhwp-studio/tests/command-error-contract.test.ts).

기존 생성 목록으로 최소 글꼴 회귀 12,320건 통과·45건 건너뜀을 확인했다. 생성 목록에서 누락된 #8410 두 모듈의 6건을 목록 재작성 후 같은 환경에서 별도로 실행해 모두 통과했다. 두 실행을 구분하며 한 번의 전체 실행으로 기록하지 않는다.

2026-11-01 고정 후보의 #8389·#8397 전쪽 정밀 대조에서 2,153행 중 Native 12행, WASM 8행이 공개 기준보다 낮았다. 이 수치는 이후 GIF·Canvas 보정 전 결과이며 최종 후보의 재검증 결과가 아니다. #7905 3쪽은 GIF 오른쪽 경계 누락, sample16 35쪽 WASM은 메타파일 내부의 미대응 3픽셀이 확인됐다. #8040 두 원본은 Native·WASM 전쪽이 90% 이상이나 Native 정밀 기준 하락이 남는다. 진행과 근거는 [작업 기록](../../working/task_m100_planet6897_system_fonts_20261031.md)을 따른다. 하락 보정과 전쪽 대조, 각 PR의 독립 Print·Native/fresh WASM 및 직접 판독을 완료해야 한다. #8410·#8412의 새 base 기능도 별도로 보존 검증한다.

원 PR의 최신 head·required check와 통합 후보의 새 검증 결과를 확인하기 전에는 승인이나 merge를 진행하지 않는다.

## Merge 후 contributor PR comment 계획

통합 PR #8438의 merge `b8338ba38be288b6e9385f5372d184649323aa9f`, 코드 head CI와 해당 원 PR의 기여·메인터너 보정을 구분해 존댓말로 안내한다. `crypto.randomUUID`가 없는 비보안 컨텍스트에서 ready 상태와 세션 식별자 구분이 통과한 Studio 검사를 설명한다. 조판 변경이 아니므로 이미지를 사용하지 않는다. 게시 시 명시적 지시를 확인하고, 게시 뒤 API로 본문과 링크를 재조회한다.
