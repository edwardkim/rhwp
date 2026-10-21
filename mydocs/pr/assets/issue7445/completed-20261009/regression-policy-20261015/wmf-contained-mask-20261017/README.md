# WMF 안쪽 흑백 마스크의 원본 색 소유

Issue: #7445

원본 `samples/hwp3-sample16-hwp5.hwp`와 사용자가 제공한 독립 한컴2024 Print
`pdf/hwp3-sample16-hwp5-lossless-2024.pdf` 전64쪽을 같은192DPI/compat2024에서 비교했다.
`9415b920` 리베이스 후 `ac5d062c`+WMF 포함 마스크 후보의 실제 Native/fresh WASM PNG다.

## 원인과 범위

BIN0003.WMF의 두 동일 PATINVERT 사이에 DPA 흑백 패턴이 있다. 안쪽 사각형
`[3823,387,2431,1161]`은 바깥 `[3822,387,2432,1161]`에 완전히 포함된다.
기존 코드는 이 관계를 특정8×8 checker 무늬에서만 인정하여, 다른 유효한 흑백 무늬가
분홍색 브러시를 회색으로 덮었다. 안쪽은 `(D XOR P) AND M XOR P`, 바깥은
`D XOR P XOR P`이므로 완전 포함이면 무늬 내용·크기와 무관하게 역흑백 mask가
원본 브러시를 선택한다. 좌표 확대나 문서별 색상 예외가 아니다.

실물 원본 WMF SHA1927d7c64d345e4467f5936b3e9a28cfe1b395bf5a2301f5fef6fb1ff829cb5d와
Windows GDI+ Metafile.DrawImage의 [원본 그림](windows-gdiplus-original-wmf.png)을 대조했다.
GDI+의 변환 후 header 분류를 원본 EMF라는 근거로 쓰지 않았다. 이 그림의 원본 바이트는
표준 WMF이며 사용자 HWP와 PDF는 보존했다. GDI+ 이미지는 전체 HWP Print의 대체 기준이 아니다.

## 시각 결과

Native 전64쪽 최저93.07399%,WASM95.87967%,각 미달0·쪽수64/64다.
18쪽 raw는98.01466/97.96086%이며 분홍색 패널과 소속 마스크를 양쪽 review/overlay에서
직접 확인했다. 미세 패턴·윤곽 차이는 남는다. 양쪽98%가 아니므로18쪽을 px로 고정하지 않는다.
19쪽의 기존 중요한 제목은99.43242/99.61477%를 유지하고 새 그림으로 직접 판독했다.
이전 candidate 전64쪽 대비0.01%p 하락0쪽은 [별도 비교](verification.md)이며
새 upstream과의 비교 통과를 뜻하지 않는다. 새 devel 자체는65쪽이라 전체 비교가 불일치한다.

| 항목 | review | overlay |
|---|---|---|
| 이전 Native18쪽 | [review](before-native-p018-review.png) | [overlay](before-native-p018-overlay.png) |
| 보정 Native18쪽 | [review](after-native-p018-review.png) | [overlay](after-native-p018-overlay.png) |
| 보정 WASM18쪽 | [review](after-wasm-p018-review.png) | [overlay](after-wasm-p018-overlay.png) |
| Native19쪽 유지 | [review](after-native-p019-review.png) | [overlay](after-native-p019-overlay.png) |
| WASM19쪽 유지 | [review](after-wasm-p019-review.png) | [overlay](after-wasm-p019-overlay.png) |

[Native 전쪽 TSV](native-all64.tsv) · [WASM 전쪽 TSV](wasm-all64.tsv) · [검증 상태](verification.md)

정식 WMF17검사는 수정 전15PASS/2FAIL(0.137초) → 수정 후17PASS(0.097초)다.
정상 비checker 바깥 마스크·변경된 XOR·색상 팔레트·중단 연산을 유지하며 새 실물 마스크 소유를 확인했다.
리베이스된 글꼴/탭16검사도16PASS다. 실제 source가 동결 PNG producer와 같은지 SHA로 확인했다.
최신 source 전체 회귀·48조건·실제 소비 경로는 아직 대기 중이다.
이 단계는 PR 준비 완료가 아니다.
