// <auto-generated>
//  automatically generated by the FlatBuffers compiler, do not modify
// </auto-generated>

namespace librawfilereader
{

using global::System;
using global::System.Collections.Generic;
using global::Google.FlatBuffers;

public struct ExtendedSpectrumDataT : IFlatbufferObject
{
  private Table __p;
  public ByteBuffer ByteBuffer { get { return __p.bb; } }
  public static void ValidateVersion() { FlatBufferConstants.FLATBUFFERS_25_2_10(); }
  public static ExtendedSpectrumDataT GetRootAsExtendedSpectrumDataT(ByteBuffer _bb) { return GetRootAsExtendedSpectrumDataT(_bb, new ExtendedSpectrumDataT()); }
  public static ExtendedSpectrumDataT GetRootAsExtendedSpectrumDataT(ByteBuffer _bb, ExtendedSpectrumDataT obj) { return (obj.__assign(_bb.GetInt(_bb.Position) + _bb.Position, _bb)); }
  public void __init(int _i, ByteBuffer _bb) { __p = new Table(_i, _bb); }
  public ExtendedSpectrumDataT __assign(int _i, ByteBuffer _bb) { __init(_i, _bb); return this; }

  public double Mass(int j) { int o = __p.__offset(4); return o != 0 ? __p.bb.GetDouble(__p.__vector(o) + j * 8) : (double)0; }
  public int MassLength { get { int o = __p.__offset(4); return o != 0 ? __p.__vector_len(o) : 0; } }
#if ENABLE_SPAN_T
  public Span<double> GetMassBytes() { return __p.__vector_as_span<double>(4, 8); }
#else
  public ArraySegment<byte>? GetMassBytes() { return __p.__vector_as_arraysegment(4); }
#endif
  public double[] GetMassArray() { return __p.__vector_as_array<double>(4); }
  public float Noise(int j) { int o = __p.__offset(6); return o != 0 ? __p.bb.GetFloat(__p.__vector(o) + j * 4) : (float)0; }
  public int NoiseLength { get { int o = __p.__offset(6); return o != 0 ? __p.__vector_len(o) : 0; } }
#if ENABLE_SPAN_T
  public Span<float> GetNoiseBytes() { return __p.__vector_as_span<float>(6, 4); }
#else
  public ArraySegment<byte>? GetNoiseBytes() { return __p.__vector_as_arraysegment(6); }
#endif
  public float[] GetNoiseArray() { return __p.__vector_as_array<float>(6); }
  public float Baseline(int j) { int o = __p.__offset(8); return o != 0 ? __p.bb.GetFloat(__p.__vector(o) + j * 4) : (float)0; }
  public int BaselineLength { get { int o = __p.__offset(8); return o != 0 ? __p.__vector_len(o) : 0; } }
#if ENABLE_SPAN_T
  public Span<float> GetBaselineBytes() { return __p.__vector_as_span<float>(8, 4); }
#else
  public ArraySegment<byte>? GetBaselineBytes() { return __p.__vector_as_arraysegment(8); }
#endif
  public float[] GetBaselineArray() { return __p.__vector_as_array<float>(8); }
  public float Charge(int j) { int o = __p.__offset(10); return o != 0 ? __p.bb.GetFloat(__p.__vector(o) + j * 4) : (float)0; }
  public int ChargeLength { get { int o = __p.__offset(10); return o != 0 ? __p.__vector_len(o) : 0; } }
#if ENABLE_SPAN_T
  public Span<float> GetChargeBytes() { return __p.__vector_as_span<float>(10, 4); }
#else
  public ArraySegment<byte>? GetChargeBytes() { return __p.__vector_as_arraysegment(10); }
#endif
  public float[] GetChargeArray() { return __p.__vector_as_array<float>(10); }
  public float Resolution(int j) { int o = __p.__offset(12); return o != 0 ? __p.bb.GetFloat(__p.__vector(o) + j * 4) : (float)0; }
  public int ResolutionLength { get { int o = __p.__offset(12); return o != 0 ? __p.__vector_len(o) : 0; } }
#if ENABLE_SPAN_T
  public Span<float> GetResolutionBytes() { return __p.__vector_as_span<float>(12, 4); }
#else
  public ArraySegment<byte>? GetResolutionBytes() { return __p.__vector_as_arraysegment(12); }
#endif
  public float[] GetResolutionArray() { return __p.__vector_as_array<float>(12); }
  public float SampledNoise(int j) { int o = __p.__offset(14); return o != 0 ? __p.bb.GetFloat(__p.__vector(o) + j * 4) : (float)0; }
  public int SampledNoiseLength { get { int o = __p.__offset(14); return o != 0 ? __p.__vector_len(o) : 0; } }
#if ENABLE_SPAN_T
  public Span<float> GetSampledNoiseBytes() { return __p.__vector_as_span<float>(14, 4); }
#else
  public ArraySegment<byte>? GetSampledNoiseBytes() { return __p.__vector_as_arraysegment(14); }
#endif
  public float[] GetSampledNoiseArray() { return __p.__vector_as_array<float>(14); }
  public float SampledNoiseBaseline(int j) { int o = __p.__offset(16); return o != 0 ? __p.bb.GetFloat(__p.__vector(o) + j * 4) : (float)0; }
  public int SampledNoiseBaselineLength { get { int o = __p.__offset(16); return o != 0 ? __p.__vector_len(o) : 0; } }
#if ENABLE_SPAN_T
  public Span<float> GetSampledNoiseBaselineBytes() { return __p.__vector_as_span<float>(16, 4); }
#else
  public ArraySegment<byte>? GetSampledNoiseBaselineBytes() { return __p.__vector_as_arraysegment(16); }
#endif
  public float[] GetSampledNoiseBaselineArray() { return __p.__vector_as_array<float>(16); }
  public float SampledNoiseMz(int j) { int o = __p.__offset(18); return o != 0 ? __p.bb.GetFloat(__p.__vector(o) + j * 4) : (float)0; }
  public int SampledNoiseMzLength { get { int o = __p.__offset(18); return o != 0 ? __p.__vector_len(o) : 0; } }
#if ENABLE_SPAN_T
  public Span<float> GetSampledNoiseMzBytes() { return __p.__vector_as_span<float>(18, 4); }
#else
  public ArraySegment<byte>? GetSampledNoiseMzBytes() { return __p.__vector_as_arraysegment(18); }
#endif
  public float[] GetSampledNoiseMzArray() { return __p.__vector_as_array<float>(18); }

