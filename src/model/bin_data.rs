//! 바이너리 데이터 (BinData, 이미지/OLE 참조)

/// BinData 개별 항목의 압축 해제 상한 (바이트).
///
/// [#2550] 저장·클립보드·렌더 경로가 무제한 `load()` 로 deflate bomb 에 노출되어
/// 있었다. 폰트 전용 상한(`MAX_EMBEDDED_FONT_BYTES` 32MB)보다 큰 실문서 임베드
/// (이미지·OLE)가 존재하므로, HWP3 레코드 상한(`check_record_count` 의
/// `HWP3_MAX_RECORD_SIZE`)과 같은 256MB 로 정합한다.
///
/// 초과 시 동작은 경로별로 다르다:
/// - 렌더·클립보드·질의: placeholder (누락 이미지와 동일 경로)
/// - HWP5 저장: 압축 해제 없이 원본 저장 바이트 통과 ([`BinDataBytes::load_raw`])
pub const MAX_BIN_DATA_BYTES: usize = 256 * 1024 * 1024;

/// 원본 컨테이너에 저장된 형태 그대로의 BinData 바이트 (압축 해제 없음).
///
/// [#2550] 상한 초과 항목을 저장 경로가 손실 없이 통과시킬 때 쓴다. 저장 형태를
/// 그대로 다시 쓰려면 **그 바이트가 압축된 상태인지**를 함께 알아야 한다 — 소비 측이
/// 기대하는 압축 상태와 다르면 압축 바이트를 원본으로 오해해 조용히 깨지기 때문이다.
#[derive(Debug, Clone)]
pub struct StoredBinData {
    /// 저장된 형태의 바이트. 암호 문서는 복호화(크기 1:1)만 거친다.
    pub bytes: Vec<u8>,
    /// `bytes` 가 압축된 상태인지 여부.
    pub compressed: bool,
}

/// 바이너리 데이터 아이템 (HWPTAG_BIN_DATA)
#[derive(Debug, Clone, Default)]
pub struct BinData {
    /// 원본 레코드 바이트 (라운드트립 보존용)
    pub raw_data: Option<Vec<u8>>,
    /// 속성 비트 플래그
    pub attr: u16,
    /// 데이터 타입
    pub data_type: BinDataType,
    /// 압축 방식
    pub compression: BinDataCompression,
    /// 접근 상태
    pub status: BinDataStatus,
    /// 연결 파일 절대 경로 (LINK 타입)
    pub abs_path: Option<String>,
    /// 연결 파일 상대 경로 (LINK 타입)
    pub rel_path: Option<String>,
    /// BinData 스토리지 내 ID (EMBEDDING/STORAGE 타입)
    pub storage_id: u16,
    /// 확장자 (EMBEDDING 타입: jpg, bmp, png 등)
    pub extension: Option<String>,
}

/// 바이너리 데이터 타입
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum BinDataType {
    #[default]
    /// 외부 파일 참조
    Link,
    /// 파일 포함
    Embedding,
    /// OLE 포함
    Storage,
}

/// 바이너리 데이터 압축 방식
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum BinDataCompression {
    #[default]
    /// 스토리지 디폴트
    Default,
    /// 무조건 압축
    Compress,
    /// 무조건 비압축
    NoCompress,
}

/// 바이너리 데이터 접근 상태
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum BinDataStatus {
    #[default]
    /// 아직 접근하지 않음
    NotAccessed,
    /// 접근 성공
    Success,
    /// 접근 실패
    Error,
    /// 접근 실패했으나 무시됨
    Ignored,
}

/// BinData 스토리지에서 로드된 실제 데이터
#[derive(Debug, Clone)]
pub struct BinDataContent {
    /// 스토리지 ID
    pub id: u16,
    /// 바이너리 데이터
    pub data: BinDataBytes,
    /// 파일 확장자
    pub extension: String,
}

