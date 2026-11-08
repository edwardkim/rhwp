# PR #7941 리뷰 — 통합 후 원본 프레임·HFT 출력 회귀 보정

## 최종 판정

**승인 — #7902 메인터너 보정 후 로컬 검증·정확한 code head CI 완료.**

원래19문서의 Native/fresh WASM 전835쪽이 각각90% 이상이며 독립 Print와 쪽수가 같다.
devel 대비0.01%p 이상 하락은0쪽이다. 중요한 완료 페이지의 관계·±2px 보호와 실제 수정 전
FAIL/후 PASS를 확인했다. 아래 잔여 차이는 완벽한 픽셀 일치로 보고하지 않는다.

code PR은 `84764fa21c96cd5f8a0e158658e4162a4fb3ce9e`로 병합됐다. 사용자 선택 옵션2에 따라
이 개별 review·오늘할일은 별도 문서 PR로 반영한다. 그 문서 PR의 CI·merge는 이 기록 뒤에
별도로 확인한다. GitHub approve나 관리자 우회로 CI를 대신하지 않았다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| PR / 작성자 / base | [#7941](https://github.com/edwardkim/rhwp/pull/7941) / jangster77 / devel |
| 검증 devel | `f5251e7de4e31ee0fb939f352c6ef144bb340bdb` |
| 처음 제출 / 보완 후 최종 head | `9e3fce70048607b11525ddcecd030d961a25134c` / `554021bee72b8b5cf30deec20c8ac5a9b15e54c0` |
| renderer / 실제 전체 Rust 검증 source | `40ef22cbe4d137398149b296ff3af42c0af0b607` / `1c38fe4bc7b0b2e14dce150ab573e387bd601822` |
| 관련 이슈 | Closes [#7902](https://github.com/edwardkim/rhwp/issues/7902); #7445 하위 이슈는 검증 범위에 맞춰 참조 |
| 권한 / reviewer | collaborator self PR, reviewer 지정 없음, 사용자 자동 PR·merge·후속 승인 |
| 병합 직전 | head 재조회, MERGEABLE/CLEAN, 성공33·정책상 skipped5, 실패·대기0 |

PR 준비 직전과 CI 보완 push 전, merge 직전 fetch에서 devel SHA가 같았다. 변경되지 않은
head를 반복 리베이스하지 않았다. 마지막 보완은 CI 배선·Python 계약·fixture 원장이며
Rust 제품 source/test·실제 producer·입력 bytes를 바꾸지 않았다.

## 조판 원칙과 실제 소비 경로 검토

판정은 아래 해결 범위에서 **충족**이다. [구현과 경계 기록](../../working/task_m100_7902_stage2.md),
[원본 프레임 복원](../../working/task_m100_7902_stage3.md),
[소수 원점 정상 대조군](../../working/task_m100_7902_stage5.md),
[HFT paint 경로](../../working/task_m100_7902_stage9.md)에 코드·독립 기대값·수정 전후 실행을 연결한다.

| 변경 주장 | 실제 생산·측정·배치와 반례 |
| --- | --- |
| 생성 번호의 원본 스타일·기준선 보존 | 원본 Page Number 스타일/언어별 face → `build_page_number` → 공통 TextStyle → 실제 glyph 원점. 굵게·한/두 자리 번호와 별도 원본 변형의 내용/쪽 소유를 검사한다. |
| 선언 조각의 물리 공간 보존 | `stored_text_frame_owner`가 원본 저장 프레임의 유효성을 확인하고 표 측정·조각 요구 높이/예약·배치가 이를 소비한다. 분할 종료의 물리 잔여는 원본 선언 높이에서 생산하며 이미 소비한 줄간격을 잔여로 다시 넘기지 않는다. |
| 종료 조각 뒤 그림·본문 배치 | `emit`의 컷·물리 잔여 → 다음 조각 예산/예약 → 실제 표 높이·후속 내용 원점을 대조했다. 원문 셀의 내용 누락/중복, 마지막 유닛 종료·후속 본문·불필요한49쪽을 검사한다. |
| 원본 HFT의 소수 glyph 원점 | 원본 언어별 HFT 선언 → ResolvedCharStyle/TextStyle → 기존 glyph positions → 선택된 CFF contour → 같은 크기·CTM·clip/fill. 이동/맞춤·advance·줄 높이를 새로 추정하지 않는다. |
| 실제 paint face 소유 | Canvas FontFace 적재 완료 → 같은 bytes의 문서 세션 등록 → immutable paint snapshot. 옛한글 전용 face 선택·화면·일반 TTF·효과/미지원 프로그램은 기존 painter를 유지한다. |

특정 문서 ID·좌표 조합으로 분기하지 않고 원본 프레임/줄 정보의 유효성과 실제 선택된 face를
근거로 삼는다. clamp·숨김·임계값 완화로 겹침/하락을 덮지 않았다. 활성 원본·PDF와 정상
대조군을 보존하며 시험지 회귀를 새로 추가하지 않았다.

## 검증 입력과 전쪽 결과

[입력·producer SHA와19문서별 결과](../assets/issue7902-final-20261016/README.md),
[기준/후보 Native/fresh WASM TSV 원본76개](https://gist.github.com/jangster77/c62ec80625417cef370c6258132baee2).
Gist revision: `be3ab9d87653fca28a1c7c3751e803c126583901`; README 포함77파일을 API로 재확인했다.

- 원본19개·독립 Print19개는 실제 HEAD Git blob과 SHA가 같다. 새 대용량 PDF는 추가하지 않았다.
- Native CLI SHA: `30f68b405f71a9b95581cd9c16c0d06dc3685d229551f5082af17bee03a1d089`.
  실제 source3a6c7d0e0 이후 제품 보완은 Native가 소비하지 않는 WASM 전용 child다.
- fresh WASM SHA: `ec9b6c4d7f90f7caba9438725946daa5ee1613367e338f84651ea9b0368ae519`.
  dev/no wasm-opt로 실제 새 빌드했으며 배포 최적화 package로 보고하지 않는다.
- 기준 Native runtime source e4e6d0c8d의 src/crates/Cargo/build.rs Git 객체는 고정 devel과 같다.
  기준 WASM은 후보와 같은 최신 optional host bridge로 전쪽 다시 출력했다.
- 96dpi·Print·2px 관용·threshold32·같은 실제 공급 글꼴·흰 배경 alpha 합성이다.
  원본 HFT는 대상 정답 PDF에서 가져오지 않고 별도 전체 한글 표본의 program으로 준비했다.

| 항목 | Native | fresh WASM |
| --- | ---: | ---: |
| 문서 / 전쪽 | 19 / 835 | 19 / 835 |
| 최저2px 관용 실루엣 | 91.39728% | 90.20061% |
| 90% 미만·누락·쪽수 불일치 | 0 | 0 |
| devel 대비0.01%p 이상 하락 | 0쪽 | 0쪽 |

Decimal 비교의 두 backend 전체 최대 하락은0.00846%p다. 평균으로 미달을 상쇄하지 않았다.
TSV numeric gate와 아래 직접 판독·내용 관계를 별도로 확인했다. 원시 명령/manifest는
[최종 실행 기록](../../working/task_m100_7902_stage11.md#실행-명령과-제출-경계)에 연결한다.

## 회귀와 로컬 실행

| 검사 | 최종 결과 |
| --- | --- |
| release-test 전체 nextest | 10,427 PASS·45 skipped·실패0, compile4분40초·실행697.251초 |
| Native Skia 공식3종 | lib4,043 PASS·12 ignored; 그림/직접 PDF·glyph 원점11 PASS |
| Native 원본 경계 / focused | 글꼴 추적·원본 내용16 PASS; 분류·저장 bytes·완료 페이지20 PASS |
| 실제 fresh WASM/Chrome | 4 PASS; 새 face 선택은 Screen/Print/HQ×현대 한글/옛자모6조합 |
| Python / workflow / Node | Visual Sweep152, HFT 준비10, CI Node224, workflow Python44 PASS |
| CI 보완의 관련 검사 | workflow42, browser2, 공개 font/notice2 PASS; actionlint PASS |
| Rust 필수 lint·정책 | fmt, 기본/WASM Clippy, workspace build/all-target Clippy, manifest/unit-tier 전부 exit0 |

Cargo는 jobs8·공용 `/home/tsjang/rhwp/target/pr-review`에서 순차 실행했다. 마지막 CI 보완은
Rust/제품/fixture bytes 변경이 없어 전체 Rust·835쪽×2 로컬 반복을 생략했으며 원격 새 head의
전체 CI 성공은 아래에서 따로 확인했다.

완료 페이지14개 검사 함수와 기존 원본 경계 검사가 소유·순서·보존·포함·비겹침을 보호한다.
독립 좌표±2px를 쓰는 완료 페이지27문서·쪽 조합의54출력은 모두 양쪽98% 이상이다.
기존 판독과 최종 PNG50개는 바이트가 같고, 변경된 HWP/HWPX104쪽4개는 다시 판독했다.
번호·stack·scattered의 양쪽100% 쪽을 우선하고 동일한 중요 경계의99%대 쪽을 선정했다.
기존 px 핀의 관계 검사 교정과 새 좌표 보호는 [페이지별 표](../../working/task_m100_7902_stage3.md#문서별-완료-페이지의-px-회귀)에 구분했다.

[수정 전/후 증거](../../working/task_m100_7902_stage10.md#font-only-실제-수정-전-fail--수정-후-pass):
실제 CLI의19HU 이동은 이전1.0px FAIL→후보0.2436279137px PASS(독립 기대0.253333px).
옛한글 전용 face 검사는 이전 source3a의 실제 WASM에서 의도한 사각형 대체로 FAIL했고,
보완 후 같은 정식 검사가 PASS했다. 저장 IR/실제 HWP bytes 불변도 검사했다.
font-only 합성 계약은 실제 한컴 문서 정확도 증거와 구분한다.

## 직접 판독과 남은 차이

[최종 review·standalone overlay12쌍](../assets/issue7902-final-20261016/README.md#대표-png와-직접-판독)을
실제 열었다. 쪽번호29쪽 양쪽100%, overfill 마지막48쪽 양쪽99%대, nested104쪽
Native99.95192%/WASM99.93860%다. 목차·부호·목록/괄호·표→본문→그림→후속 문단과
종료 내용을 대조했다. HWP/HWPX는 별도 원본·기준 PDF와 실제 같은 raster/좌표로 확인했다.

글꼴 획·그림 색/점선 아래 변·일부 미세 간격 차이는 남는다. nested102쪽 하단 표 높이 차이는
base/현재의 그 영역 pixels·bbox가 같으며 이번 수정의 해결로 주장하거나 새 px 핀을 만들지 않았다.
실루엣100%를 엄격 픽셀 동일로 설명하지 않는다. #7445의 다른 Print/축소본·인쇄 배율·편집/캐시
계약은 별도 범위이므로 이19문서 결과로 하위 이슈 전체를 닫지 않는다.

## 정확한 code head CI와 최초 실패 처리

최종 `554021bee`의 다음 run은 모두 **success**다.

- [CI](https://github.com/edwardkim/rhwp/actions/runs/38839284502): lint·Native Skia·default archive A/B/C/D·frontend package·Build & Test.
- [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/38839284410): 실제 글꼴 등록과 Canvas visual diff.
- [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/38839284424): 네 언어 분석 및 최종 CodeQL, 실패 없음.
- [Adapter](https://github.com/edwardkim/rhwp/actions/runs/38839284474), [Proptest](https://github.com/edwardkim/rhwp/actions/runs/38839284409), [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/38839282291).

첫 head9e3의 Chromium sandbox 시작 실패와 fixture2개 원장 누락은 숨기지 않는다.
[CI 보완 기록](../../working/task_m100_7902_stage12.md)에 실제 실패 run과 관련 검증을 연결했다.
설치 Chrome의 sandbox를 쓰도록 계약 검사를 옮기고 원본 MIT/SHA를 원장에 등록했다.
검사·폰트 bytes·측정 Chromium pin·허용치를 완화하지 않았다. 최종 head의 PR 본문·한글·SHA
고정 PNG24개를 API/HTTP 및 실제 Chrome PR 화면에서 확인한 뒤 일반 squash merge했다.

## 후속 처리

사용자 선택 옵션2의 이 문서 PR은 review-only B 경로에서 실제 최신 head의 preflight/집계를
확인한 뒤 merge한다. 이후 devel sync, #7902·원 PR에 merge SHA 고정 증적/검증 안내,
전용 worktree·중간 산출물 정리를 마무리한다. 공유 target·원본/Print·글꼴·최종 증적과 사용자
Studio JS를 보존한다. 기본 작업트리의 사용자 dirty JS 때문에 원격 branch 정리의 clean 조건을
충족하지 않으면 해당 임시 branch는 사유와 최종 SHA를 기록하고 유지한다.
