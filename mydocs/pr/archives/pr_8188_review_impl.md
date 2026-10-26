# PR #8188 검토·병합·후속 처리 계획

Issue: [#8160](https://github.com/edwardkim/rhwp/issues/8160), [#8267](https://github.com/edwardkim/rhwp/issues/8267)

## 고정한 작업과 승인

- 작업 브랜치: `task/8160-sample16-raw100`.
- 제출 head: `678ba12d1695bddb81becc393f0a396e21285ee3`; 검사 원본 `949195dba653cdef8d8fa1506131f5db2f669e78`; 렌더 producer `e01e7540fd4c8a67ce5d1d1e5802170973acb998`.
- 사용자 승인: 2026-10-26 원격 push, 이어 CI 완료 후 후속 처리 전체. 추가로 review·오늘할일은 옵션 2의 별도 문서 PR로 처리하도록 지정했다. 최신 승인은 이전 머지 승인 대기를 해제한다.
- 사양 정오표와 편람 HWP/HWPX 두 파일의 검토는 범위 밖이다. 원본·기준 PDF와 공유 target을 보존한다.

## 보정 이력과 실제 계약

| 커밋 | 변경과 검증 |
| --- | --- |
| `74c5a813c` | 새 devel의 탭·글꼴·클립·이전 여백 경로를 기존 해결과 통합하고 generic envelope 검사를 복원 |
| `952cd8ee2` | K-water 정상 검사 본문을 자동 수집되는 case 경로로 이동 |
| `e01e7540f` | WMF 원래 그룹 클립은 남기고 공유 인쇄 image frame 재생을 유지; 문서 전용 분기 없음 |
| `5d8188dea` | 저장 메타데이터 정상 검사의 runtime env 우선 CLI helper 복원 |
| `f017a4df4` | 세 목표 파일의 Native 192쪽 raw100 증거 |
| `2ac3a98f1` | 새 base/후보 Native 35조건·2,041쪽 및 실제 비시각 기능 보존 증거 |
| `949195dba` | Native/fresh WASM 전체 384개 raw100 뒤 WMF 상위 클립 경계 등록 |
| `2d3ec7bbe` | 실제 브라우저·CanvasKit 소비와 같은 최종 WMF 검사 본문의 수정 전 FAIL/후 PASS 고정 |
| `11aff27a2` | 새 base/후보 fresh WASM 35조건·2,041쪽, 0.1%p 하락 기준 통과 |
| `076b19d36` | 최종 14개 로컬 gate·전체 nextest·Native Skia 필수 검사 완료 고정 |
| `a01381d0e` | Native 전용 #8160 검사 누락을 기존 Native job·분류기 소유 목록에 연결 |
| `115f2df43` | #8267 Archive E builder/worker·집계·duration 연결과 worker 60분 제한 |
| `678ba12d1` | root 지침 fast-pass의 trigger·preflight·trusted 정책, 합쳐진 head Node 643·Python 146·actionlint 통과 |

기존 상세 구현과 원 해결 추적은 [10단계 기록](../../working/task_m100_8160_stage10.md)을 따른다. `src/paint/metafile_print/images.rs`의 배치 판정과 원 SVG 그룹 소유, `src/renderer/{skia,web_canvas}/metafile_print.rs`의 실제 소비를 함께 확인했다. HFT/글꼴 프로그램·저장 줄·탭의 측정과 배치는 공통 자료를 사용하며 Native/Canvas/CanvasKit을 서로의 대체 증거로 쓰지 않는다.

## 실행 stage — 사용자 지정 옵션 2

1. 완료: 최종 head `678ba12d1695bddb81becc393f0a396e21285ee3`의 전체 Actions·CI Impact Policy와 실제 Native 전용 assertion PASS를 확인했다. 기본 archive 11,379개 실행/통과가 예상 수와 일치했다.
2. 완료: 해당 exact head를 지정해 원 PR #8188을 `2546c8b2ab7041399fb15fe82e0f50c1ade21724`로 일반 merge했다. 사용자가 옵션 2를 지정했으므로 원 PR에 review·오늘할일 trailing commit을 추가하지 않는다. admin 우회는 사용하지 않는다.
3. merge SHA의 upstream/devel 포함과 duration 갱신 결과를 확인하고 최신 devel에서 `task/8160-sample16-review-records`를 만든다. 원 PR review·이 계획·20261026 오늘할일·최종 CI/병합 기록만 별도 문서 PR로 반영한다.
4. 문서 PR은 실제 번호가 발급된 뒤 기록한다. source·test·workflow·기존 baseline을 섞지 않고, 정확한 base/head의 merge tree·공백·Markdown 링크·오늘할일 양쪽 보존과 review-only preflight·최신 required CI를 확인해 merge한다. 문서 기록 PR을 위한 또 다른 기록 PR은 만들지 않는다.
5. 최종 문서가 devel에 존재하고 local devel을 fast-forward한 뒤 #8160·#8267 close·중복 comment 유무를 확인한다. 원 코드 PR·issue에 실제 merge SHA·CI·raw100 및 범위의 한국어 후속 기록을 게시하고 API로 다시 확인한다.
6. 영구 보존과 사용 종료 확인 후 전용 worktree·브랜치·저장소 루트 output을 정리한다. 정리 전후 경로·용량·해시는 로컬 영구 기록에 남긴다.

## 보존과 안전한 정리

- 정식 TSV·대표 PNG·빌드/입력/정수 비교 증거는 이 PR의 `mydocs/pr/assets/issue_8160/rebase-8231/`에 포함한다.
- 전체 목표 PNG와 Print raster는 로컬 `pdf/issue8160-rebase8231-artifacts/`에 보존하고 원래 proof 해시와 재대조한다. 이 폴더는 로컬 전용으로 정확한 경로를 exclude하며 새 원본·PDF를 숨기는 용도로 쓰지 않는다.
- 원본 부가 실행·빌드·음성 대조·CI 로그와 relocation/cleanup receipt는 `/home/tsjang/rhwp-pr-evidence/pr-8188/`에 보존한다. 재현에 필요한 글꼴 프로그램은 Git에 넣지 않는다.
- `rhwp-8160-validation`과 `rhwp-8160-wasm-candidate6/12/29`는 사용 종료·소유·변경 내용을 확인해 제거한다. generated public JS만 dirty이면 사용한 패키지 해시를 먼저 보존하고 이번 빌드가 만든 변경만 정리한다.
- 원격 임시 head는 MERGED·최종 SHA·devel 포함·comment/duration 완료·다른 PR/활성 작업 미사용을 확인한 뒤 exact lease로 삭제한다. 다른 작업의 `rhwp-7105-verify`와 공유 `target/pr-review`는 삭제하지 않는다.
- 최종 결과는 병합·원격 CI·duration·issue/comment·devel sync·각 정리의 실제 완료 상태를 분리한다. 보존·정리 gate가 미완료이면 후속 처리 완료로 쓰지 않는다.

## 병합 후 확인한 상태

- 원 PR은 MERGED이며 최신 devel에서 코드와 기준 자료를 확인했다.
- 문서 PR 브랜치는 `task/8160-sample16-review-records`로, 원 코드 merge commit에서 생성했다.
- 새 원본·새 baseline·workflow를 문서 PR에 포함하지 않는다. 입력 해시 목록은 이미 수용한
  75개 source/PDF와 code/merge Git blob을 대조한 영구 검증 색인이다. 임시 실행 로그·생성기는 제외한다.
- 코드 PR 뒤 기본 검증 CI는 재실행하지 않는다. 자동 metadata refresh와 문서 PR의 fast-pass만 확인한다.

- 실제 metadata refresh 40290628059에서 CLI의 B/C/D 전용 최종 검사가 남아 E를 거부했다. [#8272](https://github.com/edwardkim/rhwp/issues/8272)·[PR #8273](https://github.com/edwardkim/rhwp/pull/8273)의 별도 CI 보정에서 실제 CLI 네 입력을 보호하고 갱신 성공을 확인한다. 문서 PR에는 이 코드 보정을 섞지 않는다.

- CLI 보정 PR #8273은 `61ec35daa`로 머지됐고 run 40298571437에서 B/C/D/E의 46 target·1,730 case·7,578 testcase를 실제 정책 브랜치에 발행했다. [보정 검토](pr_8273_review.md)에 정확한 CI·발행 commit을 연결한다.