/// 지연 로딩 대상 BinData 를 원본 컨테이너에서 다시 읽어오는 주체.
///
/// [Task #2263] 압축 해제된 이미지 바이트를 IR 에 상주시키지 않기 위해,
/// 원본 컨테이너(HWPX ZIP / HWP5 CFB)를 보유한 파서 측이 이 트레이트를
/// 구현하고, 실제 바이트가 필요한 시점에만 압축을 푼다.
pub trait BinDataResolver:
    std::fmt::Debug + Send + Sync + std::panic::RefUnwindSafe + std::panic::UnwindSafe
{
    /// `key` 가 가리키는 BinData 의 바이트를 압축 해제하여 반환한다.
    ///
    /// 원본이 손상되었거나 엔트리가 없으면 빈 벡터를 반환한다
    /// (파싱 시점의 placeholder 의미를 그대로 유지한다).
    fn resolve(&self, key: &str) -> Vec<u8>;

    /// 공유 가능한 바이트를 반환한다.
    ///
    /// 컨테이너 리졸버의 기존 `Vec` 계약은 유지한다. 같은 할당을 이미 공유하는
    /// 리졸버만 이 메서드를 재정의하면 `Vec -> Arc<[u8]>` 재할당을 피할 수 있다.
    fn resolve_shared(&self, key: &str) -> std::sync::Arc<[u8]> {
        self.resolve(key).into()
    }

    /// 최대 `max_bytes` 바이트까지만 materialize하여 반환한다.
    ///
    /// 기본 구현은 안전하게 실패한다. 컨테이너별 리졸버가 압축 해제 경계에서
    /// 상한을 보장할 수 있을 때만 이 메서드를 구현해야 한다.
    fn resolve_limited(&self, _key: &str, _max_bytes: usize) -> Option<Vec<u8>> {
        None
    }

    /// 공유 가능한 bounded read 결과를 반환한다.
    fn resolve_limited_shared(&self, key: &str, max_bytes: usize) -> Option<std::sync::Arc<[u8]>> {
        self.resolve_limited(key, max_bytes).map(Into::into)
    }

    /// 바이트 길이만 필요한 호출을 위한 경로다.
    fn resolved_len(&self, key: &str) -> usize {
        self.resolve(key).len()
    }

    /// 빈 항목인지 판정하기 위한 경로다.
    fn resolved_is_empty(&self, key: &str) -> bool {
        self.resolve(key).is_empty()
    }

    /// 원본 컨테이너에 저장된 형태 그대로의 바이트를 반환한다 (압축 해제 없음).
    ///
    /// [#2550] 저장 경로가 상한 초과 항목(deflate bomb 포함)을 데이터 손실 없이
    /// 통과시키기 위한 경로다. 복호화(크기 1:1)는 수행하되 압축 해제는 하지 않으므로
    /// 반환 크기는 저장된 스트림 크기에 묶인다. 저장 형태를 노출할 수 없는
    /// 컨테이너는 `None` 을 반환한다 (기본값).
    fn resolve_raw(&self, _key: &str) -> Option<StoredBinData> {
        None
    }
}

/// 세션 중 추가된 바이트를 기존 공개 enum 모양을 바꾸지 않고 공유한다.
///
/// `BinDataBytes`의 tuple variant와 `load()` 반환형은 Rust 공개 계약이므로 그대로
/// 두고, 저장소가 소유하는 신규 바이트만 기존 `Lazy` resolver 슬롯을 통해 Arc로
/// 보관한다. 외부 코드가 직접 만든 `Loaded(Vec<u8>)`의 관찰 의미도 바뀌지 않는다.
struct InMemorySharedBinDataResolver {
    bytes: std::sync::Arc<[u8]>,
}

impl std::fmt::Debug for InMemorySharedBinDataResolver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InMemorySharedBinDataResolver")
            .field("len", &self.bytes.len())
            .finish()
    }
}

impl BinDataResolver for InMemorySharedBinDataResolver {
    fn resolve(&self, _key: &str) -> Vec<u8> {
        self.bytes.to_vec()
    }

