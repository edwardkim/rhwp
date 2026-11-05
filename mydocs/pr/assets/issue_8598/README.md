# 이슈 #8598 — 기존 미달 문서와 부적합한 행 분할 검사 보존

[이슈 #8598](https://github.com/edwardkim/rhwp/issues/8598)에 같은 원본의 31·33쪽 원점 미달과
회귀 재구축을 등록했다. 이관은 출력 결함 해결이나 시각 통과가 아니다. 통합 PR의 완료·Closes
범위에 포함하지 않는다.

## 제거 전 모듈과 복원 정보

| 항목 | 값 |
| --- | --- |
| 이전 경로 | `tests/issue_713.rs` |
| 보존 경로 | [retired-tests/issue_713.rs.txt](retired-tests/issue_713.rs.txt) |
| 직전 파일 변경 commit | `ea564999e1e8a343c3129fa9c3a8bda88ab3e668` |
| Git blob | `ceaf84a497f9f15cb5590acdfc4295ad42fa9d07` |
| SHA-256 | `b43b84698db03567b5996102cf17943042fb4106ee4960b4f400bfbe03335be8` |
| 제거 함수 | `issue_713_rowbreak_table_no_intra_row_split` 1개 |
| 전체 모듈 확인 | import·helper·함수·assertion을 포함한 source blob과 바이트 동일 |
| 공개 원문 | [고정 base의 전체 모듈](https://github.com/edwardkim/rhwp/blob/7fdd5441ff1e77c083818e920a48089f294de35e/tests/issue_713.rs) |

[복원 원장](retired-tests/ledger.tsv)에 제거·유지 범위를 기록한다. `.rs.txt`는 원본의 잘못된
`RowBreak` 설명과 `clip=false` 기대값까지 역사 자료로 보존한 것이며 현재 정본 계약이 아니다.
사본 추가만으로 자동 수집 모듈 집합이 바뀌지 않음과, 활성 제거 후 정확히 이 모듈만 제외됨을
확인한다. 파생 `tests/generated/`·suite manifest는 검토 worktree에서 정본 절차로 재생성하며
그 파생 변경을 source PR에 커밋하지 않는다.

## base·후보의 실제 증거

- base: `7fdd5441ff1e77c083818e920a48089f294de35e`.
- 후보: `895205d5a7c9a545997e4d4248dcceac6c9314f9`; 제품 source는
  `4b149b6cdfc514b9858ed644be2cf8120c731cd4`와 같다.
- base Native SHA-256: `a02afdf7440e8cb987fc8d2bd074023278422a91067ffd0bc2e5b8f1601d1bd1`.
- 후보 Native SHA-256: `71a6e065770d741971ad618e614a2d0bc1759cf2eefd4ce9b72ad395032f8d8f`.
- 2026-11-04, 96 DPI, Print profile, 호환 모드 2022, 색상 경계 보정 전 2px raw 실루엣.
  독립 Print 비교에는 시스템 등록 글꼴을 공급했다. Rust는 시스템·private 글꼴을 가린
  저장소 허용 글꼴 5개 환경에서 실행했다.
- Native 전 35쪽 / Print 전 35쪽, 양쪽 Native 최저 40.25603%.
  fresh WASM은 이번 이관 근거로 측정하지 않았다. Native 미달만으로 이관하는 정본 절차를
  적용했으며 두 backend를 검증했다고 보고하지 않는다.
- 전체 원시 TSV: [base](native/base.tsv) · [후보](native/candidate.tsv).

| 물리 쪽 | base Native raw | 후보 Native raw | 직접 판독 |
| ---: | ---: | ---: | --- |
| 31 | 78.27216% | 78.32835% | 기존 원점 미달 |
| 32 | 99.94707% | 99.94705% | 대상 행 5열과 지원·교원 자격·연수 내용이 같은 쪽에 한 번씩 표시됨 |
| 33 | 40.25603% | 40.25603% | 제목 상자와 뒤 본문의 세로 위치 차이; 양쪽 PNG 바이트 동일 |

- 31쪽 [review](png/p031-native-review.png) · [overlay](png/p031-native-overlay.png).
- 32쪽 [review](png/p032-native-review.png) · [overlay](png/p032-native-overlay.png).
- 33쪽 [review](png/p033-native-review.png) · [overlay](png/p033-native-overlay.png).

33쪽 PNG SHA-256은 양쪽 모두
`c4f43616d53ebdf4c498fd4906cb8abab06d691792d5f19b33508416687eca6e`다.
기본 전체 회귀는 12,558건 중 12,557 PASS / 이 검사 1 FAIL / 45 skipped였다.
같은 전체 빌드 구성을 유지하고 이 검사만 선택했을 때도 clip 5개 때문에 exit 100으로 실패했다.
Native 집중 검사는 base·후보 둘 다 통과했다. 구현 경로의 viewport 플래그를 실제 행 분할의
대용 지표로 고정한 조건이 부적절하며, 올바른 행·내용 소유 보호까지 폐기하는 근거로 쓰지 않는다.

## 원본·PDF와 유지 범위

| 공유 입력 | SHA-256 | 처리 |
| --- | --- | --- |
| [원본 HWP](../../../../samples/2022년%20국립국어원%20업무계획.hwp) | `ab59c95dde8cd42e490f7b9a3deb13a9142969706e784053237bf9dc625150e9` | 기존 경로·바이트 유지 |
| [형식별 Print PDF](../../../../pdf/2022년%20국립국어원%20업무계획-hwp-2020.pdf) | `96815defdd3a7a05e1f827a75e781150ffe378b11b76cefb6c15eb6732c3c513` | 기존 경로·바이트 유지 |
| [원래 2022 PDF](../../../../pdf/2022년%20국립국어원%20업무계획-2022.pdf) | `94695a77c2e68dedaa9e0658c1d490a1a89b561ab6419aa39cd99bc9d4a1a153` | 기존 경로·바이트 유지 |

형식별 PDF의 metadata 작성 제품은 한컴 2022이며, 원래 2022 PDF와 31·32·33쪽을 별도로
래스터화해 픽셀이 같음을 확인했다. 다른 기준을 골라 미달을 숨기거나 새로 Print하지 않았다.

이 원본은 제목·추출·보안·parser/serializer·편집 등의 공유 입력이다. 해당 정상 계약들과
다른 문서를 함께 제거하지 않는다. 제거 범위는 잘못된 clip 기대값을 가진 #713 모듈 하나다.
다른 검사들의 통과를 이 문서의 피델리티 통과로 확대하지 않는다.

## 복원 조건과 재현

같은 원본·독립 Print의 전쪽 Native/fresh WASM raw 90% 이상·쪽수 일치와 31·32·33쪽·변경 경계의
확대 판독을 완료한다. 이후 새 이름의 관계형 검사로 셀 존재·열 소속·전체 내용·누락/중복·실제
조각 소속과 앞뒤 내용을 보호한다. 잘못된 `clip=false`나 과거 25px 임계값은 복원하지 않는다.

각 source의 Native Skia 제품을 공용 `target/pr-review`에서 빌드하고 해시로 고정한 뒤 실행한다.
기존 checkpoint 재개에는 `--resume`을 사용한다.

```bash
venv/bin/python scripts/visual_sweep.py --silhouette-only \
  --hwp "samples/2022년 국립국어원 업무계획.hwp" \
  --pdf "pdf/2022년 국립국어원 업무계획-hwp-2020.pdf" \
  --compat 2022 --dpi 96 --key issue713-workplan \
  --out "output/pr-review/<검토-id>/<base-or-candidate>" \
  --rhwp-bin "<해당-source의-고정-Native-제품>" --page-workers 1
```

원 해결 PR·사양·구성별 검사의 상세 대조는
[검증 기록](../../../working/task_m100_713_clip_contract_revalidation.md)을 따른다.
