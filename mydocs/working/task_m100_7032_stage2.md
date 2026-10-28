# #7032 Stage 2 — 빈 셀 문단 배치 구현

- 일자: 2026-09-11 (KST)
- 승인 근거: 메인테이너의 구현계획 승인
- 계획: [구현계획서](../plans/task_m100_7032_impl.md)
- 현재 범위: **R1 시각 판정 및 R2 구현·focused 검증 완료. Stage 3 전체 검증은 미실행.**

## R1 변경 원리

저장 LINE_SEG가 없고 합성 줄도 없는 실제 빈 문단을, 온전한 셀에서는 누락시키지 않는다.
기존 `empty_no_lineseg_paragraph_metrics`의 대상 판정을 재사용해 빈 TextLine/TextRun 및
캐럿을 만드는 `layout_composed_paragraph`로 전달한다. y만 직접 가산하지 않는다.

controls가 있는 문단, `char_count=0`, 저장 LINE_SEG가 있는 문단, HWP3에는 새 경로를 적용하지
않는다. 세로쓰기는 기존 전용 경로를 유지한다. 파서·IR·스타일 해석·전역 합성은 변경하지 않는다.

## 검증 기록

- RED 테스트 commit: `c98c7f8e3` (제품 소스는 baseline 그대로)
- review worktree: `/home/edward/mygithub/rhwp-review-7032`
- 고정 target: `/home/edward/mygithub/rhwp-shared-review-target`
- 새 원본: `tests/cases/issue_7032_cell_empty_paragraph_flow.rs`
- 파생 suite는 review worktree에서만 준비. source checkout 및 커밋에는 포함하지 않음.
- 구현 commit: `611efa702`, 테스트 기대 범위 정정 후 검증 HEAD: `b7feedffa`.
- 아래 focused 실행은 모두 준비된 review worktree에서 `run-rust-test.mjs --cargo-test`와
  `--target-dir /home/edward/mygithub/rhwp-shared-review-target`를 사용한 **debug focused** 검사다.
  전체 release-test/Clippy/WASM 통과를 뜻하지 않는다.

| 검사 | 결과 | 증적 (`output/7032/` 아래) |
| --- | --- | --- |
| 수정 전 #7032 | 2 통과·2 실패. HWP 빈 줄 누락, HWPX·IR 보존 통과 | `r1-red.log` |
| 수정 후 #7032 | **4/4 통과**, 모든 6쪽의 빈 줄·캐럿·다음 원점 검사 | `r1-green-prepared.log` |
| #7028 반복 제목행·대각선·분할 반례 | **9/9 통과** | `r1-7028.log` |
| #2146 셀 선언 높이 | **1/1 통과** | `r1-2146.log` |
| #6110 control-only 그림 | **1/1 통과** | `r1-6110.log` |
| 전체 fmt check / prepared manifest check | 통과 | `r1-fmt.log`, `r1-manifest-final.log` |

합계 15개 실행·통과, 0건 실행을 합계에 넣지 않았다. #6660/#6035/#2279 등 계획의 나머지
보호 검사와 전체 회귀는 후속 R2/Stage 3에서 진행한다.

### 재실행과 테스트 기대값 정정

- 첫 GREEN 실행을 source checkout에서 시작한 경로 오류를 확인하여 해당 Cargo/rustc만 중지했다.
  그 실행(`r1-green.log`)은 검증 증거에서 제외하고 review worktree에서 다시 실행했다.
- 최초 테스트는 두 형식의 줄 **높이 자체**도 같아야 한다고 요구했다. 실제 HWP의 Fixed fallback은
  빈 줄 박스 18.88px, HWPX 저장 줄은 높이 13.33px + spacing 5.55px로 표현하지만 다음 원점은 같다.
  전 형식의 내부 줄 표현까지 통일하지 않는 승인 계획에 맞게 **줄 원점과 진행**만 대조하도록 정정했다.
  원본 fixture·기준 PDF·제품 줄 메트릭을 테스트에 맞춰 변경하지 않았다.
- 테스트 source 크기 변경 후 manifest 배정이 달라져 한 실행(`r1-green-final.log`)이 0건이었다.
  이 결과는 PASS에서 제외했다. 동일 SHA에서 `--prepare`/`--check` 후 다시 실행한
  `r1-green-prepared.log`에서 실제 4건을 확인했다.

## 실제 산출 및 시각 판정 결과

1쪽 r0c0의 셀 bbox `(72.0,225.3,56.2,52.4)`는 유지된다. 수정 전 HWP는 `직렬`의 TextLine이
235.3px에서 시작했지만 수정 후 빈 문단 235.3px → `직렬` 254.2px가 된다. 반올림 전 진행은
18.88px이며, 메인테이너가 정상 판정한 HWPX와 두 문단의 원점이 6쪽 모두 일치한다.
빈 문단 TextRun에는 셀 context와 문단 끝 캐럿이 존재한다.

