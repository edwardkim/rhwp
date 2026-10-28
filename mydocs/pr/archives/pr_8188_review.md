# PR #8188 리뷰 — sample16 세 문서 전쪽 raw100과 #8231 기능 보존

## 최종 판정

**승인 — 최종 code head의 전체 CI·시각·로컬 검증을 확인하고 원 PR을 머지했다.**
2026-10-26 17:12 KST에 [PR #8188](https://github.com/edwardkim/rhwp/pull/8188)을 일반 merge했다.
최종 head는 `678ba12d1695bddb81becc393f0a396e21285ee3`, merge commit은 `2546c8b2ab7041399fb15fe82e0f50c1ade21724`이다.
사용자의 최신 승인과 옵션 2 지시에 따라 이 검토·오늘할일은 원 코드 PR 이후 별도 문서 PR로 반영한다.
문서 PR 자체의 최신 CI·머지와 issue/comment·정리는 남은 종료 절차다.

## 접수 정보와 경로

| 항목 | 확인 내용 |
| --- | --- |
| PR·작성자·base | [#8188](https://github.com/edwardkim/rhwp/pull/8188), `jangster77`, `devel` |
| 관련 작업 | [#8160](https://github.com/edwardkim/rhwp/issues/8160), 세 목표 문서 각각 64쪽 |
| 제출 code head | `678ba12d1695bddb81becc393f0a396e21285ee3` |
| 최종 검사 원본 / 렌더 producer | `949195dba653cdef8d8fa1506131f5db2f669e78` / `e01e7540fd4c8a67ce5d1d1e5802170973acb998` |
| 기존·리베이스 기준 | `9418ed0c7cf289c5c4ac2b3bb8cdc18a1b637a9a` → `238de6de41a208648f4ade6b494fe433a81c1fce` (#8231) |
| 최신 base 확인 | `4e690c930105fa09fef484787ed0a2e5e182d444`, #8240은 `THIRD_PARTY.md`만 추가 변경; 런타임·입력·PDF 동일 |
| reviewer·상태 | 본인 PR, 별도 reviewer 미지정. 최종 CI·policy 성공 및 MERGEABLE/CLEAN 확인 후 MERGED. 본 문서는 이후 옵션 2 문서 PR로 반영 |
| 기본·보조 경로 | collaborator_self_merge; intake_and_review, local_validation, visual_fixture_evidence, review_only_fast_pass, post_merge |

## 변경과 범위

원본 HFT/Poppy·저장 글줄·구형 수식·탭 점선·팔레트·WMF 인쇄 경로를 공통 규칙으로 보정했다. 2022 BCP의 문구 전용 한 줄 병합 예외를 제거하고 원본 두 줄과 다음 문단 소유를 복원했다. 리베이스에서 새 devel의 탭 원점·글꼴 프로그램 슬롯·장치 클립을 유지하면서 기존 해결 의도를 함께 복원했다.

상위 SVG 그룹 클립이 있는 WMF에서 인쇄 frame 전체가 일반 SVG로 되돌아가는 후퇴를 확인했다. 그룹 클립은 원래 그룹에 보존하고 자식 image/use의 인쇄 자료를 재생하도록 공통 배치 판정을 보정했다. 자체 클립·상위 transform 제한은 유지한다. 문서명·쪽 번호·불일치 좌표에 따른 제품 분기는 없다.

sample16 기존 회귀를 #8160으로 통합하고 잘못 빠진 일반 envelope·K-water 검사와 저장 메타데이터 CLI helper를 복원했다. `2025 행정업무운영 편람(최종).hwp`·`.hwpx`는 사용자 지시로 검토·활성 자동 수집에서 제외하며 원본·PDF는 보존한다. 사양 정오표는 다른 PR 범위다.

## CI 개선 #8267

Archive C가 기존 30분 제한으로 두 번 취소돼 `115f2df43`·`678ba12d1`에서 A(lib)와
B/C/D/E(integration)의 다섯 builder/worker, 검사 수 집계·duration 생산/소비·release promotion
증거를 연결하고 worker 제한을 60분으로 조정했다. 루트 AGENTS.md·CLAUDE.md의 fast-pass는
trigger·다섯 preflight·trusted policy를 함께 맞췄다. 정책 version 8은 이전 version 7의 status를
새 허용 계약의 증거로 사용하지 않는다. 코드 혼합·rename·삭제 거부와 부분 재실행 provenance를
실제 workflow 스크립트로 검증했다. 로컬 전체 nextest 명령은 그대로이며 분할 CLI에는
integration-e를 지원한다. [구현과 검사 기록](../../working/task_m100_8267_ci_archive_e_plan.md)을 따른다.

합쳐진 최종 head에서 Node 643 PASS·Python 146 PASS·전체 actionlint PASS다.
`a01381d0e` 이후 제품 source·Rust 검사·입력/PDF·TSV에 Git 차이가 없고, #8160 Native 전용
검사 호출도 유지된다. 6개 TSV 각 64쪽의 100%를 다시 읽어 확인했다. 이 단계에서 새 렌더링을
산출한 것은 아니며 앞선 동일 런타임·입력·글꼴의 전쪽 검증을 보존한다.

## 검증 입력과 전쪽 결과

[실행 입력과 수용 commit의 바이트 대조](../assets/issue_8160/rebase-8231/inputs-in-commit.json): 원본·독립 Print 75개 고유 경로의 실제 SHA-256과 code head Git blob SHA-256이 모두 같다. 기존 저장소 경로를 재사용하며 새 원본 사본을 추가하지 않았다.

| 목표 문서 | Native | 표준 최적화 fresh WASM |
| --- | --- | --- |
| `hwp3-sample16-hwp5.hwpx` | [64/64쪽 raw100](../assets/issue_8160/rebase-8231/native/hwpx.tsv) | [64/64쪽 raw100](../assets/issue_8160/rebase-8231/wasm/hwpx.tsv) |
| `hwp3-sample16-hwp5-2024.hwp` | [64/64쪽 raw100](../assets/issue_8160/rebase-8231/native/hwp2024.tsv) | [64/64쪽 raw100](../assets/issue_8160/rebase-8231/wasm/hwp2024.tsv) |
| `hwp3-sample16-hwp5-2022.hwp` | [64/64쪽 raw100](../assets/issue_8160/rebase-8231/native/hwp2022.tsv) | [64/64쪽 raw100](../assets/issue_8160/rebase-8231/wasm/hwp2022.tsv) |

[384개 실제 PNG 정수 재검증](../assets/issue_8160/rebase-8231/raw100-proof.json)은 양방향 미대응 0/0이다. 96 DPI Print, threshold 232, 흰색 padding, 대칭 Chebyshev 2px의 raw 실루엣이며 RGB 완전 동일성을 뜻하지 않는다. [표준 빌드](../assets/issue_8160/rebase-8231/wasm/build.json)는 clean source·release·standard·wasm-opt -O 및 실제 패키지 해시를 기록한다. 세 기준 PDF는 새 MCP Print 경로로 재산출한 [engine·job·입력/PDF 해시](../assets/issue_8160/mcp-reference-refresh.json)와 일치한다.

### 새 devel의 기능 보존

#8228에 통합된 #8111·#8141·#8152·#8157·#8159·#8163·#8170·#8176·#8185·#8193의 조판·글꼴·표·원본 프레임 및 Studio/편집·저장 경로를 새 base와 같은 조건에서 대조했다. [범위·입력·명령·전쪽 TSV·실제 소비](../assets/issue_8160/rebase-8231/preservation/README.md)를 연결한다.

| backend | 대조 범위 | 후보 최저 | 최대 하락 | 0.1%p 이상 하락 / 90% 미만 |
| --- | --- | --- | --- | --- |
| Native | 35조건·2,041쪽 | 91.66238% | 0.08523%p | 0쪽 / 0쪽 |
| fresh WASM | 35조건·2,041쪽 | 91.70207% | 0.09506%p | 0쪽 / 0쪽 |

누락·쪽수 불일치가 없다. 전 35조건의 페이지·가용 소유키별 본문 문자 순서가 같다. 실제 소유키가 있는 자료를 대조했으며 그림 내부 문자·IR 전체 동일성을 주장하지 않는다. Studio 소스 계약 base/후보 각각 203 PASS, 글상자 공개 API의 생성·붙이기·제거·HWP 저장/재열기 각각 3 PASS다. Windows 체크아웃 바이트 계약 24건은 Linux Git 설정 대조이며 Windows OS에서의 실제 실행이 아니다.

### 실제 소비·판독과 회귀

[소비·판독 기록](../assets/issue_8160/rebase-8231/consumer/README.md)은 Native·WASM·Print 주요 8쪽 및 backend별 보존 주요 17쪽을 직접 대조한다. BCP 2022 두 줄·2024 한 줄과 분수·괄호·본문 간격을 확대 확인했다. Canvas 브라우저 계약 8건, CanvasKit crop 네 모드, 실제 WMF 두 그림의 네 영역 정수 0/0, Studio 후보 5종·base 3종을 통과했다. 실제 초기화 응답 WASM SHA와 readiness를 확인했다. CanvasKit의 해당 판정은 software backend의 두 그림 범위다.

전쪽 raw100 확보 후 최종 WMF 상위 클립 검사 본문을 등록했다. 같은 본문 SHA로 보정 전 의도한 단언 FAIL(exit 101), 보정 후 PASS(exit 0)를 확인했다. #8160은 41개 모듈·87개 검사(기본 86개·Native 전용 1개)이며 페이지별 중요 경계의 관계·독립 기대 px를 함께 보호한다.

## 조판 원칙 준수 검토

| 검토 항목 | 판정 | 실제 근거 |
| --- | --- | --- |
| 제출 전 로컬 검증·Open PR | 충족 | [14개 필수 단계](../assets/issue_8160/rebase-8231/validation.json) 모두 exit 0, Open 본인 PR |
| 원 이슈·해결 PR 선행 분석 | 충족 | [10단계 기록](../../working/task_m100_8160_stage10.md), 원 WMF 인쇄 해결 `4f9ab2b46`와 새 #8228 장치 클립, `e01e7540f` 보정·실패 경계 추적 |
| 독립 근거·공통 구현 | 충족 | 원본 줄·글꼴/WMF 계약·한컴 Print, 공통 paint 재생; 문구·파일명 분기 없음 |
| 측정·배치 일관성 | 충족 | 공통 글꼴 프로그램·저장 줄·PrintRasterFrame 자료를 Native/Canvas/CanvasKit에서 실제 소비 |
| 리베이스 기능 보존 | 충족 | 위 35조건·2,041쪽씩, 사용자 기준 0.1%p 적용, 비시각 기능 대조 |
| 출력 backend 소비 | 충족 | source·패키지·실제 초기화 SHA, Native/Canvas/지원 CanvasKit 실제 실행; 범위를 확대하지 않음 |
| 분할·이어받기 | 충족 | 저장 줄 경계와 이어받는 표/후속 내용 검사, 전쪽 본문 소유 대조; page count 단독 검사 아님 |
| 줄 소속·점유 | 충족 | BCP 두 원본 줄과 successor 소유, 탭 공통 폭과 새 devel 원점 동시 보존 |
| 독립성·음성 대조 | 충족 | 새 한컴 Print·원본과 합성 WMF 분리, 같은 최종 테스트 보정 전 FAIL/후 PASS |
| 기대값·baseline 변경 | 충족 | 잘못된 BCP 한 줄 핀을 원본 두 줄·내용 관계로 정정; 원본·문턱·공차 완화 없음 |
| 회귀 선행 조건·px | 충족 | 전쪽 raw100 후 신규 경계 등록; 완료 페이지 양 backend ≥98%·직접 판독·±2px·소유 관계 |
| 수식·미세 간격 확대 판독 | 충족 | 분수·괄호·주변 본문, WMF 글자·선·JPEG 격자·표제 확대 기록 |
| 입력 commit 확인 | 충족 | 75개 실제 경로/바이트와 검토 Git blob 해시 대조 |
| 주장·검증 범위 | 충족 | raw100/RGB 구분, 필터 제외·CanvasKit 두 그림·Linux 체크아웃 대조 범위 명시 |

## 로컬 검증

[최종 결과](../assets/issue_8160/rebase-8231/validation.json): 전체 nextest 11,573 PASS·45 skip; Native Skia 라이브러리 4,040 PASS·12 ignore; Native 공식 누락 그림 2건·직접 PDF 14건·#8160 문단/이중 테두리 3건 모두 PASS. #8160 Native suite 3건 중 고유 Native 전용 검사는 1건이다. Studio 단위 2,572 PASS·2 skip, TypeScript PASS, Node 등록/잠금 25 PASS, Python 시각 검증 191 PASS다. 제외·미선택 건은 통과 수에 합산하지 않는다.

## GitHub Actions와 옵션 2 문서 PR

정확한 code head `678ba12d1695bddb81becc393f0a396e21285ee3`의
[CI](https://github.com/edwardkim/rhwp/actions/runs/40286124631),
[CodeQL](https://github.com/edwardkim/rhwp/actions/runs/40286124596),
[Render Diff](https://github.com/edwardkim/rhwp/actions/runs/40286124635), Adapter·Proptest·Gym·Skill router가
모두 성공했다. Native job에서 #8160 BCP 두 줄·표 이어받기·이중 테두리 세 검사의 실제 PASS를 확인했다.

| 기본 Rust shard | 실행 / 통과 |
| --- | --- |
| A | 3,801 / 3,801 |
| B | 2,397 / 2,397 |
| C | 2,199 / 2,199 |
| D | 460 / 460 |
| E | 2,522 / 2,522 |
| 합계 | 11,379 / 11,379 |

최종 집계의 `archive_total=11379 shard_total=11379`와 각 archive별 대조가 모두 통과했다.
이 수는 root default archive이며 로컬 workspace 전체 11,573건과 구분한다.
CI Impact Policy의 최종 success와 정확한 head/base를 재확인했다.
`git merge-tree --write-tree`의 예상 tree는 `2f581aad87798cfdce57ea824c34174f05ad31f4`이고,
실제 merge `2546c8b2ab7041399fb15fe82e0f50c1ade21724`의 tree와 일치한다. 옵션 2 문서 PR의 fast-pass는 별도로 확인한다.

## Merge 후 contributor PR comment 계획

사용자 승인 범위에서 정확한 최종 head를 지정해 일반 merge했고 `2546c8b2ab7041399fb15fe82e0f50c1ade21724`의 upstream/devel 포함을 확인했다. 병합 후 duration metadata run 40290628059는 CLI의 구 B/C/D 전용 최종 검사에서 E 입력을 거부해 실패했다. 이 누락은 사용자 지시로 별도 [#8272](https://github.com/edwardkim/rhwp/issues/8272)에 등록하고 [보정 PR #8273](https://github.com/edwardkim/rhwp/pull/8273)에서 실제 네 report CLI 검사와 함께 보정한다. 보정 PR은 `61ec35daa`로 머지됐고 [갱신 run 40298571437](https://github.com/edwardkim/rhwp/actions/runs/40298571437)에서 실제 정책 발행을 확인했다. [보정 검토](pr_8273_review.md)로 원 코드 CI 성공과 metadata 오류·복구를 구분한다. [Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment), 실제 3×64쪽×2 backend의 raw100·0/0, 0.1%p 기준, CI URL과 남은 RGB 차이를 한국어 존댓말 comment로 남긴다.

본문·후속 comment는 `mydocs/pr/assets/issue_8160/rebase-8231/publication/{native,wasm}/review_018.png` 및 `overlay_018.png`를 사용한다. PR 본문은 최종 code SHA, 병합 후 comment는 실제 merge SHA의 raw image URL로 고정한다. API로 본문·한글·SHA·이미지 링크를 다시 확인하고 같은 증거의 중복 comment는 게시하지 않는다.

#8160 close와 중복 comment 유무를 확인한다. 원본·PDF·최종 증거를 영구 경로로 보존한 뒤 이번 작업 전용 branch/worktree·저장소 output을 정리한다. 공유 `target/pr-review`와 다른 작업 산출물은 보존한다. review·오늘할일은 사용자 지정 옵션 2의 별도 문서 PR로 반영하며 원 코드 PR에 trailing commit을 추가하지 않는다. 상세 단계는 [구현/후속 계획](pr_8188_review_impl.md)과 현재 canonical post_merge 절차를 따른다.
