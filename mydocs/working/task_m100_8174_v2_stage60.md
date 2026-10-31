# #8174 v2-S60 — 인라인 계획·확정 상태의 요청 예산

2026-10-27. 기준은 S59 `7034ca8baf1e0dcd68bfd1b1edf0dc58dea631e2`다. 기존 통합 승인과
다음 작업 지시에 따라 T04의 인라인 계획 생산·확정 경로를 연결하고 절편 검증을 완료했다.
T04 전체나 PR 준비 완료는 아니다. 당시 로컬 증적은
ignored `output/pr-review/issue8174/v2-s60-20261027/`에 생성했으며 종료 정리 대상이다.

## 구현과 실제 소비 경로

구역의 표 문단은 서식 계산 전에 인라인 계획을 수용하고 조기 반환할 수 있었다. 이제 이
경로와 일반 문단의 인라인 후보가 같은 Storage를 글자/개체 토큰·어절·구분 공백·배제 영역·
행과 상자 배열에 전달한다. 증가 전에 예약하며, 수용한 계획을 State에 확정할 때의
`current_items` backing과 `inline_flow_plans` 삽입도 같은 요청을 소비한다.

| 단계 | 생산·소비 연결 |
| --- | --- |
| 구역의 실제 진입 | `section/flow → typeset_table_paragraph_with_storage → controls::paragraph_flow::place_with_storage`, 일반 문단은 `typeset_inline_flow_with_storage` |
| 읽기 전용 후보 | `inline_flow/plan::build_plan_with_storage → renderer::inline_flow::{plan_with_storage, plan_plain_text_with_storage}` |
| 공통 줄 구성 | 글자/개체 위치·PUA 표시·어절/구분 공백과 스타일 사본 → 배제 영역·`LayoutFrame::try_carve` → 같은 `finish_row_with_storage`의 상자·행·끝점 |
| 실제 확정 | 수용된 같은 계획 → `commit_inline_flow_with_storage` 또는 `commit_stored_body_wide_host_with_storage` → 항목·계획 map·높이/dirty 상태 |

원래 너비·들여쓰기·기준선·행 높이/간격 계산과 수용 조건을 유지했다. 기존 일반 API는 같은
구현의 Unbounded wrapper다. `None`인 미지원/부적합 후보와 예산 오류를 구분하고, 오류를
정상 `None`/`false`로 바꾸지 않는다. BTreeMap은 새 키일 때만 논리 tree entry를 예약한다.

분할·이월의 정책은 바꾸지 않았다. 독립 표 조각 소유자는 기존 반환 경로를 쓰고, 현재
프레임에 수용되는 접두 행과 소유가 유지된다. 다음 단의 임시 프레임에서 전체 수용을 먼저
확인하고, S58의 bounded 상태 전환 뒤 실제 프레임에서 후보를 다시 구성·확인한다. 같은
계획의 끝점이 확정 높이로 이어지며 새 clamp나 출력 숨김은 없다. 표/미주 내부의 모든
상태 변경과 모든 일반 항목 생산자까지 연결한 것으로 확대하지 않는다.

## 한도 거절과 복구

공개 Native64 검사 `main_typeset_table_inline_work_uses_the_request_budget_and_recovers`를
기존 자동 수집 파일에 추가했다. 같은 검사 소스·정책의 실제 S59는 **1 FAIL**, 현재 후보는
**1 PASS**다. `multiline-cell.pdf`에서 S59 관측량 7,614,879에 16 bytes의 여유를 둔 정책이
후보의 추가 논리 계상을 거절한다. 실패 상태 유지·같은 요청 재사용 거절·새 정상 요청의
동일 쪽수/HWPX bytes 복구를 확인했다. 오늘의 관측량이나 거절 지점은 기대값으로 고정하지 않는다.

기존 Allocation의 token storage 검사에 두 bounded 공통 plan을 연결했다. 1-byte 정책의
증가 전 거절과 실패 유지, 새 요청 복구, 일반 API와의 같은 의미 결과를 확인했다. 기존
**9개** 검사 중 하나의 계약 확장이며 source unit 모듈·검사 개수 증가는 없다. 이 계약을
새 한컴 시각 golden으로 취급하지 않는다.

| 입력 | S59 논리 decoded 작업량 | 후보 | 차이 |
| --- | ---: | ---: | ---: |
| CFF | 1,633,772 | 1,633,772 | 0 |
| 중첩 셀 | 1,832,826 | 1,832,874 | +48 |
| 여러 줄 셀 | 7,614,879 | 7,614,927 | +48 |
| 문단 원점 | 7,648,635 | 7,648,683 | +48 |

기존 backing을 요청에 계상한 관측이다. 실제 heap/RSS나 로딩·편집 시간 증가 측정이 아니다.
초기 중첩 셀 cap 실험은 거절을 검출하지 못해 새 회귀로 채택하지 않았으며 로그를 보존했다.
그 실험을 경로 비도달이나 보편적인 거절 보장으로 해석하지 않는다.

## 현재 source의 검증

| 범위 | 결과 |
| --- | --- |
| 공개 요청 수명·storage/wire·XML | **47 PASS** |
| Core·두 엔진·정규화·쪽 배치·기존 inline/LayoutFrame | **657 PASS / 기존 8 ignored** |
| composer·float·model / 텍스트·도형 측정 | **244 / 38 PASS** |
| serializer·adapter / Allocation | **662 / 9 PASS** |
| frame·inline Print 기존 통합 / 기존 저장 메타데이터 | **22 / 52 PASS** |
| 저장·재열기·wire | 저장 **37개**, header/ZIP **22개** 모두 S59와 bytes 동일 |
| 정적·빌드 | Native/WASM Clippy·feature-off·fmt·절편 source unit 정책 등 **12개 실행 PASS** |
| 실제 최적화 제품 | WASM 빌드·Worker·Studio·전쪽 출력 **4개 실행 PASS** |
| Worker·Studio | **249 assertions / 14 Workers**, 활성 문서·draft·recents 보존, 실패/취소 후 복구, 15쪽 문서 표시와 HWP/HWPX 저장 PASS |

