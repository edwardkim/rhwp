# 9개 이관 이슈의 회귀 복원 — 문서별 단계 3

전체 계획은 `mydocs/plans/task_m100_7754.md`다. 단계 2의 구현·실패 진단은
`9c036063af`까지 중간 커밋했다. 시각 선행 조건을 먼저 충족한 원본부터 단계 3을 수행한다.
#7765의 보정·검증은 단계 2에서 계속하며 9개 이슈 전체의 완료나 PR 제출로 표시하지 않는다.

## #7755 시장구조조사

- 독립 Print·원본·생산 소스와 Native/fresh WASM 전쪽 TSV는
  `mydocs/pr/assets/issues7754_7765/market-source7/README.md`에 고정했다.
- Native/fresh WASM 모두 315/315쪽, 최저 raw 98.40%다. 4·5·100·247·315쪽은 양쪽
  100.00%, 246쪽은 양쪽 99.97%다. 같은 영역의 세 출력을 직접 열어 확대 판독했다.
- 기존 #6761 검사는 이미 4쪽의 본문 내부 포함과 5쪽 첫 두 줄의 내용을 검사하는 관계 검사다.
  이 관계를 유지하고 이관했던 독립 315쪽 전제만 복원했다. 임의 px 핀 교체라고 표시하지 않는다.
  첫머리에 실제 수치·근거·Git에서 확인한 직전 파일 변경 커밋 시각을 추가했다.
- 새 `issue_7755_market_flow_content`는 100쪽 빈 줄·제목·본문 순서, 246쪽 앞 표 종료와
  실제 빈 줄·억제한 빈 줄, 247쪽 부록 제목·단위·표 시작, 247–315쪽 전체 본문 행과 셀을 검사한다.
  원본의 2,195개 본문 행·각 셀 내용을 반복 머리행과 구분해 누락·중복·순서·셀 내부 포함을
  검사한다. 절대 px 좌표나 전체 SVG 해시를 고정하지 않는다. 쪽수와 내용 보존을 함께 검사한다.
- 기존 #7147의 실제 원본 표(문단 934)를 94→95쪽에서 직접 대조했다. 양쪽 raw가 각각
  100.00%이며 제목부 머리행의 실제 상자를 원본 행 선언과 비교하고 반복 머리행, 본문 행의
  분할·보존·다음 절 제목을 검사한다. 과거 코드의 0-based 94와 실제 94쪽을 혼동하지 않는다.
  전체 밀도 검사의 유효한 목적도 전체 쪽수와 실제 내용 소속으로 보호한다.
  범위 밖 다른 문서의 이관·기대값을 바꾸지 않는다.
- 최소 글꼴 bwrap은 사용자 글꼴·/opt·시스템 글꼴·비공개 ttfs를 숨기고 opensource의 5개
  글꼴만 노출한다. 기존 초안은 이 환경에서 1 PASS/0 FAIL/0 ignored, 26.52초였다.
  정식 두 파일을 같은 격리에서 다시 실행하며 그 결과와 자동 suite 수집 확인을 추가한다.
- 최종 보정 head의 전체 Rust·lint·최적화 Native/fresh WASM·보호 게이트는 아직 완료하지 않았다.

## 실제 집중 실행과 수집 확인

현재 생산 소스 `9c036063af`와 같은 소스 14 Native 라이브러리를 고정하여 정식 두 파일의
테스트를 직접 `rustc --test` harness로 실행했다. Native Skia 기능을 포함한 root opt-level 0
라이브러리이며, 필수 최종 Cargo suite·최적화 Native/fresh WASM 게이트를 대체하지 않는다.

- 최초 정식 두 함수: 2 PASS/0 FAIL/0 ignored,137.69초.
- #7147 보완 도중 `row_sizes`를 높이로 읽은 새 테스트의 잘못된 전제를 확인했다. 이 필드는
  HWP 스펙의 행별 셀 수이며, 높이는 원본 셀 LIST_HEADER의 `height`다. 생산 기대값을
  완화하지 않고 독립 원본 계약을 바로잡았다. 컴파일 자료형 오류와 그 뒤의 잘못된 전제
  실패를 보존했고, 실패 빌드 뒤 이전 harness가 시작된 실행은 중단하여 통과로 세지 않았다.
- 실제 셀 높이 선언·94–95쪽 분할·후속 절을 보완한 최종 시장 함수: 1 PASS/0 FAIL/0 ignored,
  1 filtered out,139.84초. #6761은 소스 SHA가 같은 앞선 실행의 PASS를 유지한다.
- 최소 글꼴 노출 목록은 `ttfs/opensource/`의 NotoSansKR Regular/ExtraLight,
  NotoSansSymbols2 Regular, RHWPPublicBatang Regular, RHWPOldHangul Regular 5개다.
- 무생성 `deriveManifest()` 검사에서 새 source를 발견했고 `regression_suite_002` 하나에
  배정함을 확인했다. generated harness·manifest·Cargo registry를 수정하거나 커밋하지 않았다.

고정 라이브러리 SHA-256: `109a1c756e51e389b95a306d6aa660ac9a65a9005c27fbc5fa06e6afa5025a16`.

- `tests/cases/issue_7755_market_flow_content.rs`: `da2fb2b200c877bbe02f353915954f8e8f03636d20f3998221215325e29cd73b`.
- `tests/cases/issue_6761_stored_vpos_rewind_page_break.rs`: `17e492b39ed6fe8f31d763b42e0a200f52b5be13839ee62bc028b34f703aa037`.
