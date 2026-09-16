---
kind: snapshot
status: active
canonical: mydocs/pr/archives/pr_7199_review_impl.md
last_verified: 2026-09-16
---

# PR #7199 메인터너 보정 — 1회차

## 분석

- 원 head: `5356d0c0a3f1c6e687993defc5831c3e28f3cf6d`, 통합 code `ca0db01b2`, 리뷰 기록 `ee36869bf`.
- base: `6cd3c0692a3ed9def7f7e7af1ea03cad0c1a0aaa`; 원격 devel도 동일함을 재확인했다.
- 재현 원인: `stored_tac_line_assignment`로 나눈 실제 TAC 배치를 무시하고, 소유자 탐색이
  전체 문단의 가시 문자 구간을 다시 계산해 이전 줄의 표를 끌어온다.
- 적용 규칙: 실제 방출·정렬이 사용하는 줄별 TAC 집합을 소유자 탐색에도 사용한다.
  마지막 run 끝 TAC와 저장 UTF-16 줄 경계를 모두 보존해야 한다.
- 기대값 근거: 이전 줄 바깥여백 합을 유지한 채 상하 배분만 바꾸어도 다음 줄 표의 y는 불변이다.
  합성 입력은 이 불변성만 검증하며 한컴 PDF 기준으로 주장하지 않는다.
  원본 한컴 PDF는 기존 issue2470 입력의 결재표 위치·하단차 보존을 별도로 입증한다.
- 수정 범위: 줄별 TAC 집합 전달·소유자 탐색 재사용, 정식 경계 테스트, 검토 결과 갱신.
  기존 PDF 차이·일반 표 높이 계산·다른 inline 객체 기준선 정책은 확대 수정하지 않는다.
- 검증 순서: 경계 시험 RED → 수정 → focused/음성 대조 GREEN → Native·fresh WASM
  Visual Sweep(compare/overlay/review) → 새 코드 lint/필요 검증 → 실제 결과보고 → 커밋.

## 코드 수정과 경계 검증

- `line_tac_offsets_for_width`를 `emit_line_runs`에 전달했다. 저장 줄 귀속과
  마지막 run 끝 TAC를 포함한 기존 집합을 소비하며 새 문자 범위 필터는 제거했다.
- `line_table_owner`는 run/표 반복문 밖에서 줄당 한 번 계산했다.
  서로 모순되던 독점 줄 설명과 “같은 합이면 높이·여백도 같다”는 잘못된 주석을 제거했다.
  복수 후보의 첫 후보 선택 정책 자체는 이번 보정에서 확대 변경하지 않았다.
- 정식 `previous_line_table_margin_does_not_move_the_next_line_table` 테스트를 추가했다.
  두 기존 합성 fixture를 제품 `DocumentCore::from_bytes` → page render tree로 실행하고,
  표 개수 보존과 다음 줄 표 y 불변성을 검증했다.
- RED: 보정 전 #7150 5건 중 4 PASS / 새 경계 1 FAIL(exit 100).
  관측 `318.695 → 320.028px`. 처음에는 이전 generated suite 번호로 실행되어 0건(exit 4)이었고,
  이를 통과로 세지 않았다. `run-rust-test.mjs`로 재해석한 위 5건이 실제 음성 대조다.
- GREEN: 보정 후 #7150 5건 + #7049 4건 + #6754 2건 + #6706 1건 = **12/12 PASS**.
  독립 CLI 진단도 `320.0/320.0px`, delta 0.0, exit 0이었다.
- generated suite 배치는 바뀔 수 있으므로 재현에는 고정 suite 번호보다
  `node scripts/run-rust-test.mjs <case>`를 우선한다.

## 직접 시각 검증

Native와 새 host dev/no-opt WASM에서 기존 입력 2문서 3쪽의 compare·overlay·review를 새로 만들고 직접 열었다.
WASM은 Chrome/153.0.8010.47에서 새 `rhwp.js`/`rhwp_bg.wasm`으로 실행했다.
Docker daemon 미연결로 host 진단 경로를 썼으며 최적화된 Docker 배포 빌드 완료로 표현하지 않는다.

| 입력 / 페이지 | pixel match | ink match / visual_accuracy_proxy | 판정 |
| --- | ---: | ---: | --- |
| issue2470 / 1 | 97.61166% | 46.74444% | 원 PR 결재표 개선 유지, Native SVG/tree가 보정 전과 동일 |
| issue2470 / 2 | 94.65651% | 15.32564% | 기존 사진칸·후속 본문 차이 불변 |
| hwp3-sample16-hwp5 / 18 | 77.98972% | 29.83262% | 표 제목→그림→다음 제목의 순서·위치 보존; SVG/tree가 devel과 동일 |

Native/WASM 지표가 같았고 자동 후보는 각 target 0건이다. 전체 문서의 한컴 출력 일치 주장이 아니다.
issue2470의 1·2쪽 standalone overlay를 각각 다시 열어 판독했다. 색상은 빨강=rhwp에만 있는
잉크, 파랑=PDF에만 있는 잉크, 주황=양쪽 잉크의 색상/형태 차이, 회색=임계값 내 일치다.
1쪽 결재표 위치는 개선됐지만 제목·기관명 두께와 점선 상자는 다르다.
2쪽 사진칸은 Native render tree에서 y=424.7, h=154.2px이고, PDF vector 괘선은
96DPI 환산 y=424.336–441.437px(높이 17.101px)이다. 사진칸 높이 차이 약 137.1px와
후속 본문 하향은 육안 overlay에서 확인했으며, devel/PR/보정 SVG·tree가 동일하므로
이번 줄 소속 보정에서 새로 생기거나 해결한 차이로 분류하지 않는다.
문서 전체의 PDF 시각 일치를 승인하지 않는다.

