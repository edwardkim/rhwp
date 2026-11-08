# 글자 겹침 원본 세 건의 base/current 분석

결론·명령·검증 범위는 [통합 검토 기록의 진행76](../../../archives/semanticist21_20261008_integrated_review.md#진행-중-보정-76-실패한-세-원본의-고정-base-대조와-검사-의미-재검토)에 연결한다.

- 고정 base `21ed64071f81f35f37618717693bd30f727392c9`, current code `12d96686cdd24017b44becead2c4dfeb46867bf3`.
- rowbreak: base8쪽98.14008%, current8쪽50.59834%/18→19쪽. 새 구현 회귀이며 이관으로 면제하지 않는다.
- giant HWPX: base42쪽29.00198%, current42쪽30.58735%. 둘 다 미달이다. 실제 공개 WASM paint layer의 clip 교집합 대조에서 원시 26건 모두 가시 교차가 아님을 확인했다. 출력 피델리티 통과를 의미하지 않는다.
- #3837: base1쪽99.56102%, 전4쪽최저98.29366%. 현재 Native/fresh WASM 전4쪽도 각각99.56102/98.43065/99.68445/98.29366%, 최저98.29366%이며 두 경로 PNG4개가 모두 바이트 동일하다. 전쪽 review와 사진 날인 확대를 직접 판독했다.
- `*-review_*.png`는 실제 비교3패널, `*-overlay_*.png`는 독립 overlay다. 생성 source/입력/Print는 통합 기록을 따른다.

사진 날인 확대는 `issue3837-photo-seal-review-zoom.png`의 base/current/독립 Print 순서다. 원본이 사진 설명 앞에 선언한 날인의 교차는 Print에도 존재한다. 원형 선 종류·활자 획 차이는 남는다. 이 파일을 완전한 모양 일치 근거로 사용하지 않는다.
