"""[#4340 U1] 프레임워크 통합 어댑터 — 선택 의존성.

이 하위 패키지는 rhwp 를 RAG·에이전트 프레임워크에 꽂는 어댑터를 담는다:

- :mod:`rhwp.integrations.langchain` — ``RHWPLoader`` (LangChain 문서 로더)
- :mod:`rhwp.integrations.llama_index` — ``RHWPReader`` (LlamaIndex 리더)

**여기서는 아무것도 임포트하지 않는다.** 본 패키지의 "런타임 의존성 0" 계약을
지키기 위해 각 어댑터는 자기 프레임워크를 **사용 시점에** 지연 임포트하고,
미설치면 설치 힌트를 담은 ``ImportError`` 를 낸다 — 어댑터의 존재가 rhwp 설치를
무겁게 만들면 안 된다.
"""
