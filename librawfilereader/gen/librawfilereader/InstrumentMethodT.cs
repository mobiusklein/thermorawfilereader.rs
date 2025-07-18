// <auto-generated>
//  automatically generated by the FlatBuffers compiler, do not modify
// </auto-generated>

namespace librawfilereader
{

using global::System;
using global::System.Collections.Generic;
using global::Google.FlatBuffers;

public struct InstrumentMethodT : IFlatbufferObject
{
  private Table __p;
  public ByteBuffer ByteBuffer { get { return __p.bb; } }
  public static void ValidateVersion() { FlatBufferConstants.FLATBUFFERS_25_2_10(); }
  public static InstrumentMethodT GetRootAsInstrumentMethodT(ByteBuffer _bb) { return GetRootAsInstrumentMethodT(_bb, new InstrumentMethodT()); }
  public static InstrumentMethodT GetRootAsInstrumentMethodT(ByteBuffer _bb, InstrumentMethodT obj) { return (obj.__assign(_bb.GetInt(_bb.Position) + _bb.Position, _bb)); }
  public void __init(int _i, ByteBuffer _bb) { __p = new Table(_i, _bb); }
  public InstrumentMethodT __assign(int _i, ByteBuffer _bb) { __init(_i, _bb); return this; }

  public byte Index { get { int o = __p.__offset(4); return o != 0 ? __p.bb.Get(o + __p.bb_pos) : (byte)0; } }
  public string Text { get { int o = __p.__offset(6); return o != 0 ? __p.__string(o + __p.bb_pos) : null; } }
#if ENABLE_SPAN_T
  public Span<byte> GetTextBytes() { return __p.__vector_as_span<byte>(6, 1); }
#else
  public ArraySegment<byte>? GetTextBytes() { return __p.__vector_as_arraysegment(6); }
#endif
  public byte[] GetTextArray() { return __p.__vector_as_array<byte>(6); }
  public string DisplayName { get { int o = __p.__offset(8); return o != 0 ? __p.__string(o + __p.bb_pos) : null; } }
#if ENABLE_SPAN_T
  public Span<byte> GetDisplayNameBytes() { return __p.__vector_as_span<byte>(8, 1); }
#else
  public ArraySegment<byte>? GetDisplayNameBytes() { return __p.__vector_as_arraysegment(8); }
#endif
  public byte[] GetDisplayNameArray() { return __p.__vector_as_array<byte>(8); }
  public string Name { get { int o = __p.__offset(10); return o != 0 ? __p.__string(o + __p.bb_pos) : null; } }
#if ENABLE_SPAN_T
  public Span<byte> GetNameBytes() { return __p.__vector_as_span<byte>(10, 1); }
#else
  public ArraySegment<byte>? GetNameBytes() { return __p.__vector_as_arraysegment(10); }
#endif
  public byte[] GetNameArray() { return __p.__vector_as_array<byte>(10); }

  public static Offset<librawfilereader.InstrumentMethodT> CreateInstrumentMethodT(FlatBufferBuilder builder,
      byte index = 0,
      StringOffset textOffset = default(StringOffset),
      StringOffset display_nameOffset = default(StringOffset),
      StringOffset nameOffset = default(StringOffset)) {
    builder.StartTable(4);
    InstrumentMethodT.AddName(builder, nameOffset);
    InstrumentMethodT.AddDisplayName(builder, display_nameOffset);
    InstrumentMethodT.AddText(builder, textOffset);
    InstrumentMethodT.AddIndex(builder, index);
    return InstrumentMethodT.EndInstrumentMethodT(builder);
  }

  public static void StartInstrumentMethodT(FlatBufferBuilder builder) { builder.StartTable(4); }
  public static void AddIndex(FlatBufferBuilder builder, byte index) { builder.AddByte(0, index, 0); }
  public static void AddText(FlatBufferBuilder builder, StringOffset textOffset) { builder.AddOffset(1, textOffset.Value, 0); }
  public static void AddDisplayName(FlatBufferBuilder builder, StringOffset displayNameOffset) { builder.AddOffset(2, displayNameOffset.Value, 0); }
  public static void AddName(FlatBufferBuilder builder, StringOffset nameOffset) { builder.AddOffset(3, nameOffset.Value, 0); }
  public static Offset<librawfilereader.InstrumentMethodT> EndInstrumentMethodT(FlatBufferBuilder builder) {
    int o = builder.EndTable();
    return new Offset<librawfilereader.InstrumentMethodT>(o);
  }
}


static public class InstrumentMethodTVerify
{
  static public bool Verify(Google.FlatBuffers.Verifier verifier, uint tablePos)
  {
    return verifier.VerifyTableStart(tablePos)
      && verifier.VerifyField(tablePos, 4 /*Index*/, 1 /*byte*/, 1, false)
      && verifier.VerifyString(tablePos, 6 /*Text*/, false)
      && verifier.VerifyString(tablePos, 8 /*DisplayName*/, false)
      && verifier.VerifyString(tablePos, 10 /*Name*/, false)
      && verifier.VerifyTableEnd(tablePos);
  }
}

}