    fn resolve_shared(&self, _key: &str) -> std::sync::Arc<[u8]> {
        std::sync::Arc::clone(&self.bytes)
    }

    fn resolve_limited(&self, _key: &str, max_bytes: usize) -> Option<Vec<u8>> {
        (self.bytes.len() <= max_bytes).then(|| self.bytes.to_vec())
    }

    fn resolve_limited_shared(&self, _key: &str, max_bytes: usize) -> Option<std::sync::Arc<[u8]>> {
        (self.bytes.len() <= max_bytes).then(|| std::sync::Arc::clone(&self.bytes))
    }

    fn resolved_len(&self, _key: &str) -> usize {
        self.bytes.len()
    }

    fn resolved_is_empty(&self, _key: &str) -> bool {
        self.bytes.is_empty()
    }
}

/// BinData 바이트의 보관 방식.
///
/// [Task #2263] 파싱 시점에 모든 내장 이미지를 압축 해제해 상주시키면
/// 원본 파일 크기의 수십 배에 달하는 메모리를 쓰게 된다. `Lazy` 는 원본
/// 컨테이너만 보유하고 실제 요청 시점에 해당 항목만 압축을 푼다.
#[derive(Debug, Clone)]
pub enum BinDataBytes {
    /// 메모리에 이미 올라온 바이트 (공개 Rust API 호환용 소유 `Vec`).
    Loaded(Vec<u8>),
    /// 원본 컨테이너에서 요청 시 압축 해제
    Lazy {
        /// 원본 컨테이너를 보유한 리졸버 (문서 내 모든 항목이 공유)
        resolver: std::sync::Arc<dyn BinDataResolver>,
        /// 리졸버가 해석하는 항목 키 (HWPX: ZIP 엔트리 경로, HWP5: 스토리지 스트림명)
        key: String,
    },
}

impl BinDataBytes {
    /// 저장소 내부에서 신규 바이트를 공유 보관한다.
    ///
    /// 기존 `Loaded(Vec<u8>)` variant를 바꾸거나 새 enum variant를 추가하면 외부의
    /// constructor 및 exhaustive match가 깨진다. 따라서 공유 저장은 기존 resolver
    /// 슬롯을 사용하고, 호출자는 `load_shared()`로 같은 할당을 받을 수 있다.
    pub fn from_shared(data: impl Into<std::sync::Arc<[u8]>>) -> Self {
        Self::Lazy {
            resolver: std::sync::Arc::new(InMemorySharedBinDataResolver { bytes: data.into() }),
            key: "<in-memory>".to_string(),
        }
    }

    /// 바이트를 얻는다. `Lazy` 인 경우 이 시점에 압축을 푼다.
    pub fn load(&self) -> Vec<u8> {
        match self {
            BinDataBytes::Loaded(v) => v.clone(),
            BinDataBytes::Lazy { resolver, key } => resolver.resolve(key),
        }
    }

    /// 같은 할당을 공유할 수 있는 내부 경로다.
    ///
    /// `from_shared()`로 만든 값은 Arc refcount만 증가한다. 기존 공개
    /// `Loaded(Vec<u8>)`와 압축 컨테이너 resolver는 필요할 때만 Arc로 옮긴다.
    pub fn load_shared(&self) -> std::sync::Arc<[u8]> {
        match self {
            BinDataBytes::Loaded(v) => std::sync::Arc::from(v.as_slice()),
            BinDataBytes::Lazy { resolver, key } => resolver.resolve_shared(key),
        }
    }

    /// 최대 `max_bytes` 바이트까지만 로드한다.
    ///
    /// `Loaded` 는 복제 전에 길이를 확인하고, `Lazy` 는 리졸버가 제공하는
    /// bounded read/decompression 경로만 사용한다.
    pub fn load_limited(&self, max_bytes: usize) -> Option<Vec<u8>> {
        match self {
            BinDataBytes::Loaded(v) if v.len() <= max_bytes => Some(v.clone()),
            BinDataBytes::Loaded(_) => None,
            BinDataBytes::Lazy { resolver, key } => resolver
                .resolve_limited(key, max_bytes)
                .filter(|bytes| bytes.len() <= max_bytes),
        }
    }

