---
kind: review
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-13
---

# PR #7877 self-review — 표 분할·저장 줄 흐름 보정과 #7704 통합

## 최종 판정

**승인 — 검증한 code head의 조판 보정·필수 로컬 게이트·GitHub Full CI 완료.**

작성자 메인테이너 self-review다. 검토 code head는 `152c2fcbcccd08646fa98273089833ab9237e395`, base는 `552f5228d2dbfd6046c838f14241704f1bcb8a05`다. 작업지시자는 devel 동기화와 이 PR 병합을 승인했다. 현재 기록 commit의 최신 CI·mergeability·merge-tree 확인과 실제 merge는 후속 gate이며 이전 code head 성공으로 대신하지 않는다.

## 접수와 범위

원 [PR #7704](https://github.com/edwardkim/rhwp/pull/7704), @oksure의 `896e64ff6d876e676d6ebf987d6669b47653912a`를 원 커밋 그대로 포함한다. 표 조각마다 반복하던 두 검사의 단일 순회 개선을 보존하고, 사용자 승인으로 HWP/HWPX 표 분할·원점·빈 문단·Square 배치와 기존 시각 미달을 보정했다. [원 검토 이력](pr_7704_review.md)에 수정 전 FAIL·철회 후보·재보정·최종 PASS가 있다. 관련 [#7639](https://github.com/edwardkim/rhwp/issues/7639)는 부분 성능 개선이며 전체 종료 대상으로 삼지 않는다.

## 조판 원칙과 실행 경로

| 검토 항목 | 근거·경계·검사 | 판정 |
| --- | --- | --- |
| 독립 근거와 일반성 | 정상 원본/한컴 Print, 수동 변형은 별도 분류. 문서 ID·화면 px 전용 분기 없이 실제 저장 줄·여백·물리 소유를 판정 | 충족 |
| 측정·배치 일관성 | `host_spacing` → TAC/블록 준비 → scan/budget → emit/layout이 동일 프레임·줄/개체 점유를 소비. 최종 source closing query 뒤 residual margin 결정 | 충족 |
| 최종 출력 소비 | Native/fresh WASM SVG 전체 비교, 실제 새 Docker WASM Studio 표시·해시와 사용자 시각 판정. Native Skia lib/PNG placeholder/direct PDF 별도 검사. 모든 backend paint의 100% parity를 주장하지 않음 | 충족 |
| 분할·이어받기 | 내용 컷과 빈 물리 밴드 구분. `FragmentInput::closes_terminal_content_cut`을 scan/emit이 함께 소비. 마지막 물리 여백·캡션은 조각 예약, 후속 flow margin은 한번 전진. #7336·#7470·#7518·#5543 반례와 전체 회귀 | 충족 |
| 줄 소속·점유 | 저장 TAC outgoing spacing과 다음 fresh before/top 구분. 새 단에서 이전 before 중복 제거, 실제 빈 Enter 보존. `stored_empty_square_successor_strip` → 확정 `ParagraphFloatPlacement` → post_flow/layout 공통 strip. fit은 content_height, 전진은 outgoing spacing 포함 advance | 충족 |
| 증거 독립성 | 한컴2020/2024 독립 Print와 전쪽 대조. 수동 #6761 HWPX는 합성 계약이며 정상 저장 정보 일반 근거로 승격하지 않음 | 충족 |
| baseline/golden/허용치 | 변경·완화·검사 이관 없음 | 비해당 |
| 회귀 검사 변경 선행 조건 | 출력 보정 후 전쪽 Native/fresh WASM ≥90% 확인. #6761은 행/셀/문단 소속·포함·완전 보존, #7051은 정상 원본과 의도적 child 이탈 검출. #6778은 set_document로 실제 재구성 | 충족 |
| 확대 판독 | #6761 later40·41쪽, #6778 1·4·6·7쪽, rowbreak HWP5쪽, #5543 2·4쪽. 셀 정렬·외곽선·후속 문단·그림/제목 간격·빈 문단 이월을 같은 배율로 직접 대조. 남은 글자 모양/폭·미세선 차이는 기록 | 충족 |
| 주장과 실행 범위 | 모든 formal 회귀·lint·Skia 완료, 19문서835쪽 Native/fresh WASM. #5543의2024출력은 사용자 승인으로 미검증; 한컴2020의4쪽을 정답지로 사용 | 충족 |

저장 원점은 실제 column source origin으로 무효화하고, 현재 carried frame 이전의 plain near-top reset을 새 페이지 경계로 잘못 사용하지 않는다. inline shape/명시적 쪽·단/0원점은 별도 계약을 유지한다. terminal commit은 유효한 source successor를 보존하고 소비 시 physical bottom을 확인한다. 완전한 직접 텍스트 suffix의 Center/Bottom은 실제 남은 행 상자가 소유하며 opaque 중첩 개체의 프레임에 이 규칙을 적용하지 않는다.

## 검증 입력 commit 확인

18개 문서의 실제 HWP/HWPX/PDF 36개 항목은 [정식 fixture manifest](../../../tests/fixtures/pr7704_manual_reflow/MANIFEST.json)의 상대 경로·SHA-256으로 검토 code commit 바이트와 대조했다. 새 수동 변형의 생성 방식과 독립 기대값은 [fixture README](../../../tests/fixtures/pr7704_manual_reflow/README.md)에 구분했다.

추가 #5543 입력/PDF도 같은 code commit에 포함되고 실행 파일과 SHA-256이 같다:

- `samples/issue5543_carried_anchor_ladder.hwpx`: `6adca8d39bdcf51e8791d8afa1e584b9570eb8b15df901e74ce61cfd500cdb4a`

- `pdf/issue5543_carried_anchor_ladder-2020.pdf`: `ae12f3f6ebdfe4b443001fdbb30901dc7b7c04d29768a60dc399eff730f2169d`

검증 실행 당시 `e4a7964d` 작업본의 runtime.patch를 임시 Git index에 적용해 복원한 source blob이 검토 code head와 같음을 확인했다. 커밋 뒤 Git binary diff의 index metadata 변화는 실행 코드 변화가 아니다. 입력/대표 증거를 커밋하면서 실행 코드와 formal 검사는 변경하지 않았다.

## 로컬 검증과 시각 증거

- 전체 nextest: 10,219 PASS / 0 FAIL / 45 skip.
- fmt, 기본·WASM32·workspace all-targets Clippy, workspace build, base 고정 manifest/unit-tier 모두 exit0. 공용 target/pr-review 사용.
- Native Skia lib4,046 PASS/12ignore, PNG placeholder2 PASS, direct PDF4 PASS.
- Native/fresh Docker WASM: 각각19문서835쪽, 최저90.47526%, 미달·누락·쪽수 차이0. 전체 PNG835쪽이같다. TSV 성공을 단독 시각 판정으로 삼지 않았다.
- [전체 TSV 원본38개](https://gist.github.com/edwardkim/de059a1ae29fee45bdc580611a7f6d50): 처음18문서831쪽 및 최종 #5543의4쪽. 기본 source/입력/PDF/profile/compat/DPI를 동일하게 적용했다.
- 명령: `visual_sweep.py --silhouette-only --hwp <input> --pdf <Print PDF> --rhwp-bin <candidate> --compat <2022|2024> --dpi 96`; WASM는 `--wasm-pkg <fresh Docker package>` 추가. 각 문서 전체 페이지를 비교했다.
- Docker WASM SHA-256 `7a12583cc041376ee2dcf8986dab451059e52a3669c5a47df4eb669233cccc32`, Native CLI `988c98d4f92a3f6adf6a45820ceb180e22760f60c852e6b3cdb22ef0a81030a0`. 실제 Studio 초기화·public 파일 해시를 확인했다.
- 대표 안정 asset: `mydocs/pr/assets/issue7639-maintainer/`의 rowbreak-hwp p5, 6761-later p40, 6778-square p4 Native/WASM review·overlay 12개. GitHub 본문에서 실제 로드 확인. 원시 PNG/JSON/로그·TSV는 ignored output에 유지하고 커밋하지 않았다.

## 정확한 code head GitHub CI

- [CI / 38376731402](https://github.com/edwardkim/rhwp/actions/runs/38376731402): `success`, head `152c2fcbcccd08646fa98273089833ab9237e395`.
- [Adapter inter-diff / 38376731281](https://github.com/edwardkim/rhwp/actions/runs/38376731281): `success`, head `152c2fcbcccd08646fa98273089833ab9237e395`.
- [Proptest roundtrip / 38376730968](https://github.com/edwardkim/rhwp/actions/runs/38376730968): `success`, head `152c2fcbcccd08646fa98273089833ab9237e395`.
- [CI Impact Policy Controller / 38376730795](https://github.com/edwardkim/rhwp/actions/runs/38376730795): `success`, head `152c2fcbcccd08646fa98273089833ab9237e395`.
- [Render Diff / 38376730567](https://github.com/edwardkim/rhwp/actions/runs/38376730567): `success`, head `152c2fcbcccd08646fa98273089833ab9237e395`.
- [CodeQL / 38376731166](https://github.com/edwardkim/rhwp/actions/runs/38376731166): `success`, head `152c2fcbcccd08646fa98273089833ab9237e395`.

CI의 Build & Test, 세 Clippy 포함 Lint, Native Skia, Frontend package gates, archive A/B/C/D 빌드와 회귀 worker가 실제 성공했다. fast-pass 생략으로 기록하지 않는다.

## Merge 후 contributor PR comment 계획

원 #7704에 기여자의 두 검사 순회 개선을 원 커밋 그대로 통합했다는 사실과 메인테이너 추가 보정의 범위를 한국어 존댓말로 게시한다. 통합 #7877 merge SHA·CI URL·실제 위 수치·남은 글꼴/미세선 및2024미검증을 구분한다. #7639는 후속 성능 범위를 위해 OPEN을 유지한다. 원 PR이 자동 MERGED되면 그 상태를 확인하고, OPEN이면 대체 설명 후 close한다.

[Visual Sweep merge comment 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을 연결하고 위12개 asset을 실제 merge SHA 고정 raw URL의 Markdown 이미지로 포함한다. 게시 본문은 UTF-8 파일로 보내며 API로 한글·BOM·head/merge SHA·asset을 재조회한다. devel 동기화와 duration 갱신 상태를 확인하고 전용 clean worktree/branch 정리 또는 활성 작업 유지 사유를 기록한다.
