## 메인터너 통합·검증

최신 devel의 공통 삭제·붙여넣기, Storage 자원 예산, Native 기준선·첫 줄 fit을 보존하며 원 기여의
논리 편집·범위 재조판·다단 기능을 통합했습니다. 원 기여 13커밋을 유지했고, 저장 표 조각의
문단 소유와 물리 프레임을 함께 보정해 KPS 65·66쪽의 테두리·행 위치를 복원했습니다.

동일 입력·글꼴과 독립 한컴 Print를 사용한 Native/fresh WASM 각각 **14문서·516쪽**에서 raw가 모두
90% 이상이고 기준 devel 대비 하락은 0쪽입니다. KPS 66쪽은 Native **99.99427%**, WASM **99.99510%**입니다.
저장 KPS HWPX도 원본과 양 backend 각각 77쪽 PNG가 동일합니다. 원본 giant 두 문서는 115/197쪽 전체를
검증했습니다. 글꼴/래스터 차이는 검토 기록에 구분했으며 글꼴 파일은 배포하지 않습니다.

[검토 기록](https://github.com/edwardkim/rhwp/blob/devel/mydocs/pr/archives/pr_8187_review.md),
[쪽별 원본 TSV·요약·대표 PNG](https://github.com/edwardkim/rhwp/blob/devel/mydocs/pr/assets/pr8187/README.md).

전체 회귀 12,415건, Native Skia 공식 3종 4,064건, 집중 회귀 72건과 모든 Rust lint를 통과했습니다.
실제 Chromium API 36건, Studio 2,716건과 TypeScript도 통과했습니다. Studio의 기존 Node pkg 전용
2건은 해당 빌드 부재로 skip됐으며 optimized WASM 브라우저 검증은 별도로 완료했습니다.
최종 head `8c97771760`의 CI·self-review 성공 후 merge `3724ab9dc4`로 통합했습니다.
#7444의 전체 Studio 선택·음영·서식 연결까지 해결한 것으로 종료하지 않습니다.

CI의 Color Emoji 환경에서 발견한 기존 이미지 자원 과집계는 테스트 글꼴을 공개 Noto Sans KR로 명시해 보정했습니다. 제품 코드·원본 이미지·정확히 21개 기대값은 유지했고 최소 글꼴·Color Emoji 반복 검사와 Rust lint를 통과했습니다.
