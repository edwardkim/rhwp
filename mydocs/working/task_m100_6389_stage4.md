# Issue #6389 단계 4 — 대체 글꼴 환경의 저장 다중행 보존

## 분석

- 지시: jeong-sik 작성자의 열린 이슈를 오래된 순으로 한 건씩 처리한다. 2026-09-16 조회한
  10건 중 #6389가 가장 오래되어 이 이슈부터 재검증한다. 다음 이슈는 이 회차 커밋 후 판단한다.
- 시작 base: `f1b53e82c502b8d7795a783d79cd4830168efcd7`, branch `codex/jeongsik-6389-20260916`.
- 기존 개선: #6412 저장 줄 압축, #6490 KoPub 0.872em, #7179 명시적 폰트 환경은 이미 병합됐다.
  원래의 셀 넘침·재래핑과 남아 있는 폰트 환경/시각 차이를 구분하고 실제 미해결 원인을 찾는다.
- 입력은 기존 편람 HWP·KoPub PDF·no-ttf PDF, 86712 HWP·Hancom 2024 PDF를 재사용한다.
  편람 기본 대응은 p68, no-ttf 대응은 p69다. KoPub PDF의 cairo 메타데이터와 계보 제한을 유지한다.
- 독립 근거: PDF의 실제 글꼴/글자 위치/줄 경계와 원본 저장 LineSeg. 결과를 맞추기 위한
  문서 ID 분기·허용 오차/기준값 완화·임의 폰트폭 상수 조정은 하지 않는다.
- 먼저 현재 코드로 재현하고 기준 PDF의 환경과 대조한다. 확인한 원인에 한정해 수정 전 실패/수정 후
  통과 경계를 만들고, Native/fresh WASM Visual Sweep과 standalone overlay를 함께 산출한다.
- 실제 수정 범위가 확정되면 관련 검증을 수행하고 결과를 이 문서에 기록한 뒤 함께 커밋한다.
  미검증 전체 시각 일치를 이슈 해결 완료로 주장하거나 GitHub 이슈를 자동 종료하지 않는다.
- raw log/JSON/TSV는 `/private/tmp/rhwp-6389-followup-20260916`에만 저장한다.
  이미 Git에 있는 입력은 복제하지 않는다.

## 수정 전 재현과 원인

- 현재 base에서 전용 target으로 새 native CLI를 빌드했다. KoPub 돋움/바탕 세 weight를
  `바탕`으로 대체한 명시적 세션 환경에서 편람 rhwp p68을 추출하면 대상 셀이 **17줄**이다.
  독립 `-2010-no-ttf.pdf` p69는 Haansoft Batang으로 **16줄**이며, `※ 분리등록한 첨부물`
  문단은 3줄이다. 현재 rhwp는 이 문단만 4줄로 바꾼다.
- 원인은 `composer::recompose_stored_single_line_if_overflowing_cached`의 #5952 구제 분기가
  이름·원래 목적과 달리 이미 여러 줄로 구성된 정상 저장 문단까지 다시 나누는 데 있다.
  1.10배 추정 폭은 대체 글꼴의 메트릭 차이도 부실 저장으로 오인한다. #6412의 다줄 저장
  프레임 압축 경로가 실행되기 전에 줄 경계를 파괴한다.
- 수정 범위: 이 **단일 줄 구제 함수**는 합성 결과가 실제 한 줄인 경우에만 동작하도록 한다.
  이미 유지된 저장 다중행은 공통 저장 프레임 경로에 맡긴다. 기존 #5952의 저장 2줄→합성 1줄
  붕괴와 #2291의 실제 부실 단일행 구제는 보존한다. 폭 임계·기준값을 바꾸지 않는다.
- 실물 입력의 독립 16줄 기대값을 기본 KoPub 및 명시적 no-ttf 환경에 공통 적용한다.
  수정 전 no-ttf 실패, 수정 후 두 환경 통과와 #5952 붕괴/빈 말미행 반례를 확인한다.
- 86712 p28에서는 대체 PDF의 HCRBatang-Bold와 일반 함초롬바탕 출력 차이도 관측했다.
  원본은 KoPub Light이므로 원래 bold를 잃었다는 가설은 확정할 수 없으며 이번 수정에 섞지 않는다.

