---
kind: snapshot
status: active
canonical: mydocs/working/task_m100_6970_open_pr_stage2.md
last_verified: 2026-09-16
---

# 열린 PR 메인터너 보정 — 2회차

## 분석

시작 head는 `788ab292b`이며 같은 로컬 통합 브랜치에서 보정한다.
#7118의 저장 프레임 회귀와 합성 Square 문단 겹침을 먼저 재현하고,
폭의 6% 보정·그림 모양에 따른 문단 재배열·높이 덮어쓰기의 잘못된 가정을 제거한다.
저장 정보와 합성 정보를 구분하고 측정과 배치가 같은 줄 구성과 실제 객체 점유를 소비하게 한다.
기준은 기존 Git 입력과 한컴 PDF이며 기존 기대값·허용치는 완화하지 않는다.

#7184는 실제 선택 컷에 따른 요구 높이와 예산 재시도 경로를 검토한다.
시작/끝 컷, 원래 유닛만 fit하는 예산, 마지막 유닛/후속 내용의 경계를 독립 기대값으로 확인한다.
선행 focused 및 직접 Native Visual Sweep 뒤 최종 코드의 전체 회귀·lint·WASM·Skia 검증을 수행한다.
원 PR 검토 문서는 실제 결과로 갱신한다. 분석·보정·검증·결과보고를 한 회차로 커밋한다.
이 요청에는 원격 push/PR 생성/merge가 포함되지 않는다.

## 원인 분리와 선행 결과

- #7118 프레임 회귀: 그림 높이 추가와 빈 문단 보정을 각각 제외해도 5/6 실패가
  유지됐다. NO_LS 프레임 허용 확장이 TopAndBottom 그림 문단까지 침범한 것이 원인이다.
  Square owner가 있는 경로로 제한한 대조에서는 기존 #7095 6개가 모두 통과했다.
- #7118 배너 재배열: 원본 ColumnDef raw_attr `0x1808`의 bit 10–11 값은 2다.
  저장소 [HWP 5.0 사양](../tech/한글문서파일형식_5.0_revision1.3.md) 표 139는 이를 `맞쪽`으로 정의한다. 파서가 이를 LeftToRight로
  잃고 renderer도 단 방향을 적용하지 않았다. 문단 순서를 바꾸지 않고 짝수 쪽의
  물리 단 순서를 반전하는 것이 독립 사양/PDF에 맞는 수정이다.
- #7118 글줄: typeset의 폭 6% 가산과 paint의 원래 단 폭 재조판이 다른 줄을 만들었다.
  실제 Square 가용 폭과 문단 여백 흡수를 공통 계산으로 바꾸고 수치 가산을 제거했다.
  source 맞쪽 값·2쪽의 단 순서·100번 문단 두 줄·뒤 문단 보존 테스트를 추가했다.
- 배너에 따른 문단 재배열·빈 문단 삭제·높이 덮어쓰기와 무관한 편집의 저장 간격 추정은 제거했다.
- #7184는 실제 end cut으로 예약을 구하고 예산이 부족하면 컷을 다시 선택한다.
  마지막 유닛의 종결 viewport 예약을 0으로 만드는 기존 `len()-1` 추정을 제거한다.
  source-frame 확장 분기는 별도의 저장 프레임 계약이며 그 뒤 실제 높이 측정은 유지한다.

최종 전체 검증과 시각 결과는 아래에 별도로 기록한다. 선행 성공은 최종 head 승인으로 보지 않는다.

## 수정 전후 결함 검출

수정 전 대조는 `788ab292b`의 독립 worktree에서 같은 기대값을 실행했다.

| 경계 | 수정 전 | 보정 후 선행 검사 |
| --- | --- | --- |
| source 단 방향 2 | `LeftToRight != Mirror`로 실패 | 맞쪽·짝수 쪽 단 순서·두 줄 점유 통과 |
| 마지막 mixed 유닛 예약 | 기대 24px 대신 0px로 실패 | legacy/native·1×1/다열·끝 컷·예산 재선택 통과 |
| #2004 5쪽 그림 프레임 | 기존 회차 및 원인 분리 대조에서 ±2px 위반 | #7095 6/6 통과 |

