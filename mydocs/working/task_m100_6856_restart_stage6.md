# #6856 재착수 Stage 6 — 전체 검증과 PR 준비

- Issue: #6856. 2026-09-08 메인테이너의 “전체 검증과 PR 준비를 하세요” 지시에 따른다.
- 선행 결과: [Stage 5](task_m100_6856_restart_stage5.md).
- 기준 계약: [재착수 구현계획](../plans/task_m100_6856_restart_impl.md), 특히 §6.5의 HWP 0문단 손상 거부.
- 제품·시험 후보: `492bdb3f75` (`task_m100_6856_baseline`).
- 이 승인은 로컬 전체 검증과 PR 초안 준비이며 원격 push·PR 생성·merge·issue close는 포함하지 않는다.

## 검증 기준과 원격 정합

`upstream/devel`을 fetch해 `91147aec33`을 확인했다. 후보와 공통 조상은 `b5eee9c50`이며,
후보 고유 7커밋·devel 고유 4커밋이다. devel 추가 변경은 `.github/codeql/rust-pr.yml`,
오늘할일, PR #6877 리뷰 기록뿐이고 제품 source·test·Cargo 변경은 없다.
`git merge-tree --write-tree upstream/devel HEAD`는 충돌 없이 종료했으며 통합 tree는
`2763cf48762211a494c86eaf519db2f0e89e4f8e`다. 후보와 통합 tree의 제품 검증 입력은 같다.
후보 PR 범위 및 통합 tree의 `git diff --check`도 통과했다. 실제 merge/rebase는 하지 않았다.

기존 검증 worktree `rhwp-review-6856`의 clean 상태를 확인하고 후보 commit으로 전환했다.
이전 detached head `538257c07`은 보존 branch `task_m100_6856`에 포함되어 있다.
기본 checkout의 작업 branch·과거 구현은 되돌리거나 삭제하지 않았다.

## 실행 환경과 필수 게이트

- WSL2 Linux, 논리 CPU 16개, RAM 31 GiB. 시작 시 가용 메모리 약 20 GiB, 디스크 약 136 GiB.
- 고정 target: `/home/edward/mygithub/rhwp-6812-review-target` 재사용. 이동·삭제하지 않는다.
- Cargo는 순차 실행한다. 전체 nextest는 host 메모리를 고려해 test threads 8개를 사용한다.
- `--prepare`는 review worktree에서만 실행했다. 1,201 source, 28 suite + 20 exception target이다.
  generated suite·manifest는 제출 파일이 아니다.
- 실행 로그는 review worktree의 `output/6856/pr-validation/`에만 보관하며 커밋하지 않는다.

| 검증 | 상태 |
| --- | --- |
| 전체 fmt 및 fmt check | 통과, 원본 파일 포맷 변경 없음 |
| native root Clippy (`--locked`, `-D warnings`) | 통과, 52.65초 |
| WASM32 library Clippy | 최초 후보 통과, 48.57초 |
| workspace build / all-target Clippy | build 통과 1분 26초, all-target에서 새 시험 `clippy::box_default` 1건 검출 |
| manifest check / 배정 규칙 계약 시험 | 대기 |
| release-test 전체 nextest | 대기 |
| Native Skia 3종 | 대기 |
| Docker WASM / 최신 산출물 확인 | 대기 |
| 대표 시각 산출물·문서 정합 | 대기 |

위 대기 항목을 통과하기 전 PR 준비 완료로 보고하지 않는다. 실패 시 원인과 후보 귀속을 먼저 확인하며,
실패를 피하기 위한 baseline 변경·테스트 제외·오류 무시는 하지 않는다.

### 전체 lint에서 발견한 시험 작성 오류

`issue_6856_rectangle_structure.rs`의 `Box::new(Picture::default())`가 `clippy::box_default`에
걸렸다. `Box::default()`로 정정하고 불필요해진 import를 제거한다. 제품 동작·시험 기대값은 변경하지
않으며 lint를 allow하지 않는다. 정정 commit을 검증 worktree에 반영하고 lint 묶음부터 재실행한다.

정정 뒤 formatter가 위 호출을 한 줄로 줄여 test source의 크기가 바뀌었다. 이 때문에 포맷 전 생성한
가중치 기반 suite 배정과 포맷 후 manifest의 배정이 달라져 drift가 검출되었다. 포맷 결과를 source
commit에 반영하고 그 commit에서 다시 `--prepare`한 뒤 fmt check·manifest check가 모두 통과했다.
이는 파생 준비 순서의 문제이며 생성기·정책·시험 기대값은 수정하지 않았다.

## PR 범위와 보고 경계

### 전체 회귀 1차 결과와 #5797 기대값 정정

