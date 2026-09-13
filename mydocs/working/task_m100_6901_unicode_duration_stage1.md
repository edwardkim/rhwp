# #6901 Unicode duration 증거 계약 보정 - 단계 1

## 분석

- 실증 대상: [PR #7044](https://github.com/edwardkim/rhwp/pull/7044), merge `78bdfd9aa69417b4a88ffdd43aa76d3cad471f44`.
- [CI verifier](https://github.com/edwardkim/rhwp/actions/runs/34669914465/job/103489207024)는 `candidate-duration-artifact-check-error`로 Full 전환했다.
- 원 PR CI `34662415596`의 실제 B/C/D artifact를 소비기로 재현했다. 이름 규칙만 위반한 한글 테스트 13개(B 3, C 2, D 8)가 원인이다. 중복이나 잘못된 duration이 아니다.
- JUnit 발행기는 Unicode 이름을 보존하지만 소비기는 ASCII만 허용한다. merge-tree 증거는 유효했고 CodeQL은 실제 재사용됐다.

## 수정 범위와 검증 계획

- 소비기의 이름 계약을 Unicode XID 식별자의 `::` 경로로 한정한다. 공백, 경로 문자, 빈 segment, 보이지 않는 제어 문자는 허용하지 않는다.
- 기존 저장소/PR/head/attempt/merge SHA, ZIP, 크기, 중복, 수치, target 소유 검증을 유지한다.
- JUnit 발행 → ZIP → 소비 → policy 갱신 계약과 악성 이름 거부를 테스트한다.
- 실제 #7044 B/C/D artifact를 수정된 소비기로 재검증한다. 계약 테스트도 실행한다.
- 이 단계는 로컬 계약 보정이다. 배포 후 실제 fork PR의 CI reuse/heavy skip/duration refresh가 입증되기 전 #6901은 종료하지 않는다.

## 결과

- `trusted-postmerge-duration-evidence.mjs`만 제품 로직을 수정했다. 발행기는 이미 이름을 보존하므로 변경하지 않았다. workflow 권한이나 재사용 gate는 완화하지 않았다.
- Unicode XID segment와 `::`만 허용하고 default-ignorable 문자를 거부한다. 빈 segment, 숫자/결합문자 시작, 경로, 공백, NUL, bidi, ZWJ, variation selector 거부를 회귀 계약으로 잠갔다.
- 집중 JavaScript 계약 89 PASS, post-merge/duration 전체 JavaScript 계약 520 PASS, Python workflow 계약 17 PASS. 중복 집계이므로 89를 520에 더하지 않는다.
- 실제 PR CI `34662415596` ZIP의 B 1,973 / C 1,555 / D 1,909개, 총 5,437개 test case를 승인했다. 이전에 실패했던 Unicode 13개도 모두 원문 그대로 보존했다.
- `git diff --check` 통과. Rust/브라우저를 수정하지 않았으므로 이 보정에서 Rust 회귀·WASM 빌드는 반복하지 않았다.
- 한글 JUnit → ZIP → 소비 → duration policy 갱신을 검증했다. 실제 원격 post-merge 재사용 성공은 아직 미검증이며 #6901은 OPEN이다.
