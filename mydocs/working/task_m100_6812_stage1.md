---
kind: investigation
status: active
canonical: mydocs/plans/task_m100_6812.md
issue: 6812
last_verified: 2026-09-07
---

# #6812 Stage 1 — 회귀 분류와 배치 경로 조사

## 1. 결론

**#6798이 새로 만든 회귀는 아니다.** 기존 devel과 #6798 결합 후보, 밤사이 변경을
포함한 갱신 devel 모두 1쪽 render tree JSON이 byte-identical이다. 현재 처리 분류는
**#6798 이전부터 존재하는 미해결 결함, 과거 최초 발생 시점 미확정**이다.

결함 경로는 다음과 같다.

```text
동일 문단: 종이 기준 non-TAC Square 그림 → TAC 표
  그림: paper_images로 출력, 본문 flow y는 그대로
  회피 정보: Para 기준 그림·col_node 탐색만 지원 → Paper 그림 누락
  표: 독립 PageItem::Table 경로, 같은 문단의 그림 점유 영역을 참조하지 않음
  결과: 가용 폭 부족으로 줄을 내려야 하는데 기존 y에서 표를 그림과 겹쳐 배치
```

저장 vpos를 무조건 복원하거나 기존 owner 조건 하나만 제거하는 수정은 충분하지 않다.
현재 밴드는 top/bottom만 있고 가로 구간·control 식별자가 없어, 어울림 그림 **옆의
남은 폭**과 **자기 자신이 아닌 선행 형제 개체**를 판별할 수 없다.

## 2. 기준선과 실행 결과

| 기준 | 소스 SHA | 1쪽 결과 |
| --- | --- | --- |
| 9/6 고정 devel | `3960844b2f4a546a120cbbb50ae72ef2e5e7239f` | 겹침 재현; 이전 조사 실행에서 확인 |
| #6798 결합 후보 | `d6eda71014be6896500c5832e28c0c3a9e5d23d7` | 기존 바이너리를 재실행해 동일 결과 확인 |
| 9/7 갱신 devel | `08bf41c69e4fbdc43b7bf4e2566cbe42a05b2bf5` | 이번에 native 빌드·재실행해 동일 결과 확인 |

첫 대조의 제품 source 차이는 `src/renderer/layout.rs`의 #6798 hunk 55줄뿐이었다.
이번 빌드는 작업 branch `a21b9d127964fabfc2743f87fb1ac75feb2ec2ac`에서 수행했고,
`git diff --exit-code upstream/devel -- src crates Cargo.toml Cargo.lock`이 0으로 끝나
갱신 devel과 제품 source·의존성 일치를 확인했다. #6798 후보 역시 실행 전에 제품 diff가 없었다.

### 실행 명령과 출처

```bash
cargo build --locked -p rhwp --bin rhwp --target-dir target/pr-review
RHWP_DEBUG_TAC_CURSOR=1 target/pr-review/debug/rhwp export-render-tree \
  /home/edward/mygithub/rhwp-review-6798/samples/issue6797/156160455-social-pig-farm-income.hwp \
  -p 0 -o output/6812/stage1/latest
target/pr-review/debug/rhwp dump-pages \
  /home/edward/mygithub/rhwp-review-6798/samples/issue6797/156160455-social-pig-farm-income.hwp \
  -p 0 --json
```

- 이번 native build는 정상 종료했다(exit 0, 1m03s). 두 render-tree 내보내기와
  최신 dump-pages도 exit 0이었다.

CLI는 페이지0, 기본 compat 2022, 문단 기호·control 표시·vpos-reset 추가 설정 없이 실행했다.
`load_document`는 동일한 `HwpDocument::from_bytes` 경로이며 CLI에서 외부 font source를
주입하지 않았다. 아래 결과는 이 CLI 조건의 geometry 비교이며 별도 폰트 환경의 Studio
pixel 비교와 구분한다.

| 산출물 | SHA-256 |
| --- | --- |
| 고정 devel native(선행 실행) | `1602f5a97d84ad67c3d1ef6b25e0ae71eed334181d00300e7368c116231167c3` |
| PR 후보 native | `213c307bb4bf88b05dd2539fd34b3a9230020de583c6ba984ba4f0325623336d` |
| 갱신 devel native | `625890932853d861b7218dfe40f25c555172ebe0f198b773cef76fca67ea57ae` |
| 3개 기준 공통 1쪽 JSON | `7333991996ac6e4d8b40993e62627209639f662259544b546613de718a7c3740` |

