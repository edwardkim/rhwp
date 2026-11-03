---
kind: report
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-11-03
---

# PR #8187 검토 — 논리 편집·범위 재조판과 다단 경계

## 최종 판정

**머지 보류 — 승인된 메인터너 통합 보정과 필수 로컬 검증을 진행 중이다.**
새 논리 편집·재조판 API는 최신 devel에도 없어 수용할 기능이 남아 있다. 기능의 기각이나
PR 종료 판정이 아니다. 11개 파일의 충돌을 보정한 통합 후보가 없고, 현재 devel의 KPS 66쪽도
Native 진단에서 90%에 미달했다. 통합 코드·fresh WASM·전쪽 시각 검증을 완료한 상태가 아니다.

후속 로컬 작업은 [통합·검증 계획](pr_8187_review_impl.md)에 정리했다. 2026-11-03 작업지시자가
메인터너 보정·필수 검증을 한 단위로 승인했다. 원격 push·댓글·GitHub approve·merge는 수행하지 않았다.
아래 접수 단계의 관측과 통합 후보에서 수행할 검증을 구분한다.

## 접수 정보

| 항목 | 확인값 |
| --- | --- |
| PR / 작성자 / base | [#8187](https://github.com/edwardkim/rhwp/pull/8187) / semanticist21 / devel |
| 원 head | `32df7b71a6e731f767e4945e3e1d79ca0cac3e58` |
| 작성자가 검증한 제품 source | `d166697310779a8cc02a3091b6e1481d0d4ab182` |
| PR의 실행 base / 현재 공통 조상 | `60972534680a0c5e51e54ec189b92e252e81a867` |
| 이번 최신 devel | `4e2cc6bb36576b95d58d4c76cd833281ea652554` |
| 로컬 검토 branch / 위치 | `review/pr-8187-logical-editing` / `/home/edward/mygithub/rhwp` |
| 원 head 보존 ref | `refs/remotes/upstream/pr8187-head` |
| Reviewer | `edwardkim` 지정 후 requested-reviewers API로 확인 |
| 접수 시 원격 상태 | OPEN / non-draft / CONFLICTING / DIRTY |
| 규모 | 128개 파일, +5,650 / −235줄. source·test는 52개 파일, +4,063 / −235줄 |
| 선택 경로 | maintainer_general + intake_and_review, local_validation, visual_fixture_evidence, rework_and_exceptions, post_merge |
| 관련 이슈 | [#7444](https://github.com/edwardkim/rhwp/issues/7444)의 논리 선택 삭제·붙여넣기 잔여 범위. PR 본문과 closing-issue 연결은 없음 |

기본 작업공간에서 최신 devel을 기준으로 검토 branch만 만들었다. contributor source를 checkout해
덮어쓰거나, 충돌 난 merge를 작업트리에 적용하지 않았다. 다른 작업의 워크트리와 배포된 WASM을 보존했다.

## 기능 주장과 최신 devel 대조

| 범위 | 현재 확인 | 통합 방침 |
| --- | --- | --- |
| `deleteRangeLogical`, `clipboardIsSingleText`, `pasteInternalTextLogical`, `pasteHtmlTextLogical` | 최신 source에 해당 공개 API가 없음 | 원 기여의 논리 좌표·필드 소속·변경 전 거부 계약을 공통 삭제·삽입 경로에 통합 |
| `reflow_body_paragraph_range_native`, `refresh_section_native` | 최신 source에 해당 API가 없음 | 범위 밖 저장 줄 보존, 배치 중 지연, 실패 시 부분 변경 방지를 확인해 수용 |
| 분할 셀 `model_para_range` | 최신 RenderTree에는 `model_cell_index`가 있으나 새 범위 필드는 없음 | 최신 분할 셀의 물리 프레임·clip 소유와 함께 전달 |
| Skia TextBlob 기준선 | 최신 `text_shaping.rs`에 실제 선택 face의 ascent를 쓰는 `baseline_offset` 보정이 이미 있음 | 동일 목적의 옛 구현을 덮어쓰지 않고 기존 개선을 유지 |
| 문단 앞 간격을 포함한 첫 줄 fit | 최신 `split_entry.rs`에 #8149의 일반 첫 줄 앞 간격 차감이 이미 있음 | PR의 부분 쪽 Normal 단 수용과 최신 일반 fit 계약을 함께 보존 |
| KPS 66쪽 위 테두리·내부 행 위치 | 최신 devel의 Native 1쪽 진단에서도 남음 | 글꼴 차이를 분리하고 공통 표 조각/괘선 경로를 조사. 현재 PR이 만든 새 회귀라고 단정하지 않음 |

API 부재는 source 대조 결과다. 이를 Studio의 모든 기존 편집 동작이 실패한다는 실행 판정으로
확대하지 않는다. 같은 이름의 API가 없다는 사실과 기존 API로 가능한 동작도 구분한다.

## 기존 해결 이력과 보존할 계약

- #7444의 본문과 전체 댓글을 확인했다. [원 PR #7509](https://github.com/edwardkim/rhwp/pull/7509)
  → 실제 merge `fc36d71452a1aedae5f1f2b05737efe60a00f37f`
  → [독립 검토 기록](archives/pr_7509_review.md)에서 개체 뒤 삽입·논리 범위 복사만 수용한 것을 확인했다.
  선택 삭제·음영·서식·Studio 연결층까지 해결된 것으로 간주하지 않는다. #8187은 이 중 삭제와
  서식 글 붙여넣기 등을 보완한다. 이 PR만으로 #7444 전체를 닫지 않는다.
- 최신 `ca5ed2a48e8f3213df5f914b2c49842676eb7452`는 실제 face 경계와 SkShaper 기준선을
  보정했다. 현재 `src/renderer/skia/text_shaping.rs`의 `baseline_offset` 소비를 확인했다.
- 최신 `f9acf61ec9`는 #8149의 첫 줄 fit에서 앞 간격을 차감한다. PR의 `empty_text_zone`
  조건만 선택해 일반 문단의 앞 간격 차감을 되돌리지 않는다.
- 최신 `c417310cab6`의 `insert_text_from_paragraph`는 원본 문단 메타데이터를 함께 옮긴다.
  논리 붙여넣기의 필드 소속 보정을 추가하면서 이 경로를 단순 텍스트 삽입으로 되돌리지 않는다.
- #8174 이후의 `Storage` 기반 자원 예산과 실패 전달, 공통 측정·배치 프레임은 통합 시 유지한다.
  개별 선행 PR의 실제 fixture와 검증 근거는 보정 단계에서 해당 변경마다 연결해 재검증한다.

## 실제 수행한 검증

| 검사 | 실행 결과와 한계 |
| --- | --- |
| 최신 base와 원 head 고정 | upstream fetch 후 위 SHA를 확인했다 |
| 통합 simulation | `git merge-tree --write-tree <현재 devel> <원 head>` exit 1. 11개 파일의 content conflict를 확인했다. 작업트리에는 충돌을 적용하지 않았다 |
| 원 head CI | [CI 40092729836](https://github.com/edwardkim/rhwp/actions/runs/40092729836)의 head가 `32df7b71…`이며 completed/success인 것을 확인했다. 최신 devel 통합 후보의 CI는 아니다 |
| 원 PR 증적 무결성 | manifest의 파일별 SHA-256을 원 head blob과 대조했다. 검증한 수와 전체 목록은 [접수 증거](assets/pr8187/intake-evidence.json)에 기록했다 |
| 원 PR 전쪽 TSV | KPS 77쪽의 base/후보 × Native/WASM 원본을 직접 읽었다. 각 경로의 90% 미만은 66쪽 하나다 |
| 원 PR PNG 직접 판독 | Native·WASM KPS 66쪽과 Normal 다단 전환 2쪽 review PNG를 열었다. 작성자 측 KPS 위 테두리 누락과 행/하단 위치 차이, 전환 사례의 본문 순서를 확인했다. 직접 재출력한 후보의 결과가 아니다 |
| 현재 devel Native 진단 | 같은 KPS 원본·PR의 독립 Print PDF, print/96dpi/compat 2022로 66쪽을 실제 재출력했다. raw **89.98789%**, 실제 문서 쪽수 77/77. 위 테두리 누락과 행/날짜/서명 위치 차이를 직접 판독했다 |

### 현재 devel 진단의 source·입력·글꼴

기존 Native producer SHA-256 `6d2cf962975ff04f6e965e8596e579ea99f1a47d201c9c6132d89fa4237d0d24`를
다시 확인했다. 빌드 source `d78048a6f0e5f71aa3bdb7d32e703bd5971e3019`와 현재 devel의
`src/`·`crates/`·Cargo manifest/lock diff는 없다. source가 같은 바이너리를 재사용했으며 새 빌드는 하지 않았다.

- 원본: `rhwp-studio/public/samples/kps-ai.hwp`, SHA-256
  `9b0fceb3d96956f27c893e15a72a1ad94f7ee005bd581381a1aadfcb1f57a7b9`.
- 기준: 원 PR head의 `mydocs/pr/assets/platy-native-editing/inputs/kps-ai.hwp.hancom.pdf`, SHA-256
  `6d95c2f18c275f77f5294bacec5ec15ead1b7dbbfe4a39d1df586bcaf54a4824`.
  기존 `pdf/kps-ai-2022.pdf`와 바이트가 다르므로 임의로 바꾸지 않았다.
- 글꼴 공급: `/home/edward/mygithub/ttfs/hwp`, `/home/edward/mygithub/ttfs/windows`,
  `/mnt/c/Windows/Fonts`. 원 작성자의 15개 subset 환경과 동일하다고 주장하지 않는다.
  따라서 이전 점수와의 차이를 코드 개선량으로 계산하지 않는다. 글꼴 프로그램은 추가하지 않았다.
- 명령: `venv/bin/python scripts/visual_sweep.py --hwp rhwp-studio/public/samples/kps-ai.hwp
  --pdf output/pr-review/pr8187/inputs/kps-ai.hwp.hancom.pdf --key kps-ai-current
  --rhwp-bin target/pr-review/release/rhwp --compat 2022 --dpi 96 --pages 66 --embed-fonts full
  --font-path /home/edward/mygithub/ttfs/hwp --font-path /home/edward/mygithub/ttfs/windows
  --font-path /mnt/c/Windows/Fonts --out output/pr-review/pr8187/current-devel-native`.
  exit 1은 실행 장애가 아니라 raw 90% 미달 gate 판정이다.

[현재 devel Native review](assets/pr8187/current-devel-kps-p066-review.png)와
[overlay](assets/pr8187/current-devel-kps-p066-overlay.png)를 보존했다. 이것은 **66쪽 진단**이며
현재 문서 전체의 최저값이나 fresh WASM 통과를 뜻하지 않는다.

작성자 증적의 KPS 최저값은 Native **89.27622%**, WASM **88.90567%**이며 base와 후보가 같다.
두 giant 문서는 115/197쪽 중 각각 4쪽만 제출되어 있다. 이 수치를 현재 devel 또는 통합 후보의
전쪽 검증으로 재사용하지 않는다. 실제 전쪽 Native/fresh WASM·focused·필수 lint/회귀는 보정 후보에서 남아 있다.

## 조판 원칙 준수 검토

| 항목 | 접수 단계 판정과 근거 |
| --- | --- |
| 선행 분석 | #7444 → #7509 → 실제 merge와 독립 검토 기록 확인. 최신 겹치는 보정은 위에 구분했다. 통합 때 각 해당 반례 재실행 필요 |
| 구현 근거·일반성 | 논리/글자/UTF-16 제어 슬롯과 명시적 Normal 단 계약을 구분한 접근은 유효한 검토 대상. 원 head 전체 규칙 준수 확정은 아직 하지 않음 |
| 측정·배치 및 줄 점유 | 적용 대상. PR의 내용 하단·뒤 간격 계산과 최신 공통 프레임의 최종 소비를 보정 후 대조해야 함 |
| 분할·이어받기 및 소유 | 적용 대상. 새 `model_para_range`와 현재 컷/clip/예약 높이의 결합이 미검증 |
| 최종 backend 소비·rebase 기능 보존 | 미검증. 충돌 없는 통합 후보가 아직 없음 |
| 회귀 추가·기준값 | 새 회귀 5파일/34함수. 첫머리의 측정·유형·시각 근거를 현행 규약에 맞춰 보완해야 함. 미달 문서로 임의 기대값을 새로 고정하지 않음 |
| 시각 gate·직접 판독 | 원 PR KPS 미달과 현재 Native 66쪽 미달을 확인. 현재 전쪽/fresh WASM 미실행. 승인 근거로 승격하지 않음 |
| 주장과 검증 범위 | source 대조, merge simulation, 원 증적 감사와 현재 Native 1쪽 진단까지만 수행. 통합 성공·전체 API 회귀 통과는 주장하지 않음 |

## 다음 조건

11개 충돌에 대해 [통합 계획](pr_8187_review_impl.md)의 최신 공통 로직 보존 방침을 결정한다.
승인 뒤 같은 검토 branch에서 원 contributor 커밋 이력을 보존해 통합하고, API·저장 왕복·분할·
전쪽 Native/fresh WASM과 필요한 직접 판독·필수 lint/회귀를 한 단위로 완료한다.
미달을 이유로 기여 기능을 자동 기각하거나, 기존 결함이라는 이유만으로 시각 기준을 자동 면제하지 않는다.

## Merge 후 contributor PR comment 계획

아직 수용 판정과 원격 처리를 승인받지 않았다. 향후 보정·검증이 완료되면 원 기여가 추가한 API,
메인터너의 최신 devel 통합 보정, #7444의 남은 Studio 범위를 구분한 존댓말 문안을 작성한다.
실제 전쪽 수치·source/merge SHA·CI·대표 PNG와
[Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을 연결한다.
새 head에 해당하는 증적과 사용자 승인 전에는 댓글·approve·push·merge를 수행하지 않는다.
