---
kind: snapshot
status: active
canonical: mydocs/pr/archives/pr_7201_review.md
last_verified: 2026-09-16
---

# PR #7201 검토 — #6389 대체 글꼴의 저장 다중행 보존

## 최종 판정

**승인** — 이미 다중행으로 구성된 셀 문단을 단일행 구제 함수가 다시 나누는 결함의 개선 범위다.
전체 문서 fidelity나 #6389 종료를 승인하지 않는다. 최신 PR head의 GitHub Actions 통과와
작업지시자의 merge 승인은 별도 조건이다. 이번 지시는 #6389 PR 생성이며 #6611은 포함하지 않는다.

## 대상과 경로

- [PR #7201](https://github.com/edwardkim/rhwp/pull/7201), [Issue #6389](https://github.com/edwardkim/rhwp/issues/6389), `Refs #6389`.
- 작성자 jangster77의 self-review. reviewer는 지정하지 않았다.
- base route: `collaborator_self_merge`; modifiers: `intake_and_review`, `local_validation`,
  `visual_fixture_evidence`, `review_only_fast_pass`. 모 문서·선택표·해당 자식 문서를 읽었다.
- branch: `codex/jeongsik-6389-20260916`; base `devel=f1b53e82c502b8d7795a783d79cd4830168efcd7`.
- 검증 code head: `78340390db6f8e8d9ffe35cf1fe9775cb9950ac1`.
- 생성 직후 참고값: OPEN, non-draft, MERGEABLE/BLOCKED, CI 진행 중. 최신 상태는 merge 전에 재확인한다.
- 최초 변경은 source 1, test 1, 결과보고 1, 최종 PNG 20개다. 후속 기록은 이 검토·오늘할일과
  PNG의 PR별 경로 정리 및 보고서 링크 변경뿐이며 코드·테스트·이미지 내용은 불변이다.

## 구현과 검증 근거

[단계별 분석·실행 명령·결과](../../working/task_m100_6389_stage4.md)를 정본으로 삼는다.
`recompose_stored_single_line_if_overflowing_cached`의 `composed.lines.len() != 1` 보호를
함수 입구로 옮겨 다중행 `※`/`☞` 문단도 보호했다. 정상 저장 줄의 폭 조정은 기존 공통 경로에 맡겼다.
한 줄로 실제 붕괴한 #5952와 부실 단일행 #2291의 구제는 유지했다.

- 실제 편람 no-ttf: 수정 전 17줄/새 assertion 실패(exit 101) → 수정 후 독립 PDF와 같은 16줄.
  기본·대체 환경 2건과 관련 반례 13건이 통과했다.
- 전체 release-test **9,939 통과 / 51 skipped / 실패 0**. Native Skia lib **4,112 통과 / 13 ignored**,
  그림 2, 직접 PDF 4건이 통과했다. skip/ignored를 통과 건수에 합산하지 않았다.
- fmt, native·WASM32·workspace all-targets Clippy, workspace build, manifest 고정 base 비교를 통과했다.
  최초 manifest drift는 파생 harness 재준비 후 해소됐으며 파생 파일은 커밋하지 않았다.
- 최적화 fresh WASM을 Chrome에서 실제 실행했다. 변경 8쪽의 Native/WASM PNG가 동일했다.
  기본 KoPub 환경 384쪽 SVG는 수정 전후 불변이었다. 전체 원장의 악화는 없고 p69 table_footer 후보가 1→0이었다.
- 실제 Visual Sweep·standalone overlay로 대상 셀, 표 외곽과 뒤 내용을 대조했다.
  p68/원 PDF p69의 17→16줄 및 p69/원 PDF p70의 뒤 내용 분리를 확인했다.
  머리말·쪽번호 방향·글꼴 모양·단별 배분 등 남은 차이는 위 정본의 8쪽 대응표에 기록했다.
  픽셀 보조값이나 후보 수만으로 전체 시각 일치를 판정하지 않았다.

## 조판 원칙 준수 검토

| 항목 | 실제 근거 | 판정 |
| --- | --- | --- |
| 구현 근거와 일반성 | 독립 no-ttf PDF의 16줄과 저장 LineSeg; 함수의 단일행 계약 복원. 문서 ID·글꼴 분기·임계 조정 없음 | 충족 |
| 측정·배치 일관성 | 정본의 호출 경로 표: height_measurer, table_layout, table_partial이 같은 셀 줄 재구성 결과 소비. 실제 Native/WASM 비교 | 충족 |
| 분할·이어받기 계약 | 컷 선택·유닛 소유·예약·종료 알고리즘은 변경하지 않음. 셀 줄 결과 소비는 공통 경로 유지, 전체 회귀와 변경 페이지의 후속 내용 대조 | 충족(변경 범위) |
| 줄 소속·점유 높이 | 저장 다중행 보존; 선행 stale 판단 유지. 대상 셀 h416.6→407.5px, 16줄. NO_LS 편집 환경 반례 별도 통과 | 충족 |
| 증거 독립성 | 기존 실물 HWP/한컴 no-ttf PDF; 새 회귀 RED→GREEN 및 붕괴·빈 말미행·부실 단일행 반례 | 충족 |
| 기준값 변경 | 기존 assertion·golden·래칫·오차 완화 없음 | 비해당 |
| 주장과 검증 범위 | 정확한 code head의 전체 검증·fresh WASM·직접 비교. 86712 글꼴 굵기 및 전체 fidelity는 잔여 범위 | 충족(부분 개선) |

## 입력과 영구 증적

검증 입력 커밋 확인: **충족**. 아래 파일은 `78340390d`의 Git blob과 실제 실행 파일이 바이트 단위로
같음을 확인했다. 기존 파일을 재사용하며 이름을 바꾼 입력이나 PDF 사본을 추가하지 않았다.

| 기존 경로 | 역할 / SHA-256 |
| --- | --- |
| `samples/2025 행정업무운영 편람(최종).hwp` | 실물 입력, 한컴 2024 13.0.0.3622 저장 / `40d6d05eac4d55bdc4b0c62c42d93af104d5123b447581246f36fd15de7bd46f` |
| `pdf/2025 행정업무운영 편람(최종)-2010-no-ttf.pdf` | 독립 한컴 PDF, 389쪽, Hancom PDF 1.3.0.404 / `c33fbf6be76ae43e8254321ec99b5ab3963e492ff4b6179d7c9717ecac2a05e0` |
| `pdf/2025 행정업무운영 편람(최종)-hwp-kopub-2020.pdf` | 기존 KoPub 참고 출력, cairo 1.18.0, 한컴 직접 출력 계보 미확인 / `6c7be7602cb92bb9b5e6a0b66e9cd80700fceabeade89fabd1a0fcd32adc4413` |

최종 PNG는 [pr7201_review](../assets/pr7201_review/)에 있다. 임시 산출물은
`/private/tmp/rhwp-6389-followup-20260916`에서 관리하며 raw log/JSON/TSV/SVG는 커밋하지 않았다.
각 PNG SHA-256은 아래 표에 기록한다. 중복 사본 대신 기존 작업 증적을 이 경로로 이동했다.

## Merge 후 PR·이슈 comment 계획

병합 승인을 받아 최종 head CI와 merge를 확인한 뒤, 실제 merge SHA·CI URL·이번 저장 다중행
수정 및 잔여 차이를 한국어로 설명한다. #6389는 부분 개선 기록만 남기며 OPEN을 유지한다.
[Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을 연결한다.

PR comment에는 p68·69 전후 review/overlay 8개를 표시하고, 나머지 영향 6쪽의 최종 WASM
review/overlay 12개는 펼침 영역에서 표시해 저장 줄 복원과 남은 차이를 확인할 수 있게 한다.
이슈 comment에는 대표 p68·69 최종 review/overlay 4개와 전체 증적 링크를 표시한다.
원 PDF 대응은 각각 69·70·202·236·242·375·378·382쪽이며 8쪽의 보조 지표는 정본 표를 인용한다.
낮은 내용 픽셀 일치율과 기존 단별 배분 차이를 숨기지 않는다.

이미지는 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr7201_review/<아래 파일명>`
형식의 SHA 고정 URL로 넣는다. UTF-8 본문 파일과 `--body-file`을 사용하고 게시 뒤 API로
한국어·실제 merge SHA·이미지 URL을 확인한다. 현재는 계획이며 comment·merge는 실행하지 않았다.

| 최종 PNG | SHA-256 |
| --- | --- |
| [before_overlay_068.png](../assets/pr7201_review/before_overlay_068.png) | `b5b6a326d40d5ada7421c31b63a48c1b0454ec4053c1369d169395d9128b92ed` |
| [before_overlay_069.png](../assets/pr7201_review/before_overlay_069.png) | `69b42a90d33303ecd5829772e9ef8318deb09e7b900c0377fafa1ee50a226cb5` |
| [before_review_068.png](../assets/pr7201_review/before_review_068.png) | `645df468c1240d3ef349647cc178d5668fd34b89a26b3e69f501d7504f286da0` |
| [before_review_069.png](../assets/pr7201_review/before_review_069.png) | `ff73259ebccb4df6afcdef40b9a9eea1ea1bf47acb2f41624599ee6c53d05d2d` |
| [wasm_overlay_068.png](../assets/pr7201_review/wasm_overlay_068.png) | `4ba0629b78f27d4dd8742d433992ff3678290a2d8adf3ed9e0bb367f8bab726a` |
| [wasm_overlay_069.png](../assets/pr7201_review/wasm_overlay_069.png) | `9789c22c23ac00f660c0fd404f3e775a38e5a1a1a1159826432a8c671b11a7da` |
| [wasm_overlay_200.png](../assets/pr7201_review/wasm_overlay_200.png) | `6de27979b2d139bac2833722340697a3e4533853ac84e35f261ec78ee4ece09c` |
| [wasm_overlay_234.png](../assets/pr7201_review/wasm_overlay_234.png) | `09425c61742e129137d5e8cb9873029afabe04f6772eb02db88f94e56071a61a` |
| [wasm_overlay_240.png](../assets/pr7201_review/wasm_overlay_240.png) | `8153ecf0ecc4358f187606437f1755a233a4ec4cbcc83a1faf755da5a7c70fe5` |
| [wasm_overlay_369.png](../assets/pr7201_review/wasm_overlay_369.png) | `30dbab1904d41bd616cece75b56f7ee23debae7056ff0d774a3284151b040809` |
| [wasm_overlay_372.png](../assets/pr7201_review/wasm_overlay_372.png) | `788efa5dd0467dfefd0080d869795f17071f610eb5a5293457ad08e5c6d319d0` |
| [wasm_overlay_377.png](../assets/pr7201_review/wasm_overlay_377.png) | `dab1e0d46ee76572c16bc254bc8d68e3dfc14a0f198d06ed61601171c651b477` |
| [wasm_review_068.png](../assets/pr7201_review/wasm_review_068.png) | `62c4d8882ba2ebc120fd59e0b2eb5235b567f26223768c26f29728599c55b091` |
| [wasm_review_069.png](../assets/pr7201_review/wasm_review_069.png) | `864bbc93a02de1193490581693a109ee66814d3cc4c7d64f585a5e16f27cd54e` |
| [wasm_review_200.png](../assets/pr7201_review/wasm_review_200.png) | `9f8035b25808c55f476176b856cc1ca13ec2bf632cd3a153755c603bdeae1b4c` |
| [wasm_review_234.png](../assets/pr7201_review/wasm_review_234.png) | `e52bc74039e6754e04ddcea581682b854302f6325f4a3bc6d4a707569fc44311` |
| [wasm_review_240.png](../assets/pr7201_review/wasm_review_240.png) | `baa1d7705a8656c708635486f6193568947ef698934ddaed9fb32ab8d19a86b1` |
| [wasm_review_369.png](../assets/pr7201_review/wasm_review_369.png) | `a4905c5a9bfed02dd93aaec6f393008d5c90141b839ff3f12c44d30cf7fb7fcb` |
| [wasm_review_372.png](../assets/pr7201_review/wasm_review_372.png) | `71a46901d5fe8a8bedc3df96ef7caee892f2b06dbb0e4656f63308e249e3667f` |
| [wasm_review_377.png](../assets/pr7201_review/wasm_review_377.png) | `b5af136d594513498eff2304190ccdce0652d060da0146911245374864666612` |