  public static Offset<librawfilereader.ExtendedSpectrumDataT> CreateExtendedSpectrumDataT(FlatBufferBuilder builder,
      VectorOffset massOffset = default(VectorOffset),
      VectorOffset noiseOffset = default(VectorOffset),
      VectorOffset baselineOffset = default(VectorOffset),
      VectorOffset chargeOffset = default(VectorOffset),
      VectorOffset resolutionOffset = default(VectorOffset),
      VectorOffset sampled_noiseOffset = default(VectorOffset),
      VectorOffset sampled_noise_baselineOffset = default(VectorOffset),
      VectorOffset sampled_noise_mzOffset = default(VectorOffset)) {
    builder.StartTable(8);
    ExtendedSpectrumDataT.AddSampledNoiseMz(builder, sampled_noise_mzOffset);
    ExtendedSpectrumDataT.AddSampledNoiseBaseline(builder, sampled_noise_baselineOffset);
    ExtendedSpectrumDataT.AddSampledNoise(builder, sampled_noiseOffset);
    ExtendedSpectrumDataT.AddResolution(builder, resolutionOffset);
    ExtendedSpectrumDataT.AddCharge(builder, chargeOffset);
    ExtendedSpectrumDataT.AddBaseline(builder, baselineOffset);
    ExtendedSpectrumDataT.AddNoise(builder, noiseOffset);
    ExtendedSpectrumDataT.AddMass(builder, massOffset);
    return ExtendedSpectrumDataT.EndExtendedSpectrumDataT(builder);
  }

