# PR #8784 self-review — crates.io 필수 설명과 업로드 전 검사

## 판정과 승인

승인 — code head `4a765822988575099104c30081516b2bb7c4ddb5`의 Full CI와 CodeQL,
Adapter, Proptest, 최종 정책 감사가 모두 성공했다. 실제 crates.io의 업로드 거부 원인을
보정했다. 이 문서-only trailing head의 required check와 최신 base 병합 가능성은 별도로 확인한다.

- PR: https://github.com/edwardkim/rhwp/pull/8784
- 검증 base: `2a9a88a0ddc59eb6757ee73e03eee42b4e3ac497`.
- 경로: collaborator_self_merge; intake_and_review, local_validation, review_only_fast_pass,
  post_merge를 함께 적용했다.
- 작업지시자는 보정 PR·CI·self-review·병합과 미게시 12개 패키지의 게시·공개 소비 검증을
  승인했다. v0.9.0 태그 및 이미 게시된 두 패키지의 불변성, 나머지 패키지의 보정 source 사용도
  명시 승인했다. main PR #8783의 CI·리뷰 개수 예외는 이 PR에 적용하지 않는다.

## 검토 결과

1. 세 번째 패키지 `rhwp-jpeg-decoder` 게시가 HTTP 400으로 거부된 실제 응답에서 필수
   `description` 누락을 확인했다. Cargo dry-run은 경고만 출력하므로 업로드 서버의 거부를
   선행 검증하지 못했다. 실제 포장 대상 manifest에 설명 한 줄을 추가했다.
2. 공통 preflight는 `.crate`의 Cargo 정규화 manifest를 읽어 설명이 문자열이고 공백 제거 후
   비어 있지 않은지 확인한다. 원본 manifest만 검사하거나 서버 경고를 무시하지 않는다.
   누락·빈 문자열·공백만 있는 설명의 세 대조군을 추가했다.
3. 변경은 3개 파일, 14줄이다. Rust 제품 소스·feature·의존 버전·workflow·검사 실행 범위는
   바뀌지 않았다. 조판 원칙과 새 시각 fixture, Visual Sweep·편람 gate는 비해당이다.
4. 태그 `c9095c5f67632e6721e800ccc7fb43ea49d26682`와 게시된
   `rhwp-codec-resource 0.1.0`, `rhwp-contracts 0.1.0`은 보존한다. 미게시 12개는
   승인된 code head에서 포장하고 패키지별 source SHA·archive checksum을 기록한다.
   서로 다른 source를 단일 태그 산출물이라고 주장하지 않는다.

## 완료한 로컬 검증

- 세 실패 대조군은 수정 전 FAIL, 수정 후 PASS이며 기존 계약 포함 40개 검사가 통과했다.
- 실제 `cargo publish --locked --dry-run`이 성공했고 archive SHA-256은
  `1f2bd23d396ee20440f3909fe3248dc749272f0952b7387b11eababecf9ec9af`다.
  archive의 VCS source는 검증한 code head와 같다.
- 14개 포장의 필수 설명·라이선스·정규화 manifest를 확인했다.
  [포장 검증](../assets/pr8784/package-validation.json).
- 최신 base와 code head의 merge simulation은 충돌 없이 성공했고 tree
  `b0da59cd65f729e390c8ece6b35df4a313c32845`는 GitHub의 합성 merge tree와 같다.
- 녹색 code head 뒤에 제품 변경이 없어 로컬 전체 회귀를 반복하지 않는다.
  문서-only trailing push 전에 실제 merge tree의 공백·링크·오늘할일 보존을 검사한다.

## GitHub CI와 복구

- [Full CI](https://github.com/edwardkim/rhwp/actions/runs/42471035995) attempt 2 성공:
  기본 회귀 **12,803건**, 선택 Native **237건**, Studio **2,778건 통과·기존 2건 skip**.
  Lint·Frontend 후속 검사와 Build & Test 집계도 성공했다.
- [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/42471036101) attempt 2,
  [Adapter](https://github.com/edwardkim/rhwp/actions/runs/42471036023),
  [Proptest](https://github.com/edwardkim/rhwp/actions/runs/42471035929),
  [최종 정책 감사](https://github.com/edwardkim/rhwp/actions/runs/42475907513)가 성공했다.
  Render Diff는 이번 diff의 path filter상 비대상이며 성공으로 집계하지 않는다.
- 최초 CodeQL JS·Actions와 Frontend는 공식 도구 다운로드 HTTP 504로 검사 전에 실패했다.
  Native는 140건 통과 후 후속 컴파일 중 30분 한도로 취소됐다. 각각의 실제 로그를 확인하고
  명시 승인된 실패 작업·종속 집계만 한 번 재실행했다. 공식 파일 전체 다운로드와 무결성을
  재확인했으며 성공한 다른 job을 유지했다. 추가 코드·workflow·시간 예산 변경은 없었다.
- 수치는 실제 로그와 shard artifact에서 산출했다.
  [CI 증거](../assets/pr8784/ci-results.json) · [승인된 재실행 영수증](../assets/pr8784/recovery.json).

## 후속 조건

검토 기록·오늘할일 trailing head의 실제 fast-pass와 required check를 확인해 병합한다.
그 뒤 미게시 12개 패키지를 의존 순서대로 게시하고 공개 index·owner·checksum 및
Rust·CLI·WASM 소비를 검증한다. 아직 이를 게시 완료로 기록하지 않는다.
게시에는 문서 trailing head가 아니라 승인된 code source `4a76582298`을 사용한다.
현재 릴리즈가 사용하는 output·target은 사용 종료 뒤 정리한다.
