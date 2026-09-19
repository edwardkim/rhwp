---
kind: snapshot
status: active
canonical: mydocs/pr/archives/pr_7238_review.md
last_verified: 2026-09-17
---

# PR #7238 검토

## 최종 판정

**승인 — 메인터너 보정으로 보류 사유 해소.** 명시적 저장 속성 setter에 보정을 적용하고 Studio의 기존 분할 명령을 보존하는 범위로 승인한다. 새로운 UI 전체 호환성 보장을 주장하지 않는다. 최종 검증은 아래 공통 실행 기록을 따른다.

로컬 통합 검토이며 GitHub APPROVE·remote push·통합 PR 생성·merge·issue close는 수행하지 않았다.

## Metadata·계보·CI

| 항목 | 확인값 |
| --- | --- |
| PR | [#7238: 수정: 문단 시작의 쪽 나눔은 문단을 가르지 않고 그 문단에 건다 (#7218)](https://github.com/edwardkim/rhwp/pull/7238) |
| 작성자 / reviewer | planet6897 / jangster77 |
| base / state | devel / OPEN, non-draft |
| 규모 | 4 files, +252/-9, 1 commit |
| source head | `103ab177ab3c536a6d74d05ed59c10c67f37c605` |
| 적용 commit / 통합 code head | `3127bcce9` / `4d38c9a7b29dd87edf9228d668f4cb83928f6e87` |
| 조회 상태 | MERGEABLE / CLEAN; merge 직전 재조회 필요 |

- [Build & Test](https://github.com/edwardkim/rhwp/actions/runs/35204539449/job/105150854246): **SUCCESS**.
- [Native Skia tests](https://github.com/edwardkim/rhwp/actions/runs/35204539449/job/105147135139): **SKIPPED**.
- [CodeQL](https://github.com/edwardkim/rhwp/runs/105151551208): **NEUTRAL**.

SKIPPED/NEUTRAL은 해당 검사가 실행되어 통과했다는 의미로 세지 않는다. source CI는 통합 head의 CI가 아니다.

## 충돌 조정·통합 심사

원 #7230/#7238의 겹친 테스트는 각 CLI·코어 계약을 보존하도록 통합했다. 자동으로 병합된 production 함수의 의미 차이는 별도로 보정했다. [분석·수정·검증·결과보고 기록](../pr_7238_review_impl.md).

최종 구현은 #7238의 유효한 **다른 break 축 보존, synthesized 해제, 재배치**를 CLI/MCP의 명시적 속성 setter로 옮긴다. `insert_page_break_native`는 base의 기존 분할 구현을 그대로 복원한다. 저장 속성의 코퍼스 통계를 근거로 Studio Ctrl+Enter 의미를 바꾸지 않는다. core 4개는 setter 계약으로 검증하고 중간 분할·header/footer 보호는 유지했다.

**독립 명령 측정:** Windows 10, PowerShell→Python COM `11, 0, 0, 9136`에서 `First\r\nSecond`의 첫 문단 시작 `BreakPage`를 실행했다. 실제 커서 `(0,0,16)` → `(0,1,0)`, 2문단/1쪽 → 빈 선행 문단을 포함한 3문단/2쪽이었다. 이 한 사례는 저장 속성과 편집 명령을 구분할 근거다. 나머지 COM 경계는 기존 Hwp 프로세스 때문에 실행하지 않았고, 전체 한컴 호환성 통과로 세지 않는다.

**fresh WASM 제품 경계:** Chrome 153.0.8010.47에서 커밋된 outline_headings 입력의 첫 문단·일반 시작·내용을 지운 빈 첫 문단·반복·중간 5종을 Studio의 실제 `insertPageBreak` export로 호출했다. 각각 분할/반환 커서와 텍스트, HWPX/HWP 저장·재열기 문단 수를 검사했다. 기존 UI 동작 보존 검사이며 5종 모두가 독립 한컴 oracle와 일치한다고 주장하지 않는다.

**초기 영향 분석 정정:** npm `@rhwp/hwpctrl`의 `Run("BreakPage")`는 `breakAtCursor` → `break_at_cursor` 경로이며 Studio의 insert_page_break_native와 다르다. 최종/보정 전 WASM에서 5종의 커서·문단 수·페이지·저장 재열기 결과가 모두 같았다. 이 별도 경로의 기존 즉시 페이지 갱신 및 첫 문단 HWP 재열기 구역/문단 수 차이는 남아 있다(첫 구역 6→1문단, 전체 텍스트는 보존). 이를 이번 수정의 신규 회귀나 해결 완료로 쓰지 않는다.

CLI의 실제 최초 문단 저장, 다른 비트, synthesized, HWPX/HWP5 검사는 [#7230](pr_7230_review.md#코드독립-실행-심사)와 같은 입력·증적을 공유한다. **보류 해제 방식은 원 검토에서 허용한 “명시적 저장 속성에 범위를 한정하고 기존 편집 명령을 보존”이다.** UI 전체의 새로운 호환성 보장을 조건으로 붙이거나 미검증 결과를 통과로 바꾸지 않는다.

## 공통 조판 원칙 준수

렌더 영향 있음. Visual Sweep을 실행했고 합성 계약·독립 PDF·실제 제품 경계를 구분한다.

| 항목 | 판정 | 근거 |
| --- | --- | --- |
| 구현 근거와 일반성 | 충족 | 독립 PDF/COM 및 저장 사양, 문서별 숫자 조건 없음 |
| 측정·배치 일관성 | 충족 | paint geometry 보존 또는 setter reflow/vpos 공통 갱신 |
| 분할·이어받기 계약 | 충족 | 명시적 속성과 기존 사용자 분할 의미 구분; 표 조각 규칙 비해당 |
| 증거의 독립성과 범위 | 충족 | 원 face/한컴 PDF와 제품 테스트, 미실행 COM 경계 별도 표시 |
| 기준값 변경 | 충족 | 허용치 완화 없음, 실제 저장 fixture와 같은 독립 PDF 재확인 |
| 남은 문서 차이 | 별도 범위 | 표 높이/본문, 개요 번호 및 HwpCtrl 기존 경로를 해결로 주장하지 않음 |

[공통 최종 실행과 해시](pr_7212_review.md#통합-검토-공통-실행)를 따른다.

## Visual Sweep·입력 커밋 확인

**충족(공유 입력)** — [#7230의 입력 SHA-256·확인 commit·4쪽 전체 PNG](pr_7230_review.md#visual-sweep-직접-검토)를 사용한다. 두 PR에 동일 HWPX/PDF/PNG를 중복 추가하지 않았다. COM 첫 문단 1건과 fresh WASM 경계 검사는 위에서 구분하며, 저장 속성으로 범위를 한정해 기존 명령을 보존했다.

## Merge 후 contributor PR comment 계획

최종 승인·CI·실제 merge가 완료된 뒤에만 게시한다. 한국어로 기여에 감사하고 실제 merge SHA·최종 head CI URL·수정 범위·실제 검증 범위·남은 차이를 설명한다. [Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을 직접 연결한다.

- `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr7230_review/outline_after_wasm_review_001.png`
- `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr7230_review/outline_after_wasm_overlay_001.png`
- `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr7230_review/outline_after_wasm_review_002.png`
- `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr7230_review/outline_after_wasm_overlay_002.png`

CLI/MCP의 명시적 저장 속성 보정과 Studio 분할 동작 보존, COM 첫 문단 1건의 실제 검증 범위를 설명한다. 개요 번호 renderer 및 별도 HwpCtrl 경로의 기존 차이는 해결 완료로 쓰지 않는다.

페이지·후보 수·pixel/ink 지표와 사람의 판정을 함께 적는다. review 패널만으로 standalone overlay를 대체하지 않으며 모든 위 대표 쪽을 댓글 본문에 실제 이미지로 표시하고 `<details>` 밖에 둔다. PR과 관련 issue 댓글 모두 같은 해시 고정 경로를 사용한다. UTF-8 파일+`gh ... --body-file`로 게시한 뒤 API/렌더된 본문에서 한국어·실제 head·이미지 URL과 표시를 재확인한다. 해결하지 않은 issue를 일괄 close하지 않는다.
