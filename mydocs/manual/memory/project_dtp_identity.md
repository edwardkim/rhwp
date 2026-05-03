---
name: rhwp 정체성 — 한국형 DTP 엔진 + 워드프로세서
description: 아래아한글의 개발 의도 = 쿽 익스프레스 (QuarkXPress) 대체. 단순 워드프로세서가 아닌 DTP 도구 정체성. 다층 레이어 / WebGPU / 마스터 페이지 등은 DTP 인프라 토대.
type: project
originSessionId: b2ba0b26-0926-49a2-a3fb-6ff2062a8527
---
**작업지시자 통찰 (2026-05-02, Discussion #529 Appendix)**:

> 아래아한글은 개발 목적이 쿽 익스프레스를 대체하는 것이었으니, 사실 엄밀히 따지면 '워드프로세서' 만을 충족하기 위한 프로젝트가 아니었다. 그런 면에서 볼 때 다층 레이어 접근법은 더 흥미로워질 수 있겠다.

**Why:** 아래아한글 1.0 (1989) 출시 시점, 동시대 DTP 표준은 QuarkXPress 1.0 (1987). HWP 의 다음 특성에 DTP 정체성 흔적:
- 글상자 (TextBox) — text frame 직역
- 개체 자유 배치 (BehindText / InFrontOfText / 어울림) — box 자유 배치 + z-order
- 누름틀 / 양식 / 차례 / 색인 — 구조화 문서 + DTP 자동화
- 단(column) 정의 + 절(section) 분리 — 잡지/책 다단 조판
- 페이지 분할 정밀 제어 + 머리말/꼬리말/바탕쪽 — DTP 마스터 페이지 원형

→ rhwp 는 단순 "HWP 뷰어/에디터" 가 아니라 **"한국형 DTP 엔진의 오픈소스 재현 + 워드프로세서"** 의 통합 도구.

**How to apply:**
- 인프라 결정 (다층 레이어 / GPU 가속 / 색공간 / 협업 편집) 시 DTP 사용 사례를 함께 평가 — 워드프로세서 한정 비용 합리화로는 후보 B (WebGPU) 같은 큰 투자가 어려우나 DTP 정체성 본격화 시 (M200 v2.0.0+) 합리화됨
- M200 의 후보 B (WebGPU) 도입은 DTP 정체성 본격화의 중심 인프라 — SDF 텍스트 + GPU shader 합성 + 색공간 변환 + Pantone/CMYK 처리
- M150 / M200 단계의 작업 우선순위 결정 시 DTP 영역 (마스터 페이지 / 출판 layer / 협업 편집) 을 워드프로세서 기능과 동등한 가치로 평가
- 본 정체성에 따라 다층 레이어 (Task #516 옵션 C → M200 후보 B) 단계적 마이그레이션 경로는 단순 비용 회피가 아니라 **rhwp 본질 실현의 길**

**상세**: `mydocs/tech/multi_layer_rendering_strategy.md` Appendix A + GitHub Discussion [#529](https://github.com/edwardkim/rhwp/discussions/529) 참조.