## 현재 진행 상태

수정 전 회귀 테스트는 1 통과/1 실패(exit 101), 수정 후 두 환경 모두 통과(exit 0)했다.
관련 #5952·#2430·#2291·#6389 환경 계약 13건도 통과했다. 전체 회귀·Native Skia·fresh WASM
검증까지 완료했다. 아래 실제 결과와 증적을 코드·테스트와 함께 커밋한다.

## 적용 경로 검토

`recompose_horizontal_cell_lines_for_width`는 먼저 공통 프레임 재구성을 실행하고, 실제
한 줄인 경우만 저장 줄 붕괴 복구로 보낸다. 편집으로 dirty해진 줄·과밀한 부실 저장의 유효성
검사는 선행 `stored_rows_are_stale`에 남아 있다. 새 문서 ID·글꼴 이름·수치 분기는 없다.

| 경로 | 실제 소비와 적용 |
| --- | --- |
| `height_measurer::measure_table_impl` | 셀 전체/말미/이어받기 높이 계산의 4개 호출이 같은 공통 재구성 결과를 사용 |
| `table_layout::calc_cell_paragraphs_content_parts`, `layout_table_cells` | 실제 셀 줄 배치와 높이가 같은 결과를 사용 |
| `table_layout::nested_table_mixed_fragment_heights`, `cell_units_uncached` | 중첩 셀·분할 유닛에도 같은 공통 호출; 컷 선택·예약·종료 알고리즘 자체는 변경 없음 |
| `table_partial`의 line cache 및 `layout_partial_table_cells` | 부분 표 배치도 같은 공통 호출 |
| 세로쓰기·HWP3·HWPX 전용 경로 | 기존 호출자의 repair 여부와 분기는 변경하지 않음 |
| 저장 줄 없는 편집 후 재조판 | 선행 프레임의 줄 나눔을 사용; 환경 계약의 NO_LS 테스트로 별도 확인 |

## 선행 Native 시각 대조

현재 수정의 새 CLI와 수정 전 base CLI로 각각 SVG·render tree 384쪽을 생성했다.
8쪽만 SVG가 달라졌고 모든 변경 쪽에서 공백을 제외한 전체 문자열은 보존됐다.
기존 HWP/PDF 파일을 그대로 사용한다. PDF 대응은 본문을 대조해 정했으며 원 PDF를
수정하거나 다른 이름으로 저장소에 추가하지 않는다.

| rhwp 쪽 | 원 no-ttf PDF 쪽 | 전 → 후 내용 픽셀 보조값(%) | 관측 |
| --- | --- | --- | --- |
| 68 | 69 | 60.43 → 61.11 | 대상 셀 17→16줄, ※ 문단 4→3줄 |
| 69 | 70 | 35.93 → 42.80 | 유의사항 문단 16→13줄, 뒤 본문과 분리 |
| 200 | 202 | 31.32 → 32.01 | ☞ 문단의 저장 줄 경계 복원 |
| 234 | 236 | 63.69 → 64.66 | ※ 문단 3→2줄 |
| 240 | 242 | 23.10 → 23.26 | 전체 문자·줄 경계는 동일, 셀 내부 기하 차이 |
| 369 | 375 | 15.38 → 15.44 | 서식 유의사항 줄 경계 복원 |
| 372 | 378 | 10.02 → 10.05 | 서식 유의사항 줄 경계 복원, 기존 단별 배분 차이는 남음 |
| 377 | 382 | 14.97 → 15.15 | 보고내용 주의문 줄 경계 복원 |

`visual_sweep.py`의 `svg_raster_command`(webfont), `make_compares`, `make_overlay_page`,
`make_review_panels`, `analyze_page`를 사용해 원 PDF의 대응 쪽을 직접 래스터 대조했다.
문항 marker 자동 페이지 대응은 이 수동 대응 검사에서 사용하지 않았다.
처음 p68·69 CLI sweep에는 원 PDF 표지 1쪽을 제외한 임시 사본을 사용했으며, 최종 대응
대조에서는 원 PDF p69·70을 직접 지정했다. 두 경로의 출력은 동일하다.

