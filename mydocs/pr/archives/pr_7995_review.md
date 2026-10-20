# PR #7995 리뷰 — Studio 생성 JS·타입 바인딩 동기화

## 최종 판정

**승인 — 검증된 WASM과 Studio 생성 바인딩을 일치시키는 범위에서 작성자 self-review를 완료했다.**
검토한 code head는 `8fa28a67e862b433e79cc793089edb5a878dc2a2`다. 실행으로 확인한 추가 결함이나
코드 검토상 blocker는 없다. 이 기록 이후 문서-only trailing head의 실제 CI와 최신 base 병합 가능 여부를
확인해야 한다. 작업지시자는 2026-10-17 self-review와 병합을 승인했다. 아직 병합 완료 기록은 아니다.

## 접수 정보와 검토 범위

- [PR #7995](https://github.com/edwardkim/rhwp/pull/7995), 작성자 `edwardkim`, base `devel`.
- 검토 base: `f9d2ab17ab438c6bed97d91c3c2a98a5be27e573`. 직접 종료할 issue는 없다.
  [#7980](https://github.com/edwardkim/rhwp/pull/7980)·[#7981](https://github.com/edwardkim/rhwp/pull/7981)은 산출물·기본 feature 동등성의 참조다.
- 기본 경로 `collaborator_self_merge`; 보조 `intake_and_review`, `local_validation`,
  `review_only_fast_pass`, `post_merge`. 생성 diff 1,000줄 초과로 `rework_and_exceptions`도 읽었다.
  reviewer는 지정하지 않는다. 원격 APPROVE event를 작성자 승인으로 위장하지 않는다.
- code diff는 `rhwp-studio/public/rhwp.js`, `rhwp.d.ts`, `rhwp_bg.wasm.d.ts` 세 파일,
  1,897줄 추가·40줄 삭제다. Rust·호출부·workflow·검사·fixture·기준값 변경은 없다.

## 코드·조판 원칙 검토

생성 JS diff는 기존 메서드를 제거·수정하지 않고 레이어 트리 V2·리소스 반환 wrapper 18개와
그 구현에 필요한 import를 추가한다. 선언은 현재 Rust/wasm-bindgen 계약에 맞췄으며,
JS·두 선언·WASM은 기존 검증 producer의 package와 파일별로 바이트 단위 일치했다.
`getPdfImportPolicy` 누락은 기존 바인딩 검사로 발견했고 검사 기대값을 유지한 채 올바른 산출물로 보정했다.

**조판 원칙 검토: 비해당.** 줄 구성·측정·pagination·paint의 계산과 Studio의 실제 호출 선택을
바꾸는 PR이 아니다. 이미 병합된 Rust 구현을 노출하는 생성 wrapper와 타입 계약을 동기화했다.
공통 helper 뒤 별도 좌표 보정·clamp·샘플별 예외, baseline·golden·허용치 변경도 없다.
새로운 문서의 조판 개선이나 전 backend 시각 일치 완료를 주장하지 않는다.

## 검증 입력·산출물 동일성

**충족.** 실제 브라우저에서 사용한 [samples/hwp_table_test.hwp](../../../samples/hwp_table_test.hwp)는
검토 head의 Git blob과 실행 파일이 같으며 29,184바이트,
SHA-256 `dc3e57d7577447ae4b47f59bdb0f499a5ac29066b62af8a650bcdd8d70eee32a`다.
새 샘플 사본은 추가하지 않았다. 이번 Studio 바인딩 검증은 독립 Print PDF의 새 비교를 필요로 하는
조판 계산 변경이 아니므로 PDF/전쪽 TSV를 새로 산출하지 않았다.

| 산출물 | SHA-256 |
| --- | --- |
| JS | `7dcd8775b5ff2e6d6d46f6811d03cd3e0f0db9d7d0f0d1a0606741e762bce06e` |
| 고수준 선언 | `c56c89a9fe1060e04ca5acd84b3eb1d2f2b56e17ecb0ab945d0b9d1230fcd382` |
| 저수준 선언 | `53002a58e6455e6937bf091e84797c81c2e50d909b51eebd4d63402a4f94398e` |
| Docker WASM (18,863,606바이트) | `50ead3ca44c8f807f44aec6e575b4d61426b1b8828bd182d1864fcd440770f57` |

[#7980의 산출물과 시각 기록](pr_7980_review.md)을 재사용했다. #7981의 기본 활성 `wasm-api`
feature를 전개한 Rust 선택 소스·manifest 동등성을 대조했고 renderer·PDF backend·model·paint·
글꼴·lockfile은 동일하다. 기존 Native/fresh Docker WASM 41쪽 검증을 이번 head의 새 빌드·캡처로
기록하지 않는다. 동일 WASM 재빌드·Rust 전체 검사는 반복하지 않았다.

## 로컬 검증과 정확한 code-head CI

로컬 성공 이후 세 제출 파일의 해시가 바뀌지 않았음을 확인했다. 기존 로그·ABI·브라우저 JSON은
로컬 영구 증적 `/home/edward/mydev/rhwp-review-evidence/studio-wasm-types-pr7995-20261017/`에
후속 보존한다. 이것은 public 재현 입력을 대체하는 경로가 아니다.

| 로컬 검사 | 실제 결과 |
| --- | --- |
| Studio `./node_modules/.bin/tsc --noEmit` | 성공 |
| Studio `npm test` | 2,316 PASS / 2 skip / 0 FAIL |
| `node --test scripts/frontend-wasm-bindings.test.mjs scripts/frontend-editor-embed.test.mjs` | 3 PASS |
| 실제 WASM ABI | 선언한 export 520개 모두 존재, 정책 JSON 반환 |
| Chrome 152·Studio 7700 | 표 샘플 3쪽·Canvas 표시, 문서 객체의 정책 API 호출, pageerror 0 |
| diff·merge-tree | 공백 오류·충돌 없음 |

처음의 오래된 pkg 검사에서는 `getPdfImportPolicy` 선언 누락 1건이 실패했다. 생성 산출물 일치 보정 뒤
위 검사를 통과했다. 진단 스크립트의 inspector cache·별도 미초기화 모듈 문제와 제품 결함을 구별했다.
최종 실제 Studio 객체를 사용한 브라우저 검증이 성공했다. 이 self-review에서 동일 검사를 다시 실행하지 않았다.

정확한 code head `8fa28a67e862b433e79cc793089edb5a878dc2a2`의 다음 6 workflow가 모두 성공했다.

- [CI](https://github.com/edwardkim/rhwp/actions/runs/39036810049)
- [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/39036810162)
- [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/39036809968)
- [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/39036810074)
- [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/39036810087)
- [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/39036809715)

35 check는 19 success·15 skipped·CodeQL 요약 1 neutral이며 실패·미완료는 0건이다.
CI preflight는 `classified:studio-render-package`, `frontend_mode=package`를 선택했다.
Frontend package·Canvas readiness·final Build & Test의 실제 성공과, Rust archive·lint·Native Skia의
정책상 skip을 구분한다. Rust Full CI가 새로 실행됐다고 기록하지 않는다.
PR merge ref의 열린 code-scanning alert는 조회 시점 0건이다.

## 남은 순서와 병합 후 기록 계획

1. 이 review와 오늘할일만 문서-only commit으로 추가한다.
2. 최신 base/head의 merge-tree·문서 링크·기존 오늘 기록 보존을 확인한 뒤 push한다.
3. trailing head의 실제 required CI·fast-pass 재사용 결과와 mergeability를 확인한다.
4. 이미 승인된 정상 squash merge를 정확한 head에 한정해 수행한다. 보호 정책 우회는 하지 않는다.
5. merge 뒤 duration metadata 갱신 결과 또는 증거 부족의 보류 사유를 확인하고 devel을 동기화한다.
6. PR에 merge SHA·CI·이 검토 기록의 고정 링크를 한국어 존댓말로 남긴다. 별도 종료 issue는 없다.
   시각 개선 PR이 아니므로 새 review PNG나 시각 점수를 추가하지 않는다.
7. 이 PR 전용 branch·증적을 안전하게 정리한다. 공용 target·사용자 Studio·다른 작업은 보존한다.
   root output의 다른 자료 소유가 불명확하면 전체 정리 미완료 사유를 남긴다.
