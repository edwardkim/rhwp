# #7028 Stage 2 — 온전한 셀 대각선 연결 및 focused 검증

- 이슈: https://github.com/edwardkim/rhwp/issues/7028
- 날짜: 2026-09-11 (KST)
- [구현계획](../plans/task_m100_7028_impl.md): 메인테이너 승인 후 수행.
- 작업 브랜치: `task_m100_7028`.
- 검증 worktree: `/home/edward/mygithub/rhwp-review-7028` (detached).
- 검증 후보: `1ae766455adeeaf9ee4779fa310de08dc2882bda`.
- 상태: 구현·focused 회귀·native SVG 준비 완료. 메인테이너 시각 판정 대기.

## 1. 구현

제품 변경은 `src/renderer/layout/table_partial.rs` 한 파일이다.

- `partial_cell_has_complete_rows`: 원본 셀의 모든 행이 현재 표시 행 목록에 순서대로
  들어 있는지 판단한다. 표 continuation 여부나 제목 셀 여부만으로 분기하지 않는다.
- `partial_cell_intersects_diagonal_zone`: 활성 대각선/중심선 zone과 실제로 겹친 셀만
  이번 변경에서 제외한다. 무관한 zone·비활성 zone 때문에 전체 표를 제외하지 않는다.
- 기존 cut·height override 조건과 결합해 온전한 셀만 `render_cell_diagonal`에 연결했다.
- 대각선 노드는 가로·세로쓰기 분기 전에 한 번 생성하고, 각 경로에서 Cell 뒤에 한 번
  추가한다. 세로쓰기의 조기 continue로 누락되던 경로도 포함한다.
- 좌표는 표 격자의 `cell_x/y/w/h`를 사용하며 사후 확장된 내용 clip bbox를 사용하지 않는다.

파서·IR·제목행 선택·높이 측정·페이지네이터·일반 표/zone 렌더 코드는 변경하지 않았다.
실제로 잘린 셀·활성 zone 영향 셀은 승인된 범위대로 기존 출력 유지다.

## 2. 수정 전 실패와 테스트 전제 정정

1. `38ba874f6`은 제품 코드를 변경하지 않고 새 회귀 7개만 추가한 commit이다.
   별도 review worktree에서 prepare 후 실행: **1 통과 / 6 실패**.
   대표 실패는 r0c0 대각선 기대 1개, 실제 0개이며 셀 bbox는
   `(71.9600,225.2667,56.1600,52.3733)`이었다. 선 없음 검사는 통과했다.
2. 제품 수정 `38eb14166`에서 SVG 출력 및 병합 셀/분할 경계 검사를 더한 9개 중
   **8 통과 / 1 실패**. 실패는 구현 결과가 아니라 "row9가 두 쪽에 나타난다"는 테스트
   전제 assertion이었다. 실패를 숨기거나 baseline 기대값을 새 구현에 맞춰 바꾸지 않았다.
3. 수정 전 6쪽 RenderTree를 재확인하니 두 쪽에 나타나는 셀은 r6c0, r20c0, r60c0이었다.
   row9는 end-cut이 있지만 첫 쪽에서 소비를 마쳐 두 번째 인스턴스가 없다.
4. `1ae766455a`에서 테스트만 정정했다. r9c2는 page_fragment 메타데이터와 한 번 표시를,
   r6c0은 page_fragment와 두 쪽 이상 표시를 assert한다. 두 경우 모두 새 사선이 없고
   온전한 제목 셀에만 사선이 있는지 검사한다. 제품 구현 조건은 바꾸지 않았다.

이 과정은 **cut 메타데이터가 있다고 반드시 다음 쪽에 같은 셀이 다시 나타나는 것은 아니다**라는
경계 구분을 고정한다. 용지 크기를 바꾼 합성 파일은 만들지 않았다. IR 변형 테스트는 알고리즘
검사이며 한컴 정답지로 사용하지 않는다.

## 3. focused 검증

source commit 후 같은 review worktree를 해당 SHA로 전환하고 파생 suite를 다시 준비했다.
공유 Cargo target은 `/home/edward/mygithub/rhwp-shared-review-target`이며 Cargo 실행은 순차다.

| 검사 | 결과 |
| --- | --- |
| 새 #7028 회귀 | 9/9 통과 |
| #2146 실물 제목 셀 높이 | 1/1 통과 |
| #1623/#1633 기존 zone·개별 셀·중심선 | 19/19 통과 |
| manifest generator 계약 검사 | 23/23 통과 |
| 준비된 review worktree의 `cargo fmt --all -- --check` | 통과 |
| 파생 suite manifest `--check` | 통과 (원본 1258개, 실행 target 48개) |
| native `release-test` 빌드 | 통과 (2분 03초) |

