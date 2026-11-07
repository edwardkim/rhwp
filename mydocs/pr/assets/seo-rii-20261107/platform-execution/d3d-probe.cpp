#include <windows.h>
#include <d3d12.h>
#include <dxgi1_4.h>
#include <wrl/client.h>
#include <cstdint>
#include <cstring>
#include <sstream>
#include <stdexcept>
#include <string>
#include "rhwp_native_ffi.h"
#include "NativeReply.h"
using Microsoft::WRL::ComPtr;
using rhwp::d3d::NativeReply;
void check(HRESULT hr, const char* text) { if (FAILED(hr)) throw std::runtime_error(text); }
std::string read(char* p) { if (!p) throw std::runtime_error("null native response"); std::string s(p); rhwp_string_free(p); return s; }
void wait_queue(ID3D12Device* device, ID3D12CommandQueue* queue) {
  ComPtr<ID3D12Fence> fence; check(device->CreateFence(0, D3D12_FENCE_FLAG_NONE, IID_PPV_ARGS(&fence)), "create fence");
  HANDLE event = CreateEventW(nullptr, FALSE, FALSE, nullptr);
  if (!event) throw std::runtime_error("create event");
  check(queue->Signal(fence.Get(), 1), "signal fence");
  check(fence->SetEventOnCompletion(1, event), "set fence event");
  DWORD result = WaitForSingleObject(event, 10000); CloseHandle(event);
  if (result != WAIT_OBJECT_0) throw std::runtime_error("verification copy fence timeout");
}
extern "C" __declspec(dllexport) int rhwp_review_d3d_probe(const char* request, char* output, size_t capacity) {
  std::string result; int code = 0; uint64_t presenter = 0;
  try {
    ComPtr<IDXGIFactory4> factory; check(CreateDXGIFactory1(IID_PPV_ARGS(&factory)), "DXGI factory");
    ComPtr<IDXGIAdapter1> adapter; ComPtr<ID3D12Device> device; bool software = false;
    for (UINT i = 0; i < 16; ++i) {
      ComPtr<IDXGIAdapter1> candidate; if (factory->EnumAdapters1(i, &candidate) == DXGI_ERROR_NOT_FOUND) break;
      DXGI_ADAPTER_DESC1 desc{}; check(candidate->GetDesc1(&desc), "adapter desc");
      if (desc.Flags & DXGI_ADAPTER_FLAG_SOFTWARE) continue;
      if (SUCCEEDED(D3D12CreateDevice(candidate.Get(), D3D_FEATURE_LEVEL_11_0, IID_PPV_ARGS(&device)))) { adapter = candidate; break; }
    }
    if (!device) { software = true; check(factory->EnumWarpAdapter(IID_PPV_ARGS(&adapter)), "WARP adapter"); check(D3D12CreateDevice(adapter.Get(), D3D_FEATURE_LEVEL_11_0, IID_PPV_ARGS(&device)), "WARP device"); }
    ComPtr<ID3D12CommandQueue> queue; D3D12_COMMAND_QUEUE_DESC q{}; q.Type = D3D12_COMMAND_LIST_TYPE_DIRECT;
    check(device->CreateCommandQueue(&q, IID_PPV_ARGS(&queue)), "command queue");
    auto created = read(rhwp_d3d_presenter_create(adapter.Get(), device.Get(), queue.Get(), 1, reinterpret_cast<const uint8_t*>("{}"), 2));
    if (!NativeReply(created).field("ok").boolean()) throw std::runtime_error(created);
    presenter = NativeReply(created).field("presenter").integer();
    D3D12_HEAP_PROPERTIES heap{}; heap.Type = D3D12_HEAP_TYPE_DEFAULT; heap.CreationNodeMask = heap.VisibleNodeMask = 1;
    D3D12_RESOURCE_DESC desc{}; desc.Dimension = D3D12_RESOURCE_DIMENSION_TEXTURE2D; desc.Width = desc.Height = 4;
    desc.DepthOrArraySize = desc.MipLevels = 1; desc.Format = DXGI_FORMAT_B8G8R8A8_UNORM; desc.SampleDesc.Count = 1;
    desc.Flags = D3D12_RESOURCE_FLAG_ALLOW_RENDER_TARGET | D3D12_RESOURCE_FLAG_ALLOW_SIMULTANEOUS_ACCESS;
    ComPtr<ID3D12Resource> texture; check(device->CreateCommittedResource(&heap, D3D12_HEAP_FLAG_SHARED, &desc, D3D12_RESOURCE_STATE_COMMON, nullptr, IID_PPV_ARGS(&texture)), "target resource");
    auto rendered = read(rhwp_d3d_presenter_render(presenter, texture.Get(), reinterpret_cast<const uint8_t*>(request), std::strlen(request)));
    if (!NativeReply(rendered).field("ok").boolean()) throw std::runtime_error(rendered);
    if (!NativeReply(rendered).field("frame").field("producerGpuComplete").boolean()) throw std::runtime_error("producer incomplete");
    std::string stale_request(request);
    const std::string view_token = "\"viewGeneration\": 1";
    auto view_offset = stale_request.find(view_token);
    if (view_offset == std::string::npos) throw std::runtime_error("missing probe stamp token");
    stale_request.replace(view_offset, view_token.size(), "\"viewGeneration\": 2");
    auto stale = read(rhwp_d3d_presenter_render(presenter, texture.Get(), reinterpret_cast<const uint8_t*>(stale_request.data()), stale_request.size()));
    if (NativeReply(stale).field("ok").boolean()) throw std::runtime_error("stale view unexpectedly rendered");
    D3D12_PLACED_SUBRESOURCE_FOOTPRINT footprint{}; UINT rows = 0; UINT64 row_size = 0, bytes = 0;
    device->GetCopyableFootprints(&desc, 0, 1, 0, &footprint, &rows, &row_size, &bytes);
    heap.Type = D3D12_HEAP_TYPE_READBACK; D3D12_RESOURCE_DESC bd{}; bd.Dimension = D3D12_RESOURCE_DIMENSION_BUFFER;
    bd.Width = bytes; bd.Height = bd.DepthOrArraySize = bd.MipLevels = 1; bd.SampleDesc.Count = 1; bd.Layout = D3D12_TEXTURE_LAYOUT_ROW_MAJOR;
    ComPtr<ID3D12Resource> readback; check(device->CreateCommittedResource(&heap, D3D12_HEAP_FLAG_NONE, &bd, D3D12_RESOURCE_STATE_COPY_DEST, nullptr, IID_PPV_ARGS(&readback)), "verification readback resource");
    ComPtr<ID3D12CommandAllocator> allocator; ComPtr<ID3D12GraphicsCommandList> list;
    check(device->CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT, IID_PPV_ARGS(&allocator)), "copy allocator");
    check(device->CreateCommandList(0, D3D12_COMMAND_LIST_TYPE_DIRECT, allocator.Get(), nullptr, IID_PPV_ARGS(&list)), "copy list");
    D3D12_TEXTURE_COPY_LOCATION src{}; src.pResource = texture.Get(); src.Type = D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX;
    D3D12_TEXTURE_COPY_LOCATION dst{}; dst.pResource = readback.Get(); dst.Type = D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT; dst.PlacedFootprint = footprint;
    list->CopyTextureRegion(&dst, 0, 0, 0, &src, nullptr); check(list->Close(), "close copy list");
    ID3D12CommandList* lists[] = {list.Get()}; queue->ExecuteCommandLists(1, lists); wait_queue(device.Get(), queue.Get());
    void* raw = nullptr; D3D12_RANGE range{0, static_cast<SIZE_T>(bytes)}; check(readback->Map(0, &range, &raw), "map verification pixels");
    const auto* b = static_cast<const uint8_t*>(raw); std::ostringstream samples;
    samples << '[';
    for (UINT y = 0; y < 4; ++y) for (UINT x = 0; x < 4; ++x) {
      if (x || y) samples << ','; const auto* px = b + footprint.Offset + y * footprint.Footprint.RowPitch + x * 4;
      samples << '[' << unsigned(px[0]) << ',' << unsigned(px[1]) << ',' << unsigned(px[2]) << ',' << unsigned(px[3]) << ']';
    }
    samples << ']'; D3D12_RANGE none{0, 0}; readback->Unmap(0, &none);
    auto released = read(rhwp_d3d_presenter_request(presenter, reinterpret_cast<const uint8_t*>("{\"op\":\"release\"}"), 16));
    if (!NativeReply(released).field("ok").boolean()) throw std::runtime_error(released);
    presenter = 0;
    result = "{\"ok\":true,\"softwareAdapter\":" + std::string(software ? "true" : "false") + ",\"verificationReadbacks\":1,\"native\":" + rendered + ",\"staleRejected\":" + stale + ",\"bgraSamples\":" + samples.str() + '}';
  } catch (const std::exception& error) {
    code = 1; result = error.what();
    if (presenter) { auto ignored = read(rhwp_d3d_presenter_request(presenter, reinterpret_cast<const uint8_t*>("{\"op\":\"abandon\"}"), 16)); }
  }
  if (result.size() + 1 > capacity) return 2;
  std::memcpy(output, result.c_str(), result.size() + 1); return code;
}
