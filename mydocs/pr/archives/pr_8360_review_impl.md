# PR #8360 구현 검토

## 최종 판정

**승인 — 검토 code head `f1437377645256367fd53bc1eae724a3c4cb5e19`의 구현·검증 근거를 확인했다.** 원격 최종 조건은 [리뷰](pr_8360_review.md)를 따른다.

## 요청 수명과 실패 경계

| 생산 / 전달 | 확인한 소비와 실패 경계 |
| --- | --- |
| `src/parser/pdf/policy.rs` | standard 8,000,000·선택 쪽당 250,000과 constrained/낮춘 host cap을 검증한다. operator Vec는 capacity 확장 전 검사하며 단일 stream 한도를 같이 올리지 않았다. |
| `src/document_core/pdf_import_session.rs` | 같은 ImportBudget을 parser·Core·실제 Storage·첫 저장·진단·응답 사본에 전달한다. 예산 실패 후 sticky 상태와 정상 새 요청의 독립 수명을 구분한다. |
| `src/cli/commands/pdf_import` / `src/resource_supervisor` | 입력/설정 검증, 제한된 worker 요청·응답, wall/CPU/address space/output 감시, 종료·취소와 원자적 출력 교체를 확인했다. 다른 CLI 명령의 일반 process 실행과 구분한다. |
| `src/wasm_api/pdf_request.rs` | prepare → 입력 사본 예약 → once-only open → 제한된 SVG/진단/저장 → 응답 backing 예약. JS UTF-16 사본 계상도 반환 전에 수행한다. |
| `rhwp-shared/pdf/import-session` / Studio `pdf-open.ts` | worker 세대·취소·timeout·transfer 오류와 정상 결과를 구분하고 제한·잔여 credit을 편집 재열기로 전달한다. 실패 후 이전 문서와 다음 정상 요청을 실제 검증했다. |
| Studio `wasm-bridge.ts` | 새 문서를 준비·검증한 뒤 publish한다. 이전 문서는 commit 뒤 해제하며 observer/정리 실패가 publish한 문서의 소유를 되돌리지 않는다. renderer backend 전환의 자원 정리도 실제 대조했다. |

T04의 전체 할당 감사와 T05 handoff 검증은 초기 PDF/첫 저장에서의 실제 생성·소비 경로를 추적했다. 셀 cache·notes· 일부 shaping의 비도달 증명과 표본 breakpoint 0회를 구분했다. 새 devel이 추가한 reset/소유 원장도 실제 Storage에 연결했다. 카운터가 전체 heap/time 증명은 아니며 legacy XML parser·host 격리 책임을 문서에 유지했다.

## PDF 조판 공통 결과

`src/parser/pdf/editable.rs`의 관측 row grid와 glyph CHAROFFSET 생산 결과가 저장 LineSeg와 실제 editable 문단에 전달된다. 선행 위치 고정 본문은 `take_positioned_prefix`가 같은 단의 첫 flow와 함께 보존하고 기존 continuation owner 경계가 이를 소비한다. 모든 그림을 흐름 점유로 취급하지 않으며 확정 본문 owner·배경·머리말을 구분한다.

마지막 줄의 accepted content end와 inferred interline tail을 구분하고, 다음 관측 원점 앞의 가용 tail을 같은 LineSeg spacing·flow bottom·후속 frame 원점에 반영했다. accepted 줄 상자를 자르거나 이미 확정한 본문 owner를 demote하지 않는다. 문서 식별자·우연한 값 조합으로 맞추지 않고 기존 50 HU 원점 witness를 유지했다.

분할/이어받기에 대해서는 선행 owner → 첫 flow → continuation 원점 선택과 일반/특수 paragraph frame 소비를 대조했다. 이번 PDF 보정은 이후 측정에서 paint 제한을 흉내 내는 보정을 추가하지 않는다. 기존 표/미주 분할 알고리즘의 계산을 바꾼 것이 아니라 같은 Storage의 누적 예약과 소유 수명을 연결한 부분도 따로 검토했다. 입력 순서·줄 소속·누락/중복·다음 본문과 쪽수를 정식 회귀 및 전쪽 실제 출력에서 확인했다.

macOS/2010/2020 세 PDF의 독립 paint baseline·전체 페이지·Native/fresh WASM 98% 조건과 직접 확대 판독을 먼저 확인한 뒤 `issue_8220_pdf_editable_layout`에 1건을 추가했다. 기존 정상 단/쪽 연결·7쪽·편집/양쪽 저장 4건은 유지했다. 수정 전 FAIL → 수정 후 PASS와 정상 대조군 5건을 [검증 원장](../assets/issue_8174/t09/verification_manifest.json)에 연결한다. 규제분석서 28·29쪽의 기존 ±2px 보호는 양쪽 98% 이상에서 유지했다.

## 코덱·SVG·보호 기능

코덱/ICC/xref/SVG·링크·제한된 양식 동작·보호 복구의 기본 제공과 명시 제외 경로를 확인했다. 모든 codec 조합·URI scheme·임의 JavaScript·모든 손상 문서 복원을 지원한다는 뜻은 아니다. JPEG 공통 기하와 alpha는 독립 pixel/geometry·예산·복구 및 실제 Native/fresh WASM 출력으로 검증했다. 한컴의 기존 alpha 출력 차이는 메인테이너 판정과 원본 기대를 구분했다.

SVG glyph는 실제 공통 bounded raster API를 소비한다. API의 예산/검증 실패를 기존 path fallback으로 우회하지 않는다. CI 하네스는 renderer와 같은 WASM instance를 초기화한 뒤 image replay를 검사하고, 별도 단색 glyph로 path 계약을 보호한다. 같은 하네스가 이전 API의 base 제품에서도 통과했다. production renderer와 생성 제품 hash는 바뀌지 않았다.

## 재사용 근거와 한계

최종 E2E 보정은 하네스/문서뿐이며 Rust·Studio runtime·generated binding/JS/WASM·package payload는 이전 검증 source와 같다. 따라서 이미 완료한 로컬 Rust/포장/전쪽 증거를 새 출력으로 위장해 재생성하지 않았다. 변경한 하네스·연관 coverage를 실제 실행했고 새 code head의 Full GitHub CI·Render Diff·GHAS도 성공했다.

보안 173건은 동일 metric flag의 이름 기반 오탐이며 전체 source/flow를 보존하고 사용자 승인으로 metadata를 분류했다. 기존 DES는 실제 HWP3 호환 알고리즘이라는 제한을 유지한다. 필수 기능/시각/포장 게이트, 추가 415쪽 성능 미판정, 실제 registry 게시의 별도 승인 범위를 혼동하지 않는다.