새 회귀의 범위는 실제 6쪽의 제목 셀 5개·높이·y·사선 끝점·검정 실선 및 SVG 한 번 방출,
반복 끄기, 선 없음/type=0, 세로쓰기, 일반 본문 셀, 온전한 rowspan,
완료된 end-cut 행과 실제 straddle rowspan, 활성/비활성/비교차 zone이다.
height override만 단독으로 발생시키는 실물 테스트는 아직 추가 실행하지 않았으며,
기존 해당 계열 회귀는 Stage 3 영향 검증 대상으로 남긴다.

실행(각 case 이름을 바꾸어 3회):

```bash
node scripts/rust-test-suite-manifest.mjs --prepare
cargo fmt --all -- --check
node scripts/run-rust-test.mjs issue_7028_partial_table_diagonal -- \
  --cargo-profile release-test --target-dir /home/edward/mygithub/rhwp-shared-review-target
node scripts/run-rust-test.mjs issue_2146_no_ls_label_cell_declared_height -- \
  --cargo-profile release-test --target-dir /home/edward/mygithub/rhwp-shared-review-target
node scripts/run-rust-test.mjs issue_1623_cellzone_diagonal -- \
  --cargo-profile release-test --target-dir /home/edward/mygithub/rhwp-shared-review-target
```

배정 suite 번호는 원본 weight 변화에 따라 달라지므로 고정하지 않는다. 테스트 필터로 제외한
동일 suite의 다른 검사 수는 성공 검사 수에 넣지 않았다. 로컬 nextest 0.9.137은 권장
0.9.140보다 낮으며 CI-duration-observation의 JUnit 옵션 경고가 있었지만 위 focused 실행은
완료됐다. 도구 버전 변경이나 전체 CI 통과로 확대하지 않는다.

로그: `output/7028/stage2/red.log`, `green.log`(초기 전제 실패), `green-final.log`,
`2146.log`, `1623.log`, `manifest-tests.log`.

## 4. 실제 6쪽 산출과 변경 영향

동일 review 후보에서 다음 명령으로 native를 빌드했다.

```bash
cargo build --locked --profile release-test --bin rhwp \
  --target-dir /home/edward/mygithub/rhwp-shared-review-target
```

실행 파일 SHA-256: `a5614823f20bba5164aeaea1fd641cc621e996559dd18aba369f34404f920f46`.
원본과 기존 한컴 PDF를 그대로 사용해 프로젝트 `fidelity_compare.py`로 6쪽 전체를 내보냈다.
PDF의 파일명은 hwp-2020이지만 실제 metadata는 Hwp 2022 / Hancom PDF다. 새 변환 PDF가 아니다.

- SVG: `output/7028/stage2/fidelity/svg/21761835_jeonjik_exemption_table_001.svg`부터
  `_006.svg`까지. 파일명 번호는 실제 쪽 번호다.
- 표준 비교 이미지: `output/7028/stage2/visual/cmp-p000.png`부터 `cmp-p005.png`까지.
  이미지 번호는 0-based다. 왼쪽 한컴 PDF, 오른쪽 수정 native 출력이다.
- baseline 대조: `output/7028/stage2/geometry.json`. 6쪽 각각 대각선 0→1개이며,
  이 선을 제외한 **모든 내보낸 RenderTree 노드가 동일**하다. 셀·텍스트 좌표를 바꾸지 않았다.
- `text-report.tsv`는 Stage 1 파일과 byte 단위로 동일하다.
- 1·2·6쪽 비교 이미지를 확인했다. 첫 제목 셀의 좌상단→우하단 선이 후속 쪽에도 보인다.
  기존 제목 라벨의 수직 위치, 글꼴, 본문 행 높이 차이는 남아 있다. 특히 `직렬` 글자는
  한컴보다 위에 있어 새 선과 가까우므로 메인테이너가 확대 확인해야 한다.
  선 방출을 확인한 것을 문서 전체의 한컴 일치나 최종 시각 통과로 보고하지 않는다.

1·2·6쪽 pixel diff는 각각 14.02%, 16.56%, 14.84%다. 기존 차이를 포함한 fidelity 지표이며
visual sweep ink 정확도나 합격 기준이 아니다. 로그는 `build.log`, `fidelity.log`,
`manifest-check.log`, 비교 도구와 기하 대조 스크립트는 위 output 아래에 로컬 보존한다.

## 5. 다음 게이트