    /// 최대 크기를 지키면서 공유 가능한 바이트를 얻는다.
    pub fn load_limited_shared(&self, max_bytes: usize) -> Option<std::sync::Arc<[u8]>> {
        match self {
            BinDataBytes::Loaded(v) if v.len() <= max_bytes => {
                Some(std::sync::Arc::from(v.as_slice()))
            }
            BinDataBytes::Loaded(_) => None,
            BinDataBytes::Lazy { resolver, key } => resolver
                .resolve_limited_shared(key, max_bytes)
                .filter(|bytes| bytes.len() <= max_bytes),
        }
    }

    /// 원본 컨테이너에 저장된 형태 그대로의 바이트를 얻는다 (압축 해제 없음).
    ///
    /// [#2550] `load_limited` 가 상한 초과로 실패한 항목을 저장 경로가 데이터 손실
    /// 없이 통과시킬 때 사용한다. `Loaded` 는 이미 해제된 메모리 바이트라 저장
    /// 형태가 없으므로 `None` 이다 (해당 항목은 폭탄 위험도 없다).
    pub fn load_raw(&self) -> Option<StoredBinData> {
        match self {
            BinDataBytes::Loaded(_) => None,
            BinDataBytes::Lazy { resolver, key } => resolver.resolve_raw(key),
        }
    }

    /// 바이트 길이. `Lazy` 인 경우 압축 해제가 발생하므로 렌더 경로의
    /// 반복 호출은 피하고 `load()` 결과를 재사용한다.
    pub fn len(&self) -> usize {
        match self {
            BinDataBytes::Loaded(v) => v.len(),
            BinDataBytes::Lazy { resolver, key } => resolver.resolved_len(key),
        }
    }

    /// 빈 항목인지 판정한다.
    ///
    /// `Lazy` 는 "원본 컨테이너에 엔트리가 있을 것"이라는 기대일 뿐 보장이 아니다.
    /// 매니페스트에는 있으나 실제 엔트리가 없거나(엔트리 누락) 읽기에 실패하는
    /// 경우([#1917] 상한 초과 등) 리졸버가 빈 바이트를 반환하므로, 여기서
    /// 실제로 해석해 봐야 placeholder 의미가 보존된다.
    pub fn is_empty(&self) -> bool {
        match self {
            BinDataBytes::Loaded(v) => v.is_empty(),
            BinDataBytes::Lazy { resolver, key } => resolver.resolved_is_empty(key),
        }
    }
}

impl Default for BinDataBytes {
    fn default() -> Self {
        BinDataBytes::Loaded(Vec::new())
    }
}