focused 19개 묶음은 66/66 통과했다. 단 방향 HWP/HWPX 재저장과
LTR/RTL/맞쪽의 사각형·짝홀 반복 적용도 포함한다. source unit 총량은
기존 4205개를 유지했다. 영문으로 추가했던 코드 주석은 한국어로 수정했다.

## 실제 호출 경로

| 변경 | 선택·측정·수용·배치 연결 | 적용 경계 |
| --- | --- | --- |
| Square 폭 | `synthetic_wrap_column_width`를 typeset의 frame 구성과 `layout_partial_paragraph`의 frame 구성에서 함께 소비 | 합성 whole-paragraph anchor; 저장 LineSeg/band별 앵커는 기존 경로 |
| NO_LS 그림 frame | `recompose_stored_lines_in_frame_with_known_square_band`에서 확인된 밴드 또는 문단 소유 Square 그림이 있을 때만 그림 frame 허용 | TopAndBottom-only 문단 제외, TAC 기준선 점유 유지 |
| 단 방향 | 파서 값 2 → `ColumnDirection::Mirror` → 페이지별 물리 단 배열 → 최종 쪽번호 보정과 paint | source 문단 순서를 바꾸지 않음; 단 구분선은 물리 좌우로 계산 |
| mixed 예약 | 실제 `advance_row_cut`의 end cut → `row_cut_mixed_nested_reserve` → 부족한 예산 재선택 → 같은 cut의 `row_cut_content_height` → `consumed` 누적 및 배치 | 일반 컷과 재시도에 적용; 저장 source-frame 확장은 별도 소유 계약과 후속 실측을 유지 |

종결 유닛 테스트의 10px 단위는 합성 입력이다. 24px는 기존 42065 종결 뷰포트
계약(첫 유닛 두 개와 4px clip 여유)의 예약 전달 검사이며, 이 숫자만으로
한컴 시각 일치를 주장하지 않는다. 원본 42065·3637·거대 셀을 별도로 비교한다.
분할 불가능한 첫 유닛을 소비하는 기존 진행 규칙은 유지하므로, 모든 유닛이
임의의 작은 예산 안에 들어간다는 주장도 하지 않는다.

## 최종 Native 직접 비교

Visual Sweep의 실제 비교 PNG를 열어 확인했다. 입력/PDF는 모두 기존 Git 경로를 사용한다.
`fidelity_compare --text-only --export-all-svg --layout-ledger`도 문서 전체에 실행했다.
첫 호출의 페이지 index를 1-based로 넘긴 도구 오류는 성공으로 세지 않고, 0-based 범위로
다시 실행해 6개 문서 모두 exit 0을 확인했다.

| 입력 | 직접 비교 쪽 | 관측 |
| --- | --- | --- |
| #7118 익명화 fixture | 1–3 | 2쪽 맞쪽 단 순서·두 줄 분리, 앞뒤 내용 유지. 글꼴에 따른 줄끝·수직 위치 차이는 남음 |
| #7184 giant HWPX | 18–21, 47–48 | 보정 전 6쪽과 픽셀 동일. 본문 하단·최종 내용 유지. 19쪽 표 전체 이월 차이는 기존 제한 |
| #3637 nested HWPX | 26–30 | 보정 전 5쪽과 픽셀 동일. 29쪽 표 내용 누락은 보정 전에도 존재하므로 개선 완료로 세지 않음 |
| #7183 picture-only HWP | 1–4 | 그림 자리와 표의 앞뒤 내용 유지. PDF 대비 글꼴·행 높이 차이는 기존 제한 |
| #2004 picture-stack HWP | 5 | 그림 프레임 하단 회귀 해소, PDF의 그림 끝과 프레임 대응 |
| 42065 HWP | 15–17 | 이어받은 표 뒤 내용과 17쪽 마지막 4) 문단 유지, 불필요한 후속 빈 쪽 없음 |

전수 페이지 원장은 각각 3/48/31/4/8/17쪽으로 PDF와 일치한다. 페이지 수 및
픽셀 동일성은 보조 증거이며, 위 내용·프레임·겹침 관측을 대신하지 않는다.

## 최종 검증 기록

기준 base는 `8d45f242baa1a565357aaa38e9f459595b1e756c`이며, 실행 코드의 파일별
SHA-256과 CLI 해시는 [소스 원장](../pr/assets/pr7118_7187_review_stage2/validation-source.json)에 있다.
파생 suite drift는 최종 source로 `--prepare` 후 전체 대상 Clippy·base 고정 정책 검사를
다시 통과시켰다. 파생 suite와 manifest는 commit에 포함하지 않는다.