현재 실행 산출물은 `output/6812/stage1/{latest,pr-candidate}/render_tree_001.json`과
같은 폴더의 진단 로그·`latest-pages.json`이다. 이전 `/tmp/rhwp-6812-stage1` 파일들은
이번 재개 시 존재하지 않아 PR 후보를 다시 실행했다. 고정 devel 값은 앞선 실행 기록이며,
이번에 고정 devel을 다시 빌드했다고 기록하지 않는다. 위 수치·hash를 이 문서에 보존하고
중간 JSON·로그·native binary는 제출하지 않는다.

## 3. 실제 배치와 기대 관계

| 대상 | 96dpi bbox/흐름 |
| --- | --- |
| 본문 단 | x=75.5867, y=79.36, w=642.5333, h=963.7867 |
| 그림 pi=0 ci=3 | x=75.9, y=83.1, w=641.0, h=60.4 |
| TAC 표 pi=0 ci=4 | x=76.6, y=81.2, w=639.7, h=154.7 |
| 다음 제목 표 pi=1 ci=0 | x=77.5, y=239.7, w=638.8, h=87.0 |
| 본문 pi=3 첫 줄 | y=380.7 |

진단 로그는 그림이 `y_in=79.4 → y_out=79.4`, 표가
`79.4 → 237.9`임을 보여준다. 표는 그림 아래로 줄이 이동한 약 64.13px를 소비하지 않는다.
저장 LineSeg 4810 HWPUNIT를 단 상단에 더한 `143.4933px`는 그림의 실제 하단과 일치한다.
이는 기대 관계의 보조 증거이며, 표 테두리 top은 바깥 여백과 구분해야 한다.

첫 문단에는 일반 텍스트가 없고 그림은 non-TAC이다. 따라서 #6754의 **TAC 그림+TAC 표**
폭 합산 경로에서도 이번 그림은 제외되고, 표가 독립 PageItem으로 배치된다.
그림이 본문 폭의 대부분을 점유하므로 옆 여유는 수 px 수준이며 약 640px인 표가 들어갈 수 없다.
다만 일반 구현에서는 실제 문단 여백·표 advance·개체 bbox로 가용 구간을 계산해야 한다.

## 4. 소스에서 확인한 누락 지점

갱신 기준 소스의 위치이며 이후 수정으로 줄 번호가 달라질 수 있다.

1. `src/renderer/layout.rs:12214`의 `is_paper_based` 분기는 그림을 임시 parent에서
   렌더한 뒤 `paper_images`에 넣는다. `result_y`는 입력 y를 유지한다. 절대 위치의
   그림이 그 자체 높이만큼 본문을 무조건 미는 것은 아니므로 이 사실 단독은 버그가 아니다.
2. `layout.rs:620`의 `square_picture_side_wrap_exclusion`은 Para 상대만 허용하고
   bbox를 `col_node`에서만 찾는다. 이번 Paper 그림은 조건·탐색 위치 모두에서 제외된다.
3. `layout.rs:582`의 `VisibleFloatExclusion`에는 y 구간·owner_para·blocks_text만 있다.
   x 구간 및 owner control이 없어 같은 문단 내 어울림 줄 배치를 직접 표현하지 못한다.
4. `layout.rs:9701`의 TAC 회피는 `!blocks_text || owner_para >= para_index`이면
   건너뛴다. 기존 Square 그림 밴드는 blocks_text=false이며 동일 문단 owner도 제외된다.
5. `height_cursor.rs:192`의 vpos 보정은 이전 문단이 없거나 같은 문단이면 입력 y를
   반환한다. 이번 첫 문단의 Shape→Table 사이 줄 이동을 inter-paragraph 보정이 대신하지 않는다.
6. `height_measurer.rs:388`의 inline 분류는 빈 host의 TAC 개체 수·폭을 본다.
   non-TAC 그림의 어울림 공간은 그 합산 대상이 아니다.

이 중 하나만 완화하면 다른 조건에서 계속 누락되거나, 반대로 옆 배치 가능한 개체까지
무조건 아래로 밀 수 있다. 수정은 개체 식별·실제 가로 점유와 세로 흐름을 함께 다뤄야 한다.

