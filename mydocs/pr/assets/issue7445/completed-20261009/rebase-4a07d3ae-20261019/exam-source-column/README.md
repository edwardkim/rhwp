# 시험 원본 단의 표 소유·글꼴 체인 검사

원PR8064의 Print96DPI/2px 조건으로 고정base4a07 Native44쪽과 후보 Native/fresh WASM 각각44쪽을 비교했다.
전쪽90% 이상·4/20/20쪽수 일치·정식83PASS다. 과학3쪽은base95.06581→94.73771로0.32810%p하락하여 **PR 보류**다.
192DPI 진단은 별도로 보존하며 Native 미달과WASM ENOSPC를 통과 증거로 사용하지 않는다.

원본 입력/PDF/글꼴 SHA와 생산자는 producer·validation.json, 원문 TSV는 backend/문서명 파일을 따른다.
과학의 마지막 표는2쪽 왼쪽 단에 남고10번 문제는같은쪽오른쪽단에서시작한다.
국어의 본문·구두점은 같은 기준선을 유지한다. 표시 fallback체인은원본첫face소유를없애지않는다.

[과학2쪽 review](native-science-p002-review.png) · [overlay](native-science-p002-overlay.png)

[국어14쪽 review](native-kor-hwp-p014-review.png) · [overlay](native-kor-hwp-p014-overlay.png)

[잔여 과학3쪽 review](native-science-p003-review.png) · [overlay](native-science-p003-overlay.png)
