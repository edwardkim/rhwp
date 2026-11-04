# PR #8499 리뷰 — 시험지 ingest 이미지 실제 삽입

## 최종 판정

**승인 — #665 이미지 삽입 범위의 로컬 검증과 code head CI를 완료했다.**
검토한 code head는 `1f65acd967ca6793997ba8a1e8665550be09574c`다. 아래는 작성자 edwardkim의 self-review이며 외부 reviewer나
GitHub approve 이벤트를 대신하지 않는다. 남은 merge 조건은 이 기록의 최신 trailing head CI와
작업지시자의 별도 merge 승인이다. 이번 원격 승인에는 merge가 포함되지 않았다.

## 접수 정보

| 항목 | 확인값 |
| --- | --- |
| PR / 작성자 / base | [#8499](https://github.com/edwardkim/rhwp/pull/8499) / edwardkim / devel |
| 기준 devel | `ea86816b5316dfeeb47ecf20df97abfd4b02da73` |
| 제품 producer source | `d78048a6f0e5f71aa3bdb7d32e703bd5971e3019` |
| 검토 code head | `1f65acd967ca6793997ba8a1e8665550be09574c` |
| 적용 경로 | collaborator self, intake/local/visual, trailing review-only |
| 작성 시점 | 2026-11-03T06:06:16.315286+09:00 |
| PR 상태 참고값 | Open / reviewer 지정 없음 / MERGEABLE / CLEAN |

## 변경과 검토 범위

CLI와 byte-only Rust API에서 실제 Picture/BinData를 생성하고 배치 4종·문장 내 runs·공유 지문·
보기 안팎 소유를 지원한다. 경로·치수·손상·자원 한도·참조 오류를 실패 처리한다. 기존 텍스트 API와
스타일, 엄격한 JSON 입력 계약을 유지한다. 공개 StemBlock의 새 Paragraph variant에 대한 Rust
exhaustive match 보완은 소비자 호환 안내에 적었다. renderer/pagination/paint 알고리즘은 바꾸지 않았다.

## 검증과 code head CI

| workflow | 실제 실행 | 결과 |
| --- | --- | --- |
| CodeQL | [run 41515945915](https://github.com/edwardkim/rhwp/actions/runs/41515945915) | success |
| Render Diff | [run 41515945245](https://github.com/edwardkim/rhwp/actions/runs/41515945245) | success |
| Adapter inter-diff | [run 41515945154](https://github.com/edwardkim/rhwp/actions/runs/41515945154) | success |
| Proptest roundtrip | [run 41515945234](https://github.com/edwardkim/rhwp/actions/runs/41515945234) | success |
| Skill router gate | [run 41515945294](https://github.com/edwardkim/rhwp/actions/runs/41515945294) | success |
| CI Impact Policy Controller | [run 41515945196](https://github.com/edwardkim/rhwp/actions/runs/41515945196) | success |
| CI | [run 41515945506](https://github.com/edwardkim/rhwp/actions/runs/41515945506) | success |

Main CI는 attempt 2에서 성공했다. 최초 E는 checkout의 git fetch에서 30분 한도를 초과해
Rust 빌드·테스트 시작 전에 취소됐다. 작업지시자가 승인한 E·종속 테스트/집계 1회만 재실행했다.
기존 성공 job의 실행 시각이 유지됨을 API로 확인했다. [고정 code CI 증적](../assets/issue_665_ingest_images/code-ci.json)에 run/attempt/check를 보존했다.

- 로컬 전체 release-test 12,378 PASS / 45 기존 skip, Native 공식 3종 4,064 PASS / lib 기존 ignore 12.
- 새 그림 계약 9개·기존 ingest 14개·HTML 그림 8개, CLI 거부/기존 출력 보존 14개 통과.
- fmt·Native/WASM/workspace Clippy·workspace build·manifest·unit-tier와 최적화 fresh WASM을 완료했다.
- 정식 문서 24개 × 실제 보안 탐지 3종 72회 clean. samples 전용 입력 없는 검사를 이 수치로 세지 않았다.
- 같은 제품·테스트 source의 이미 완료된 로컬 검사를 CI 뒤에 반복하지 않았다.
- [전체 실행 기록](../../tech/investigations/issue-665/rust-validation.json)과 [최종 보고서](../../report/task_m100_665_report.md)에 명령·개수·source 동일성을 연결한다.

## 공통 조판 원칙 준수 검토

| 항목 | 근거와 실제 판정 |
| --- | --- |
| 선행 분석 | 충족. #668 → 실제 선별 반영 80535a3be1 → #666 보정 f708a3cb9f, #667·#3355·#3358을 추적했다. #2724 원 PR #2741 → 통합 #2797 → 독립 삭제 실증을 대조했다. |
| 구현 근거·일반성 | 충족. 명시 placement, mm × 7200 / 25.4, 원본 종횡비와 기존 crop/imgDim(px × 75) 계약이다. 문서 ID·실측 px를 맞추는 분기나 clamp를 추가하지 않았다. |
| 측정·배치 | 충족. 공통 Picture와 실제 UTF-16/8-unit 제어 슬롯을 기존 composer·inline flow가 소비한다. 저장 LineSeg를 임의 생성하지 않았다. 실제 소비 위치는 기술 조사 README에 기록했다. |
| 분할·이어받기 | 비해당. pagination/rowspan/clipping/continuation 알고리즘·컷·예약 높이를 변경하지 않았다. 새 그림 문단의 순서·후속 본문과 소유는 실제 저장/출력으로 확인했다. |
| 회귀 선행 조건 | 충족. 각 관련 문서의 Native/fresh WASM 전쪽 최저 90% 이상과 직접 판독 뒤 새 회귀를 등록했다. 관계·내용 보존 및 독립 쪽수를 검사하며 실제 px 핀은 추가하지 않았다. |
| 기준값·기존 검사 | 충족. 렌더링 golden/baseline은 바꾸지 않았다. 이동한 무효화는 Picture 7 + model 1곳을 계속 검사하고 helper 삭제 음성 대조가 실제 실패했다. |
| 최종 backend 소비 | 충족. Native PNG·fresh WASM Canvas 전쪽 비교와 실제 SVG 소비를 확인했다. 같은 입력의 devel/candidate PNG 30개가 byte-identical이다. |
| rebase 기능 보존 | 비해당. 기준 devel이 유지돼 rebase하지 않았다. 기존 ingest·HTML 및 전체 회귀, 같은 입력/출력으로 기존 동작을 확인했다. |

## 검증 입력·독립 기준·확대 판독

입력은 공개 합성 JSON·패턴 그림이며 기준 HWPX는 새 builder 출력에서 복사하지 않고 base의
텍스트 builder + 기존 편집 API로 별도 작성했다. 한컴 2024 13.0.0.3901의 Print(method 0,
one-up) 결과를 사용했다. [입력 해시](../../tech/investigations/issue-665/inputs.sha256)의 62개 파일은
실제 실행 파일과 code head blob이 같다. 대용량 PDF 예외는 없다.

15개 시나리오의 Native/fresh WASM 30개 전쪽 비교에서 devel 대비 최대 하락은 0.00%p,
2%p 초과·누락·쪽수 차이는 0이다. 각 입력은 독립 Print와 1쪽으로 일치한다. 최저 raw는
shared-boxed의 Native 91.04% / WASM 91.00%다. 나머지 배치·runs·mixed·image-first·저장 왕복·
JPEG/GIF/BMP/TIFF는 양쪽 100.00%다. [전쪽 비교](../../tech/investigations/issue-665/devel-comparison.tsv)와
[입력/PDF/producer/조건](../../tech/investigations/issue-665/devel-comparison.json)을 확인했다.

같은 배율로 그림 색·종횡비·문자 순서·문항 번호·선택지·공유 지문 1회·보기 안팎 소유를 직접
판독했다. shared-boxed의 기존 border/간격 차이는 같은 입력의 base에도 같으며 이미지·본문 소유의
새 결함은 없었다. 이 차이를 숨기거나 px 핀으로 고정하지 않았다. 수식이 없는 합성 입력이므로
편집 가능한 수식·실제 업무 문서 전반의 피델리티를 주장하지 않는다.

대표 [배치 비교](../assets/issue_665_ingest_images/placement-review.png),
[문장 내/보기 확대](../assets/issue_665_ingest_images/detail-review.png),
[추가 형식 비교](../assets/issue_665_ingest_images/codec-review.png)는 기존 기술 증적과 같은 바이트다.
PR 본문과 raw 파일 72개의 바이트 일치, 원본 TSV 60개와 비교표, 실제 PR 이미지 3개의 로드를
확인했다([공개 증적 확인](../assets/issue_665_ingest_images/publication-code-head.json)).

## 이번 trailing 기록과 merge 전 조건

이 기록·오늘할일·동일 PNG의 정식 PR asset 연결·원시 로그 발췌의 구조화 요약만 보완한다.
제품·테스트·fixture·기준 PDF·workflow·측정값을 바꾸지 않는다. 현재 base의 merge-tree와
링크·공백·기존 기록 보존을 push 전에 확인하고, push 뒤 최신 head의 fast-pass와 required check를
확인한다. code head CI 성공을 아직 실행 전인 trailing head의 성공으로 표시하지 않는다.
merge와 #665 종결·후속 branch 삭제는 별도 승인 범위다.

## Merge 후 contributor PR comment 계획

본인 task PR이므로 외부 contributor 감사 댓글은 비해당이다. 후속 댓글이 별도로 승인되면
[Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment),
#665 해결 범위·전쪽 수치·기존 차이·code CI를 적고 대표 PNG를 최종 merge SHA의
`https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/issue_665_ingest_images/placement-review.png`
형태로 표시한다. UTF-8 body-file로 게시하고 API로 확인한다. 현재 댓글·merge는 수행하지 않았다.