  public static void StartExtendedSpectrumDataT(FlatBufferBuilder builder) { builder.StartTable(8); }
  public static void AddMass(FlatBufferBuilder builder, VectorOffset massOffset) { builder.AddOffset(0, massOffset.Value, 0); }
  public static VectorOffset CreateMassVector(FlatBufferBuilder builder, double[] data) { builder.StartVector(8, data.Length, 8); for (int i = data.Length - 1; i >= 0; i--) builder.AddDouble(data[i]); return builder.EndVector(); }
  public static VectorOffset CreateMassVectorBlock(FlatBufferBuilder builder, double[] data) { builder.StartVector(8, data.Length, 8); builder.Add(data); return builder.EndVector(); }
  public static VectorOffset CreateMassVectorBlock(FlatBufferBuilder builder, ArraySegment<double> data) { builder.StartVector(8, data.Count, 8); builder.Add(data); return builder.EndVector(); }
  public static VectorOffset CreateMassVectorBlock(FlatBufferBuilder builder, IntPtr dataPtr, int sizeInBytes) { builder.StartVector(1, sizeInBytes, 1); builder.Add<double>(dataPtr, sizeInBytes); return builder.EndVector(); }
  public static void StartMassVector(FlatBufferBuilder builder, int numElems) { builder.StartVector(8, numElems, 8); }
  public static void AddNoise(FlatBufferBuilder builder, VectorOffset noiseOffset) { builder.AddOffset(1, noiseOffset.Value, 0); }
  public static VectorOffset CreateNoiseVector(FlatBufferBuilder builder, float[] data) { builder.StartVector(4, data.Length, 4); for (int i = data.Length - 1; i >= 0; i--) builder.AddFloat(data[i]); return builder.EndVector(); }
  public static VectorOffset CreateNoiseVectorBlock(FlatBufferBuilder builder, float[] data) { builder.StartVector(4, data.Length, 4); builder.Add(data); return builder.EndVector(); }
  public static VectorOffset CreateNoiseVectorBlock(FlatBufferBuilder builder, ArraySegment<float> data) { builder.StartVector(4, data.Count, 4); builder.Add(data); return builder.EndVector(); }
  public static VectorOffset CreateNoiseVectorBlock(FlatBufferBuilder builder, IntPtr dataPtr, int sizeInBytes) { builder.StartVector(1, sizeInBytes, 1); builder.Add<float>(dataPtr, sizeInBytes); return builder.EndVector(); }
  public static void StartNoiseVector(FlatBufferBuilder builder, int numElems) { builder.StartVector(4, numElems, 4); }
  public static void AddBaseline(FlatBufferBuilder builder, VectorOffset baselineOffset) { builder.AddOffset(2, baselineOffset.Value, 0); }
  public static VectorOffset CreateBaselineVector(FlatBufferBuilder builder, float[] data) { builder.StartVector(4, data.Length, 4); for (int i = data.Length - 1; i >= 0; i--) builder.AddFloat(data[i]); return builder.EndVector(); }
  public static VectorOffset CreateBaselineVectorBlock(FlatBufferBuilder builder, float[] data) { builder.StartVector(4, data.Length, 4); builder.Add(data); return builder.EndVector(); }
  public static VectorOffset CreateBaselineVectorBlock(FlatBufferBuilder builder, ArraySegment<float> data) { builder.StartVector(4, data.Count, 4); builder.Add(data); return builder.EndVector(); }
  public static VectorOffset CreateBaselineVectorBlock(FlatBufferBuilder builder, IntPtr dataPtr, int sizeInBytes) { builder.StartVector(1, sizeInBytes, 1); builder.Add<float>(dataPtr, sizeInBytes); return builder.EndVector(); }
  public static void StartBaselineVector(FlatBufferBuilder builder, int numElems) { builder.StartVector(4, numElems, 4); }
  public static void AddCharge(FlatBufferBuilder builder, VectorOffset chargeOffset) { builder.AddOffset(3, chargeOffset.Value, 0); }
  public static VectorOffset CreateChargeVector(FlatBufferBuilder builder, float[] data) { builder.StartVector(4, data.Length, 4); for (int i = data.Length - 1; i >= 0; i--) builder.AddFloat(data[i]); return builder.EndVector(); }
  public static VectorOffset CreateChargeVectorBlock(FlatBufferBuilder builder, float[] data) { builder.StartVector(4, data.Length, 4); builder.Add(data); return builder.EndVector(); }
  public static VectorOffset CreateChargeVectorBlock(FlatBufferBuilder builder, ArraySegment<float> data) { builder.StartVector(4, data.Count, 4); builder.Add(data); return builder.EndVector(); }
  public static VectorOffset CreateChargeVectorBlock(FlatBufferBuilder builder, IntPtr dataPtr, int sizeInBytes) { builder.StartVector(1, sizeInBytes, 1); builder.Add<float>(dataPtr, sizeInBytes); return builder.EndVector(); }
  public static void StartChargeVector(FlatBufferBuilder builder, int numElems) { builder.StartVector(4, numElems, 4); }
  public static void AddResolution(FlatBufferBuilder builder, VectorOffset resolutionOffset) { builder.AddOffset(4, resolutionOffset.Value, 0); }
  public static VectorOffset CreateResolutionVector(FlatBufferBuilder builder, float[] data) { builder.StartVector(4, data.Length, 4); for (int i = data.Length - 1; i >= 0; i--) builder.AddFloat(data[i]); return builder.EndVector(); }
  public static VectorOffset CreateResolutionVectorBlock(FlatBufferBuilder builder, float[] data) { builder.StartVector(4, data.Length, 4); builder.Add(data); return builder.EndVector(); }
  public static VectorOffset CreateResolutionVectorBlock(FlatBufferBuilder builder, ArraySegment<float> data) { builder.StartVector(4, data.Count, 4); builder.Add(data); return builder.EndVector(); }
  public static VectorOffset CreateResolutionVectorBlock(FlatBufferBuilder builder, IntPtr dataPtr, int sizeInBytes) { builder.StartVector(1, sizeInBytes, 1); builder.Add<float>(dataPtr, sizeInBytes); return builder.EndVector(); }
  public static void StartResolutionVector(FlatBufferBuilder builder, int numElems) { builder.StartVector(4, numElems, 4); }
  public static void AddSampledNoise(FlatBufferBuilder builder, VectorOffset sampledNoiseOffset) { builder.AddOffset(5, sampledNoiseOffset.Value, 0); }
  public static VectorOffset CreateSampledNoiseVector(FlatBufferBuilder builder, float[] data) { builder.StartVector(4, data.Length, 4); for (int i = data.Length - 1; i >= 0; i--) builder.AddFloat(data[i]); return builder.EndVector(); }
  public static VectorOffset CreateSampledNoiseVectorBlock(FlatBufferBuilder builder, float[] data) { builder.StartVector(4, data.Length, 4); builder.Add(data); return builder.EndVector(); }
  public static VectorOffset CreateSampledNoiseVectorBlock(FlatBufferBuilder builder, ArraySegment<float> data) { builder.StartVector(4, data.Count, 4); builder.Add(data); return builder.EndVector(); }
  public static VectorOffset CreateSampledNoiseVectorBlock(FlatBufferBuilder builder, IntPtr dataPtr, int sizeInBytes) { builder.StartVector(1, sizeInBytes, 1); builder.Add<float>(dataPtr, sizeInBytes); return builder.EndVector(); }
  public static void StartSampledNoiseVector(FlatBufferBuilder builder, int numElems) { builder.StartVector(4, numElems, 4); }
  public static void AddSampledNoiseBaseline(FlatBufferBuilder builder, VectorOffset sampledNoiseBaselineOffset) { builder.AddOffset(6, sampledNoiseBaselineOffset.Value, 0); }
  public static VectorOffset CreateSampledNoiseBaselineVector(FlatBufferBuilder builder, float[] data) { builder.StartVector(4, data.Length, 4); for (int i = data.Length - 1; i >= 0; i--) builder.AddFloat(data[i]); return builder.EndVector(); }
  public static VectorOffset CreateSampledNoiseBaselineVectorBlock(FlatBufferBuilder builder, float[] data) { builder.StartVector(4, data.Length, 4); builder.Add(data); return builder.EndVector(); }
  public static VectorOffset CreateSampledNoiseBaselineVectorBlock(FlatBufferBuilder builder, ArraySegment<float> data) { builder.StartVector(4, data.Count, 4); builder.Add(data); return builder.EndVector(); }
  public static VectorOffset CreateSampledNoiseBaselineVectorBlock(FlatBufferBuilder builder, IntPtr dataPtr, int sizeInBytes) { builder.StartVector(1, sizeInBytes, 1); builder.Add<float>(dataPtr, sizeInBytes); return builder.EndVector(); }
  public static void StartSampledNoiseBaselineVector(FlatBufferBuilder builder, int numElems) { builder.StartVector(4, numElems, 4); }
  public static void AddSampledNoiseMz(FlatBufferBuilder builder, VectorOffset sampledNoiseMzOffset) { builder.AddOffset(7, sampledNoiseMzOffset.Value, 0); }
  public static VectorOffset CreateSampledNoiseMzVector(FlatBufferBuilder builder, float[] data) { builder.StartVector(4, data.Length, 4); for (int i = data.Length - 1; i >= 0; i--) builder.AddFloat(data[i]); return builder.EndVector(); }
  public static VectorOffset CreateSampledNoiseMzVectorBlock(FlatBufferBuilder builder, float[] data) { builder.StartVector(4, data.Length, 4); builder.Add(data); return builder.EndVector(); }
  public static VectorOffset CreateSampledNoiseMzVectorBlock(FlatBufferBuilder builder, ArraySegment<float> data) { builder.StartVector(4, data.Count, 4); builder.Add(data); return builder.EndVector(); }
  public static VectorOffset CreateSampledNoiseMzVectorBlock(FlatBufferBuilder builder, IntPtr dataPtr, int sizeInBytes) { builder.StartVector(1, sizeInBytes, 1); builder.Add<float>(dataPtr, sizeInBytes); return builder.EndVector(); }
  public static void StartSampledNoiseMzVector(FlatBufferBuilder builder, int numElems) { builder.StartVector(4, numElems, 4); }
  public static Offset<librawfilereader.ExtendedSpectrumDataT> EndExtendedSpectrumDataT(FlatBufferBuilder builder) {
    int o = builder.EndTable();
    return new Offset<librawfilereader.ExtendedSpectrumDataT>(o);
  }
}


static public class ExtendedSpectrumDataTVerify
{
  static public bool Verify(Google.FlatBuffers.Verifier verifier, uint tablePos)
  {
    return verifier.VerifyTableStart(tablePos)
      && verifier.VerifyVectorOfData(tablePos, 4 /*Mass*/, 8 /*double*/, false)
      && verifier.VerifyVectorOfData(tablePos, 6 /*Noise*/, 4 /*float*/, false)
      && verifier.VerifyVectorOfData(tablePos, 8 /*Baseline*/, 4 /*float*/, false)
      && verifier.VerifyVectorOfData(tablePos, 10 /*Charge*/, 4 /*float*/, false)
      && verifier.VerifyVectorOfData(tablePos, 12 /*Resolution*/, 4 /*float*/, false)
      && verifier.VerifyVectorOfData(tablePos, 14 /*SampledNoise*/, 4 /*float*/, false)
      && verifier.VerifyVectorOfData(tablePos, 16 /*SampledNoiseBaseline*/, 4 /*float*/, false)
      && verifier.VerifyVectorOfData(tablePos, 18 /*SampledNoiseMz*/, 4 /*float*/, false)
      && verifier.VerifyTableEnd(tablePos);
  }
}

}
