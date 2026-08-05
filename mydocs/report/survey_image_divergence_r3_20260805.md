# 이미지 발산 스윕 r3 — 잔여 4부류 수정 후 flagged 33→10

- **표본**: `hwpdocs/` 전수 10,000건 — [r2](survey_image_divergence_r2_20260805.md) 동일
- **바이너리**: `fix/image-conversion-remainder 843f54399` (#4060 stack 위에 PCX #4065 ·
  WMF #4063 · TIFF/BMP #4064 · DOS EPS #4062 일부), release
- **실행 환경**: `RHWP_FONT_PATH=ttfs/hwp:ttfs/windows`, 8갈래 xargs (n=25)
- **하네스**: [이미지 발산 스윕 가이드](../manual/verification/image_divergence_sweep.md)

## 결과

| 지표 | r1 (수정 전) | r2 (EMF 후) | **r3** |
|---|---|---|---|
| flagged 문서 | 131 | 33 | **10** |
| 신규 악화 | — | 0 | **0** |
| 파서 실패(`ok:false`) | 52 | 52 | 52 |

부류별 해소: WMF 변환실패 6문서(폰트 힌트 관용) · TIFF 11문서(팔레트 폴백 +
svg 경로 배선) · 초대형 BMP 4문서(다운스케일) · DOS EPS 2문서(내장 TIFF 프리뷰) ·
PCX v2.8 1문서(판별) — 총 23문서 해제, r2 목록 기준 전이 확인.

## 잔여 10건 (예측과 일치)

- **텍스트 PostScript 9문서** — mime 은 `application/postscript` 로 정직해졌지만
  변환기(인터프리터)가 없어 여전히 그릴 수 없다. #4062 의 잔여 스코프.
- **손상 헤더 BMP 1문서** (기획재정부 선진통상국가용역보고서) — 헤더가
  w=16,318,939 를 담은 깨진 바이트라 거부가 정답. 경성 한도(한도의 8배) 밖
  치수는 다운스케일 폴백에서도 의도적으로 거부한다.

## 시각 증적 (export-png, native-skia)

- 성동구 표창장 별지서식: 의회 휘장·직인 씰(팔레트 TIFF 2장) 정상 렌더
- 재정경제부 2012 경제정책방향 0-based p21: 【그림 2-2】 가로 막대 차트(WMF,
  FontQuality 0x06 관용 후) 정상 렌더
- (r2 문서의 함평군 시추보고서 PCX·EDCF EMF 증적은 그대로 유효)
