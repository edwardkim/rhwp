# PR #8889 self-review — 잔여 렌더링의 원본 소유·실제 출력 복구

## 최종 판정

**승인.** 정확한 code head의 로컬·Full CI·자체 검토를 완료한 뒤 일반 squash merge로 병합했다.
CI timeout 변경이 있어 옵션 2로 이 기록을 별도 문서 PR에 반영한다. 원 PR에 trailing commit을
추가하거나 branch protection을 우회하지 않았다.

## 접수 정보와 병합

| 항목 | 실제 확인값 |
| --- | --- |
| PR·작성자·base | [#8889](https://github.com/edwardkim/rhwp/pull/8889) / jangster77 / devel |
| 관련 이슈 | [#8856](https://github.com/edwardkim/rhwp/issues/8856), 이번 해결 범위만 종료 |
| code head | `b9d22dfbffc4ccf0dab9bcc9078b30e902a06dab` |
| 검증 base | `3170fddc48cc4fcac0c18761d146b619d0f90c7b` |
| merge SHA·시각 | `050e78455ea22857665973be3cdb68981fd71cb0` / 2026-11-10 12:58:35 KST |
| merge 직전 | OPEN / non-draft / MERGEABLE / CLEAN / reviewer 미지정 |
| 승인·방법 | 사용자 자동 승인 / exact-head 일반 squash / admin 옵션 없음 |

[code CI 원장](../assets/issue8856/postmerge-20261110/code-ci.json)에 head·run·job·판정을 보존했다.
#8857의 원 기여 PR 검토·종료는 기존 완료 이력이며 반복하지 않는다.

## 변경과 선행 해결 검토

서식 말미의 inline 선택지 소유, 표 원형 점선의 pen, 이전 인쇄 세대의 연속 그림자,
그림 soft-edge의 원시 속성·왕복 저장·공통 paint·2024 색/마스크 전달, 명시 대체 기호의
전진 칸, HFT 글머리표 뒤 본문 원점, 캡션 그림 마커, 참조 번호의 언어 슬롯·크기·장평을 보정했다.
원 해결 PR·실제 통합 커밋·원본 바이트/속성·독립 Print·적용 및 비적용 경로는
[최종 증적의 단계별 연결](../assets/issue8856/final-20261110/README.md#원인별-실제-보정과-선행-해결)과
[구현 기록](pr_8889_review_impl.md)에 있다.

표 CIRCLE은 원본 HWPX와 독립 PDF의 실제 원형 점이다. 실선처럼 보인 축소 이미지로
SOLID로 바꾸지 않고, 표용 물리 패턴·두 번 도색과 기존 문단 패턴을 분리했다.
원본 글꼴 대조는 최소 공개 글꼴과 별도 조건이며 픽셀 완전 일치를 주장하지 않는다.

## 검증 결과

- 최소 글꼴 전체 nextest: **13,153 PASS / 0 FAIL / 45 skip**, 334.650초.
  앞선 2개 SVG 조회 실패는 출력100%와 원 해결 커밋을 먼저 확인한 후 조회만 보정했다.
- fmt·Native/WASM32/Native-feature/workspace all-target Clippy·workspace build PASS.
- Native Skia root3,852 PASS/12ignore + 다른 libs182 PASS, Native CI 진입 모듈22 PASS,
  PNG2/PDF35 PASS, manifest·unit tiers·Node 글꼴/브라우저 그림 계약 PASS.
- Studio TypeScript PASS, unit2,784 PASS/2skip. 실제 소비는 별도 PNG·backend·응답 hash로 검증했다.
- Mac fresh WASM은 locked wrapper `--target web --out-dir pkg --no-opt`의 로컬 대체 빌드다.
  Docker 최적화 빌드로 표시하지 않는다.
- 개인 글꼴을 삭제하지 않고 sandbox에서 CoreText528 OS face / 비OS0을 확인했다.
  원본 OTF 공급 보호 대조를 일반 기여자의 최소 글꼴 검사로 혼동하지 않았다.

[전체 로컬 gate](../assets/issue8856/final-20261110/final-gates.json),
[입력 커밋 감사](../assets/issue8856/final-20261110/input-audit.json),
[producer·Runtime 동일성](../assets/issue8856/final-20261110/committed-source-equivalence.json).
실제 빌드 parent와 자산/검사 commit을 구분했다. 최종 code head까지 제품 Runtime2,634개 파일이 같다.

### GitHub Actions

[CI](https://github.com/edwardkim/rhwp/actions/runs/42748908063),
[CodeQL](https://github.com/edwardkim/rhwp/actions/runs/42748907908),
[Render Diff](https://github.com/edwardkim/rhwp/actions/runs/42748908025),
[Adapter](https://github.com/edwardkim/rhwp/actions/runs/42748907906),
[Proptest](https://github.com/edwardkim/rhwp/actions/runs/42748907860),
[정책](https://github.com/edwardkim/rhwp/actions/runs/42748905003)이 모두 성공했다.
기본 회귀 Archive A~E·Lint·Frontend package가 통과했다. 선택적 skipped job을 실행 PASS로 세지 않았다.

첫 Native job은 캐시가 없는 상태에서17개 실제 묶음173건 PASS 후30분 제한으로 취소됐다.
실패 assertion이나 디스크 부족이 아니었다. 시간 상한만45분으로 조정했고 다른 YAML 의미 필드는
동일하다. 로컬 workflow100건·정책95건도 PASS다.
새 Native job은 캐시가 없는 상태에서 **32분24초, 실제22묶음242 PASS/0 FAIL**로 완료했다.
추가3개 라이브러리의0건 선택은 양쪽 실행 건수에서 제외했다.
[시간 예산 보완 근거](../assets/issue8856/native-ci-budget-20261110.json).

## 전쪽 Visual Sweep과 리베이스 보존

Native/fresh WASM 각9조건219쪽, 합계438쪽 전부90% 이상이며 쪽수·누락·중복 불일치는0이다.
최저는 각각92.32561% /92.15037%이며 원본·쪽수·개별 최저·전쪽 TSV·raw·review·overlay는
[최종 전쪽 표](../assets/issue8856/final-20261110/README.md#전체-페이지devel-대조)에 있다.
같은 fixed devel·PDF·글꼴 환경 대비 최대 raw 하락0.04934%p, 2%p 초과0이다.
정본 보호36데이터셋3,542쪽의 공식 두 자리 하락0, 편람 HWPX Native383쪽 gate PASS다.

새 base3170fddc48의 유입은 문서/보안 종료 기록뿐이다. 이전 fixed base b5064fae와 제품 Runtime 및
검증 입력이 같음을 확인해 해당 base의 독립 빌드·전쪽 TSV를 재사용했다. 자동 리베이스·컴파일
성공만으로 기능 보존을 선언하지 않았다. [리베이스 동등성](../assets/issue8856/final-20261110/rebase-equivalence.json).

sample10의 WASM 첫 진단에는 Native가 자동 읽던 원본 외부 그림3개 공급이 빠져 있었다.
기존 `--linked-image`로 같은 tracked 파일·SHA를 공급해763쪽 전체를 다시 산출했다.
PDF·영역·문턱을 바꾸지 않았고 무효 입력 실행은 수용 근거에서 제외했다.
[공급 감사](../assets/issue8856/final-20261110/sample10-linked-assets.json).

## 조판 원칙과 실제 소비 판정

| 검토 항목 | 판정·구체적 근거 |
| --- | --- |
| 사양·원본 소유 | 충족 — 원본 효과·글꼴 언어 슬롯·전진·저장 플래그로 측정/paint를 공유. 문서명/쪽 번호 예외 없음 |
| 읽기/저장/공통 트리/실제 paint | 충족 — soft-edge 각 전달 단계와 실제 Native·Canvas 출력으로 소실 경계를 검증 |
| 정상 경로·반례 | 충족 — flags0/flags8, 표/문단 pen, 원본/명시 대체 프로그램, 외부 그림 유무, screen/print를 구분 |
| 회귀 보호 | 충족 — 실제 FAIL→PASS와 기존 대조군. report95 새 배치는 관계형이며 기존 완료 쪽의 독립±2px는 유지 |
| 검증 입력 commit | 충족 — 원본/PDF hash·producer를 제출 commit과 대조. 비공개 글꼴 바이너리는 포함하지 않음 |
| 분할·이어받기 | 비해당 — 새 분할 알고리즘이 아닌 원본 소유/paint 복구. 기존 nested·긴 표 보호 TSV 하락0 |
| 실제 backend | 충족 — 브라우저28 capture + Native5 실제 PNG. Canvas2D24/CK compat2/CK default2 구분 |
| 미지원 fallback | 충족 — report95 CK 요청은 자원 준비 실패로 Canvas2D. CK 지원 PASS로 주장하지 않음 |

[실제 소비·초기화 hash](../assets/issue8856/final-20261110/actual-consumers/validation.json)에
root pkg·Studio public·실제 JS/WASM 초기화 응답을 연결했다. Vite URL/sourceMap 재작성과
WASM 바이트를 구분했다. 같은 쪽 raw/review/overlay로 request 선택지, NewYear 마커,
report95 참조·110쪽 bullet 뒤 본문, 표 원형 점과 그림 외곽을 확대 판독했다.

최소 공개 글꼴의 실제 윤곽 차이와 raster AA를 분리했다. 같은 원본 Print 공급에서는
interview99.99453%, KTX99.94065/99.94015%, report95·96·110은99.98% 이상이다.
clip5587의 원 Print 조건4쪽 전체는 Native98.75608% /WASM98.76342% 이상,
legislation의 명시 Print 대체 조건10쪽은 양쪽100%다. 이를 최소 환경의 수치로 대체하지 않았다.

## Merge 후 contributor PR comment 계획

원 코드 [#8889](https://github.com/edwardkim/rhwp/pull/8889)와 [#8856](https://github.com/edwardkim/rhwp/issues/8856)에
merge SHA·실제 CI·위 검증 범위·글꼴 조건 구분을 한국어 존댓말로 게시한다.
[Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)을 연결하고,
해당 주장과 같은 문서·쪽의 raw·review·overlay를 사용한다. 대표 request1과 report95는
`../assets/issue8856/final-20261110/focus/{native,wasm}/{request,report-628}/`의
`raw-001.png`, `review-001.png`, `overlay-001.png` 또는095 파일이다.
다른 주장도 같은 final focus/단계별 자료를 연결하며 다른 페이지의 그림으로 대체하지 않는다.
raw 이미지 URL은 `https://raw.githubusercontent.com/edwardkim/rhwp/050e78455ea22857665973be3cdb68981fd71cb0/`
뒤에 정확한 저장소 자산 경로를 붙인다. 게시 전 hash·이미지 응답과 devel 존재를 확인하고
`--body-file`로 게시한 뒤 API 본문과 표시를 재확인한다. #8857의 원 PR 종료/댓글은 반복하지 않는다.

자동 close된 #8856의 CLOSED를 확인했다. [duration 갱신](https://github.com/edwardkim/rhwp/actions/runs/42752855786)은
success이며 실제44개 target을 갱신했다. [후속 운영 증적](../assets/issue8856/postmerge-20261110/postmerge.json)에
실제 run·job·로그 요약을 보존했다. 검증 workflow를 병합 후 재실행하지 않았다. 종료 시 루트 output·공유 debug 및 작업 전용 branch를 정리하되
원본·PDF·글꼴·다른 profile과 다른 작업은 보존한다.
