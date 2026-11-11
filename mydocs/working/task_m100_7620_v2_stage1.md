# #7620 후속 1단계 — RowBreak 자리 차지 표의 첫 조각 바닥 예산

2026-11-10 KST. [이슈 #7620](https://github.com/edwardkim/rhwp/issues/7620)의 남은 항목 중
첫째(본문 바닥 경계의 예산 판정)를 처리한다. base는 `upstream/devel`
`e118de8abadd2dffb2798036b9ffd381b96ed7d5`(2026-11-11 `b5064fae1`에서 리베이스), 브랜치는 `task/7620-rowbreak-margin-budget`이다.
선행 작업은 #7644(#7681로 통합)와 planet6897의 #8149(빈 host 첫 조각 바닥 예약)다.

## 확정 범위

- 대상: 쪽 경계 나눔(RowBreak, `TablePageBreak` 2)·비TAC·자리 차지·문단 기준·오프셋 0 표가
  쪽 바닥에 걸릴 때 첫 조각에 실을 행이 있는지 판정하는 예산.
- 비범위: 56345 문서의 7·8·9·17·18쪽 부족분(#7806, jangster77 담당), footer 문서 11쪽
  높이 초과(다음 단계), CellBreak 실제 이월(다음 단계).
- 초기 가설이던 `entry.rs`의 `whole_frame_budget` 실패 fallback(상자 소실)은 생성 대조군과
  기존 회귀에서 재현되지 않았다. 실제 결함은 아래의 분할 판정이었다.

## 독립 기준(입력 생성 → 기준 → 관측값)

입력은 `samples/issue7620/generate-budget.mjs`(devel web WASM API)로 만든 HWP5 대조군이다.
앞 문단 N개 · R×2 표 · 빈 문단 · 뒤 문단이며, 표는 바깥 여백 위·아래 2835HU(37.8px)다.
기준은 같은 입력을 한컴독스 웹 한글에서 인쇄한 PDF(`pdf/issue7620/*-hancomdocs.pdf`)다.

| 입력 | N·R | 한컴 관측(96dpi) |
|---|---|---|
| b | 37·2 | 1쪽: 앞 문단 37 뒤에 표 문단과 뒤 문단이 이어짐. 2쪽: 표 전체가 170.1px(본문 윗변 + 바깥 위 여백)에서 시작 |
| c | 39·2 | 표가 2쪽 170.1px에서 시작, 뒤 문단은 표 아랫변 + 바깥 아래 여백(242px) |
| d | 20·40 | 1쪽 표 596–955px(아래에 37.8px 남김), 2쪽 170–495px, 뒤 문단 554px |

따라서 한/글은 `바깥 위 여백 + 첫 행 + 바깥 아래 여백`이 현재 쪽에 들어갈 때만 행 경계에서
나누고, 첫 행도 들어가지 않으면 표만 다음 쪽 위로 옮긴다. 표 문단과 뒤 글줄은 현재 쪽에 남는다.
보조 탐침(쪽 나눔 None 표, 여러 행 경계)은 기존 출력과 같아 변경하지 않았다.

## 원인 경로

`TypesetEngine` 표 배치(`src/renderer/typeset/table/block/entry.rs:107` → `prepare.rs`)에서
첫 행 이월 판정(`remaining_on_page_for_first_unit`)은 저장 원점 상자 `p`를 쓰는
`fragment_host_placement` 분기에서 바깥 아래 여백만 빼고 #8149의 빈 host 바닥 예약을 빼지 않았다.
b에서 이 판정은 49.7px가 남는다고 보고 첫 행(약 17.1px)이 들어간다고 결론냈다. 반면 실제 행
스캐너(`continuation/fragment/budget.rs`)는 첫 조각에서 같은 예약을 무조건 빼 11.9px 예산을
계산했고, 0행 예산에서 첫 행을 억지로 실어 표를 나눴다. 측정(이월 판정)과 배치(행 스캐너)가
다른 예산을 소비한 것이 원인이다.

## 수정

1. `prepare.rs`: 첫 조각의 바닥 예약을 `fragment_outer_bottom + empty_host_reserve`로 묶어
   `fragment_host_placement` 분기에서도 스캐너와 같은 값을 뺀다. 두 경로가 같은 예약을 소비한다.
2. `deferred_float.rs`: 첫 행의 최소 단위(첫 줄 + 안 여백)와 바깥 위·아래 여백이 현재 쪽에
   들어가지 않고 표 상자 전체는 새 쪽에 들어가는 RowBreak 표를, 기존 None 표의 이월 경로
   (`try_defer`)로 보낸다. 표만 다음 쪽 위로 옮기고 host·뒤 글줄은 현재 쪽에 남긴다.
   쪽 하나보다 큰 표는 여기서 제외되어 기존 분할 경로를 따른다.

경로·컷·종료 대조:

| 분기 | 시작/끝 컷 | 요구 높이 | 예산 실패 시 | 실제 배치 |
|---|---|---|---|---|
| b(첫 행 불가, 통째 가능) | 컷 없음 | 바깥 위 + 표 + 바깥 아래 | 표만 다음 쪽으로 이월 | 2쪽 170.1px, 행 1·2 모두 |
| c(앵커만 가능) | 컷 없음 | 같음 | 같음 | 2쪽 170.1px, 뒤 문단 표 아래 |
| d(여러 행 가능) | 행 경계 | 행 누적 + 바깥 아래 | 남은 행 이월 | 1쪽 1–N행, 2쪽 나머지, 행 1–40 각 1회 |
| None 표 | 기존과 같음 | — | — | 변경 없음 |

## 검증

### 시각 증적

- Visual Sweep: `mydocs/pr/assets/issue_7620_rowbreak_bottom_margin_budget/README.md`.
  b는 수정 전 Native 89.88/59.65%, fresh WASM 90.21/60.49%에서 수정 후 양쪽 100.00/100.00%다.
  c·d는 수정 전후 100.00%다. 1쪽 바닥과 2쪽 표 위치를 review PNG로 직접 판독했다.
  측정 뒤 커밋은 테스트(`732d53626`)와 TSV 개행(`2c825f08a`)만 바꿨고 `src/`는 그대로다.
- `layout-anomaly`: b·c·d 모두 0건. 래칫 baseline 추가가 필요 없다.

### 게이트(review worktree, head `2c825f08a7a0247be25493543545e1a6798fc1a2`)

2026-11-10~11, base `b5064fae1eefff92d60464c51844a38debec09f9`, `--target-dir target/pr-review`.

| 단계 | 명령 | 결과 |
|---|---|---|
| 준비 | `node scripts/rust-test-suite-manifest.mjs --prepare` | 성공 |
| fmt | `cargo fmt --all -- --check` | 통과 |
| Clippy native | `cargo clippy --locked -- -D warnings` | 통과 |
| Clippy WASM | `cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown -- -D warnings` | 통과 |
| workspace | `cargo build --locked --workspace`, `cargo clippy --locked --workspace --all-targets -- -D warnings` | 통과 |
| manifest | `node scripts/rust-test-suite-manifest.mjs --check --base-ref b5064fae1` | 통과 |
| 수정 후 | `node scripts/run-rust-test.mjs issue_7620_rowbreak_float_bottom_margin_budget` | 3건 통과 |
| 수정 전 | 같은 명령, `prepare.rs`·`deferred_float.rs`만 base로 되돌림 | b 실패(아래), c·d 통과 |
| 전체 | `cargo nextest run --locked --cargo-profile release-test --tests` | 13116건 통과 |
| Skia lib | `cargo test --locked --profile release-test --features native-skia --lib` | 3851 통과, 1 실패(아래) |
| Skia 대상 | `issue_2225_missing_picture_placeholder`, `render_p37_direct_pdf_export`(`--features native-skia`) | 2건, 35건 통과 |
| 공백 | `git diff --check b5064fae1 HEAD` | 통과 |

- 수정 전 실패의 원인: `issue_7620_rowbreak_without_first_row_moves_table_and_keeps_text`가
  `표가 나뉘지 않고 한 쪽에만 있어야 한다`에서 실패한다. 표 조각이 1쪽 959.4px와 2쪽 170.1px
  두 곳에 있었다(빌드·환경 실패 아님). c·d는 수정 전에도 같은 배치를 내는 대조군이다.
- Skia lib 1건 `renderer::skia::renderer::tests::custom_font_paths_select_style_within_the_first_family_owner`
  (`renderer.rs:3851`, 합성 Bold 굵기 400 ≠ 700)은 base src로 같은 테스트만 돌려도 같은 위치에서
  실패한다. 이 변경과 무관한 기존 실패다.

### 코퍼스 래칫과 보안 검사

- `samples/`를 훑는 다섯 래칫(IR sweep, overflow cell, off canvas, text overlap, body overflow)은
  위 전체 nextest에 포함되어 통과했다. 새 sample b·c·d는 `layout-anomaly` 0건이라 행이 없다.
- `oracle_page_count_baseline`: 비해당. 원장은 `<원본>-<hwp|hwpx>-<2020|2024>.pdf` 형식의
  한/글 데스크톱 엔진 PDF만 고른다(`tools/oracle_pdf_selection.py`). 새 기준 PDF는 한컴독스 웹
  인쇄본(`-hancomdocs.pdf`)이므로 엔진을 꾸며 이름을 바꾸지 않았다. 재생성 결과에는 이 작업과
  무관한 로컬 `pdf/` 체크아웃 차이가 섞여 커밋하지 않았다.
- 새 sample 보안 검사: `RHWP_SECURITY_SWEEP_SAMPLES_JSON`에 b·c·d 세 경로를 넣고
  `-E 'test(/security_corpus_regression/)'` 실행, 6건 통과
  (`new_sample_documents_are_clean_across_all_three_detectors` 포함).

### 편람 `--handbook-gate`

2026-11-11, head `2c825f08a`, `scripts/visual_sweep.py --handbook-gate`(Native, 383쪽, 지원 웹폰트).

| 환경 | 실행 파일(SHA-256) | 결과 |
|---|---|---|
| WSL Ubuntu 22.04, `cargo build --release --features native-skia`, pypdfium2 5.13.0 | `4d5aa4934e0d3045b5ace67bd021e8d172538e42d7313eb2825eb4e220dd0b1d` | **passed**, 하락 0쪽. raw 100.00%가 아닌 쪽은 130쪽(99.96%, 알려진 WMF 차이) 하나 |
| Windows 11, release-test native-skia, pypdfium2 5.14.0과 5.13.0 | head `5d9ec9b3…`, base `deed3274…` | 둘 다 `re_review_required`(9쪽 100.00 → 99.99%, 25픽셀) |

Windows의 9쪽 하락은 이 변경과 무관한 실행 환경 차이다.

- base(`b5064fae1`) 실행 파일로 같은 환경에서 돌려도 383쪽 raw 값이 head와 전부 같았다.
- head와 base의 편람 383쪽 `export-svg` 출력은 바이트 단위로 같았다.
- 9쪽을 확대해 보면 배치는 PDF와 같고, 큰 제목 글자의 안티앨리어싱만 다르다.
- 같은 head를 Linux에서 빌드해 돌리면 통과한다. 그래서 Windows Skia의 글자 래스터 차이로 본다.
