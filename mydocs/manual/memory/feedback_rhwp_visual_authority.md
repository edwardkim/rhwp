---
name: rhwp 자체 시각 해석 권위 — IR 충실보다 시각 본질 우선 가능
description: 한컴 출력은 권위 미입증 (feedback_pdf_not_authoritative). 같은 IR 값에 대한 한컴 시각이 환경/버전별로 다를 때, rhwp 가 자체 시각 해석을 정의 가능. IR 데이터 충실보다 시각 본질 보장 우선.
type: feedback
originSessionId: b2ba0b26-0926-49a2-a3fb-6ff2062a8527
---
**작업지시자 정합 (2026-05-02, Task #516)**:

> 한컴을 믿으면 안 됨. 자체적으로 D-1 로 우리는 해석하면 될듯합니다.

**상황**: 복학원서.hwp 의 가운데 엠블럼 IR 값 (`effect=GrayScale, brightness=-50, contrast=70, watermark=custom`) 을 rhwp 가 그대로 시각 적용 → 진한 회색. 한컴 출력은 같은 IR 값에 대해 연한 회색 + 흐릿함. 같은 IR, 다른 시각.

**Why:**
- 한컴 출력은 환경별 (버전/폰트/OS) 로 다름 (메모리 `feedback_pdf_not_authoritative`)
- IR 값과 한컴 GUI 표시값의 매핑이 swap 또는 비선형일 가능성
- 한컴이 워터마크 효과 ON 시 IR 에 저장 안 되는 추가 시각 처리 (opacity 보정 등) 가능
- 결과: 한컴 출력을 정답지로 삼고 매핑을 맞추려 하면 환경별로 회귀가 누적됨 (메모리 `feedback_v076_regression_origin`)

→ **rhwp 는 IR 데이터 충실보다 시각 본질 (예: 워터마크 = 흐릿함 + 텍스트 가독성) 을 우선** 할 수 있다. 한컴 시각은 참고 자료로만 활용 (정답지 아님).

**How to apply:**
- IR 충실 적용으로 시각 본질이 손상되는 케이스 식별 — 워터마크 / 회색조 / blend mode / opacity 등
- rhwp 자체 시각 알고리즘 정의 가능 (예: Task #516 의 D-1 권장안 — multiply blend + opacity 강제 + IR b/c 보존)
- 단순 IR 충실 매핑이 시각 정합 깨는 경우, **편집자 의도 보존 (IR 값) + 시각 본질 보장 (rhwp 자체 처리) 의 균형** 으로 결정
- 이 권위는 한컴 환경별 출력 차이가 누적될 때 회귀 origin 차단의 기반 — 한컴 PDF 를 정답지로 사용한 외부 PR 이 작업지시자 환경에서 회귀 (메모리 `feedback_v076_regression_origin`) 의 본질 해결책

**관련 task**: Task #516 (다층 레이어 도입) 의 결함 2 (워터마크 시각) 가 본 권위의 첫 적용 사례. 시각 정합 정정은 분리 task #535 에서 rhwp 자체 D-1 (multiply + opacity) 또는 D-2 (한컴 자동 프리셋 시각으로 강제) 로 진행.
