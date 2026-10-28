---
kind: working
status: done
issue: 6016
---

# task_m100_6016 stage1 - visual_sweep Linux 한글 라벨 폰트 분석

## 배경

PR #6015 검토 증적 `mydocs/pr/assets/pr_6014_issue5712_p1_review.png`에서 오른쪽 overlay/contact
sheet 하단의 metric 설명 라벨이 한글 glyph 대신 네모 박스(tofu)로 표시됐다. 문서 본문 렌더와 기준 PDF의
한글은 깨지지 않았고, `visual_sweep.py`가 PNG 위에 그리는 설명 라벨만 깨졌다.

## 원인

`scripts/visual_sweep.py`의 `label_font()`는 macOS Arial 계열 경로 두 개만 확인하고, Linux에서는
`ImageFont.load_default()`로 fallback했다. Ubuntu 개발 서버에는 `Noto Sans CJK KR`와 `NanumGothic`이
설치되어 있었지만, 기존 구현이 fontconfig나 Linux CJK 폰트 경로를 조회하지 않아 사용할 수 없었다.

확인한 현재 서버 fontconfig 결과:

```text
NotoSansCJK-Regular.ttc: "Noto Sans CJK KR" "Regular"
NanumGothic.ttf: "NanumGothic" "Regular"
```

따라서 설치 폰트 부족이 아니라 증적 생성 도구의 라벨 폰트 선택 경로 누락이다.

## 보정 방향

`label_font()`를 Linux/macOS/Windows 공통으로 다음 순서로 변경한다.

1. `RHWP_VISUAL_SWEEP_LABEL_FONT` 환경변수로 명시한 폰트가 있으면 우선 사용한다. 값은 실행 OS의
   `os.pathsep`으로 여러 후보를 나열할 수 있다.
2. `fc-match`가 있는 환경에서는 `Noto Sans CJK KR`, `NanumGothic`, `UnDotum`, Windows/macOS CJK 계열
   family 후보를 찾는다.
3. `platform.system()` 기준 현재 OS의 알려진 CJK 폰트 경로를 먼저 확인하고, 그 뒤 다른 OS 후보도
   fallback으로 확인한다.
4. 위 후보가 모두 실패할 때만 기존처럼 `ImageFont.load_default()`를 사용한다.

## 회귀 테스트

시스템 폰트 설치 상태에 직접 의존하지 않도록 `scripts/tests/test_visual_sweep.py`에서 다음 계약을 mock으로
검증한다.

- `fc-match`가 반환한 기존 파일 경로를 `fontconfig_label_font_path()`가 선택한다.
- 환경변수 지정 폰트가 fontconfig 결과보다 앞서며, OS별 path separator로 여러 후보를 받을 수 있다.
- 현재 OS의 알려진 font path 후보가 다른 OS 후보보다 먼저 평가된다.
- 환경변수·fontconfig·고정 경로가 같은 파일을 가리키면 중복 후보를 제거한다.
- `label_font()`는 후보 TrueType 폰트를 먼저 로드하고, 성공하면 Pillow 기본 폰트로 fallback하지 않는다.

## 영향 범위

제품 렌더러나 HWP/PDF 변환 결과는 변경하지 않는다. 변경 범위는 visual sweep 증적 PNG의 라벨 폰트 선택과
해당 Python 단위 테스트에 한정된다.
