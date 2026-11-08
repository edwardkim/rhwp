# #7207 저장 앞 프레임 정렬의 최소 보정 — #7544 재검토의 선행 작업

Issue: #7207. 관련 PR: #7544 (원 이슈 #7486).
기준: `upstream/devel` `48ff4bb9353bf8d29b7e1c0ac44df9bc5800dce8`.

## 범위

원래 사용자 요청은 #7544의 리뷰·병합이다. 그 재검증에서 최신 devel에도 어구
문서의 저장 앞 프레임 정렬이 사라지는 기존 결함을 확인했다. 사용자 승인에 따라
이미 분리한 이 작은 보정만 선행 PR로 준비하며 #7207 전체 또는 #7445 교육과정
전체 개선을 인수하지 않는다. 소스 변경은 `table_partial.rs`의 기존 정렬 소유
조건을 줄 구성과 최종 원점 선택에서 함께 소비하게 하는 13줄이다.
기존 검사를 보강하며 컷·높이·guide·Enter·Studio source는 변경하지 않는다.
이 선행 PR은 #7207 또는 #7486을 닫지 않는다.

#7207은 postmelee에 할당돼 있다. #7574의 기존 병합 증거를 역사적 결과로
유지하며 최신 devel의 Native/fresh WASM 전21쪽을 같은 한컴 PDF와 다시 비교한다.
7쪽 표 하단, 9쪽 다열 셀 문구, 닫는 선과 19쪽 범례선의 기존 차이는 이번
최소 보정의 범위 밖이다. 90% 점수나 기존 세 회귀 통과로 이를 해결됐다고 쓰지 않는다.
다른 두 문서의 #7207 판정도 이 선행 PR에 포함하지 않는다.

#7544는 원격 head `e22b099a28`를 유지한다. 최신 base와의 병합 시뮬레이션은
`mydocs/orders/20261003.md`의 add/add 한 건이며 Rust source는 자동 병합된다.
새 로컬 후보에서 양쪽 운영 기록을 보존하고 정확한 source의 Enter·저장/재열기·
실제 저장 guide 경로를 검증한다. 교육과정 전체 결함을 대신 수정하거나 필수
시각 근거 부족을 면제하지 않는다. 원격 push·PR 갱신·댓글·승인·병합은 미수행이다.

## 검증 순서

1. 정확한 최신 devel Native 빌드와 같은 원문/독립 PDF의 전21쪽 TSV·대표 PNG.
2. fresh WASM 인쇄 출력과 Native 의미/위치·시각 비교.
3. #7544를 로컬 병합한 후보에서 관련 기존 회귀와 합성 Enter7쪽·실제 저장본 영향 비교.
4. 적용/비적용 경로와 독립 기준으로 필요한 #7207 보정만 분리한다. 이전 교육과정
   후보를 통째로 합치거나 문서 ID·수치 예외, baseline 완화로 통과시키지 않는다.
5. 실제 실행 결과·소스/입력/바이너리 해시·남은 범위는 이 기록에 연결한다.

아래 1회차 결과는 이미 실행한 정렬 보정의 결과다. #7544 로컬 후보 재검증은
별도 branch에 보존했으며 이 선행 PR에는 포함하지 않는다. 선행 code head
`cf2b0508fc13812b51b345eacf16a47baa6db6ec`의 필수 로컬 검증을 완료했다.
원격 CI와 mergeability는 PR 생성 후 정확한 최종 head에서 확인한다.

## 1회차 결과 — 저장 프레임의 정렬 소비 복원

정확한 최신 base에서도 어구 11~15/17~18쪽이 Native/fresh WASM 각각
82.66/82.75/82.42/82.99/81.71/34.67/26.39%로 실패했다. #7544 적용 전
검증이므로 그 PR의 새 회귀로 세지 않는다. 원본 지도 셀은 Center이고
독립 PDF17쪽은 첫 지도 앞에 빈 밴드를 보존하지만 현재 최종 배치는 Top으로
되돌려 그 밴드를 없앴다. 이전 교육과정 후보의 다른 코드를 가져오지 않았다.

`table_layout.rs::stored_full_width_row_declared_height`가 원본 저장 프레임을
확인하고 `fragment/emit.rs`가 소비한 프레임과 가용 예산으로 요구 높이·
`end_row_height_override`·예약 높이를 만든다. 기존 생산·예산·내용 컷은 유지한다.
`table_partial.rs`에서 이 프레임을 줄 구성 창에는 인정하지만 최종
`effective_align`에는 이어받기 프레임만 인정하던 가정을 제거했다.
두 단계가 `frame_owns_alignment`를 함께 소비하며 이후 정렬 높이는 기존
`centered_content_height`의 같은 저장 프레임 계약을 따른다. 좌표 clamp나
문서/행/쪽 번호 분기를 추가하지 않았다. 편집·재조판/Top/여러 셀 소유 경로는
기존 생산자의 수용 조건을 유지하고 이번 앞 프레임 분기를 적용하지 않는다.