impl From<Vec<u8>> for BinDataBytes {
    fn from(v: Vec<u8>) -> Self {
        BinDataBytes::Loaded(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn legacy_loaded_and_from_vec_keep_the_vec_variant_contract() {
        let direct = BinDataBytes::Loaded(vec![1, 2, 3]);
        let converted: BinDataBytes = vec![4, 5, 6].into();

        assert!(matches!(direct, BinDataBytes::Loaded(ref bytes) if bytes == &[1, 2, 3]));
        assert!(matches!(converted, BinDataBytes::Loaded(ref bytes) if bytes == &[4, 5, 6]));
        let loaded: Vec<u8> = direct.load();
        let limited: Option<Vec<u8>> = direct.load_limited(3);
        assert_eq!(loaded, vec![1, 2, 3]);
        assert_eq!(limited, Some(vec![1, 2, 3]));
    }

    #[test]
    fn shared_in_memory_bytes_keep_one_allocation_across_clones() {
        let bytes = BinDataBytes::from_shared(vec![1, 2, 3, 4]);
        let cloned = bytes.clone();

        let first = bytes.load_shared();
        let second = cloned.load_shared();
        assert!(std::sync::Arc::ptr_eq(&first, &second));
        assert_eq!(bytes.load(), vec![1, 2, 3, 4]);
        assert_eq!(bytes.load_limited(4), Some(vec![1, 2, 3, 4]));

        let limited_first = bytes.load_limited_shared(4).expect("within limit");
        let limited_second = cloned.load_limited_shared(4).expect("within limit");
        assert!(std::sync::Arc::ptr_eq(&limited_first, &limited_second));
        assert!(bytes.load_limited_shared(3).is_none());
    }

    #[test]
    fn document_snapshot_clone_shares_in_memory_bin_data() {
        let mut document = crate::model::document::Document::default();
        document.bin_data_content.push(BinDataContent {
            id: 1,
            data: BinDataBytes::from_shared(vec![0x89, b'P', b'N', b'G']),
            extension: "png".to_string(),
        });

        let snapshot = document.clone();
        let current = document.bin_data_content[0].data.load_shared();
        let snapshotted = snapshot.bin_data_content[0].data.load_shared();
        assert!(std::sync::Arc::ptr_eq(&current, &snapshotted));
    }

    #[test]
    fn test_bin_data_default() {
        let bd = BinData::default();
        assert_eq!(bd.data_type, BinDataType::Link);
        assert_eq!(bd.compression, BinDataCompression::Default);
        assert_eq!(bd.status, BinDataStatus::NotAccessed);
    }

    #[test]
    fn test_bin_data_embedding() {
        let bd = BinData {
            data_type: BinDataType::Embedding,
            storage_id: 1,
            extension: Some("jpg".to_string()),
            ..Default::default()
        };
        assert_eq!(bd.data_type, BinDataType::Embedding);
        assert_eq!(bd.extension.as_deref(), Some("jpg"));
    }

    #[test]
    fn load_raw_exposes_stored_form_only_for_container_resolvers() {
        #[derive(Debug)]
        struct RawResolver;

        impl BinDataResolver for RawResolver {
            fn resolve(&self, _key: &str) -> Vec<u8> {
                vec![0; 8]
            }

            fn resolve_raw(&self, key: &str) -> Option<StoredBinData> {
                Some(StoredBinData {
                    bytes: key.as_bytes().to_vec(),
                    compressed: true,
                })
            }
        }

        let lazy = BinDataBytes::Lazy {
            resolver: std::sync::Arc::new(RawResolver),
            key: "BIN0001.jpg".to_string(),
        };
        let stored = lazy
            .load_raw()
            .expect("컨테이너 리졸버는 저장 형태를 노출한다");
        assert_eq!(stored.bytes, b"BIN0001.jpg".to_vec());
        assert!(stored.compressed);
        // Loaded/인메모리 공유 바이트는 저장 형태가 없어 None 이다 (#2550).
        assert!(BinDataBytes::Loaded(vec![1, 2]).load_raw().is_none());
        assert!(BinDataBytes::from_shared(vec![1, 2, 3])
            .load_raw()
            .is_none());
    }

    #[test]
    fn limited_lazy_load_never_falls_back_to_unbounded_resolution() {
        #[derive(Debug)]
        struct LimitedOnlyResolver {
            requested_limit: AtomicUsize,
        }

        impl BinDataResolver for LimitedOnlyResolver {
            fn resolve(&self, key: &str) -> Vec<u8> {
                panic!("bounded load must not call unbounded resolver: {key}")
            }

            fn resolve_limited(&self, _key: &str, max_bytes: usize) -> Option<Vec<u8>> {
                self.requested_limit.store(max_bytes, Ordering::SeqCst);
                Some(vec![0; max_bytes + 1])
            }
        }

        let resolver = std::sync::Arc::new(LimitedOnlyResolver {
            requested_limit: AtomicUsize::new(0),
        });
        let bytes = BinDataBytes::Lazy {
            resolver: resolver.clone(),
            key: "compressed-font".to_string(),
        };

        assert!(bytes.load_limited(16).is_none());
        assert_eq!(resolver.requested_limit.load(Ordering::SeqCst), 16);
    }
}
