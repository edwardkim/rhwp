# 최신 리베이스 이전의 검증·분석 이력

이 문서는 이전 source의 실행과 당시 남은 조건을 보존한다. 현재 진행 상태는 [증거 목록](README.md)을 따른다.
과거의 진행 중·최신 등의 표현을 현재 head의 상태로 사용하지 않는다.

# planet6897 원 PR 39건 통합 검토 증거

PR 생성 전 승인 대기까지 준비하는 로컬 작업이다. 검토 범위는 접수한 원 PR 39개로 고정했으며,
새 PR을 추가로 체리픽하지 않는다. 원 PR별 입력·주장 쪽·원 head와 메인터너 보정을 분리해 기록한다.
원격 PR 생성·push·review·comment·close는 수행하지 않았다.

## 추가 보정의 실제 완료 범위 — 2026-11-05

제품 코드 `c183a1ac61c2aa4188989fd93032cae68d3538c1`에서 기본 개요 예약 폭과 저장 줄
재시작 표의 하단 보정을 검증하고 있습니다. 같은 원본·기존 독립 Print의 13개 조합·73쪽은
Native/fresh WASM 모두 완료했고 최저 95.98%·96.12%, 이전 표시값 하락은 양쪽 0쪽입니다.
[번호 전쪽 증거](intermediate-evidence-summary.md),
[같은 fresh WASM 증거](intermediate-evidence-summary.md),
[표 하단과 이어받기 증거](intermediate-evidence-summary.md)를 구분합니다.