`fidelity_compare.write_layout_ledger`로 각 새 render tree의 384쪽을 전수 검사했다.
입력 누락은 없으며 p69 `table_footer` 후보 1→0 외에는 원장 변화가 없다.
자동 후보 0이나 위 픽셀 점수를 전체 시각 일치로 해석하지 않는다. 직접 이미지 대조에서
대상 글자가 셀 안에 있고 다음 내용이 유지됨을 확인했다. 머리말·쪽번호 방향, 글꼴 모양,
기존 표 간격 및 단별 페이지 배분 차이는 남아 있다.


## 새 WASM 및 증적 보존

macOS 네이티브 `scripts/wasm-pack-locked.sh --target web --out-dir <전용 임시 경로>`를 사용했다.
Docker daemon은 연결되지 않았으나 `--no-opt` 없이 **wasm-opt 최적화까지** 완료했다(3분 50초).
Chrome 153.0.8010.47의 실제 WASM 인스턴스로 384쪽 SVG와 render tree를 다시 생성했다.
변경 8쪽의 Native/WASM 래스터 PNG는 바이트 단위로 동일하다. 대상 셀은 양쪽 모두
16줄, `x=118.7, y=469.5, w=521.6, h=407.5px`이며 수정 전 높이는 416.6px였다.
기본 KoPub 환경의 별도 수정 전후 sweep에서는 384쪽 SVG가 모두 바이트 단위로 동일하다.

PNG 20개(약 6.7 MB)를 이 문서의 [assets/issue6389_stage4](assets/issue6389_stage4/)에 보존한다.
현재 Native/WASM이 동일하므로 수정 후 중복 이미지는 추가하지 않고 실제 WASM 증적을 남긴다.
기존 HWP/PDF를 중복 추가하지 않는다. raw SVG/tree/JSON/TSV/log, 임시 페이지 대응 PDF,
컴파일 산출물도 커밋에서 제외한다.

| 비교 쪽(rhwp / 원 PDF) | 수정 전 | 수정 후 실제 WASM |
| --- | --- | --- |
| 68 / 69 | [review](assets/issue6389_stage4/before_review_068.png), [overlay](assets/issue6389_stage4/before_overlay_068.png) | [review](assets/issue6389_stage4/wasm_review_068.png), [overlay](assets/issue6389_stage4/wasm_overlay_068.png) |
| 69 / 70 | [review](assets/issue6389_stage4/before_review_069.png), [overlay](assets/issue6389_stage4/before_overlay_069.png) | [review](assets/issue6389_stage4/wasm_review_069.png), [overlay](assets/issue6389_stage4/wasm_overlay_069.png) |
| 200 / 202 | 위 원장과 동일 입력 | [review](assets/issue6389_stage4/wasm_review_200.png), [overlay](assets/issue6389_stage4/wasm_overlay_200.png) |
| 234 / 236 | 위 원장과 동일 입력 | [review](assets/issue6389_stage4/wasm_review_234.png), [overlay](assets/issue6389_stage4/wasm_overlay_234.png) |
| 240 / 242 | 위 원장과 동일 입력 | [review](assets/issue6389_stage4/wasm_review_240.png), [overlay](assets/issue6389_stage4/wasm_overlay_240.png) |
| 369 / 375 | 위 원장과 동일 입력 | [review](assets/issue6389_stage4/wasm_review_369.png), [overlay](assets/issue6389_stage4/wasm_overlay_369.png) |
| 372 / 378 | 위 원장과 동일 입력 | [review](assets/issue6389_stage4/wasm_review_372.png), [overlay](assets/issue6389_stage4/wasm_overlay_372.png) |
| 377 / 382 | 위 원장과 동일 입력 | [review](assets/issue6389_stage4/wasm_review_377.png), [overlay](assets/issue6389_stage4/wasm_overlay_377.png) |

원본 HWP의 `lastSavedWith`는 `hancom-office-2024 13.0.0.3622`다.
no-ttf PDF는 389쪽, producer/creator `Hancom PDF 1.3.0.404`다.
KoPub PDF는 383쪽, producer `cairo 1.18.0`이며 한컴 직접 출력 계보는 확인되지 않았다.
파일명의 2010/2020을 실제 변환 엔진 확인 결과로 대체하지 않는다.