Native는 root opt-level 1·codegen-units 64의 기능 검증 제품이다. 최종 성능 빌드로 표시하지
않는다. 기존 inline Print 5개 검사는 실제 S59와 후보 양쪽에서 통과했고 소스가 동일하다.
CLI 오류 6건·Native preflight 6건도 통과했다. 전체 최종 nextest/Skia·workspace·성능 게이트는
T08에서 수행한다.

최적화 fresh WASM은 **25,387,932 bytes**, S59보다 **47,942 bytes(+0.18920%)**다.
SHA-256은 `efbb05039a9951b46553caab574ed344bc40f0df939d57083a782779b60a4bc7`다.
JS bytes와 공개 선언의 서명을 유지했다. `InitOutput` 내부 같은 멤버의 순서만 다르다.
root pkg와 Studio의 실제 JS/WASM bytes를 대조했다. 공개 검사 library는 header·counter
producer와 현재 파일 모두 `5914799933a487c4554a0a2bf30e681b3c8b1629f0ef94fc7cfd057184443eb2`로 일치한다.

## 전쪽 출력과 직접 판독

| 범위 | Native | fresh WASM |
| --- | --- | --- |
| 기존 PDF 가져오기·본문/표·글꼴 17개·49쪽 | 최저 raw **98.34664%** | 최저 raw **98.79246%** |
| inline Print 진단 3개·35쪽 | **100%** | 최저 **99.81680%** |
| 원본 배포 문서 1개·3쪽 | 최저 **99.66991%** | 최저 **99.17555%** |
| 합계 | **21개·87쪽** | **21개·87쪽** |

각 runtime의 실제 PNG **87개 모두 S59와 bytes 동일**하다. 추가 38쪽은 같은 최종 글꼴
공급 조건에서 S59와 후보를 각각 다시 출력했다. 전체 쪽수·전쪽 TSV·90%·누락/측정 불가를
확인했고, 기존 좌표 회귀가 보호하는 진단 35쪽과 원본 p3는 양쪽 **98% 이상**을 유지했다.
기존 조판 기대값·fixture/golden·수치 한도를 바꾸지 않았다. 별도 Native fallback 5개·5쪽은
모두 **100%**, 실제 PNG도 S59와 동일하다.

현재 CFF 기호/합자·Type3 도형/설명과 p11 행 26–30·중첩 A/B/C/D·여러 줄 셀의 review와
overlay를 직접 확인했다. fallback 5개도 현재 panel 또는 같은 현재 픽셀의 판독에 연결했다.
추가로 양쪽 runtime에서 개체 공백/우측 단위 표기·overflow·빈 줄/서식 있는 개행·원본
p1–p3의 선택한 확대/전체 panel **16개**를 확인했다. 표 외곽·내용 순서·본문/뒤 문단·
131,200 합계·주석의 새 누락/겹침/줄 소유 변화는 관찰하지 않았다. 독립 PDF 대비 원본
p2 도입부와 p3 본문의 글자 굵기 및 가는 획 차이는 남고 실제 S59 출력에도 동일하다.
점수 통과를 모든 글자 모양의 일치나 최종 PR 승인으로 확대하지 않는다.

초기 원본 p1의 **89.50497%**는 검증 글꼴 공급 부족 상태였다. 설치된 GulimChe의 실제
TTC face 1과 HY견고딕·HY견명조·HY헤드라인M을 로컬 우선 디렉터리에서 공급하고,
같은 face 이름 매핑을 양쪽에 적용해 다시 실행했다. 전체 추출 글꼴 디렉터리는 기존
Native catalog의 **64 MiB** 한도를 넘겨 필요한 face가 빠질 수 있었다. 필요한 face를 먼저
공급했으며 제품의 catalog 한도·코드는 바꾸지 않았다. 미사용 Magic R 진단 후보를
휴먼명조로 대체한 것으로 주장하지 않는다. 초기 실패와 중간 시도는 증적 폴더에 보존했고
최종 TSV와 구분한다. 글꼴 파일은 커밋·배포하지 않았다.

실행 명령과 로그는 `checks.json`, `wasm-checks.json`, 각 validation driver와 log에,
전쪽 TSV/PNG는 `visual/`에 있다. `direct-review-*.json`은 실제 선택 판독 파일을,
`font-supply-final.json`은 공급 순서·실물 face/hash를 기록한다. 소스 동결·제품·검사·문서
원본·독립 PDF·전쪽 TSV·현재 판독의 hash 대조는 `evidence-manifest.json`과
`verify-evidence.log`, `verify-inline-evidence.log`로 연결한다.

## 남은 작업

다음 T04 대상은 일반 State 항목·문단 원점/계산 메타데이터의 생산·원장과 표/미주 내부
상태 호출이다. 셀 유닛/행 색인 캐시·일부 일반 frame/커닝/shaping도 남는다. 이번 plan
확정을 전체 항목/ledger 연결 완료로 표시하지 않는다. [TODO](task_m100_8174_pr_remaining_todo.md)의
T05–T09와 최신 devel 보존·최종 PR 게이트를 유지한다. 기존 host 책임·논리 작업량/실제
메모리 구분과 `parserAllocationBoundsProven=false`도 유지한다. 이번 절편은 로컬 처리이며
원격 push·GitHub 게시·registry 업로드·태그는 수행하지 않았다.