[최소 글꼴 집중 회귀 6개](rust-validation-c183a1ac61.md)와
[실제 최적화 WASM의 공개 API·브라우저 검사](runtime-validation-c183a1ac61.md)는 통과했습니다.
최신 Native의 [80개 입력·406쪽 전쪽 감사와 TSV 원본](current-c183a1ac61/native-validation.md)은
완료했습니다. 최저 raw는 92.45232%이며 90% 미만·누락·쪽수·입력/PDF 해시 오류는 0건입니다.
[당시 고정 base와의 동일 환경 대조](current-c183a1ac61/native-validation.md)에서도
비교 가능한 396쪽의 0.1%p 초과 하락은 0쪽입니다. base 파싱 거부 6쪽의 출력과
누락 4쪽의 복원은 수치 비교와 구분합니다. 최신 devel 전체를 새로 출력한 결과로 표시하지 않습니다.
[최신 제품의 전체 최소 글꼴 Rust 회귀](rust-validation-c183a1ac61.md)는
12,563/12,563 PASS·45 skipped·exit 0으로 완료했습니다.
[Rust·Native 집중 검사 40개 묶음](rust-validation-c183a1ac61.md)은 모두 exit 0이며,
Native Skia 라이브러리 4,040 PASS·12 ignored와 CI integration 21개 모듈을 포함합니다.
[별도 필수 편람 Native 383쪽 게이트](current-c183a1ac61/native-validation.md)도
최저 99.96%, 기존 기준 표시값 하락 0쪽으로 통과했습니다. fresh WASM 80개 입력의
[전쪽 80입력·406쪽 감사](current-c183a1ac61/wasm-validation.md)는 완료됐으며 최저 92.87288%, 90% 미만·누락·쪽수·해시 오류 0건입니다.
[PR #8389·#8397의 Native 보호 범위](current-c183a1ac61/pr8389_8397/native-validation.md)는
17문서·1,770쪽과 원 TSV 2,153행을 모두 대조해 표시값 하락·누락 0행입니다.
`issue7905` 6쪽도 전쪽 98% 이상이며 최저 98.86%입니다.
[새 base 영향 범위의 Native 3문서·7쪽 실제 대조](current-c183a1ac61/rebase-scope-native-validation.md)도
0.1%p 초과 하락 없이 완료했습니다. fresh WASM의 해당 최종 대조는 완료 전입니다.
아래 직전 완료 결과를 새 보정 제품의 전체 통과로 표시하지 않습니다. PR 생성·원격 작업은
수행하지 않았으며 최종 준비 후 작업지시자 승인을 기다립니다.

직접 판독 원장은 현재 [158건](direct-reading-current.md)입니다. 실제로 연 review·별도 overlay만 기록하며, 한 PR의
전체 검증 완료 건수와 혼동하지 않습니다. 기존 producer `4b149b6cdf`의 PNG 재사용은
최신 제품의 재출력과 구분합니다. PR #8475의 세로쓰기 병합 칸에서 마지막 글자가 보이지 않는
차이와 PR #8512의 상자 선 표현, PR #8535의 각주 번호 위치 차이를 확인해 기록했습니다.
이 기록을 완전 일치나 최종 승인으로 승격하지 않습니다.

PR #8475는 별도 보정 대상으로 유지합니다. 새 관계 검사 2개가 기존 제품 소스에서
실제로 실패하며 세로줄 분산·칸 밖 글자 배치를 재현했습니다. 초기 검증 폴더의 자료 누락으로
발생한 컴파일 오류는 이 재현 결과에서 제외했습니다. 보정 소스 `37e5bba300`은 별도
worktree에서 검증 중이며, 위 `c183a1ac61`의 통과 결과로 대신하지 않습니다.
[기본 옵션의 집중 검사](pr_8475/vertical-axis-focused-37e5bba300.md)는 같은 검출 함수의
수정 전 2 FAIL·수정 후 2 PASS와 기존 대조 5 PASS를 확인했습니다. 보정 후 실제
Native/fresh WASM 전쪽은 각각 6입력·29쪽을 완료했습니다. 두 문제 쪽의 직접 판독도
마쳤으며, 실물·정상 대조의 WASM 판독과 최종 공통 검증은 아직 완료 전입니다.

## 추가 보정 전의 완료된 공통 검증 — a7e5ce7112

| 구분 | 실제 source·범위·상태 |
| --- | --- |
| 최신 검증 기준 | `upstream/devel` `22947f6a90c998e95fa51238d5b907a0ebd37162` |
| 보정 전 필수 검증 제품 | `a7e5ce7112ea8b227c317a9cbfb9d6fed0b00973` |
| 전쪽 시각 측정 producer·당시 base | `4b149b6cdfc514b9858ed644be2cf8120c731cd4` / `7fdd5441ff1e77c083818e920a48089f294de35e` |
| 전쪽 fresh WASM 실제 build head | `7bb9e66fd006109ca9f7ee4c4d3f752c82e60fae` — 위 제품과 runtime source 동일성 확인 |
| 보정 전 기본 전체 Rust 회귀 | 12,561/12,561 PASS, 45 skipped, exit 0 |
| 보정 전 Native Skia | unfiltered library 4,040 PASS·12 ignored, CI integration 모듈 21개 모두 exit 0 |
| 보정 전 Rust 기타 게이트 | fmt·suite/tier·workspace all-target Clippy·WASM Clippy·workspace build·doc test 모두 exit 0 |
| 전쪽 Native | 80입력·406쪽, 최저 91.85%, 90% 미만·누락·쪽수·입력/PDF 해시 불일치 0건 |
| 전쪽 WASM | 중단 입력 36개 복구 완료. 80입력·406쪽, 최저 raw 92.03052%, 90% 미만·누락·쪽수·입력/제품 해시 오류 0건 |
| PR #8389·#8397 보존 범위 | Native/fresh WASM 각각 17문서·1,770쪽, 원 TSV 2,153행 표시값 하락 0행 |
| 보정 전 제품 후속 검사 | 최적화 WASM·실제 브라우저/API 통과. 새 base/후보와 리베이스 전 후보의 영향 3입력·7쪽씩, 총 14쪽 raw 동일. 당시 base의 전쪽 WASM 대조는 진행 중 |
| 개별 리뷰 | 39개 모두 현재 증적·최신 검증/대기 상태 갱신. 조건부 수용 기록이며 최종 승인 확정 0개 |

[전쪽 Native/fresh WASM 산출물 감사](intermediate-evidence-summary.md),
[Rust 34개 실제 성공 게이트](intermediate-evidence-summary.md)와
[원 PR 39개 head 재확인](source-head-recheck-20261105.tsv)을 연결한다. 모두 Open·non-Draft이고
리뷰어 요청이 유지되며 원 head 변경은 없다. 현재 devel은 PR metadata의 `base.sha`가 아닌
실제 `git/ref/heads/devel`을 따로 조회해 확인했다.

충돌 없는 리베이스 뒤 전체 비교를 반복하지 않는 작업지시자의 범위를 따른다. 재사용한 전쪽
TSV·PNG는 원래의 producer·base·package 해시를 유지한다. 최신 source에서 재출력한 증거처럼
표시하지 않는다. 새 base의 기능은 영향 대상과 정상 대조군을 최신 제품과 새 기준 제품으로
실제 실행해 확인하며, 기존 보정 대상 PR #8498의 영향 5쪽은 리베이스 전 후보와 대조한다.
최신 후보 집중 범위는 3문서·7쪽이고 새 base 자체 대조는 그중 2문서·2쪽이다.

[최신 제품 실행](intermediate-evidence-summary.md),
[새 base 제품과의 실제 14쪽 대조](rebase_scope_22947f6a90/README.md)와
[리베이스 영향 경계의 실제 전쪽 TSV·대표 PNG](rebase_scope_22947f6a90/README.md)를 별도로 보존한다.

Native/fresh WASM 대표 증적의 중복 제외 93개 입력·쪽 조합은 실제 원본·PDF·backend·source PNG 해시와
대응함을 확인했다. 파일 대응 감사와 직접 판독·최종 승인 판정은 구분한다. [상속 기능 WASM 32입력·72쪽 대조](intermediate-evidence-summary.md)는 같은 원본·PDF·글꼴
프로그램·공급 순서와 출력 조건에서 정밀 raw가 72쪽 모두 같다. 주 검증 대상 80입력의
WASM base 대조는 진행 중이다.

설치 글꼴이 없는 환경의 회귀와 전체 시스템 글꼴을 공급한 독립 Print 시각 측정은 별도 계약이다.
원본·독립 PDF·실제 글꼴 공급/대체·DPI·프로필·조판 세대와 Native/fresh WASM 해시는 최종
출처 기록에서 구분한다. 폰트 파일을 추가하거나 재배포하지 않는다.

아래는 이전 source의 검사와 판단 이력이다. 최신 source의 통과 결과나 현재 남은 작업과 혼동하지 않는다.

## 이전 source의 실행·보정 이력

### 코드와 실행 파일

- 기준 devel: `7d06d5c706f1d89f1662e739abf2b1b25d2a98e4`.
- 제품 코드: `dc87f4fa2e`. 이후 `fd97ae040f`는 두 호환 모드 회귀와 문서·증적 보강이며
  제품 코드를 바꾸지 않는다.
- Native Skia 실행 파일 SHA-256:
  `24c7e929b04c14adc46fbcc80b384e74fffd5524bdd2484068ebe6e974952337`.
- 같은 코드의 최적화 완료 fresh WASM SHA-256:
  `c7e166e1b087641e727d326dd5ff504653f96134808242f9d1fd83228bbae903`.
- fresh JS SHA-256:
  `219cdf77a61e995569350fc81c8258f104bd177301a80bfe3a59670ff3804a06`.
  `rhwp-studio/public/rhwp.js`와 같은 바이트다.

### 현재 검증 상태

2026-11-04, 96 DPI, 색상 경계 보정 전 2px 내용 실루엣을 사용한다.
직접 대응시킨 78개 입력/PDF 쌍의 Native 404쪽은 모두 90% 이상이다.
별도로 접수한 PR #8540의 A/B 개요 문서도 Native 각 1쪽 100.00%다.
fresh WASM은 완료한 입력부터 개별 리뷰에 연결하고 있으며 아직 전체 완료 판정은 하지 않는다.
최종 PR 제출 전에 현재 코드의 모든 필수 TSV와 대응 PNG로 수집을 마친다.

2026-11-04 후속 확인에서 개별 리뷰의 80쌍(직접 78쌍과 편람 2쌍), 각 경로 1,170쪽의
Native/fresh WASM TSV를 모두 확보했다. 누락·90% 미만은 양쪽 모두 0쪽이며 최저값은
Native 90.29%, fresh WASM 90.59%다. 별도 PR #8540의 두 입력은 Native/fresh WASM
각 1쪽 100.00%다. 이는 제품 코드 `dc87f4fa2e`의 리베이스 전 결과다.

검토 중 `upstream/devel`이 `1924d371d6757392d063b8dc2e9bbe9a298be817`로 전진했다.
PR #8565의 Native 웹폰트·측정 경로와 PR #8568의 기록을 반영한 뒤 기능 보존을
검증한다. `7d06d5c706` 비교를 새 base 비교로 표시하지 않는다. 구 base의 진행 중이던
추가 비교와 대기 lint 큐는 새 base/head 검증으로 교체하며, 이미 완료한 결과는 보존한다.

[입력·PDF SHA 대조표](input-integrity.tsv)의 148개 파일은 증적 고정 커밋
`fd97ae040f`의 내용과 일치한다. 기존 입력과 PDF는 같은 저장소 파일을 재사용한다.
실제 한컴 Print 출처와 제품 버전은 해당 원 PR 리뷰 및 기존 기준 자료의 출처 기록을 따른다.

Native/fresh WASM 시각 비교에는 설치된 시스템 글꼴 전체를 공급한다. 회귀는 상용·개인
글꼴을 숨기고 저장소의 허용된 기본 글꼴 5개만 노출한 환경에서 별도로 실행한다.
글꼴 프로그램은 이 증적에 포함하지 않는다.

### 기존 해결 범위 보존

[PR #8389·#8397 전쪽 비교](pr8389_8397/README.md)는 기존 TSV의 모든 기준 행을
대조한다. Native는 원 PR 기준 2,153행 모두 유지·개선했다. 직전 통합 기록 일부의 수치가
동일 코드 재측정과 다른 사실은 PNG SHA 대조와 함께 해당 기록에 명시했다.
현재 devel과의 Native/fresh WASM 대조 및 최종 Rust·lint 검증은 완료 후 연결한다.

### 최종 회귀와 base 대조 진행 기록

최소 글꼴 환경의 전체 Rust 검사는 `fd97ae040f`에서 12,498개를 실행해 12,497개 통과,
1개 실패, 45개 건너뛰기로 끝났다. 실패한 `body_overflow_baseline` partition 14는 새 실물
입력 `issue8470/42565_입법예고문.hwp`의 기준 누락이었다. 원 PR의 세로쓰기 표 나눔 금지와
독립 Print의 16쪽·표 끝을 대조해 [보정 근거](body-overflow-42565-proof.md)를 남겼다.
`70a9d870ee`에서 해당 입력의 body overflow 5건만 등록한 뒤 같은 최소 글꼴 환경의 해당
검사 1개가 통과했다. 기존 입력의 허용치나 검사 구현은 바꾸지 않았다. 새 샘플 68개를
명시적으로 공급한 보안 corpus 검사도 6개 모두 통과했다.

이후 Native Skia 라이브러리 검사도 통과했다. 이번 통합이 바꾼 회귀 파일 40개의 첫머리에
관계형·고정형 구분, 실제 갱신 시각과 직전 파일 커밋, 현재 원본별 전쪽 최저값·주장 쪽 수치와
증거 경로를 보강했다. 모든 파일에서 `//!` 주석을 제외한 내용이 직전 커밋과 같음을 확인했다.
이 주석 보강 때문에 전체 Rust 검사를 반복하지 않으며, 새 devel의 제품 변경에 대한 검증은
별도로 완료한다.

[Native base 비교](base-native-comparison.tsv)는 직접 대응 입력 406쪽을 대조한다.
양쪽 수치가 있는 396쪽에 2%p 초과 하락이 없다. 나머지 10쪽은 미실행이 아니다. base가
선언 0·실제 1 문단 글상자를 거부한 두 문서의 6쪽과, base의 잘못된 쪽수로 없었던 4쪽이
후보에서 복원된 경우다. 이 행에 가상의 0% 기준값을 넣지 않고 실제 base 상태를 기록했다.
fresh WASM의 같은 대조와 Native Skia·lint는 아직 완료 전이다.

### 실제 가져오기·편집 경로

[WASM 실제 API 대조](wasm-producer-parity.md)는 최적화한 fresh package에서 PDF
가져오기 3종과 EMC 문서의 실제 글자 삭제·HWPX 저장을 실행한 결과다. 네 산출물이
Native의 같은 API 동작 산출물과 바이트 단위로 같아 해당 HWPX의 전쪽 시각 검증을
공유한다. 렌더링만 실행한 결과를 가져오기·편집 성공으로 대신하지 않는다.

[Native 변경 전후 대조](native-producer-before-after.md)는 같은 원본에 base importer와
editor를 실행한 산출물을 별도로 비교한다. 2010·2020판은 정상 대조군으로 바이트가 같다.
무표기본의 제목→본문 간격은 base의 2808HU에서 원본 PDF의 2424HU(5쪽)·2364HU(13쪽)로
맞춰졌으며, 실제 생성 HWPX의 저장 줄에서도 같은 차이를 확인했다.

PR 생성은 작업지시자의 별도 승인 전까지 대기한다. 로컬 검증·증적과 개별 리뷰를 먼저
완료하며, 현재 기록은 최종 승인이나 원 PR의 닫기를 뜻하지 않는다.

## 세로쓰기 보정 Native 증거 추가 — 2026-11-05

PR #8475의 별도 보정 소스 `37e5bba300`을 실제 컴파일한 Native 제품에서 두 문제
입력의 각 5쪽은 모두 100.00%다. 6입력·29쪽 대조에서 두 문제 쪽만 개선됐고 나머지
27쪽은 직전 `c183a1ac61` 후보의 PNG 바이트와 같다.
[전쪽 대조](pr_8475/vertical-axis-37e5bba300/native/README.md)와
[주장 쪽 review·overlay 및 직접 판독](pr_8475/vertical-axis-37e5bba300/native/README.md)을 보존했다.
이 보정은 아직 통합 브랜치의 제품 코드에 적용하지 않았으며 fresh WASM과 해당 소스의
최종 필수 검증은 진행 중이다. 기존 `c183a1ac61` 전체 검증을 새 보정 소스의 통과로
표시하지 않는다.

## 세로쓰기 보정 fresh WASM 전쪽 추가 — 2026-11-05

[보정 source 37e5bba300의 fresh WASM 전쪽 TSV](pr_8475/vertical-axis-37e5bba300/wasm/README.md)를
확보했다. 두 축약 입력의 각 5쪽은 모두 100.00%이며, 대조군을 포함한 29쪽은 모두 90%
이상이다. 두 문제 쪽만 개선됐고 나머지 27쪽은 이전 c183 WASM PNG와 바이트가 같다.
주장 쪽 fresh WASM review/overlay의 판독과 해당 source의 전체 Rust 검증은 아직 진행 중이다.

보정 source 37e5bba300의 Native/WASM 대표 판독은 두 문제 쪽·실물 10/11쪽·정상 대조
1쪽씩을 모두 완료했다. 실제 관측은 [판독 목록](direct-reading-current.md)에 구분했다.
필수 전체 Rust·통합 적용·최신 base의 남은 확인을 완료하기 전에는 최종 승인으로 바꾸지 않는다.

## 세로쓰기 보정의 필수 편람·doctest 복구 완료 — 2026-11-05

[보정 source 37e5bba300의 필수 편람 383쪽 게이트](pr_8475/vertical-axis-37e5bba300/handbook-validation.md)는
최저 99.96%, 기준 하락·fallback 0건으로 통과했다. ENOSPC로 실패한 첫 집계는 성공으로
계산하지 않았으며 같은 완료 구간의 canonical resume으로 결과를 복구했다. 같은 코드의
doctest도 동시 링크만 제한해 8 PASS·3 IGNORE·exit 0으로 완료했다. lint·workspace
빌드는 통과했고 전체 Rust·Skia·CI 통합 묶음은 실패 단계 뒤에서 이어 실행 중이다.