검증 소스는 위 base에 이 커밋의 코드·테스트 변경을 적용한 상태다. 이후 생산 코드·테스트는
변경하지 않았다. SHA-256:

- `src/renderer/composer.rs`: `9639b9da7a66a0a890f78e16bccef6d2cda30d58a91189ae04be857a35ec6d0c`
- 회귀 테스트: `a6c9b50dfb70c96d055fba0b1bfb3adbaed52523cde0385592d4d9e832909888`
- 편람 HWP: `40d6d05eac4d55bdc4b0c62c42d93af104d5123b447581246f36fd15de7bd46f`
- 기존 no-ttf PDF: `c33fbf6be76ae43e8254321ec99b5ab3963e492ff4b6179d7c9717ecac2a05e0`


## 검증 명령과 결과

모든 Cargo 명령은 `DEVELOPER_DIR=/Library/Developer/CommandLineTools`,
`CARGO_TARGET_DIR=target/issue6389-followup-20260916`에서 순차 실행했다.

| 검사 | 결과 |
| --- | --- |
| 수정 전 `cargo test --locked --test regression_suite_005 issue_6389_manual` | 기본 KoPub 통과, 새 no-ttf 줄 경계 assertion 실패; exit 101 |
| 수정 후 같은 focused 검사 | 2 통과; exit 0 |
| 관련 suite 010/017/026의 #5952·#6389 환경·#2430·#2291 focused nextest | 13 통과; exit 0 |
| `cargo fmt --all -- --check` | 통과 |
| `cargo clippy --locked -- -D warnings` | 통과 |
| `cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown -- -D warnings` | 통과 |
| `cargo build --locked --workspace` | 통과 |
| `cargo clippy --locked --workspace --all-targets -- -D warnings` | 통과 |
| manifest `--check --base-ref f1b53e82c502b8d7795a783d79cd4830168efcd7` | 파생 harness 재준비 후 통과 |
| `cargo nextest run --locked --cargo-profile release-test --tests --no-fail-fast` | **9,939 통과 / 실패 0 / 기존 skip 51**, 테스트 실행 374.398초, 빌드 포함 903초 |
| Native Skia lib·missing picture·direct PDF 3종 | lib 4,112 통과(기존 ignored 13), 그림 2 통과, 직접 PDF 4 통과; 각 exit 0 |
| fresh WASM wrapper 및 wasm-opt | 통과 |
| Native/WASM Visual Sweep 및 standalone overlay | 변경 8쪽 직접 대조 완료, PNG 동일, 남은 시각 차이는 위에 명시 |
| `git diff --check`, 이 보고서의 로컬 증적 링크 | 통과 |

최초 manifest 확인은 `cargo fmt --all`이 파생 harness의 module 순서를 바꿔 drift를 보고했다.
제품 코드의 실패가 아니며 `--prepare`로 파생 파일만 다시 생성한 뒤 고정 base 비교를 통과했다.
파생 파일은 커밋하지 않는다. baseline·허용 오차·기존 assertion을 완화하지 않았다.

## 판정과 남은 범위

- **충족**: 실제 no-ttf 환경에서 재현한 정상 저장 다중행의 불필요한 재래핑, +1줄,
  그에 따른 후속 내용 밀림을 개선했다. 기본 환경은 384쪽 SVG가 불변이다.
- **충족**: 실제 한 줄 붕괴 구제·끝의 빈 저장 줄·부실 단일 줄·저장 캐시 없는 재조판 반례가 통과한다.
- **잔여**: 두 기준 환경의 전체 문서 fidelity, 86712의 글꼴 굵기·위치 차이는 이 수정의
  해결 판정에 포함하지 않는다. 이번 부분 개선만으로 #6389 전체를 닫거나 정확한 한컴 폰트
  환경의 전역 자동 판정을 완료했다고 주장하지 않는다.
- 이번 회차의 분석 → 코드 수정·검증 → 결과보고를 완료했다. 이 문서와 코드·테스트·PNG를
  함께 커밋한 뒤 후속 회차를 판단한다. 원격 push·PR 생성·GitHub 이슈 상태/코멘트 변경은 하지 않았다.
