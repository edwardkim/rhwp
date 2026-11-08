---
kind: report
status: archived
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-11-03
---

# PR #8187 검토 — 논리 편집·범위 재조판과 다단 경계

## 최종 판정

**승인 — 메인터너 보정 head의 CI·self-review 성공 후 원 PR merge 완료.**
원 기여 13커밋을 보존해 11개 충돌을 해결했고 논리 편집·다단 기능과 KPS 65·66쪽을 보정했다.
전체 회귀 **12,415 PASS**, Native Skia 공식 3종 **4,064 PASS**, 집중 회귀 **72 PASS**와 모든
Rust lint를 확인했다. 독립 Print 대비 Native/fresh WASM 각각 **14문서·516쪽**이 전쪽 raw 90% 이상이며
같은 원본의 기준 devel 대비 하락은 0쪽이다. 실제 브라우저 API·Studio·TypeScript도 통과했다.

2026-11-03 작업지시자는 [통합 계획](pr_8187_review_impl.md)의 **메인터너 보정과 필수 로컬 검증**을
승인했고, 로컬 완료 뒤 **원 기여 branch push → 본문 보충 → 새 head CI·self-review → 원 PR merge →
감사 댓글·기록·정리**도 승인했다. [원격 처리안](../assets/pr8187/remote-action-plan.md)에 따라 진행한다.
원 contributor branch에 후보를 push하고 본문을 보충했다. 최신 head의 CI와 충돌 상태를 따로 확인하며, 실패한 검사를 우회하지 않는다.

## source와 접수 정보

