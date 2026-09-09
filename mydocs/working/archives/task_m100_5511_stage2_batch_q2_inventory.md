# #5511 Stage 2 기능군 배치 Q2 — 출력 adapter inventory와 복잡도 중단

- 이슈: [#5511](https://github.com/edwardkim/rhwp/issues/5511)
- 브랜치: `task_m100_5511`
- 시작 기준선: `73811a7bc8c0b9e2b419a45314f5fe75ca1cb11c`
- characterization 커밋: `46579796b`
- 수행일: 2026-08-19
- 상태: 중단 조건 발동 — 책임 분해안 승인 대기

## 1. 조사 범위

Q2는 root의 다음 일곱 출력 handler와 전용 helper를 조사했다.

| 책임 | handler | 이동 후보 |
|---|---|---|
| vector/structure | `export_svg`, `export_render_tree`, `export_structure` | `cli/outputs/vector.rs` |
| raster/GPU | `export_png`, `export_png_gpu`, `gpu_info` | `cli/outputs/raster.rs` |
| PDF | `export_pdf` | `cli/outputs/pdf.rs` |

trial move에서 모듈 크기는 vector 770줄, raster 774줄, PDF 448줄로 모두 1,200줄 상한
이하였다. `src/main.rs`도 38,405줄에서 36,457줄로 1,948줄 줄어드는 구조였다. 기본
`cargo check --all-targets`와 `cargo check --features gpu --bin rhwp`가 통과했고, 관련 generated
suite 7개와 `cli_exit_codes`를 함께 실행한 focused 범위 807/807도 통과했다.

그러나 이 결과만으로 move를 커밋하지 않았다. 승인된 배치 계획의 CC>25 중단 조건을 별도로
계측했기 때문이다.

## 2. 보호 계약 inventory

기존 계약은 다음 사용자-visible 축을 이미 보호한다.

- SVG: JSON manifest, 기본 출력, 읽기·쓰기 실패의 stdout 순수성, 옵션 순서
- render tree: 고정 파일명, 페이지·옵션·실패 종료 코드
- structure: auto/outline/clause, JSON 봉투, 단건·batch 동형성, 쓰기 실패
- PNG: feature gate, native 종료 코드, profile·placeholder·페이지 산출
- PDF: 기본/명시적 SVG backend 바이트·stdout 동등성, direct backend와 backend 전용 옵션,
  JSON manifest, HML 입력

기본 build에서 직접 보호되지 않던 GPU feature stub 두 개는 기존 `tests/cli_exit_codes.rs`에
characterization을 추가했다. `export-png-gpu`와 `gpu-info`가 feature 미활성 시 종료 코드 2,
빈 stdout, 명령별 stderr와 build 안내를 유지하는 계약이며 1/1 통과 후 독립 커밋했다. 새 integration
source나 generated 산출물은 추가하지 않았다.

## 3. 중단 조건 증거

Clippy `cognitive_complexity`를 기본·`native-skia`·`gpu` feature에서 각각 계측했다.

| handler | feature 축 | CC | 판정 |
|---|---|---:|---|
| `export_svg` | 기본/native-skia/GPU | 38 | 중단 |
| `export_pdf` | 기본/native-skia/GPU | 32 | 중단 |
| `export_png_gpu` | GPU | 35 | 중단 |
| `export_png` | native-skia | 26 | 중단 |

`export_render_tree`, `export_structure`, `gpu_info`는 CC>25 경고에 나타나지 않았다. 하지만 네 개의
고복잡도 handler를 그대로 새 파일로 옮기면 기능군의 물리 위치만 바꾸고 복잡도를 숨기는 결과가 된다.
이는 “CC>25 함수를 분해 없이 다른 파일로 옮기지 않는다”는 Q2 승인 조건에 정확히 해당한다.

trial move는 커밋하지 않고 모두 복원했다. 따라서 작업 트리는 characterization 커밋만 가진 clean
상태이며, 제품 handler 소유권과 동작은 아직 바뀌지 않았다.

## 4. 선택지

### A. 책임 분해 후 이동 — 권장

같은 Q2 안에서 인자 파싱, 실행 준비, 페이지 산출·manifest 작성을 분리하고 각 함수 CC를 25 이하로
내린 뒤 세 output 모듈로 이동한다.

- SVG: option/parser, document/render preparation, page writer·manifest 분리
- PNG: option/parser와 native page writer 분리
- GPU: option/parser, context 준비, page raster·benchmark 집계 분리
- PDF: 공통 option/parser, backend option 검증, render·write·manifest 분리

각 분해 커밋마다 현재 807개 focused 범위와 feature별 compile/CC 계측을 적용한다. 출력 바이트·파일명·
stdout/stderr·종료 코드가 달라지면 즉시 중단한다. 최종 HEAD에서 Native Skia 3종, 전체 release-test와
정적·정책 게이트를 수행한다.

### B. 저복잡도 handler만 먼저 이동

`export_render_tree`, `export_structure`, `gpu_info`만 이동하고 나머지 네 handler를 root에 남긴다.
위험은 작지만 하나의 출력 기능군을 다시 쪼개 승인·검증 비용을 되살리고 shared helper 소유권도
임시 상태로 남긴다.

### C. Q2 보류

characterization 커밋만 유지하고 Q3로 넘어간다. 즉시 이동 위험은 없지만 root의 가장 큰 연속 출력
책임 1,950줄이 그대로 남아 Stage 2 종료 조건을 달성할 수 없다.

## 5. 원격 위험

최종 재조회에서 `origin/devel`과 `upstream/devel`은 `c5511c4e8`로 같고 작업 branch는 3커밋 뒤,
61커밋 앞이다. Q2 시작 뒤 병합된 두 커밋은 integration test 제출 문서·템플릿·`AGENTS.md` 절차를
갱신했으며 Q2 제품·test 경로와 겹치지 않는다. 최신 base와 현재 HEAD의 merge-tree는 충돌 없이
생성됐다.

열린 PR은 15개이고 Q2 대상 경로와 교집합은 0개다. 이 판정은 시점 증거이므로 구현 재개와 push
직전에는 exact SHA와 PR head를 다시 확인한다. remote push는 수행하지 않았다.

## 6. 승인 요청

권장안 A는 물리 이동만 하려던 Q2에 내부 책임 분해를 추가하지만, 제품 알고리즘이나 출력 규약을
바꾸지 않는다. 승인되면 characterization 계약을 기준으로 네 고복잡도 handler를 먼저 CC 25 이하로
분해하고, 같은 Q2 배치의 세 모듈 이동과 최종 검증까지 이어간다.
