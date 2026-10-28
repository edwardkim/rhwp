# rhwp-codec-resource

Internal fallible allocation and work hooks for the PDF codec forks in #8183.
The caller enters a synchronous, thread-local scope with the originating request's
budget. Nested scopes restore the previous budget on drop; guards cannot move to
another thread. This is not a global allocator or a process heap/RSS limit.

The `scoped` feature requires `std` and works on Native and wasm32. With that
feature disabled, helpers retain checked arithmetic and fallible allocation but
do not install a request budget. The PDF importer uses `scoped` for opted-in
codecs and checks its sticky request error before returning codec errors.