- 수정 전 HWP/HWPX: `output/7032/r1-before-hwp/`, `r1-before-hwpx/`.
  `c98c7f8e3` review Cargo 실행으로 생성한 native binary에서 내보냈다.
- 수정 후 HWP: `output/7032/r1-after-hwp/svg/21761835_jeonjik_exemption_table_001.svg`부터 `_006.svg`.
- 수정 후 HWPX: `output/7032/r1-after-hwpx/`. `diff -rq`로 수정 전후 **6쪽 SVG 모두 동일** 확인.
- 프로젝트 표준 비교: `output/7032/r1-after-hwp/cmp-p000.png`부터 `cmp-p005.png`.
  `fidelity_compare.py --export-all-svg --layout-ledger`의 6/6 완료 기록은 `run-state.tsv`에 있다.
- 기존 PDF `pdf/task2146/21761835_jeonjik_exemption_table-hwp-2020.pdf`를 재사용했다.
  기준의 실제 metadata는 Hwp 2022이며, 새 정답지를 생성하지 않았다.
- PDF/SVG/render tree 모두 6쪽. 직접 연 1쪽 비교 시트에서 첫 셀의 글자가 대각선 아래로
  내려온 것을 확인했다. 본문의 폰트·테두리 굵기 등 기존 차이는 이 절편의 수용 판정으로 삼지 않는다.

제시한 확인 대상은 **첫 셀의 빈 문단 간격, `직렬` 위치, 후속 쪽 반복 제목 셀**이다.
메인테이너가 **“시각 판정 통과입니다.”**라고 판정했다. 이에 R1 시각 게이트를 통과로 기록한다.
이는 제시한 SVG 결과에 대한 판정이며, 개별 페이지를 모두 직접 열어 확인했다는 추가 주장은 하지 않는다.
Studio의 기존 WASM은 교체하지 않았으므로 WASM 시각 검증 통과를 의미하지 않는다.

## R1 완료 시점의 후속 범위

R1에서 실제 cut 창의 의미를 변경하지 않았다. 당시 **R2는 미완료**였으며, 현재 선택된 빈 atom의
높이·가시 마지막 문단·split 여부를 일관되게 판정하는 구현과 창 안/밖·마지막/연속 빈 문단
반례 검증이 남아 있었다. 아래 R2에서 처리했다. R1의 실제 샘플 통과를 전체 구현 완료로 간주하지 않았다.

R1 SVG/PDF 대조 및 메인테이너 판정 이후 승인된 R2를 수행했다.

## R2 — 분할 창의 빈 문단 소유권과 메트릭

메인테이너의 “다음 절차 진행을 승인합니다.”에 따라 수행했다. 최종 제품·테스트 검증 SHA는
`532b74fc5`이며, R1 승인 기준은 `cb4563233`이다.

### 원인과 수정

1. 분할 원장의 빈 문단 atom은 `(0,1)`을 소유하지만 물리 `ComposedLine` 수는 0이다.
   종전 비교는 모든 atom이 선택된 셀도 실제로 잘린 것으로 판정하여 Center/Bottom을 Top으로 바꿨다.
2. cut 콘텐츠 높이 합산에서 합성 줄만 순회하여 선택된 빈 문단 높이가 빠졌다.
3. 실제 셀 마지막 빈 문단의 기존 fallback은 Fixed 간격 전체를 줄 박스로 사용했지만,
   기존 셀 높이 측정은 마지막 문단의 글꼴 em만 소비했다.

`cell_cut_empty_paragraph_owners`는 **기존 cached CellUnit 창**에서 빈 콘텐츠 atom만 선택한다.
control/중첩/gap-only 유닛을 빈 문단 소유권으로 사용하지 않는다. 실제 빈 문단 대상 판정과 결합해
미선택 문단은 노드·캐럿·y를 소비하지 않고 선택 문단만 기존 배치 함수로 전달한다.
가시 마지막 문단·split 여부·cut 콘텐츠 높이·tail 정렬 높이도 같은 소유권을 사용한다.

높이는 기존 `calc_para_lines_height`를 재사용한다. 마지막 가시 빈 문단의 실제 배치는
cell context에 한해 글꼴 em과 trailing spacing 0으로 맞췄다. body/HWP3 fallback은 유지한다.
원장 생성·분할 예산·페이지네이션 정책·파서·IR·전역 composer는 변경하지 않았다.
빈 문단이 없는 셀에서는 추가 빈 atom 탐색을 건너뛴다.

### 계약 테스트와 RED 재확인