- fmt, host/WASM/workspace-all-targets Clippy 3종, workspace build: 통과.
- base 고정 suite/unit 정책: 통과. source unit 총량 4205 유지.
- `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr7118-7187-20260916 --tests --no-fail-fast`: **9933/9933 통과**, 기존 skip 51. 테스트 실행 402.717초, 빌드·탐색 포함 763.81초.
- Native Skia 전체 lib: **4112 통과**, 기존 ignore 13(본체 3930, workspace 보조 182).
- Skia 지정 integration: `issue_2225_missing_picture_placeholder` 2/2, `render_p37_direct_pdf_export` 4/4 통과.
- fresh WASM: `CARGO_TARGET_DIR=target/pr7118-7187-20260916 scripts/wasm-pack-locked.sh --target web --out-dir pkg --no-opt` 통과(221.23초). Docker 데몬 연결 불가에 따른 문서 지정 host 진단 경로다. Docker/wasm-opt 배포 최적화 경로를 통과했다고 주장하지 않는다.
- fresh WASM Visual Sweep: Native와 같은 6개 문서 22쪽을 실행하고 실제 비교 이미지에서 문제 영역과 후속 내용을 확인했다. 제목 32px를 제외한 본문 비교 PNG는 Native와 **22/22 픽셀 동일**했다. 이는 backend 간 무회귀 보조 근거이며 PDF 완전 일치의 뜻이 아니다.

macOS arm64의 논리 CPU 10개, 메모리 32GiB에서 nextest 기본 동시성을 사용했다.
공유 target은 지우거나 옮기지 않았고, Cargo 계열 검증은 검토 전용 target에서 한 명령씩 실행했다.

## 결과보고

#7118의 기존 프레임 실패·두 줄 겹침·6% 폭 가산·배너 재배열 및 높이 덮어쓰기,
#7184의 실제 끝 컷/종결 예약과 경계 증거 부족을 해소했다. 보정 전 테스트의 의도한
실패와 보정 후 통과를 연결했고 기존 baseline·±2px 허용치는 완화하지 않았다.
#2004 5쪽 프레임 하단은 937.80px에서 929.80px로 바뀌었다(render tree 반올림값).
독립 PDF의 931.48px에 대해 기존 ±2px 테스트가 Native 최종 코드에서 통과하며 WASM 트리도 같다.

판정은 **검증한 변경 범위의 보류 해소**다. #3637의 기존 29쪽 내용 누락, #7184의
19쪽 표 전체 이월 차이, 문서별 글꼴/줄끝 차이는 이 보정으로 해결했다고 주장하지 않는다.
따라서 #6970·#7140·#3637 전체 이슈 종료의 근거로 사용하지 않는다.
불균등 단 너비와 구역 쪽번호 재시작을 함께 사용하는 맞쪽 문서의 별도 한컴 대조,
HML 재저장 실측은 이번에 추가 실행하지 않았다. 직접 검증한 단 계약은 기존 문서와
등폭 3단의 방향·짝홀 반복 적용, HWP/HWPX 재저장이다.

- 원 PR 8개 head는 검증 종료 시 다시 조회해 불변·OPEN을 확인했다.
- RED 확인용 임시 worktree는 Cargo 종료 후 제거했다. 검토 target과 본 작업 브랜치는 유지한다.
- 검증 입력/PDF 12개는 모두 기존 Git 파일을 재사용했으며 이름을 바꾼 중복 문서는 추가하지 않았다.
- 이 회차 결과를 사용자에게 보고한 뒤 코드·증적을 커밋한다. 이후 개별 review와 impl 문서를 이 보정 commit에 맞춰 갱신한다.

[검증 명령/종료 결과](../pr/assets/pr7118_7187_review_stage2/gate-results.json) ·
[RED/GREEN](../pr/assets/pr7118_7187_review_stage2/red-green-results.json) ·
[Visual Sweep 실행](../pr/assets/pr7118_7187_review_stage2/visual-results.json) ·
[입력/PDF 원장](../pr/assets/pr7118_7187_review_stage2/fixture-manifest.json) ·
[Native/WASM 본문 비교](../pr/assets/pr7118_7187_review_stage2/native-wasm-pixels-stage2.json).