## 5. 원인 계보와 최초 발생의 한계

| 기존 변경 | 원래 보호하는 동작 | 이번 조합이 포함되지 않은 이유 |
| --- | --- | --- |
| #5929 / `0c2f1cfe310e3394cd4d73bf95a88d662a131053` (2026-08-26) | Para 기준 Square 그림 아래 non-TAC T&B 표 | Paper 그림·TAC 표 대상 아님 |
| #6104 포함 `2ca9aa90f7b1463d34f07ded1981577939b74598` (2026-08-30) | 이전 문단 T&B 표 아래 TAC 표 | blocks_text 및 이전 owner로 제한 |
| #6754 / `309aa25d0ec71833d61a81329f88c9c16b820fed` (2026-09-05) | TAC 그림과 TAC 표의 옆 배치 | non-TAC 그림 제외 |
| #4639 관련 기존 경로 | HWPX/PARA 동거 float 뒤 후속 문단 | HWP5/Paper/host 내부 표와 다름 |

위 날짜는 **해당 처리 도입일**이지 이번 결함의 발생일이 아니다. #6754 이전 소스도
이번 문단에서 TAC 표 하나만 집계하므로 inline false였다는 점은 코드로 확인했다.
그것만으로 이전 전체 renderer의 실제 출력까지 같았다고 단정하지 않는다.

기존 report·working·tech·troubleshootings에서 원본 식별자를 검색했지만 과거 정상
geometry 증적은 찾지 못했다(폰트 계측 로그는 조판 정상 증거가 아니다). 따라서 정상/비정상
양끝점 없는 bisect와 과거 후보 무작위 빌드는 하지 않았다. 추가 역사 탐색은 필요한 근거가
생길 때 한정한다. 제품 수정은 현재의 구조적 처리 공백을 해소하는 목표로 진행할 수 있다.

## 6. 구현 시 반드시 검증할 미해결 항목

- 실제 그림·표의 `allow_overlap` 의미와 원본 값, 표의 바깥 여백 포함 advance를
  RED 계약에 명시한다. 기존 bool 하나로 어울림의 모든 의미를 대체하지 않는다.
- renderer에서 y만 이동하면 pagination 예산과 불일치할 수 있다. 현재
  `pagination/engine.rs`의 TAC 높이 cap과 표 배치 예산에는 그림 회피 거리의 명시적
  전달이 없다. 페이지 하단 경계 시험으로 측정·분할 경로의 보정 필요성을 판정한다.
- `dump-pages`의 usedHeight와 hwpUsedHeight 차이를 이번 그림 회피 거리와 동일시하지 않는다.
- 이번 단계는 CLI geometry·코드 조사다. 새 이미지 비교, 전체 회귀·Clippy·WASM 빌드,
  수정안의 시각 통과를 수행한 것으로 기록하지 않는다.

## 7. 산출·다음 게이트

### 메인테이너의 구현 방향 정정 (2026-09-07)

조사한 구조는 결함을 발견한 경로이지 수정 범위를 그 예제로 제한하는 근거가 아니다.
그림은 자신의 기준 좌표로 먼저 배치하고, 이어 TAC 표가 가용 공간에 들어가지 못하면
그림 아래의 다음 줄 위치를 계산하는 **엔진 규칙**을 구현한다. 저장 vpos는 그 결과의
검증 단서이며 목표 좌표로 강제하는 값이 아니다. 빈 host·특정 pi/ci·독립 PageItem만을
위한 우회 구현은 배제한다. 수행계획과 구현계획을 이 기준으로 정정했다.

계획과 오늘할일을 갱신했다. 제품 source·시험·원본·PDF는 변경하지 않았다.
기존 #6798 review worktree는 유지하고 새 worktree·고아 branch를 만들지 않았다.
Stage 1 결과를 커밋한 뒤 별도 구현계획을 제안하며, 승인 전에는 Stage 2 구현을 시작하지 않는다.

용어: bbox는 bounding box(개체를 둘러싼 좌표 상자), Paper/Para는 종이/문단 기준,
T&B는 TopAndBottom(자리차지), advance는 줄에서 개체가 소비하는 폭,
RED→GREEN은 수정 전 실패와 수정 후 통과를 증명하는 시험 절차다.