원본의 문단·스타일을 재사용하는 **메모리 내부 IR 계약 테스트**다. 파일로 저장하거나 한컴에서 생성한
정상 샘플로 주장하지 않는다. A4 페이지에 실제 `PartialTable` item과 cut 창을 전달하여 production
렌더러를 실행한다. 창 안/밖, 연속 빈 문단, 마지막 빈 문단, 완전 소진한 빈 창, 노드·캐럿·y를 검사한다.

- 창: `[0,1)`, `[0,2)`, `[1,3)`, `[2,4)`, `[3,4)`, `[4,4)`.
- 전체 창 `[0,4)`와 uncut 비교: Top/Center/Bottom × Fixed/Percent × 문단 전후 간격 유무 = 12조합.
- 마지막 문단의 em 박스: uncut, 전체 창, 마지막 빈 문단 단독, 첫 빈 문단만 선택된 창.
- 비교 시 물리 셀 높이는 기존 `end_row_height_override`로 동일하게 고정하고 높이 자체도 assert한다.
  자연 cut의 콘텐츠 높이로 작아진 상자와 uncut 상자의 정렬 차이를 버그로 오판하지 않기 위해서다.

초기 테스트의 CharShape 필드명(`height` → `base_size`) 컴파일 오류와 물리 프레임 비교 조건을
정정했다. 정정된 **동일 테스트**를 review worktree의 R1 소스 `cb4563233`에 적용해 다시 실행했다.
`r2-corrected-red.log`: 5 통과·2 실패. 동일 프레임에서도 정렬 실패(46.83px vs 3.68px), 마지막 빈
문단 높이 실패(18.88px vs 13.33px)가 재현됐다. 선택/미선택 소유권 검사는 R1에서도 통과했다.
즉 새로 고친 문제와 이미 작동하던 보호 동작을 구분했다. 이 진단용 테스트 변경은 복원 후
review worktree를 최종 SHA로 다시 전환했다.

### 최종 검증

모두 `/home/edward/mygithub/rhwp-review-7032`에서 파생 suite를 새로 준비한 뒤 순차 실행했다.
고정 target은 `/home/edward/mygithub/rhwp-shared-review-target`이다. 공통 명령:

```bash
node scripts/rust-test-suite-manifest.mjs --prepare
node scripts/run-rust-test.mjs --cargo-test <case-name> -- \
  --target-dir /home/edward/mygithub/rhwp-shared-review-target
```

| case | 실제 결과 |
| --- | --- |
| `issue_7032_cell_empty_paragraph_flow` | 7 통과 (R1 4개 + R2 3개; 내부 조합 포함) |
| `issue_7028_partial_table_diagonal` | 9 통과 |
| `issue_2146_no_ls_label_cell_declared_height` | 1 통과 |
| `issue_6110_cell_empty_para_float_anchor` | 1 통과 |
| `issue_6660_empty_para_line_not_added_to_object_height` | 1 통과 |
| `issue_6035_cell_row_line_split_keep` | 1 통과 |
| `issue_6035_cell_split_empty_band` | 2 통과 |
| `issue_2279_layout_oracles` | 3 통과·기존 ignore 1 유지 |

**합계 25 통과·1 ignored.** #2279 ignored는 `#5193` 프레임 이관에 따른 기존
`issue_2279_nested_cell_units_split_r27_not_r26`이며, 이번에 추가하거나 해제하지 않았다.
통과 건수에 포함하지 않는다. 로그는 `output/7032/r2-final-<case-name>.log`에 있다.

- `cargo fmt --all -- --check`: 통과 (`r2-final-fmt.log`).
- prepared manifest `--check`: 통과 (`r2-final-manifest.log`).
- `node --test scripts/tests/rust-test-suite-manifest.test.mjs`: 23/23 통과 (`r2-manifest-policy-tests.log`).
- source-side `#[cfg(test)]` 변경 없음. generated suite/manifest는 커밋하지 않음.
- 전체 release-test, 세 Clippy, Native Skia 게이트, Docker WASM은 **아직 실행하지 않았다**.

### R1 시각 결과 보존

최종 native로 HWP와 HWPX를 `export-svg --font-style`로 다시 내보냈다.

- HWP: `output/7032/r2-after-hwp/`, R1 `r1-after-hwp/svg/`와 6쪽 각각 `cmp` 동일.
- HWPX: `output/7032/r2-after-hwpx/`, R1 `r1-after-hwpx/`와 `diff -rq` 동일.
- 합계 **12개 SVG 바이트 동일**. R1에서 승인된 실제 출력이 바뀌지 않았으므로 새 시각 통과를
  임의 선언하지 않고 기존 메인테이너 판정의 보존 증거로 기록한다.

## 다음 단계

Stage 3 전체 회귀·순차 lint·Native Skia·Docker WASM 및 Studio 확인을 진행해야 한다.
원격 push·PR·병합·이슈 close는 수행하지 않았으며, 이번 focused 결과만으로 완료 처리하지 않는다.
