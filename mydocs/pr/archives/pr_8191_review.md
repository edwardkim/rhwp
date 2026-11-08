# PR #8191 셀프 리뷰 — 기존 완료 문서의 표·줄·글꼴·출력 경계 보존

## 최종 판정

**승인.** 동결한 기존 문서의 공통 조판·출력 보정 범위에서 필수 로컬 검증과 정확한 code head의
GitHub CI가 통과했습니다. #7445 전체 해결 판정은 아닙니다. KTX 제목 획/그림자, giant 문서의
보존된 API 회귀 복구 등 부분 해결 이슈는 열린 상태로 유지합니다.

문서 trailing head의 실제 CI/fast-pass·병합 가능 상태를 다시 확인한 뒤 머지합니다.
작업지시자는 머지·CI 모니터링·오늘할일과 해결 서브이슈 후속 처리를 승인했습니다.

## 접수 정보와 범위

| 항목 | 확인한 값 |
| --- | --- |
| PR·작성자·대상 | [#8191](https://github.com/edwardkim/rhwp/pull/8191) / jangster77 / devel |
| 검증 code head | `8ba0980feb5995dffbfdb84601e53085b49f810d` |
| 고정 base | `3536e72819e165dbe959ee42cfd4e4cf8db33ce7` |
| 기록 경로 | collaborator self-merge, 지정 reviewer 없음; 자기 PR의 GitHub 승인 review로 가장하지 않음 |
| 조회한 상태 | Open, MERGEABLE/CLEAN, 적용 CI 성공; 최종 머지 직전 다시 조회 |
| 관련 이슈 | [#7445](https://github.com/edwardkim/rhwp/issues/7445) 참조, 대표 이슈 전체 close 없음 |

표의 물리 점유·내용 컷·저장 글줄과 실제 paint 원점을 공통 경로에서 보정하고 글꼴·그림/문자
간격·공개 출력 경계를 보호했습니다. 새 HWP 보정 대상을 확장하지 않았습니다. 별도 PR 범위인
sample16 HWP5의 2024 HWP/HWPX·바이트가 같은 2010 별칭은 제외했습니다. 서로 다른
2022 저장본과 HWP3 원본은 유지했습니다. 검증/제출 조건은 **backend당 49조건·1,772쪽**입니다.

전체 구현·이력은 [Stage90](../../working/task_m100_7445_rebase_repair_stage90.md),
[Stage93](../../working/task_m100_7445_rebase_repair_stage93.md),
[이전 제출 source의 Stage127](../../working/task_m100_7445_rebase_repair_stage127.md),
[현재 Stage137](../../working/task_m100_7445_rebase_repair_stage137.md)에 연결했습니다.
과거 부적합 회귀의 이관과 현재 범위에서 출력 보정 후 관계 검사로 교정한 항목을 구분했습니다.
과거 PR 품질 래칫을 Visual Sweep에 자동 강제 연결하지 않습니다.

## 검증 입력과 생산 출처

**충족.** 실제 사용한 입력과 PDF의 저장소 경로·출처·SHA-256은
[입력 원장](../assets/issue7445/completed-20261009/final-20261024/inputs.json)과
[조건별 입력/PDF·전쪽 TSV](../assets/issue7445/completed-20261009/final-20261024/README.md)에 있습니다.
이 원장의 과거 source 표기는 보존합니다. 현재 `8ba0980f`의 같은 Git blob과 실행 바이트를 다시
대조했고, 50 MiB 이상 PDF는 로컬 경로·바이트 수·hash·독립 쪽수·생산 근거를 확인하여 PR에서
제외했습니다. 크기로 검증을 생략하지 않았습니다. 개인 글꼴 원본·HFT 복원용 표본은 로컬에 보존하며
입력 커밋 증명이나 재배포 가능 글꼴로 오인하지 않습니다.

현재 전쪽 비교는 source77/base e7의 실제 PNG 쌍과 producer/build/font hash를 보존한 뒤
**196 TSV를 현재 정본으로 전쪽 재산출**한 것입니다. 196조건의 actual/reference 7,078쌍을
각각 SHA로 검증했고 원래 쪽 순서·값이 같습니다. 이를8ba의 전쪽 재캡처로 표시하지 않습니다.
새 devel의 읽기 전용 JSON getter 밖 렌더링/파서/글꼴/캡처 경로가 같음을 대조했습니다.
별도 새 head 실제 소비 72관측과 #8180/#8182의 실제 base/head 계약 검증으로 재사용 자격을 보완했습니다.
과거 final-20261024의 50조건 표는 이력이며 그중 sample16-hwpx를 제외한 49조건을 현재 범위로 읽습니다.
현재 196 TSV 중195개는 제출한 사본과 바이트가 같습니다. 달라진 base/WASM/hwp3-alias의
16쪽 TSV는 현재 PR 본문에 원문을 별도 제출합니다. 그3·11쪽의97.01170→99.15465%,
96.33270→99.10484%를 포함한 최신 base값을 사용하며 과거 사본을 현재 base로 재표기하지 않습니다.

| 전쪽 출력 | 조건·참조 쪽수 | 최저 raw | 90% 미달·후보 쪽수 불일치·2%p 이상 하락 |
| --- | --- | ---: | --- |
| Native Skia | 49·1,772 | 93.06876% | 모두0 |
| fresh WASM Canvas | 49·1,772 | 92.99291% | 모두0 |

canonical은 PDFium152.0.7947.0 / pypdfium2 5.12.1 smooth RGB v1·96DPI·raw2px90%입니다.
base의5조건×2backend 쪽수 차이는 그대로 기록했고 같은 수치 쪽을 같은 내용이라고 간주하지 않았습니다.
후보 쪽수와 내용 소속·순서·누락/중복 검사, 독립 Print 판독을 함께 확인했습니다. 예외/마스크는 없습니다.

## 현재 head의 검증과 실제 소비

- 전체 release-test/native-skia nextest **11,766 PASS / 0 FAIL / 45 skip**, 실제 검사1,035.984초.
  현재 base 대비 새 Rust 회귀 **228개(integration224·unit4)** 모두 실제 PASS입니다.
  기존 교체/rename12·이동3은 제외했으며 소스 정의 순증가218과 구분합니다.
- prepare·manifest·unit tiers·fmt·Native/WASM/workspace/Skia all-target Clippy·workspace/release build·
  Skia lib·직접 PDF·placeholder·release lib·doc·Studio unit/TS 등22단계 exit0입니다.
  Skia root3,859 PASS/12ignore·다른 libs182 PASS, PDF14·placeholder2 PASS,
  release root3,802 PASS/12ignore·다른 libs182 PASS, doc8 PASS/3ignore입니다.
- Studio/editor/shared PDF **2,421 PASS / 0 FAIL / 2skip**. 현재 Studio/Chrome/Firefox build·VSCode compile,
  binding/font11·실제 browser글꼴10 계약도 통과했습니다. 묶음 간 중복을 고유 검사 수로 합산하지 않았습니다.
- 새 8ba 실제 소비는 **Native30·Canvas2D30·지원CanvasKit12 =72관측**입니다.
  경로별 최저93.13188%·93.13770%·99.87307%,90% 미달0입니다.
  backend/surface/readiness/글꼴·자원·입력 소유를 확인했습니다. Canvas2D fallback10관측은 CK 통과가 아닙니다.
- 실제 초기화 52관측의 JS/WASM 응답을 검증했습니다. Vite JS는 URL 변환·완전한 source-map,
  WASM은 원시 hash를 대조했습니다. 72관측의 새 PNG 쌍은 기존 판독한 쌍과 바이트가 같아
  판독을 재사용했으며 모두 새로 열었다고 하지 않습니다.
- Mac WASM은 locked wrapper `--target web --out-dir pkg --no-opt` 로컬 대체 빌드입니다.
  Docker 최적화 통과가 아닙니다. 검증 중 JS `f90e8e9655657ea0f23b467ccb89b37b6af24f8f1b387dab9055dae747aea5b9`,
  WASM `69aced2fc42726a35d62182ef7931cf4254bfcd8a388f72f3496eb1d5653ddc9`의 root/pkg·public·frozen·실제 소비를 확인했습니다.
  종료 뒤 public JS의 자동 JSDoc4줄만 복원했고 주석 제외 동일성과 frozen 원본을 보존했습니다.

## 리베이스로 반영한 기능의 보존

401개 패치는 range-diff에서 보존됐으며 자동 병합 tree와 리베이스 tree가 같았습니다.
이를 기능 검증의 대용으로 쓰지 않았습니다. 정확한3536 archive에서 Native/fresh WASM을 순차 빌드했습니다.

- #8180: base/head 각각 실제 Studio 셀 편집6진입 경로와 잘못된 대상9건의 mutation 전 거부,
  이웃·중첩/텍스트상자 소유, undo/redo를 확인했습니다.
- #8182: 전체 nextest의 Native7건, base/head 직접 WASM5계약 묶음과 Studio getter4입력에서
  displayText/displayCharX와 모델 text/charX를 분리하고 선택/삽입 오프셋·SVG 보존을 확인했습니다.
- #8190 코덱의 source77 수행 matrix114 Native관측·68시각 출력은 Stage127의 독립 생산 기록입니다.
  관련 코드 동일성을 확인하여 재사용했으며8ba에서 matrix를 다시 실행했다고 하지 않습니다.

## 공통 조판 원칙 검토

| 검토 항목 | 확인한 근거 | 판정 |
| --- | --- | --- |
| 제출 전 로컬 검증과 PR 상태 | 현재 head 22게이트·frontend6단계·전체 nextest·실제 소비 및 code CI, Open PR | 충족 |
| 기존 이슈·해결 PR 선행 분석 | 기존 Stage 기록의 원 PR/통합 SHA와 독립 Print, Stage90/93/127/137에서 현재 관측 구분 | 충족 |
| 구현 근거와 일반성 | split topology·저장 줄 출처/독립 프레임 계약으로 분기; 문서명/쪽 번호 clamp 없이 반례·정상 대조 실행 | 충족 |
| 측정·배치 일관성 | typeset.rs의 is_reparsed_single_column_cell_split_row, table_layout.rs의 stored_plain_cut_closing_advance와 물리 프레임; Stage90의 stored_square_table_anchor_offset 측정/paint 공통 소비, Stage93의5소비 경로 | 충족 |
| 리베이스 반영 기능 보존 | #8180/#8182 exact base/head 실제 제품 계약과 #8190 재사용 자격을 위 절에서 분리 | 충족 |
| 최종 출력 소비 계약 | Native/Canvas2D/지원CK 실제72관측·초기화52hash, 준비 미달10건은 별도 분류 | 충족 |
| 분할·이어받기 계약 | 공통 컷·요구/예약/배치 높이, giant115/저장197·로드맵 마지막 상자·rewind·TAC 반례의 내용 소속/다음 내용 보존 | 충족 |
| 줄 소속과 점유 높이 | 저장 LineSeg/재래핑 출처, split 물리 폭의 line_breaking 소비와 빈 밴드·padding 구분, 줄/개체 소유 회귀 | 충족 |
| 사례와 증거의 독립성 | 버전에 맞는 독립 한컴 Print와 수정 전 FAIL→후 PASS 기록, 정상/합성·실물 증거 분리 | 충족 |
| 기준값 변경 | 과거 부적합 핀/래칫은 원본·이관 기록·정상 검사 보존, 현재 완료 문서는 출력 보정 뒤 관계 검사;90%/2%p·canonical 비교 문턱의 새 완화 없음 | 충족 |
| 회귀 추가·변경 선행 조건 | 관련 완료 페이지의 Native/fresh WASM 전쪽90% 선행·직접 판독; 관계 검사 기본,98% 완료 쪽의 독립 좌표±2px 조건 구분 | 충족 |
| 90% 통과 후 확대 판독 | 기존38쌍과 현재 추가10쌍의 review/overlay; 수식·dot·간격·표/앞뒤 내용 구분, KTX 잔여는 미해결 이슈 유지 | 충족 |
| 주장과 검증 범위 | source77/e7전쪽 producer와8ba새 actual/API/전체 회귀를 구분,49조건과 이슈10개 종료 후보를 구분 | 충족 |

## 시각 증적과 남은 차이

[현재 추가10쌍의 review/standalone overlay와 판독](../../working/task_m100_7445_rebase_repair_stage137.md#추가-직접-판독과-이슈-종료-범위),
[기존 전쪽 대표 증적](../assets/issue7445/completed-20261009/final-20261024/README.md#확대-판독과-실제-소비-경로)을 함께 확인했습니다.
규제분석의 표/비표시 안내, 자산관리의 붙임·직인·연락처,2022저장본의 본문/복구 표,
보도자료 마지막 행/별첨 소속을 보존했습니다. 얇은 괘선·글리프 획·미세 간격은 남습니다.
KTX 제목 획·그림자, 원본 sample16 수식/dot, sample11 UI Print와 편집기 사진의 차이를
전부 해결한 것으로 확대하지 않습니다. raw와 엄격 RGB/글꼴 동일성은 다릅니다.

## 코드 head CI

정확한8ba head·같은 PR/저장소/branch의 완료·성공을 API로 확인했습니다.

- [Full CI](https://github.com/edwardkim/rhwp/actions/runs/40064292787): Build & Test와 적용 worker 성공.
- [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/40064292763),
  [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/40064292747),
  [Adapter](https://github.com/edwardkim/rhwp/actions/runs/40064292759),
  [Proptest](https://github.com/edwardkim/rhwp/actions/runs/40064292784),
  [Gym 계약](https://github.com/edwardkim/rhwp/actions/runs/40064292754): 성공. Full Gym은 정책상 skip입니다.

trailing의 실제 fast-pass·필수 검사와 exact-head merge는 이후 확인하며 성공을 선기록하지 않습니다.

## Merge 후 contributor PR comment 계획

자기 PR이므로 외부 기여자 코드를 통합한 것처럼 쓰거나 owner를 reviewer로 자동 지정하지 않습니다.
머지 후 최종 merge SHA·CI/fast-pass·검증/재사용 출처·대표 review/overlay의 merge SHA 고정 raw URL을
한국어 존댓말 comment로 남기고 `--body-file` 게시 뒤 API로 원문을 대조합니다.
#7445의 실제 해결 10행만 취소선 처리하고 해당 서브이슈에는 전쪽 수치·입력/Print·회귀 근거를 남겨
종료합니다. #7780/#7813은 부분 개선 comment와 잔여를 기록하며 대표7445는 열어 둡니다.
이미 닫힌 #8060의 좁은 높이 규칙과 #8100/#8095의 전쪽 잔여를 구분합니다.
공유 target/output·사용자7729 서버·원본/PDF·폰트·초안/stash는 보존합니다.