| 항목 | 고정값 |
| --- | --- |
| PR / 작성자 / base | [#8187](https://github.com/edwardkim/rhwp/pull/8187) / semanticist21 / devel |
| 원 head | `32df7b71a6e731f767e4945e3e1d79ca0cac3e58` |
| 작성자 제품 source | `d166697310779a8cc02a3091b6e1481d0d4ab182` |
| 작성자 base / 공통 조상 | `60972534680a0c5e51e54ec189b92e252e81a867` |
| 이번 기준 devel | `4e2cc6bb36576b95d58d4c76cd833281ea652554` |
| 로컬 branch / 위치 | `review/pr-8187-logical-editing` / `/home/edward/mygithub/rhwp` |
| 원 head 보존 ref | `refs/remotes/upstream/pr8187-head` |
| 통합 commit | `29b0bb11849edf92d7e2fe9592c51788dc54675d` — 원 기여 13커밋을 두 번째 parent로 보존 |
| 표 프레임 보정 | `c33037bf2ff187ab1d80dd6b73537f55b61335d8` |
| Native 기준선 보존 | `054ef1cd2dcd6060e1cf4a98ea9c65a72fb40ee1` |
| 접수 상태 / reviewer | OPEN, non-draft, CONFLICTING / edwardkim requested |
| 원 PR 규모 | 128파일 +5,650/−235줄, source·test 52파일 +4,063/−235줄 |
| 선택 절차 | maintainer_general + intake_and_review, local_validation, visual_fixture_evidence, rework_and_exceptions, post_merge |

접수·계획 commit은 `ea0346b14520a4e6d1cd5dd5450ae9d5b6cd8862`다.
접수 당시 `merge-tree`의 11개 충돌, 원 head 파일 지문과 원 CI의 source 일치는
[접수 증거](../assets/pr8187/intake-evidence.json)에 남겼다. 원 head의
[CI 40092729836](https://github.com/edwardkim/rhwp/actions/runs/40092729836)는 성공했으나 이번 보정 후보의 CI가 아니다.
다른 작업 워크트리와 기본 pkg/Studio 배포 파일은 보존했다.

## 최신 devel에서 해결됐는지와 선행 해결 이력

최신 source에는 `deleteRangeLogical`, `clipboardIsSingleText`, `pasteInternalTextLogical`,
`pasteHtmlTextLogical`, 범위 재조판·구역 갱신 API와 분할 셀의 `model_para_range`가 없었다.
따라서 수용할 기능이 남아 있다. 기존 API 전체가 동작하지 않는다는 뜻은 아니다.
KPS 66쪽의 빠진 테두리와 행 위치 차이도 같은 입력·글꼴의 기준 Native/WASM에 남아 있었다.

| 선행 해결 | 통합에서 보존한 계약과 재검증 |
| --- | --- |
| #7444 → #7509 → merge `fc36d71452a1aedae5f1f2b05737efe60a00f37f` → [독립 검토](pr_7509_review.md) | 개체 뒤 삽입·논리 복사 유지. 논리 삭제·서식 글 붙여넣기를 추가하며 필드·개체 소속/거부 원자성·왕복 검사 |
| #4698 → #4707 → 선택 보정 `c7cfaefb9` → #4712 merge `3c7b89356eef4f69cc2101d8e07507ba2ecf2425` | 분할 셀의 원본 문단 소유 유지. 기존 #4698/#1073 및 KPS의 실제 문단 내용·Layer JSON 검사 |
| `27f55e5b399e26820f8cece6ae8d7e185c750f84` → #8319 merge `af6bb05b26887135fb9da0fa4e54ca7205564fca` | Square의 글줄 컷은 열린 경계를 유지하고 온전한 자식 행 경계만 프레임을 닫음. #6549/#6756 실제 출력 대조 |
| `ca5ed2a48e8f3213df5f914b2c49842676eb7452` | 실제 선택 face 기준선 유지. 최종 Skia 두 파일은 기준 devel과 동일하며 KPS 53쪽 PNG도 바이트 동일 |
| `f9acf61ec9` / #8149 | 일반 첫 줄 fit의 문단 앞 간격 차감 유지. Normal 부분 쪽 단 전환과 기존 fit 회귀 검사 |
| `c417310cab6` / #8389 | 붙여넣기의 NBSP 원본 메타데이터 유지. 기존 #8153과 논리/서식 글 붙여넣기 검사 |
| #8174의 공통 Storage 경로 | 인라인 그림 행·단 너비 저장도 예산/취소와 실패 전달을 소비. 기존 공개 Storage 경계 검사 |

이 PR은 #7444의 잔여 API 일부를 보완한다. Studio의 모든 선택·음영·서식 연결을 해결했다고
주장하지 않으며 #7444 전체를 자동으로 닫지 않는다.

## 보정 내용과 실제 소비 경로

| 경계 | 생산·측정·배치·저장의 연결과 반례 |
| --- | --- |
| 논리 삭제·붙여넣기 | 공통 clipboard/text_editing 경로와 `Paragraph::inherit_inserted_text_controls`가 원본 문단 메타데이터·필드 소속을 함께 소비한다. UTF-16 보충 문자·각주 슬롯·개체 앞뒤·같은 위치의 여러 개체·거부 시 불변·snapshot/왕복을 실행했다 |
| 재조판·다단 | 범위 밖 저장 줄을 보존하고 batch 중 쪽 갱신을 미룬다. Normal 단 전환은 실제 내용 하단과 뒤 간격을 한 번 계상하고 새 빈 단에서 첫 줄 예산을 검사한다. Distribute/Parallel은 기존 경로로 처리한다 |
| 인라인 그림 행 | `composer/inline_picture_metrics.rs::refresh_picture_rows_with_storage` → composition의 실제 행 갱신. Storage의 collect·실패 전달을 유지한다 |
| 단 너비 저장 | `serializer/column_sizes.rs::for_export_with_storage` → `body_text.rs`와 `hwpx/section.rs`의 공통 writer. 본문 폭 전체를 차지하는 유효 절대 너비만 32768 비례값으로 변환하며 잘못된 용지·부분 폭은 원본을 유지한다. 사본도 Storage를 통해 만든다 |
| 셀 내용/괘선 소유 | `table_layout.rs`의 row filter가 내용과 괘선을 같은 선택 행 범위에 투영한다. 잘린 변에는 표의 선언 프레임을 적용하며 원래 셀 내부선·반복 제목행 스타일은 유지한다 |
| 부모 컷과 물리 프레임 | `CellUnit::starts_nested_row` → `table_partial.rs::nested_row_cut_frame_columns` → Square 상·하단 괘선의 실제 소비. 자식의 온전한 행 시작과 글줄 중간 컷을 구별하고 두 방향에서 같은 결과를 쓴다 |
| 첫 조각/이어받기 높이 | HWP5 저장 줄의 쪽 되감김·RowBreak·원본 유효성을 확인한 `table_row_frame::project_first_rowspan_frame` → `height_measurer.rs`의 행 원장 → `table_layout.rs` fallback/실제 셀 배치. HWPX의 문단 지역 0 원점·편집본은 해당 증거로 사용하지 않는다 |
| 예산/내용 하한 | 닫는 행의 저장 여유만 첫 프레임으로 옮기고, 그 경계를 지나는 rowspan의 가장 이른 종료 행에서 물리 높이를 한 번 이어받는다. 단일 셀/같은 경계에서 끝나는 병합 셀의 실제 내용 하한을 먼저 검사한다. 여러 걸침의 총높이와 다음 본문을 유지한다 |

KPS 안쪽 표의 첫 프레임 원본은 63299 HWPUNIT = 843.98667px(96dpi)이다. 종전 첫 조각 행 합은
63510 HWPUNIT으로 211 HWPUNIT를 더 점유했다. 공통 행 원장에서 이 저장 여유를 이어받게 해
65쪽 끝과 66쪽 내부 행·날짜·서명 위치를 복원했다. 문서 ID나 211이라는 상수로 분기하지 않는다.
[실행 증거](../assets/pr8187/boundary-execution.json)는 같은 실제 base/head 출력의 프레임·다음 내용 관계를
검사한다. base의 양 backend에서 실패하고 후보의 양 backend에서 통과했다.

통합 중 원 PR의 자동 적용 부분이 Native `baseline_offset`을 canvas translate와 draw origin에
중복 적용해 KPS 53쪽 결합 문자가 위로 잘렸다. raw가 98.84%→97.95%로 여전히 90%를 넘었지만
직접 판독으로 수용하지 않았다. Native Skia 두 파일을 최신 devel과 동일하게 복원한 뒤 Native 전쪽을
다시 출력했다. 수정 후 53쪽 PNG는 기준과 바이트가 같다.

## 독립 Print 전쪽 시각 검증

최종 원본 TSV는 [final/native](../assets/pr8187/final/native/)와 [final/wasm](../assets/pr8187/final/wasm/),
정확한 쪽별 값·입력/PDF/TSV 해시는 [전쪽 요약](../assets/pr8187/final-visual-summary.json)에 있다.
print, 96dpi, 색상 경계 보정 전 2px 실루엣 raw를 사용했다. compatibility는 원본 Print 출처에 맞췄다.

| 문서 | 쪽수 | Native 최저 % | fresh WASM 최저 % |
| --- | ---: | ---: | ---: |
| midrange-three-columns | 2 | 100.00000 | 100.00000 |
| continued-one-to-three-columns | 3 | 100.00000 | 100.00000 |
| zone-preserve-paragraph-after | 2 | 100.00000 | 100.00000 |
| paragraph-after-1800 | 2 | 100.00000 | 100.00000 |
| paragraph-after-5000 | 2 | 100.00000 | 100.00000 |
| shortcut | 7 | 97.19106 | 97.20982 |
| KPS 원본 HWP | 77 | 96.38238 | 96.51991 |
| rowbreak18 | 18 | 97.64506 | 97.80701 |
| giant4138-original | 115 | 93.12289 | 97.33354 |
| giant4138-tab-period | 197 | 94.43562 | 94.61644 |
| #6549 Square 정상 대조군 | 2 | 100.00000 | 100.00000 |
| #6756 rowspan 컷 정상 대조군 | 5 | 98.00828 | 98.51080 |
| #5885 body frame 정상 대조군 | 7 | 99.96441 | 99.96386 |
| fresh WASM에서 저장·재열기한 KPS HWPX | 77 | 96.38238 | 96.51991 |
| 합계 | **516** | 미달·누락 0 | 미달·누락 0 |

독립 PDF와 전쪽 쪽수가 일치한다. 같은 글꼴의 base 원본 13문서·439쪽/backend와 대조해 하락 0쪽이다.
Normal 다단 원본 5개에서 10쪽, KPS 65·66쪽이 개선됐다. shortcut·rowbreak18·두 giant와 정상
대조군은 각각 같은 backend의 base PNG와 전쪽 바이트가 같다. 저장 KPS HWPX도 원본 HWP와
양 backend 각각 77쪽 PNG가 모두 같다. [페이지별 보존 증거](../assets/pr8187/pixel-preservation.json).

KPS **65쪽 Native 99.35691% / WASM 99.33451%**, **66쪽 Native 99.99427% / WASM 99.99510%**다.
66쪽 WASM이 소수 둘째 자리에서 100.00%로 표시되는 것은 픽셀 바이트 동일을 의미하지 않는다.
기준 devel 66쪽은 같은 공급에서 Native 90.01%, WASM 89.98%였으며 최종 후보에서는 둘 다 해소됐다.

### 직접 판독과 글꼴 경계

- KPS 65·66쪽의 독립 PDF/Native/WASM을 같은 영역·배율에서 비교했다. 위·아래 괘선, 이어받는
  라벨의 문단 소유, 본문 순서, 다음 셀 시작과 날짜·서명을 확인했다.
  [Native 66쪽](../assets/pr8187/final/native/kps-ai/review_066.png),
  [WASM 66쪽](../assets/pr8187/final/wasm/kps-ai/review_066.png).
- [같은 영역 확대 PNG](../assets/pr8187/final/crops/manifest.json)의 OpenAPI·괄호·확인란·날짜/서명과
  65쪽 컷을 직접 열어 확인했다. 빠진 글자·반복·clip·새 기준선 오류는 없었다. 획 두께 등
  글꼴/래스터 차이는 남아 있으며 raw 통과를 완전한 픽셀 동일로 설명하지 않는다.
- Normal 단 전환 1·2쪽 및 다음 3쪽, 뒤 간격 대조군을 직접 비교했다. 41–43/82–84 문단이
  같은 줄의 세 단에 배치되고 44/85 문단으로 이어지는 순서·간격·누락 여부를 확인했다.
- 최종 base/head는 **동일한 15개 글꼴 프로그램, 35,166,544바이트**를 공급했다. Native 64MiB
  catalog 한도를 넘지 않게 원본과 글리프 ID·윤곽·advance·수직 메트릭·family가 같은 subset을
  검증했다. subset의 동등성 검사가 실패한 두 프로그램은 원본 단독 face를 사용했다.
  [글꼴 지문](../assets/pr8187/base-producers.json). 원 작성자와 완전히 같은 글꼴 버전이라고 주장하지 않는다.
- KPS 입력 SHA-256 `9b0fceb3d96956f27c893e15a72a1ad94f7ee005bd581381a1aadfcb1f57a7b9`,
  PR의 독립 Print SHA-256 `6d95c2f18c275f77f5294bacec5ec15ead1b7dbbfe4a39d1df586bcaf54a4824`.
  `pdf/kps-ai-2022.pdf`는 바이트가 달라 임의로 바꾸지 않았다. 원본/Print는 원 PR의 정식 경로에 있다.
- 별도 SVG 소비 경계도 KPS 65·66쪽에서 실행·직접 판독했다. 두 쪽 최저 raw는 Native SVG
  99.00291%, WASM SVG 98.75594%이며, 프레임과 본문 순서를 보존했다. 이것을 PNG/Canvas 전쪽
  gate의 대체 증거로 사용하지 않았다.
- 접수의 1쪽 진단과 작성자의 giant 대표 4쪽 결과는 최종 전쪽 증거에 포함하지 않았다.
  글꼴 프로그램 자체는 저장소·배포물에 추가하지 않았다.

## 실행 검증 대장

| 검사 | 결과 |
| --- | --- |
| 논리 편집·서식 글·필드·range reflow 집중 회귀 | **72 PASS / 19모듈**, 계획한 모듈 모두 실행 |
| KPS 프레임·문단 범위 / rowspan 하한·다중 종료·무효 원본 | 집중 회귀에 포함, 모두 PASS |
| fresh WASM 실제 Chromium API·HWP/HWPX 왕복·Canvas | **36건 PASS**, [결과](../assets/pr8187/browser-api-results.json) |
| Studio 전체 테스트 | **2,716 PASS, 2 skip, 실패 0**; fresh web pkg를 namespace 안에서 연결. 기존 Node pkg 전용 2건은 해당 빌드 부재로 skip되며 실제 optimized WASM 브라우저 검증은 별도로 완료 |
| Studio TypeScript | PASS |
| public WASM 선언 gate | fresh pkg에서 PASS. 기존 배포 pkg의 선언 부재를 새 후보 실패로 세지 않음 |
| release-test 전체 회귀 | **12,415 PASS**, 기존 ignore 45, 실패 0 |
| Native Skia 공식 3종 | **4,064 PASS** = 라이브러리 4,040 + placeholder 2 + 직접 PDF 22. lib의 기존 ignore 12 |
| Rust 전체 lint / workspace build / suite·source-unit 정책 | **전부 PASS**. source-unit 4,099로 기준 대비 증가 없음 |

[필수 로컬 실행 대장](../assets/pr8187/local-validation.json)에 11개 명령의 exit 0·시간·로그 지문과
집중 모듈별 실행 수를 보존했다. 전체 회귀의 #2063 대형 CellBreak 검사는 228.091초에 완주했다.
다른 장비·과거 실행과 조건이 달라 이를 성능 개선 또는 악화의 측정값으로 해석하지 않는다.
[Frontend 실행 대장](../assets/pr8187/frontend-validation.json)과
[TSV·쪽별 값 무결성 확인](../assets/pr8187/evidence-check.json)도 보존했다.

회귀는 bwrap 파일 시스템 격리로 상용·개인 글꼴, Windows mount와 시스템 글꼴 경로를 숨긴
최소 글꼴 환경에서 실행한다. 실제 공개 fallback 5개와 명시 fixture만 보였다. 독립 Print 비교용
글꼴 공급과 혼동하지 않는다. Cargo는 공용 `target/pr-review`, jobs 4, tests 8로 순차 실행한다.

새 KPS 검사는 전체 문단 내용·소유·테두리와 원본 HWPUNIT 높이·다음 내용 관계를 보호한다.
65·66쪽 양쪽 raw 98% 이상과 확대 판독을 확인했으며 실제 절대 화면 px 핀은 추가하지 않았다.
기존 KPS 검사들의 문단 소유·포함·원본 크기 관계는 유지했다. 독립 출력과 맞는 기존 기대값은
바꾸거나 제거하지 않았다. API·보안·합성 행 원장 검사의 첫머리는 시각 비해당 이유를 기록했고,
다른 원본의 raw를 해당 합성 입력의 측정값으로 쓰지 않았다. 새 source-unit은 추가하지 않는다.

### 실행 producer

| 산출물 | source / SHA-256 |
| --- | --- |
| 기준 Native | 제품 source가 기준 devel과 동일한 `d78048a6f0…` 빌드 / `6d2cf962975ff04f6e965e8596e579ea99f1a47d201c9c6132d89fa4237d0d24` |
| 기준 optimized WASM | 기준 devel에서 새 빌드 / `c49ee93133bdb0856dcb29af71832e186f1cb1dd18d98a98121f33792344cccf` |
| 최종 Native | `054ef1cd2d`와 동일한 제품 source / `90cfc746f2082b303e29684992d3ca72d2bf5d018e62f86a450ca98fd41bb808` |
| 최종 optimized WASM | `c33037bf2f` / `f92b4c31ae7fdd323ef15eb859e0168211db0e286d40c74b569d5a41903bc83a` |

`054ef1cd2d`의 제품 변경은 `cfg(all(not(target_arch="wasm32"), feature="native-skia"))` 전용 두 파일뿐이다.
WASM에 영향이 없어 위 optimized producer를 재사용했다. 뒤의 테스트·증거 변경도 제품에 영향을 주지 않는다.
전체 gate가 만든 Native CLI는 fingerprint `83153458…`로 달랐으나 제품 source는 같았다.
그 CLI로 KPS 53·65·66쪽과 Normal 다단 1·2쪽을 다시 출력해 전쪽 producer의 같은 쪽 PNG와
바이트가 모두 같음을 확인했다. [최종 CLI 경계 확인](../assets/pr8187/native-postgate-smoke.json).
root pkg/Studio public WASM SHA-256 `19e4729f73daa5a183d37b29eb09500cdd217c112383f0d643b786fce59bb3c1`는 보존했다.

## 원격 제출 전 조건과 승인

필수 로컬 gate는 모두 완료했다. [원격 처리안](../assets/pr8187/remote-action-plan.md)과 본문 보충·감사 댓글 초안을 준비했다. 원 contributor branch에
원 이력을 보존한 후보를 push할 수 있는지 최신 head/권한을 확인하며, 승인 범위에서 새 head CI·self-review·merge·후속 댓글을 진행한다.
[접수 시 원격 조회](../assets/pr8187/remote-read-snapshot.json)와 실제 원격 처리 결과를 구분한다.

## Merge 후 contributor PR comment 계획

[감사 댓글 초안](../assets/pr8187/merge-comment-draft.md)을 기준으로 원 기여의 논리 편집·범위 재조판·
다단 기능을 먼저 설명하고, 최신 devel 통합과 KPS 표 프레임의 메인터너 보정을 구분한다.
실제 merge SHA·새 head CI URL·archive 검토 URL을 확정한 뒤 게시한다. Native/fresh WASM 각각
14문서·516쪽, KPS 66쪽 99.99427%/99.99510%, 집중 72건·전체 12,415건·Native 4,064건을 사용한다.
숫자를 사람의 판정 정확도나 픽셀 완전 동일로 설명하지 않는다. #7444 전체를 닫지 않는다.
merge에 포함된 Native·WASM KPS 66쪽 review PNG를 이미지로 표시하고
[Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을 연결한다.


## 원격 CI와 #6612 환경 의존 검사 보정 — 2026-11-03

- 원 contributor head를 보존한 `5ce743cbc7`을 fast-forward push했다. 실제 변경 범위에 LFS
  객체가 없음을 확인한 뒤 정본의 `GIT_LFS_SKIP_PUSH=1` 경로를 사용했으며 hook 설정은 바꾸지 않았다.
- 첫 실행은 옛 base SHA `60972534`가 고정된 이벤트로 lint의 대량 diff/ENOBUFS가 발생했다.
  실제 base `4e2cc6bb`로 metadata를 갱신하고, 작업지시자가 승인한 임시 close/reopen으로 새 이벤트를
  생성했다. 커밋·workflow·검증 기준은 바꾸지 않았다. 새 lint는 통과했다.
- 작업지시자가 이전 CI `41572395775`·Render `41572395710` 취소를 승인했으나 이미 자동 취소된
  상태였다. 수동 취소·재실행 API를 실행하지 않았다.
- 새 [CI 41574106733](https://github.com/edwardkim/rhwp/actions/runs/41574106733)의 Archive D에서
  기존 #6612의 WMF 이미지 개수 검사가 **101 != 21**로 실패했다. A/B/C/E·Native·lint·Frontend는
  통과했으며 CodeQL·Render Diff·Adapter·Proptest도 통과했다. 전체 성공으로 판정하지 않는다.
- 원 해결 #6618 → 메인터너 보정 `7946a457d9` → 통합 조상 `9088bd705c`와 독립 검토 기록을 먼저
  추적했다. 현재 base/head의 PDF 내보내기 구현과 원본은 같았다. Color Emoji를 먼저 노출한 격리
  환경에서 같은 실패를 재현했으며 **base/head PDF SHA-256도 동일**했다. 추가 80개는 136×128
  글자 비트맵 40개와 그 soft mask 40개다. WMF 21개가 누락된 실패가 아니다.
- generic fallback 옵션만 명시하면 원본의 미설치 family 선택이 남아 실패가 지속됐다. 해당 이미지
  자원 검사에서만 문서의 글꼴을 저장소의 공개 Noto Sans KR로 명시하고 `set_document`로 파생 상태를
  갱신했다. 제품 글꼴 정책·배치·workflow는 그대로이며 원본 파일과 **정확히 21개** assertion도 유지한다.
- Color Emoji 환경 수정 전 FAIL → 수정 후 **2 PASS**, 최소 글꼴 **2 PASS**, Color Emoji 반복
  **2 PASS**를 확인했다. Rust lint 묶음과 manifest 검사도 모두 PASS다. 테스트 전용 변경으로
  snapshot/baseline·쪽수·좌표를 바꾸지 않았고 기존 제품의 Native/fresh WASM 증거를 유지했다.
  [진단·원인 이력·실제 명령과 로그 지문](../assets/pr8187/issue6612-font-diagnostic.json).


## 최종 통합 — 2026-11-03

- 원 PR [#8187](https://github.com/edwardkim/rhwp/pull/8187)을 head `8c977717604f31ab73115769161b836d1b7102da`에서
  merge `3724ab9dc43bd60796a20e48c00c5f5ad1f053b9`로 통합했다. 원 기여 13커밋은 ancestry로 보존됐다.
- 최신 devel `4e2cc6bb36576b95d58d4c76cd833281ea652554`와 `merge-tree --write-tree`가 충돌 없이 완료됐고, 직전 재조회에서
  `mergeable=true`, `mergeable_state=clean`, 41개 check가 pass 또는 정식 skip이었다.
- [CI](https://github.com/edwardkim/rhwp/actions/runs/41580317496)·CodeQL·Render Diff·Adapter·Proptest와
  최종 정책 집계가 모두 성공했다. GitHub 기본 archive A–E는 **12,221 PASS**다. 로컬 전체 **12,415 PASS**에는
  CLI bin unit 12건과 workspace package 182건도 포함된다. 두 개수를 같은 실행의 값으로 섞지 않는다.
- CI Archive D에서 보정한 #6612 두 테스트의 실제 PASS를 확인했다. 기대값 1개/21개는 유지했다.
- [self-review](https://github.com/edwardkim/rhwp/pull/8187#pullrequestreview-5747574077)는 위 exact head에 APPROVED로 기록됐고 차단 지적은 없다.
  제품 소스가 같아 전쪽 Native/fresh WASM·실제 브라우저 증거를 유지했으며, 테스트 보정 뒤의 Rust lint와
  최소 글꼴·Color Emoji 집중 검사는 따로 완료했다. CI 통과 뒤 추가 제품 커밋이나 로컬 재빌드는 하지 않았다.
- 병합 후에는 [duration 갱신](https://github.com/edwardkim/rhwp/actions/runs/41583408055)과 이슈 metadata
  처리만 관찰했다. duration 46개 target의 자료가 `ci-metrics/nextest-target-durations`에 갱신됐다.
  검증 CI를 다시 시작하거나 재실행하지 않았다. [#7444](https://github.com/edwardkim/rhwp/issues/7444)는
  남은 Studio 연결 범위가 있어 OPEN을 유지한다.
- 후속 경로는 maintainer 직접 반영이다. 이 archive·asset·오늘할일만 한 운영 기록 commit에 반영한다.
  감사 댓글과 종료 정리는 승인 범위에서 이어서 실행하며, 실제 정리 결과는 `/tmp/rhwp-pr8187-cleanup.json`에 기록한다.

[최종 head CI·검토·병합·실제 archive 집계 증거](../assets/pr8187/remote-merge.json).