`4784ec0a4`에서 전체 lint·manifest check 및 배정 규칙 21개가 통과했다.
전체 nextest는 컴파일 4분 04초, 시험 317.855초에 **9,247 통과·1 실패·46 skip**으로 종료했다.
실패는 `issue5797_self_closing_paragraph_does_not_swallow_next_paragraph` 한 건이다.
이슈 #6856의 집중 시험 및 #6852 보호 시험 30개는 모두 통과했다.

기존 #5797 입력은 첫 도형에 `<p/>`와 `검토/승인` 문단을 순서대로 가진다. 시험 helper가 모든
문단 텍스트를 `join("\n")`하므로, 승인된 P1의 빈 문단 보존 뒤 결과는 `"\n검토/승인"`이다.
기존 기대값 `"검토/승인"`은 빈 문단을 버린 결과였다. 실제 뒤 두 형제의 id와 글자는 그대로였다.
따라서 이번 실패는 뒤 문단 소실 회귀가 아니라 **빈 문단 보존 계약과 이전 기대값의 불일치**다.
기대 문자열과 설명을 정정하되 trim/filter로 빈 문단을 숨기지 않고 형제 검사도 유지한다.
제품 source·golden·baseline 원장은 바꾸지 않는다. 정정 뒤 필수 lint와 전체 회귀를 재실행한다.

Native Skia와 Docker WASM은 실패한 전체 회귀 뒤 실행하지 않았고 미실행으로 남겼다.
1차 실패 로그는 review worktree의 `output/6856/pr-validation/full-nextest-first.log`에 보존한다.

### 최신 대표 출력 확인

제품 코드가 같은 `492bdb3f7`의 workspace debug 바이너리로 A4 HWP/HWPX를 canonical layer SVG로
내보냈다. 두 출력은 바이트 단위 동일하며 사각형 1개·글상자 2개를 표시했다. PNG를 직접 열어
한글 label과 실선, 빈 글상자 및 글자 포함 글상자를 확인했다. legacy SVG도 선행 메인테이너 판정본
`output/6856/identification/a4-hwp-control-codes.svg`와 바이트 단위 동일했다.

- 대표 PNG: `mydocs/pr/assets/issue6856_a4_control_codes.png`.
- PNG SHA256: `c92adc23dacf6bb36b9d1f48d44afea369b2522a0f4abde1009ea955554d868a`.
- A4 layer SVG SHA256(두 형식 동일): `38f06fd85f5813db0e9a628c8cac8ad61f14d591adeab96b3725ba074c626078`.
- 바이너리 SHA256: `89951139972dd4402c13bfdbd4228f9b215c25b3321d33f9b59e45667e980238`.
- 원본 #6852 HWP/HWPX 5쪽 layer SVG도 동일(`81312df7ceace805dd48154b79d55791f4c506f026d3858befbc7db5ed875e9c`), 각 11쪽·선택 5쪽 overflow 0이었다.
- 편람 쌍은 열기·9쪽 내보내기에 성공했다. HWP 384쪽/HWPX 382쪽은 이번 실행의 관측값이며
  포맷 간 전체 출력 동일성이나 한컴 정답 판정을 의미하지 않는다.
- 동일 0문단 HWP 원본은 최신 CLI `info`에서도 구조 오류로 거부했다.

이 확인은 이미 승인된 조판부호의 유지 검증이며 새로운 PDF fidelity sweep 통과 주장이 아니다.

이번 승인 구현은 소유 내부 영역에 따른 사각형/글상자 식별, 조판부호 소비 통일, HWPX 빈 문단 보존,
확정적 소유 구조 오류의 문서 열기 전달, HWP 사각형 소유 목록의 0문단 거부다.
18조합 모델 시험은 한컴 정상 fixture 18개를 확보했다는 뜻이 아니다.

원격 #6856 본문에는 재착수 전 A 선 억제 조건 재검토와 더 넓은 수용 조건이 남아 있다.
이번 변경은 A 조건을 수정하지 않는다. PR 초안에는 승인된 최신 계획과 이 차이를 명시하고,
이슈 본문 현행화·close 여부는 별도 승인 단계에서 처리한다. 모든 과거 수용 조건이 해결됐다고
주장하거나 자동 close 문구를 먼저 넣지 않는다.

기존 조판부호 시각 판정 통과와 0문단 파일의 한컴 손상 판정은 메인테이너 근거다.
새 parser 검증 뒤의 정상 지정 샘플·출력 유지와 전체 회귀는 별도로 기록한다.
미실행 성능 측정, 전체 한컴 호환성 또는 코퍼스 전수 정답 검증을 통과로 확대하지 않는다.
