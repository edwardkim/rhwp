# Task M100 #2214 Stage 8 완료보고 — 최신 devel 통합·CI 복구

## 1. 결론

PR #2241의 CI 실패는 #2195가 만든 `render_normalized` 파생 복사본에 deferred 셀 편집이
반영되지 않은 coherence 누락이었다. #2195를 되돌리지 않고, 편집된 normalized cell paragraph와
그 셀의 pointer-key layout cache만 국소 갱신해 두 변경을 양립시켰다.

## 2. 최신 devel 통합 보정

`upstream/devel@6cfc4cec`를 병합했다. 최신 table text baseline 보정으로 cursor y 기준선이
바뀌었지만 HWP/HWPX 및 flush 전후 결과는 동일했다.

| query | x | y | bounds h |
|------|--:|--:|---------:|
| path-near | 569.7 | 344.8 | 945.9 |
| direct | 569.7 | 345.6 | 945.9 |

44번째 입력은 여전히 target paragraph의 상대 flow advance를 바꾸므로
`cellFlowChanged=true`와 cursor 전 full pagination 1회를 유지한다. #2195 이후 page 0은
deferred/flush 모두 cut 37·bounds 945.9지만 page 2~114의 continuation cut 113개가 재정렬되므로
경계 flush는 정확성 계약상 필요하다.

## 3. 영구 회귀 보정

- native cold/warm cursor 기준선을 최신 devel 값으로 갱신했다.
- 브라우저 E2E의 #2195 이전 post-flush bounds 971.5 기대값을 945.9로 갱신했다.
- HWP/HWPX 각 3회에서 44번째만 flush 1회, 1~43·45~50번째는 추가 flush 0회를 확인했다.
- IME/iOS raw stable 4건은 flush 0회, boundary 4건은 flush 1회로 통과했다.

## 4. 검증 결과

| 검증 | 결과 |
|------|------|
| `cargo test --profile release-test --tests` | 모든 test binary 실패 0 |
| #2214 native integration | 3 passed / 0 failed |
| #2214 30-case diagnostic matrix | 완료 / HWP·HWPX flush 전후 exact |
| Native Skia CI 축 | 49 lib + 2 integration passed |
| Clippy | all targets/features, `-D warnings`, 경고 0 |
| Rustfmt / diff check | 통과 |
| Studio unit | 303 passed / 0 failed |
| WASM binding/editor embed | 3 passed / 0 failed |
| Studio build | 통과 |
| #2214 browser | focused 6/6, raw 8/8 GREEN |

브라우저 계측은 dev WASM으로 실행했으므로 절대 시간은 성능 기준으로 사용하지 않고,
flush 횟수·순서와 exact state만 정확성 증거로 사용한다.

## 5. 후속 분리

이번 보완은 현재 mutable normalized clone 구조 안에서 안전한 scoped coherence를 제공한다.
장기적으로는 editable IR을 단일 권위 상태로 두고 render normalization을 revision 기반 derived
cache 또는 overlay로 전환해 경로별 mirror 코드를 제거하는 별도 설계 이슈가 필요하다. 이는
PR #2241 merge 필수 범위가 아니며 #2193의 bounded/partial paginator 설계와도 구분한다.
