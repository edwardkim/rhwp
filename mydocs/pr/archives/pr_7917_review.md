# PR #7917 — feat: add native caret and selection interaction APIs (P54)

## 최종 판정

**승인 — 메인터너 보정한 통합 PR #7969의 code head 검증 완료.**

검토 대상은 통합 code head `c5b99e2c2825f1c56e9750134fc5722f916a8a8b`와 production source `3535106051394672e93ca8401ff9eae8b449ec20`다. 원 contributor head `a0ecf253a78c2ccf106c91e9cba19115c88317f2`의 변경은 보정한 통합 head에서 수용한다. 실제 code CI와 로컬·시각 검증을 완료했다. merge 전 조건은 같은 PR의 review·오늘할일 trailing head CI 성공과 최신 mergeability 재확인이다.

## 접수 정보

- [원 PR](https://github.com/edwardkim/rhwp/pull/7917), 기존 contributor `seo-rii`, base `devel`, non-draft.
- 원 head `a0ecf253a78c2ccf106c91e9cba19115c88317f2`, 검토 base `fe5725dc384689b45d60ef8ea4c83138bc2701d8`.
- 접수 시 규모 +19732 / -1327 / 141 files는 누적 source diff다. 선행 PR을 중복 적용하지 않는다.
- reviewer `jangster77` 지정. 작성 시 mergeability는 참고값이며 실제 통합 후보와 최신 head를 다시 확인한다.
- [구현 순서](pr_7917_review_impl.md), [전체 계획](../../plans/pr_review_seo_rii_20261017.md).

## 변경·검증 범위

- 구현 주장: 본문/중첩 셀 주소와 scalar 위치를 core geometry에 전달해 hit-test·caret·selection·세로 이동을 조회한다. unsupported 주소는 출력 일부로 가장하지 않는다.
- 실제 소비 경로: `document_core/native_interaction.rs → native_host/mod.rs → FFI request`. 아래 검사는 source 단독 CI가 아닌 통합 후보에서 연결한다.
- 관련 검사: `issue_536_native_interaction; 실제 session_contract.py의 caret·selection`. 1–6단계 당시 전체 10,643건·Native 집중 277건 통과를 아래 실행 기록에 연결한다.
- 독립 기대값과 범위: 문서/쪽/문단/셀 소속과 scalar 경계를 검사한다. 조회 지원을 셀 편집 지원으로 확대 해석하지 않는다.
- 직전 source `94a7f2da7d712b0f20abc37c1981bc32d55dfd71` 대비 원 head의 delta를 확인했다. 누적 PR 전체 diff를 이 단계의 고유 변경량으로 세지 않는다. 원 commit → 체리픽 SHA는 [구현 기록](pr_7917_review_impl.md)에 보존한다.

## 조판 원칙과 입력 확인

공통 구현·통합 해소는 [1단계](../../working/task_m100_536_integration_review_stage1.md),
구조 분리와 예제 설명은 [2단계](../../working/task_m100_536_integration_review_stage2.md),
실제 Print 출처·폼 대응은 [3단계](../../working/task_m100_536_integration_review_stage3.md),
극한 이름의 생산→각 backend 소비·비적용 경계는 [4단계](../../working/task_m100_536_integration_review_stage4.md),
실제 전쪽 결과·정식 수정 전 FAIL/수정 후 PASS는 [5단계](../../working/task_m100_536_integration_review_stage5.md)에 연결한다.

검증 입력 8개는 각각 전체 1쪽이며 Native/fresh WASM 모두 90% 이상, 고정 devel 대비 0.01%p 이상 하락이 없다.
원본·독립 Print PDF·직접 판독과 metadata/합성 계약을 구분한다. Native IME 저장본은 실제 host 조합·undo/redo·저장·재열기 후 한컴 Print와 양쪽 100%를 확인했다. [최종 실행 기록](../../working/task_m100_536_integration_review_stage6.md)에 연결한다. 별도 단독 극한 진단 입력은 90% 미만으로
정상 fixture로 등록하지 않았고 완료로 표시하지 않는다. 새로운 절대 px 핀·baseline 완화는 없다.
쪽 분할·이어받기·저장 LineSeg 수용 규칙을 이 통합의 새 기능으로 변경하지 않는다.
1–6단계 당시 로컬 필수 검사는 완료했다. 전체 nextest 10,643 PASS / 0 FAIL / 45 skipped, Native 집중 277 PASS, Native Skia lib 4,043 PASS이며 format·Native/WASM/workspace lint·workspace build·doc test·실제 feature-off ABI·기존 HFT Native 검사도 통과했다. 그 당시 원격 CI는 미실행이었으며 현재 결과는 아래 통합 PR CI 절을 따른다. source SHA·명령·한계는 위 최종 실행 기록에서 구분한다.

## Merge 후 contributor PR comment 계획

1. 통합 PR의 실제 MERGED·merge SHA와 최종 head CI를 API로 확인하고, 이 개별 review·대표 asset이 devel에 존재하는지 먼저 확인한다. 원 PR을 직접 merge한 것으로 표시하지 않고 체리픽 통합 링크와 원 head `a0ecf253a78c2ccf106c91e9cba19115c88317f2`를 연결한다.
2. 원 author의 기여에 한국어 존댓말로 감사하고, 원 PR의 고유 주장·소비 경로·메인터너 보정 이유를 위 기록에서 가져온다. 로컬 검증은 최신 producer `3535106051394672e93ca8401ff9eae8b449ec20`의 전체 10779 PASS / 0 FAIL / 45 skipped, 실제 Native/브라우저 및 이후 완료한 필수 gate를 최종 기록에서 인용한다. 미완료 검사·미실행 플랫폼을 통과로 바꾸어 적지 않는다.
3. 직접 비교한 전체 입력은 13개·각 1쪽(원본8개와 독립 한컴 저장 색조 대조군5개)이며 Native/fresh WASM 모두 90% 이상, 고정 devel 대비 0.01%p 이상 하락·누락·쪽수 차이가 없음을 기록한다. 실제 host의 한글 조합·undo/redo·저장·재열기, live/saved frame byte 동일과 독립 Print의 Native/WASM 전체 1쪽 100%를 기록한다. 단일 예제를 모든 문서·OS 기능의 완료로 일반화하지 않는다.
4. 대표 stable 경로는 `mydocs/pr/assets/issue_536_seo_rii_20261017/{native,wasm}/native-ime_{review,overlay}.png`다. 실제 merge SHA로 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/issue_536_seo_rii_20261017/<backend>/native-ime_<panel>.png`를 만들어 **Markdown 이미지**로 표시한다. [Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)을 direct link로 함께 남긴다.
5. 알려진 원래 분수/폼 차이와 미해결 단독 극한 진단을 전체 해결로 표시하지 않는다. 부모 #536은 종료하지 않는다. 실제 결과 본문은 UTF-8 `--body-file`로 게시하고 API로 한글·선두 BOM·본문·merge SHA·asset URL을 재조회한다. 게시 성공 뒤 이 원 PR을 통합 링크와 함께 close하고 contributor fork branch는 보존한다.

## PR 직전 base 갱신

새 base `b7a5388034ee66f58a7e1d5363974e83b6374f57`와 통합 head `a7aac1a9ce5b2b902471ecd8f234f7d72834812d`의 [7단계 재검증](../../working/task_m100_536_integration_review_stage7.md)을 따른다. 원 head와 1–6단계 실행 기록은 그대로 보존한다. TSV 원본은 사용자 후속 지시에 따라 검토 asset으로 바이트 그대로 커밋하고 PR 본문에서 정확한 head SHA의 raw 링크로 연결한다.

## 최신 devel #7964 반영

기준 `9415b920c8dec299e8ada69b01dbd1c3c044f1f1`, 리베이스 직후 head `ec742f167991fd8c5efcd01ab7e8fe752c0a070e`의 [8단계](../../working/task_m100_536_integration_review_stage8.md)를 따른다. 이미지 캐시와 한컴 색조를 함께 보존하는 충돌 해소·raw 실루엣 재검증의 실제 완료 결과는 아래 최신 통합 source 절을 따른다. 앞의 base/통과 수치는 각 당시 실행 기록이며 새 source 결과와 구분한다.

## 최신 통합 source의 로컬 검증 완료

producer `3535106051394672e93ca8401ff9eae8b449ec20`, base `9415b920c8dec299e8ada69b01dbd1c3c044f1f1`에서 전체 회귀10779PASS /0FAIL /45skipped, Native 집중288PASS 및 필수 lint·fresh Studio·실제 ABI를 완료했다.13개 입력 전쪽의 Native/fresh WASM raw90% 이상·쪽수1:1·누락0·0.01%p 이상 하락 없음과 직접 판독을 [8단계](../../working/task_m100_536_integration_review_stage8.md)에 연결한다. 이전 본문의 실행 수와 SHA는 각 당시 역사적 결과이며 이 최신 실행과 구분한다.

### 조판 원칙 준수 검토

각 PR의 고유 구현 주장·실제 소비 경로·관련 검사와 비범위는 위 개별 변경 범위를 따른다. 아래 공통
통합 증거는 원 contributor head 단독 검증과 구분한다.

| 검토 항목 | reviewer가 확인한 근거 | 판정 |
| --- | --- | --- |
| 구현 근거와 일반성 | 위 개별 주장·실제 경로·계약 반례, 4–5단계 극한 이름의 독립 Print·glyph/em 근거와 8단계 캐시/색조 해소. 문서 ID 예외·clamp·출력 은폐를 추가하지 않았다. | 충족 |
| 측정·배치 일관성 | 극한 이름의 공통 EqLayout와 각 backend/font preflight 소비는 4단계, 개별 Native 조회는 기존 core geometry 소비 경로와 관련 검사에 연결했다. 저장 줄 수용·재조판 규칙을 새로 바꾸지 않았다. | 충족 |
| 최종 출력 소비 계약 | 8단계의 실제 Native Skia/fresh WASM Canvas, 실제 C ABI screen·Direct PDF, Studio CanvasKit font/shaping 및 색조/WMF 소비 검사. Apple SDK·C# runtime 실행은 미검증이다. | 충족(Linux 검증 범위) / 미검증(명시한 플랫폼) |
| 분할·이어받기 계약 | 이 통합의 신규 변경은 pagination·rowspan 컷·continuation 예약/종료 규칙을 바꾸지 않는다. 새 base의 해당 수정은 기준 변경이며 이 PR의 해결 성과로 표시하지 않는다. | 비해당 |
| 줄 소속과 점유 높이 | 저장 LineSeg 수용 조건·문단 표 소속을 변경하지 않았다. 극한 이름의 em/높이는 공통 결과와 원본 1쪽 회귀·Print로 확인하고 일반 문단의 줄 높이 재설계로 확대하지 않았다. | 충족(해당 수식 범위) / 비해당(저장 줄 규칙 변경) |
| 사례와 증거의 독립성 | 기존/새 한컴 Print, 사용자 수동 HWPX 폼 Print, Native API 생성 저장본과 독립 Print, 한컴 저장 색조5개를 구분했다. HWP 폼의 cross-format 대응과 합성 계약의 한계를 기록했다. | 충족 |
| 기준값 변경 | 기존 golden·baseline 기대값·래칫을 완화하지 않았다. 새 wire/API 합성 계약과 기존 정상 출력 증거를 구분한다. | 비해당(기존 기준값 완화 없음) |
| 회귀 추가·변경의 선행 조건 | 추가한 원본 극한 HWP 회귀는 5단계 양쪽90% 이상과 수정 전 FAIL/후 PASS 뒤 등록했다. 최신13개 입력 raw90% 이상을 재확인했다. 새 절대 px 핀은 추가하지 않았다. | 충족 |
| 90% 통과 후 확대 판독 | 최신 동일 좌표 Print/Native/WASM을 직접 판독했다. 분자·괄호 간격·폼 bevel·raster/테두리의 기존 차이와 미달 단독 진단은 미해결로 남겼다. 8단계 판독 원장 참조. | 충족(판독 수행; 남은 차이 별도 명시) |
| 주장과 검증 범위 | 최신 source·명령·실제 PASS와 원 PR별 고유 검사, 실제 ABI·browser 소비를 8단계에 연결했다. 단일 예제·Linux 결과를 전체 문서·OS 지원으로 일반화하지 않으며 현재 code head CI는 아래 실제 URL로 확인했다. | 충족(검증한 code head); trailing CI는 남은 merge 조건 |

## 통합 PR #7969 code head CI 완료

검증한 통합 code head `c5b99e2c2825f1c56e9750134fc5722f916a8a8b`의 [CI](https://github.com/edwardkim/rhwp/actions/runs/39007700375) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/39007700497) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/39007700494) · [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/39007700528) · [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/39007700524) · [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/39008935219)가 실제 성공했다. CodeQL의 JavaScript/TypeScript·Python·Rust·Actions Analyze와 GHAS CodeQL check도 성공했다. Full CI의 Native·lint·frontend, archive A/B/C/D builder·worker가 실제 성공했으며, 별도 WASM Build job의 skip과 frontend fresh WASM 성공을 구분한다.

같은 PR의 문서-only trailing commit과 그 최신 CI는 이후 확인한다. 원 contributor PR을 직접 merge한 것으로 표시하지 않는다.
