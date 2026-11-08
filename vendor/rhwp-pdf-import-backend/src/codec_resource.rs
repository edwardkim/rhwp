//! The opted-in codecs consume the originating synchronous import request.
use crate::error::Result;
use crate::import_limits::ImportBudget;
use std::sync::Arc;

struct Request(Arc<ImportBudget>);

impl rhwp_codec_resource::ResourceBudget for Request {
    fn allocation_limit(&self) -> usize {
        self.0.limits.codec_allocation_bytes
    }
    fn reserve(&self, bytes: usize) -> bool {
        self.0.charge_codec_allocation(bytes).is_ok()
    }
    fn work(&self, units: usize) -> bool {
        self.0.charge_codec_work(units).is_ok()
    }
    fn reject(&self) {
        self.0.reject("image codec allocation failure");
    }
}

pub(crate) fn scope(budget: Option<&Arc<ImportBudget>>) -> Option<rhwp_codec_resource::Scope> {
    budget.map(|b| rhwp_codec_resource::Scope::enter(Arc::new(Request(b.clone()))))
}

pub(crate) fn decode<T>(
    budget: Option<&Arc<ImportBudget>>,
    operation: impl FnOnce() -> Result<T>,
) -> Result<T> {
    let _scope = scope(budget);
    let result = operation();
    // A codec parser may report EOF after a work check. Preserve the actual
    // sticky request failure instead of turning a limit into a parse error.
    if let Some(budget) = budget {
        budget.check()?;
    }
    result
}
