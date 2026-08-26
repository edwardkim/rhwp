# 시각 근거 — 렌더·레이아웃 변경

렌더링·페이지네이션·조판을 바꾸면 숫자 시험만으로 끝내지 않는다.
전후 근거를 PR 에 남긴다.

## 언제 필요한가

- `src/renderer/`, `src/paint/`, 레이아웃 관련 모델
- SVG/PNG/PDF 내보내기 경로
- 페이지 수·줄나눔·표 단편에 영향을 주는 패치

스킬 문서·계약 시험만 만지는 이 파동은 **시각 근거가 필요 없다.**
그래도 레시피는 갖춘다. 에이전트가 렌더 이슈를 이 스킬로 완주할 때 쓴다.

## 최소 근거

1. 공개 샘플 경로
2. 변경 전 SVG 또는 PDF (또는 페이지 수·텍스트 해시)
3. 변경 후 같은 명령의 산출
4. 환경 (OS, 폰트, rhwp 버전)

```bash
rhwp export-svg samples/<공개샘플>.hwp -o /tmp/before.svg
# ... 패치 ...
rhwp export-svg samples/<공개샘플>.hwp -o /tmp/after.svg
```

한컴 PDF 는 정답지가 아니다 (`CONTRIBUTING.md`). 한컴 대조를 첨부할 때는
한컴 버전·OS·폰트를 같이 적는다.

## 시각 회귀 스킬

숫자 판정(`render-diff`, `ir-diff`)이 필요하면
`.claude/skills/rhwp-visual-regression/` 을 **포인터로** 따른다.
이 스킬이 그 본문을 다시 쓰지 않는다.

예제: [13_visual_evidence_render.md](../examples/13_visual_evidence_render.md).