18쪽의 글꼴·그림 색상·테두리 차이는 남아 있다. 라벨과 overlay를 직접 판독했으며
그림의 존재·위치와 뒤 본문을 확인했다. `fidelity_compare --text-only --export-all-svg --layout-ledger`도
각 대상 페이지에서 다시 실행했다.

issue2470의 갱신한 WASM PNG 4개는 기존 커밋 파일과 byte-identical하여 중복 파일을 추가하지 않는다.
18쪽의 대표 review/standalone overlay만 기존 assets 디렉터리에 추가한다.
파일 이름 `-2022.pdf`와 달리 18쪽 기준 PDF의 실제 Creator는 `Hwp 2024 13.0.0.3457`이다.
기존 PDF를 재사용했고 새 한컴 변환은 하지 않았다.

실행 루트: `/private/tmp/rhwp-pr7199-evidence-20260916`.
Native `maintainer-native/{issue2470,issue6706}`, WASM
`maintainer-wasm-2470/issue2470`, `maintainer-wasm-6706/issue6706`의 compare/overlay/review를 확인했다.
원시 로그/JSON/TSV는 임시 위치에만 보존한다.

## 검증 대상 파일 식별

실행 당시 checkout HEAD는 `ee36869bf1f21441583f2a00e9b13644556e7ec0`이며 아래 소스 수정이 working tree에 있었다.
검증 이후 구현을 바꾸지 않고 아래 내용으로 code commit을 만든다. 확정 SHA는 review 문서에 연결한다.
실행 manifest의 HEAD만으로 보정 전 바이너리를 실행했다고 해석하지 않는다.

| 파일 | SHA-256 |
| --- | --- |
| `src/renderer/layout/paragraph_layout.rs` | `ee3ea2086106b3ab317cb5f36280d88ac7cc147f1b792f8a95e688159409ca9a` |
| `tests/cases/issue_7150_tac_line_owner_anchor.rs` | `59bb25605be744807edd7b56e2fd87087d4d416a81face54bfc24dc3d38cf424` |

실제 sweep manifest의 exporter SHA-256:

- Native CLI: `a874b87116bc3bb4dce4785cefaf15e58294ea0930c881da7b5b370b667575a8`.
- fresh `rhwp.js`: `a490dc79102cbc6047252da243f0901d078997bef545d7c38d5143093e0bc781`.
- fresh `rhwp_bg.wasm`: `053e5d623570de6e0662d1bb289bd687ca71722e8a2163de52c2302fe4f06c08`.

## 최종 검증 결과보고

아래 검증은 위에 식별한 새 코드에서 순차 실행했고 전체 실행기 exit 0을 확인했다.

| 검증 | 실제 결과 |
| --- | --- |
| CLI 및 workspace build | `cargo build --locked --bin rhwp`, `cargo build --locked --workspace` PASS |
| fresh WASM | locked wrapper `--target web --no-opt --dev` PASS |
| fmt | `cargo fmt --all -- --check` PASS |
| native lint | `cargo clippy --locked -- -D warnings` PASS |
| WASM lint | `cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown -- -D warnings` PASS |
| workspace lint | `cargo clippy --locked --workspace --all-targets -- -D warnings` PASS |
| manifest | `node scripts/rust-test-suite-manifest.mjs --check --base-ref 6cd3c0692a3ed9def7f7e7af1ea03cad0c1a0aaa` PASS |
| 전체 회귀 | `cargo nextest run --locked --cargo-profile release-test --tests --no-fail-fast`: **9,938 PASS**, 51 skipped, 4 slow, exit 0 |
| Native Skia lib | `cargo test --locked --profile release-test --features native-skia --lib`: **4,112 PASS**, 13 ignored, exit 0 |
| Native Skia 그림 | `run-rust-test.mjs issue_2225_missing_picture_placeholder -- --cargo-profile release-test --features native-skia`: **2 PASS**, 206 filter skipped |
| Native Skia PDF | `run-rust-test.mjs render_p37_direct_pdf_export -- --cargo-profile release-test --features native-skia`: **4 PASS**, 199 filter skipped |

전체 회귀는 빌드 포함 892.87초, 실제 테스트 375.329초였다. Skia lib는 기본 workspace 구성원
각각 3,930 + 15 + 165 + 2건을 통과했다. skipped/ignored는 성공으로 합산하지 않았다.
기존 nextest 설정의 `report-skipped` 미인식 warning은 있었으며 모든 실행 exit는 0이었다.
코드 내용이 시각 검증 당시 SHA-256과 같음을 최종 재확인했다. 변경 문서 metadata·로컬 링크와
`git diff --check`도 통과했다.

재현된 줄 경계 P2는 해결됐고 원본 결재표 개선과 대조 입력의 위치는 보존됐다.
issue2470 2쪽의 기존 사진칸 높이 및 후속 본문, 글꼴·색상 등 명시한 시각 차이는 남는다.
최적화 Docker WASM, 원격 보정 head CI, 원격 push/merge는 이번 로컬 보정에서 수행하지 않았다.
분석·수정·검증·이 결과보고를 함께 커밋하고 확정 code SHA를 검토 문서에 연결한다.