메인테이너에게 1·2·6쪽 시각 판정을 요청한다. 이후 Docker WASM/Studio 및 전체 영향 회귀·OVR·Clippy 3종·
workspace/Native Skia 검증을 진행한다. 이번 focused 통과는 그 전체 게이트를 대체하지 않는다.
원격 push·PR 생성·GitHub 상태 변경은 하지 않았다.

## 6. 메인테이너 피드백 — 선 확인, 빈 문단의 텍스트 진행 누락

메인테이너는 대각선 렌더링을 확인했다. 동시에 편집자가 빈 두 줄 후 세 번째 줄에
`직렬`을 배치하려 했으나 Studio가 이를 무시한다고 지적했다. **대각선 확인만 기록하며
셀 전체 시각 판정 통과나 Stage 3 승인을 의미하지 않는다.**

현재 저장소 원본의 SHA-256은 앞서 사용한 값과 같다. native `dump -s 0 -p 4`와 CFB
`BodyText/Section0` 원시 레코드를 함께 확인했다.

- 첫 셀 LIST_HEADER(record 24)는 문단 수 **2**를 선언한다.
- record 25 PARA_HEADER: char_count=1, ParaShape=53. 다음 record 26은 CharShape=37이다.
  이 빈 문단에는 PARA_TEXT와 PARA_LINE_SEG가 없다. 문단 자체는 존재한다.
- record 27 PARA_HEADER: char_count=3, ParaShape=53. record 28 PARA_TEXT의 UTF-16 값은
  `[51649, 47148, 13]`, 즉 `직렬\r`이다. 별도의 선행 줄바꿈은 없다.
- native IR 요약도 `paras=2 text="|직렬"`이다. 따라서 이 저장본에서 확정할 수 있는 것은
  **빈 문단 1개 + 글자 문단 1개**이며, 메인테이너가 설명한 빈 두 줄과는 구분한다.
  편집 중인 다른 저장본 여부나 한컴 UI 줄 수의 의미는 미확인이다. 임의로 두 문단을 보충하지 않는다.

누락 경로:

1. `composer.rs::compose_lines`: LINE_SEG 없음 + text 비어 있음이면 빈 줄 목록을 반환한다.
2. `recompose_cell_lines_in_frame`: 합성 줄 목록이 비어 있으면 조기 반환한다.
3. `table_partial.rs::layout_partial_table_cells`: 온전한 셀에서도 이 문단의 줄 범위가
   `(0,0)`이 되어 `start_line >= end_line` 분기에 걸리고 y 진행 없이 continue한다.
4. 반면 `table_layout.rs::calc_para_lines_height`에는 빈 줄 목록인 문단도 글꼴·줄간격으로
   높이를 계산하는 경로가 있다. **측정에는 존재하고 표시 진행에서는 사라지는 불일치**다.

Stage 1·2의 첫 셀 RenderTree는 선을 제외하면 동일하다. 두 출력 모두 셀 y=225.3,
높이=52.4px, `직렬` TextLine y=235.3px이며 child는 pi=1 하나뿐이다. 빈 pi=0의 줄 노드는 없다.
따라서 이번 대각선 연결로 새로 발생한 회귀가 아니라 이미 존재하던 텍스트 배치 문제다.
과거 최초 유입 시점을 확정하는 bisect는 하지 않았다.

수정 방향은 텍스트를 특정 px만큼 내리는 것이 아니라 **실제 빈 문단의 줄 진행을 측정과
렌더링이 동일하게 소비하도록 하는 것**이다. 저장 LINE_SEG 유무, 순수 빈 문단과 control-only
문단, 온전한 셀과 실제 cut 범위를 구분해야 한다. 기존 #2146의 선언 셀 높이와 Fixed 줄간격
보호 조건도 함께 검증해야 한다. 이번 응답에서는 원인 조사만 했으며 제품 코드·테스트·WASM은
변경하지 않았다. 구현 범위 확대와 수정 계획은 메인테이너 결정 후 진행한다.

## 7. 범위 확정 및 완료 절차 승인

메인테이너는 빈 문단 텍스트 문제를 별도 이슈로 분리하고 #7028의 완료 절차를 진행하도록
지시했다. [후속 #7032](https://github.com/edwardkim/rhwp/issues/7032)를 등록하고 담당 edwardkim,
v1.0.0, bug/layout/rendering 및 한글 본문을 API 재조회로 확인했다. #7032 구현은 이번 변경에
추가하지 않는다. 대각선 렌더링은 메인테이너 확인 완료이며, 기존 텍스트 차이는 #7032로 남긴다.
다음은 Stage 3 Docker WASM·전체 회귀·최종 lint 검증이다. 원격 push·PR·병합은 별도 승인 대상이다.
