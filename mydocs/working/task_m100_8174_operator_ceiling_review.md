# #8174 — PDF operator 400만 상한의 근거와 800만 확대 검토

- 요청: 400만의 도입 근거를 확인하고 800만으로 확대할 수 있는지 조사한다.
- 검토 source: `8a3746a75d1156a99d116f8bacfdee2ba3a66f30`.
- 이 검토 실행에서는 제품 기본값을 변경하지 않았다. 진단용 Rust 소비자만 명시적 정책으로
  800만을 전달했다. 이후 아래 메인테이너 수용에 따라 구현할 범위를 확정했다.
- 상태: **상한 검토 완료**. 권고는 공통 수집기의 사전 버퍼 제한 보강 후 standard를 800만으로
  확대하는 것이다. S08 전체 완료·PR 제출 판정이 아니다.
- 로컬 증적: ignored `output/pr-review/issue8174/v2-s08-20261025/operator-assessment/`.

## 400만은 어디에서 왔는가

원 도입은 `a5cbae1daea6e46f6ae7898b5995fc7650a70793`, 통합은
[PR #7970](https://github.com/edwardkim/rhwp/pull/7970)의
`515bd72dcde0eecdeece333666b675354b7fa91d`다.
[당시 4단계 기록](task_m100_7957_stage4.md)은 선택 쪽수에 비례한 작업량과 별도로
operators 4,000,000·decoded 2 GiB·path 1,000,000을 유한 요청 상한으로 정했다.
이는 PDF 사양이나 정수 표현의 한계가 아니라 제품의 작업량 정책이다.

당시 기록에서 성공 입력의 operator 사용량은 274,718·421,337·56회였다. PR 본문·논의,
도입 diff와 작업 기록에서 **400만과 800만을 비교해 400만을 선택한 경계 측정 근거는
확인하지 못했다.** 정상 입력의 중복 작업 제거·쪽 선택·출력 호환 검증은 있었지만,
그 검증을 400만 부근의 메모리·실행 시간 검증으로 바꾸어 해석하지 않는다.

## 지금 무엇을 제한하는가

- [제품 정책](../../src/parser/pdf/policy.rs)의 standard 기본값이다. 신뢰 Rust 호스트는
  `parse_pdf_with_policy`에 더 큰 유한 값을 직접 전달할 수 있다. CLI·WASM의 JSON override는
  선택 프로필보다 낮추는 계약이므로 현재 `operators: 8000000`을 넣으면 정책 오류다.
- [bounded parser](../../vendor/rhwp-pdf-import-backend/src/content/parser.rs)의 어휘 해석 때
  한 번씩 계상한다. 다시 파싱한 내용은 다시 계상하지만, 같은 페이지의 해석 캐시를 재사용한
  interpreter 실행을 operator 카운터에 다시 더하는 것은 아니다. 참조·XObject 방문과
  decoded 복사 작업에는 독립된 카운터가 있다.
- 현재 유효 operator 한도는 `min(프로필/호스트 cap, 250000 × 선택 쪽수)`다. 15쪽은
  3,750,000이며 standard 상한만 800만으로 올려도 이 값은 바뀌지 않는다. 400만은 16쪽부터,
  800만은 32쪽부터 적용된다. constrained의 250,000과 명시적으로 낮춘 값은 별개다.
- 입력 64 MiB·decoded 2 GiB·단일 stream/copy 32 MiB·collection 400,000·path·font·image
  한도도 독립적으로 작동한다. operator를 늘려도 다른 한도로 거절한 입력까지 열리지는 않는다.
- [페이지 operator 캐시](../../vendor/rhwp-pdf-import-backend/src/document.rs)는 1개 페이지를
  보관한다. 문서 전체 800만 명령이 항상 동시에 남는 것은 아니다. 그러나 한 stream의
  `Vec<Operator>` 수집은 [복사 버퍼 검사](../../vendor/rhwp-pdf-import-backend/src/content/operator_copy.rs)
  전에 완료된다. 따라서 한 페이지에 몰린 입력은 거절되기 전에 큰 임시 Vec를 만들 수 있다.

## 측정 방법과 Native 결과

같은 source의 공통 PDF frontend를 외부 Rust 소비자로 호출했다. 명령을 정확히 세어 독립 생성한
40쪽 PDF에 `q Q`와 각 쪽의 사각형 한 개를 넣었다. 400만·400만+1·800만·800만+1,
한 페이지에 집중한 두 입력을 비교했다. 원본 파일은 18–27 KB 수준의 압축 PDF이며
counter·자원 경계용 합성 입력이다. 실물 문서의 조판 일치 검증으로 사용하지 않는다.

Native는 `release-test` 프로필이다. 제품과 같은 `resource_supervisor`로 Linux AS 4 GiB·CPU
90초·wall 120초를 적용했다. 각 일반 경계는 별도 프로세스에서 3회 실행했다. 아래 시간은
parse부터 HWPX 직렬화까지의 중앙값이고, RSS는 감독기가 관측한 최대값이다. RSS를 완전한
peak 측정이나 보증 상한으로 표현하지 않는다. 성능 측정 중 Cargo 빌드는 겹치지 않았다.
소비자는 `pdf-import`만 켰으며 이 입력은 이미지 코덱·ICC 분기를 실행하지 않는다.
제품 CLI의 기본 feature 조합은 별도로 대조했다.

| operator 상한 | 실제 명령 | 결과 | Native 중앙값 | 관측 RSS 최대 |
| ---: | ---: | --- | ---: | ---: |
| 4,000,000 | 4,000,000 | 40쪽·HWPX 성공 | 620.34 ms | 40.94 MiB |
| 8,000,000 | 4,000,000 | 같은 입력 성공 | 637.83 ms | 39.33 MiB |
| 8,000,000 | 8,000,000 | 40쪽·HWPX 성공 | 1,231.86 ms | 55.70 MiB |

400만+1은 400만 정책에서, 800만+1은 800만 정책에서 정확히 `content operators`로 거절했다.
성공·거절 뒤 같은 프로세스의 새 정상 요청은 모두 가져오기·HWPX 저장에 성공했다.
같은 400만 입력의 작은 시간 차이는 3회 관측이며 유의한 성능 회귀로 단정하지 않는다.
상한 숫자를 바꿨다고 그 크기의 공간을 미리 예약하지는 않는다.

한 페이지에 대부분의 명령을 집중한 입력은 두 정책 모두 `operator copy buffer`로 거절했다.
그러나 거절 전 관측 RSS는 400만 정책에서 **167.97 MiB**, 800만 정책에서 **329.43 MiB**였다.
800만 확대는 허용 작업량뿐 아니라 이 거절 경로의 임시 메모리 비용도 늘린다. 두 실패 뒤의
정상 후속 요청은 성공했다. 카운터·거절·복구 원문은 `native-results.json`에 보존한다.

제품 Native CLI의 현행 400만 정확 경계 성공, +1 거절, constrained 거절과 800만 override의
정책 오류도 별도 확인했다. 모든 실패에서 기존 출력 파일을 유지했다(`product-cli.json`).

## 이번 사용자 PDF와의 관계

실제 15쪽 원본을 같은 소비자에서 두 정책으로 다시 읽었다. 두 경우 모두 IR 추출에 성공했고
operator 사용량은 1,731,414, 유효 한도는 3,750,000이었다. HWPX 직렬화에서는 두 경우 모두
`invalid editable text clip metadata`로 실패했다. 800만으로 올려야 이 파일이 열린다는 근거는
없다. 해당 후속 clip 보정은 [S08](task_m100_8174_v2_stage8.md)에서 별도로 계속 다룬다.
원본·실행 로그는 로컬에만 보존한다(`private-cap-4000000.json`, `private-cap-8000000.json`).

## WASM과 최종 판정

같은 frontend를 실제 wasm32 소비자로 빌드해 Chromium Worker에서 실행했다. 진단 소비자는
`release`·`opt-level=3`이며 `wasm-opt`는 실행하지 않았다. 현재 기본 400만인 최적화 **제품**
WASM의 경계 동작은 별도로 대조했다. 진단 소비자 800만 결과를 제품 CLI·Studio의 800만
기본값으로 빌드·검증한 결과라고 표현하지 않는다.

| operator 상한 | 실제 명령 | 결과 | WASM 중앙값 | 실행 후 선형 메모리 용량 |
| ---: | ---: | --- | ---: | ---: |
| 4,000,000 | 4,000,000 | 40쪽 IR·HWPX 성공 | 865.90 ms | 26.31 MiB |
| 8,000,000 | 4,000,000 | 같은 입력 성공 | 847.60 ms | 26.31 MiB |
| 8,000,000 | 8,000,000 | 40쪽 IR·HWPX 성공 | 1,507.80 ms | 47.81 MiB |

WASM 시간에는 로컬 loopback의 입력 fetch·WASM 복사·parse·직렬화가 포함되며 모듈 초기화는
제외한다. Native와 시간 범위가 완전히 같지 않으므로 두 backend 사이의 속도비를 주장하지 않는다.
`memory.buffer.byteLength`는 선형 메모리의 용량이며 live heap·RSS·브라우저 전체 메모리가 아니다.
이미 커진 선형 메모리는 Rust 객체를 해제해도 줄어들지 않는다. 매 측정은 새 Worker를 사용했다.

WASM에서도 정확한 경계 +1 거절의 `used=cap`, `requested=1`, `limit=cap`과 정상 후속 요청의
가져오기·HWPX 저장을 확인했다. 일반 경계 21회·집중 입력 2회씩, Native/WASM 합계
**46회와 각각의 정상 후속 요청 46회**를 확인했다. 집중 입력은 같은 복사 버퍼 오류로 거절했지만
선형 메모리 용량은 400만의 **230.88 MiB → 800만의 457.25 MiB**로 증가했다.
wasm32 `Operator` 크기는 28바이트다. Vec 확장 중의 이전/새 공간과 allocator 동작이 있으므로
`명령 수 × 슬롯 크기`만으로 peak 비용을 단정하지 않는다.

현행 최적화 제품 WASM에서도 400만 정확 경계의 40쪽 가져오기·저장, +1 거절과 800만 JSON
override 거절을 확인했다(`product-wasm.json`). 실제 공통 `PdfImportSession`에서는 큰 입력의
`importing` 통지 후 100 ms에 취소했고 100.5 ms에 취소 응답을 받았다. 이전 2쪽 문서를 유지하고
다음 정상 2쪽 가져오기에 성공했다(`product-worker-recovery.json`). 별도 800만 소비자 Worker도
동기 parse 중 100 ms에 종료할 수 있었다. 이를 Studio 전체의 800만 완료 검증으로 승격하지 않는다.

### 권고

**800만은 수치·API상 가능하지만 현재 코드에서 숫자만 바꾸는 확대는 권하지 않는다.**
400만에 특별한 사양상 근거는 없으며, 경계 실험에서도 800만의 처리·정확한 거절·복구가
동작한다. 문제는 한 페이지에 집중한 입력을 어차피 거절하면서도 큰 Vec를 먼저 만드는 경로다.

1. 공통 `parse_content_with_budget`에서 operator Vec의 capacity 증가를 **할당 전에** 제한한다.
   기존 단일 버퍼 32 MiB 계약을 소비하도록 하고, 길이만 검사한 뒤 기하급수적으로 과예약하는
   방식은 피한다. 기존에도 복사 버퍼 한도로 거절하던 입력을 더 일찍 거절하는 보강이다.
2. 그 뒤 공통 standard operator 상한만 8,000,000으로 올린다. constrained 250,000,
   250,000/선택 쪽, 낮춘 host override와 다른 독립 한도는 유지한다. 다른 이름의 400만
   카운터를 근거 없이 함께 올리지 않는다.
3. 보강 source에서 같은 밀집 입력의 사전 거절·메모리, 정상 800만·+1 거절·복구를 재검증하고,
   실제 CLI·최적화 제품 WASM·Studio의 30초 Worker 경계와 문서 유지·저장을 대조한다.

이번 검토에서는 제품 기본값·수집 로직을 수정하지 않았다. 메모리 관측을 전체 비신뢰 PDF의
안전성 증명으로 확대하지 않으며 기존 MD05 감사·S07 시각 보류·S08 후속 clip 보정도 유지한다.
정확한 입력·소비자·결과 파일의 SHA-256과 집계는 `assessment-proof.json`에 보존했다.

## 메인테이너 수용 — 2026-10-25

사용자는 「권고안을 수용합니다.」라고 결정했다. 공통 operator 수집기의 capacity 증가 전
버퍼 제한 보강 → standard 8,000,000 확대 → CLI·최적화 제품 WASM·Studio의 실제 경계와
복구 재검증을 승인된 구현 범위로 기록한다. constrained·선택 쪽 비례식·낮춘 host cap과
다른 독립 한도는 유지한다. 수용 당시 제품 기본값은 4,000,000이었으며, 이 결정 기록을 구현
완료로 표시하지 않는다. 이후 구현·Native 계약과 남은 제품 검증은
[T01 실행 기록](task_m100_8174_t01_operator_budget.md)을 따른다.
[결정 대장 MD07](../plans/task_m100_8174_maintainer_decisions.md)과
[PR 제출 전 남은 작업](task_m100_8174_pr_remaining_todo.md)에 연결했다.
