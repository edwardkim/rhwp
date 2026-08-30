# Task M100 #6041 Stage 3 보고 — 실제 문서와 Draft PR 검증

- **상태**: 구현자 검증 완료, 작업지시자 결과 승인 대기
- **Draft PR**: [#6467](https://github.com/edwardkim/rhwp/pull/6467)
- **로컬 서버**: `http://127.0.0.1:4178/`

## 실제 브라우저 결과

- 34% 4쪽 실문서: 네 쪽 모두 `layerCount=1`, tier `screen`, effective DPR 2, 540×764 physical px
- `kps-ai.hwp` 25→34→36→50→60→100%: 100% retained 3쪽 모두 DPR 2. 페이지별 보수적 layer count
  2/1/1과 총 57,019,408 bytes가 예산 이내라 강등하지 않음
- `basic/KTX.hwp`: DOM에서 main/background/behind/front 실제 4 Canvas 확인, 진단 `layerCount=4`
- CanvasKit: layer count 1, raw DPR 2 유지, browser warning/error 없음
- 수정 전 #6040과 수정 후 #6041을 같은 1280×720 viewport에서 비교한 무손실 PNG 9개를 생성했다.
  세 문서 각각 34%·50%·100%이며 SSIM 범위는 0.999885~0.999979다.
- 합성 이미지의 왼쪽은 수정 전, 오른쪽은 수정 후다. 4쪽·kps는 모든 retained 페이지가 DPR 2이고,
  KTX는 세 배율 모두 `layerCount=4`, DPR 2다.

## 판정과 남은 게이트

일반 문서를 과도하게 저해상도로 만들던 고정 4-layer candidate는 폐기했다. 현재 candidate는 실제 페이지
구성이 예산을 넘을 때만 비포커스 페이지를 낮춘다. 브라우저 시간 표본은 변동이 커 속도 향상을 주장하지
않는다. 9개 수정 전/후 비교판, 상세 수치와 asset SHA는
`mydocs/pr/issue_6041_render_surface_evidence.md`에 있다.

작업지시자가 일반 문서 열기, 34%/50%/100% 줌, 포커스 이동을 직접 확인하기 전에는 이 Stage를 최종
승인으로 닫지 않고 #6042도 시작하지 않는다.
