# PR #8572 리뷰 — fix(edit): 칸 글로 높이가 바뀐 글자처럼 취급 표의 저장 줄과 뒤 문단 위치를 한/글처럼 맞춘다

## 최종 판정

**머지 보류 — 셀 첫 줄 간격 보정 제품의 전쪽 Native/fresh WASM·전체 회귀·보호 문서 최종 검증 진행 중.**

원 PR head와 메인터너 보정 제품을 구분한다. 원격 comment·approve·push·merge·close는 실행하지 않았다. 검증 완료 뒤 판정과 증적을 갱신한다.

## 접수 정보

- 원 PR: [PR #8572](https://github.com/edwardkim/rhwp/pull/8572), 작성자 `semanticist21`, reviewer `jangster77`.
- 원 head: `d9750b5604d3f23fd8645ae3f19dfc0cb293a70e`. 2026-11-05 재조회에서 검토 시작 때와 같고 Open·Non-Draft다.
- 고정 base: `170b886f167b705b6e630c5b34bb638d92fae0a9`(PR #8542 포함).
- 현재 리베이스 제품: `71e5e0ae2a72a6b8f51682d131077bd4df44994f`. 리베이스 전 셀 첫 줄 간격 보정 제품은 `2de033b4e42be406a01ea0b9053a4531247e147c`다. 직전 행 조각 보정 제품은 `a314d7a24d3234305abff43149f575853db31105`다. 아래 완료된 R2 측정 제품은 `dc39658f812d22c961ddd0fbc8e71c7ec7a52e5b`이며 새 후보 측정으로 바꾸어 쓰지 않는다.
- 통합 브랜치: `review/integration-20261105-semanticist21-open`.

## 원 주장과 실제 검증 범위

실행 트리거: footnote-01 제목 표의 셀 문단 앞에 표안한글 삽입 후 저장.

독립 기대 동작: 표 위쪽 유지, 아래 표의 동일 간격, 7쪽·문단 소속 및 추가 Enter 재편집 보존.

원본 열기·모델 저장 검사와 실제 편집·저장·한컴 Print 시각 증거를 구분한다. 보충 입력은 원 PR 원본을 검증한 것으로 표현하지 않는다.

## 체리픽과 메인터너 보정

| 원 commit | 170b 리베이스 후 적용 commit |
| --- | --- |
| `9ded487923f1fa8d6e49e2ef1ef85fff4145d90c` | `96500d20a06a3ed82f683fb284e7f05183253cc5` |
| `ee831a3465fd8d58c589031cc084fda95a28be80` | `deb1b7ca097c579cf268923ad981a903bcf609df` |
| `a9cd16ad09ba6a280a1e7a6456342a9741619f15` | `8f05bf6ab5e28ee3b79ead1cdfcbb355fdb5edba` |
| `410c5dba40d0dbdf27a6da6312e5530003a56bb0` | `7bb8fa38baf14b5d7420a1aad925bfd87b2d18dc` |
| `d9750b5604d3f23fd8645ae3f19dfc0cb293a70e` | `beb26383c1e1f7c2f15d28587fdf3d0ce36a3620` |

공통 보정의 원인·소비 경로·리베이스 충돌 해소는 [통합 분석](assets/semanticist21_20261105/integration-plan.md)에 기록했다.

## 검증 입력과 전쪽 결과

아래 Native·fresh WASM은 R2 제품 측정이다. 수용 판정은 최종 보정 후보 검증까지 완료한 뒤 내린다.

| 입력·용도 | 쪽수 | Native 최저 raw·TSV | fresh WASM 최저 raw·TSV | 독립 Print PDF |
| --- | ---: | --- | --- | --- |
| [fix-typed.hwp](assets/tac-host-row-cell-growth/fix-typed.hwp) — 제출된 표 셀 편집 정상 저장본과 후속 문단 배치 | 7 | [100.00%](assets/semanticist21_20261105/validation-r2/native/8572-fix-typed-hwp-2022.tsv) | 측정 진행 중 또는 실행 오류 재측정 필요 | [fix-typed-hwp-2020.pdf](../../pdf/semanticist21-20261105/pr8572/fix-typed-hwp-2020.pdf) |

입력의 실행 경로·정식 보존 경로·SHA-256·크기·역할은 [입력 대응표](assets/semanticist21_20261105/validation-inputs.json)에 있다. 새 정식 보존 파일은 최종 제출 전에 실제 commit 포함과 실행 바이트 동일성을 확인한다.

## 주장에 대응하는 페이지 증적

아래 PNG는 같은 R2 측정에서 나온 원본 PNG를 합성했다. 점수와 직접 확대 판독을 구분하며, 최종 후보에서 달라진 영역은 새 증적으로 교체한다.

- `8572-fix-typed-hwp-2022` 1쪽: [review](assets/semanticist21_20261105/validation-r2/png/8572-fix-typed-hwp-2022/review_001.png) · [overlay](assets/semanticist21_20261105/validation-r2/png/8572-fix-typed-hwp-2022/overlay_001.png).

## 회귀와 조판 원칙 준수

- [tests/cases/tac_host_row_follows_cell_growth.rs](../../tests/cases/tac_host_row_follows_cell_growth.rs)

- R2 제품의 최소 글꼴 집중 검사는 35모듈·169건 PASS이며 새 각주 관계 검사 두 건도 PASS다. 전체 nextest 통과와 구분한다.
- R2 전체 nextest는 행 조각 제품 보정이 필요해 컴파일 중 종료 코드 130으로 중단했으며 최종 후보에서 다시 실행한다.
- 새 base PR #8640의 실제 46상태·124쪽을 포함한 R2 Native 104사례·262쪽은 모두 90% 이상이다. 최저 91.36%, 새 base 대비 2%p 초과 하락 0쪽, 리베이스 전 136쪽 표시값 하락 0쪽이다.
- 현재 진행 중인 fresh WASM·기존 PR #8389/#8397 보호 문서·편람 필수 gate·Rust lint·Native Skia·필요한 브라우저 검증을 완료한 뒤 최종 수용 여부를 갱신한다. 웹폰트 시간 제한 검사는 사용자 지시에 따라 충돌이 없으므로 반복하지 않는다.
- 원본 단위·참조 보존 계약과 실물 배치 좌표 고정을 구분한다. 새 회귀·기존 좌표 핀은 실제 Native/fresh WASM·확대 판독 조건에 맞춰 점검하고 첫머리 근거를 갱신한다.

## Merge 후 처리 계획

통합 PR의 검증·CI·작업지시자 승인 후 원 PR별 해결 범위와 같은 원본·페이지의 증적을 연결한다. 한국어 존댓말의 기존 comment 수정안을 준비하고, 원격 게시·close·merge는 승인 범위에 따라 별도로 수행한다.