| 확인 | 실행 결과 |
| --- | --- |
| 어구 Native/fresh WASM 전21쪽 TSV | 최저93.22327%(7쪽), 90% 미만/누락0 |
| 본문11~15쪽·지도17/18쪽 | 98.43249/98.24875/98.29738/98.33620/98.14148 / 97.32653/94.51240% |
| 정상 RowBreak Native/fresh WASM 전18쪽 TSV | 최저92.48319%(12쪽), 90% 미만/누락0 |
| 어구 전21쪽 수정 전후 tree | 바뀐 쪽11~15/17~18, 텍스트 순서 변경0, 전체21쪽 유지 |
| 정상 대조군 전18쪽 수정 전후 tree | 차이0 |
| 어구21/정상18쪽 Native/fresh WASM tree | 구조·내용·좌표 차이0(좌표1e-5·아키텍처별pi sentinel 동등화) |
| 강화한 #7207 기존 지도 검사 | 같은 검사/원문에서 수정 전FAIL(exit101) → 후PASS(exit0); 빈 밴드 관계로 검출 |
| #7207 기존3개 + #7518 기존23개 | 실제26PASS |
| fmt·fmt check·PR 범위 공백 검사 | PASS |
| Native/WASM lib/workspace all-target Clippy (`-D warnings`) | 세 단계 모두 PASS |
| locked workspace build·manifest base 비교 | PASS; base `48ff4bb9353bf8d29b7e1c0ac44df9bc5800dce8` 고정 |
| release-test 전체 integration | 10,509 PASS / 0 FAIL / 50 skipped |
| Native Skia lib | 4,109 PASS / 0 FAIL / 13 ignored (workspace 네 lib 합계) |
| Native Skia 누락 그림·직접 PDF | 2 PASS + 4 PASS |

테스트 강화는 양쪽 출력39쪽의90% 이상과 대표 이미지 직접 판독 뒤 수행했다.
원본의 Center 속성과 독립 PDF17쪽의 빈 밴드를 기대 근거로 사용하며
실물 절대px/전체SVG hash 또는 구현의 정렬 offset을 assertion으로 고정하지 않는다.

Native17/19/7쪽과 fresh WASM17쪽 review를 직접 판독했다. 지도 위치·캡션·
뒤 제목과 내용 소속은 복원됐고 7쪽 하단 괘선 및19쪽 범례선 차이는 여전히
남는다. 해당 차이는 이번 변경 전후 동일하며 #7207 전체 종료를 주장하지 않는다.

명령은 `cargo build --locked --profile release-test --target-dir target/pr-review
--bin rhwp`, 루트 wrapper `--target web --out-dir ../pkg-alignment --mode no-install`,
공식 `visual_sweep.py --silhouette-only`의 Native/`--wasm-pkg` 전쪽 비교와
`run-rust-test.mjs`의 두 모듈 실행이다. WASM은 호스트 wrapper의 wasm-opt
최적화 성공이며 Docker 표준 경로 통과로 보고하지 않는다. 최초 패키징 실행의
샌드박스 제한, 오래된 samples 링크로 인한 fixture 누락, 검사 길이 변경 후
파생 suite 이동에 따른0건 선택은 모두 성공/결함 증거에서 제외하고 재실행했다.

정확한 source/patch/binary/package hash·전후/Native-WASM tree 대조·TSV·
RED/GREEN 로그는 ignored `output/pr-review/issue7207-7544-focus-20261007`과
해당 임시 sweep 디렉터리에 보존한다. 새 fixture·baseline 변경은 없다.
필수 lint는 `cargo fmt --all -- --check`, Native Clippy, WASM32 lib Clippy,
`cargo build --locked --workspace`, workspace all-target Clippy를 순차 실행했다.
모든 Cargo 실행은 공유 `target/pr-review`를 사용했다. manifest 검사는
`node scripts/rust-test-suite-manifest.mjs --check --base-ref 48ff4bb9353bf8d29b7e1c0ac44df9bc5800dce8`이다.
전체 integration은 `cargo nextest run --locked --cargo-profile release-test
--target-dir target/pr-review --tests --test-threads 4 --no-fail-fast`, Native lib는
`cargo test --locked --profile release-test --target-dir target/pr-review
--features native-skia --lib`로 실행했다. Native focused 두 모듈은
`issue_2225_missing_picture_placeholder`, `render_p37_direct_pdf_export`다.

첫 workspace build의 저장공간 부족과 앞선 전체 회귀의 sparse worktree fixture
누락은 환경 실패로 기록하고 성공 증거에서 제외했다. 동일 commit의 누락 파일을
복원하고 다시 실행한 최종 결과가 위 표다. 제품 source·기대값·baseline은 바꾸지
않았다. source-side cfg(test) 변경은 없어서 unit-tier 검사는 비해당이다.
원격 CI·push·PR·승인·병합은 로컬 검증과 구분한다. #7544는 별도 로컬 후보에서
재검증하며 이 선행 보정을 #7544 완료로 보고하지 않는다.

대표 증적: [변경 전17쪽](../working/assets/issue7207-center-frame-20261007/before-native-map17-review.png),
[변경 후Native17쪽](../working/assets/issue7207-center-frame-20261007/after-native-map17-review.png),
[fresh WASM17쪽](../working/assets/issue7207-center-frame-20261007/after-wasm-map17-review.png).
