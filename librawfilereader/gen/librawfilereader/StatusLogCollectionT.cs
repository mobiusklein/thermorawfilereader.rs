// <auto-generated>
//  automatically generated by the FlatBuffers compiler, do not modify
// </auto-generated>

namespace librawfilereader
{

using global::System;
using global::System.Collections.Generic;
using global::Google.FlatBuffers;

public struct StatusLogCollectionT : IFlatbufferObject
{
  private Table __p;
  public ByteBuffer ByteBuffer { get { return __p.bb; } }
  public static void ValidateVersion() { FlatBufferConstants.FLATBUFFERS_25_2_10(); }
  public static StatusLogCollectionT GetRootAsStatusLogCollectionT(ByteBuffer _bb) { return GetRootAsStatusLogCollectionT(_bb, new StatusLogCollectionT()); }
  public static StatusLogCollectionT GetRootAsStatusLogCollectionT(ByteBuffer _bb, StatusLogCollectionT obj) { return (obj.__assign(_bb.GetInt(_bb.Position) + _bb.Position, _bb)); }
  public void __init(int _i, ByteBuffer _bb) { __p = new Table(_i, _bb); }
  public StatusLogCollectionT __assign(int _i, ByteBuffer _bb) { __init(_i, _bb); return this; }

  public librawfilereader.StatusLogFloatT? FloatLogs(int j) { int o = __p.__offset(4); return o != 0 ? (librawfilereader.StatusLogFloatT?)(new librawfilereader.StatusLogFloatT()).__assign(__p.__indirect(__p.__vector(o) + j * 4), __p.bb) : null; }
  public int FloatLogsLength { get { int o = __p.__offset(4); return o != 0 ? __p.__vector_len(o) : 0; } }
  public librawfilereader.StatusLogBoolT? BoolLogs(int j) { int o = __p.__offset(6); return o != 0 ? (librawfilereader.StatusLogBoolT?)(new librawfilereader.StatusLogBoolT()).__assign(__p.__indirect(__p.__vector(o) + j * 4), __p.bb) : null; }
  public int BoolLogsLength { get { int o = __p.__offset(6); return o != 0 ? __p.__vector_len(o) : 0; } }
  public librawfilereader.StatusLogIntT? IntLogs(int j) { int o = __p.__offset(8); return o != 0 ? (librawfilereader.StatusLogIntT?)(new librawfilereader.StatusLogIntT()).__assign(__p.__indirect(__p.__vector(o) + j * 4), __p.bb) : null; }
  public int IntLogsLength { get { int o = __p.__offset(8); return o != 0 ? __p.__vector_len(o) : 0; } }
  public librawfilereader.StatusLogStringT? StringLogs(int j) { int o = __p.__offset(10); return o != 0 ? (librawfilereader.StatusLogStringT?)(new librawfilereader.StatusLogStringT()).__assign(__p.__indirect(__p.__vector(o) + j * 4), __p.bb) : null; }
  public int StringLogsLength { get { int o = __p.__offset(10); return o != 0 ? __p.__vector_len(o) : 0; } }

  public static Offset<librawfilereader.StatusLogCollectionT> CreateStatusLogCollectionT(FlatBufferBuilder builder,
      VectorOffset float_logsOffset = default(VectorOffset),
      VectorOffset bool_logsOffset = default(VectorOffset),
      VectorOffset int_logsOffset = default(VectorOffset),
      VectorOffset string_logsOffset = default(VectorOffset)) {
    builder.StartTable(4);
    StatusLogCollectionT.AddStringLogs(builder, string_logsOffset);
    StatusLogCollectionT.AddIntLogs(builder, int_logsOffset);
    StatusLogCollectionT.AddBoolLogs(builder, bool_logsOffset);
    StatusLogCollectionT.AddFloatLogs(builder, float_logsOffset);
    return StatusLogCollectionT.EndStatusLogCollectionT(builder);
  }

  public static void StartStatusLogCollectionT(FlatBufferBuilder builder) { builder.StartTable(4); }
  public static void AddFloatLogs(FlatBufferBuilder builder, VectorOffset floatLogsOffset) { builder.AddOffset(0, floatLogsOffset.Value, 0); }
  public static VectorOffset CreateFloatLogsVector(FlatBufferBuilder builder, Offset<librawfilereader.StatusLogFloatT>[] data) { builder.StartVector(4, data.Length, 4); for (int i = data.Length - 1; i >= 0; i--) builder.AddOffset(data[i].Value); return builder.EndVector(); }
  public static VectorOffset CreateFloatLogsVectorBlock(FlatBufferBuilder builder, Offset<librawfilereader.StatusLogFloatT>[] data) { builder.StartVector(4, data.Length, 4); builder.Add(data); return builder.EndVector(); }
  public static VectorOffset CreateFloatLogsVectorBlock(FlatBufferBuilder builder, ArraySegment<Offset<librawfilereader.StatusLogFloatT>> data) { builder.StartVector(4, data.Count, 4); builder.Add(data); return builder.EndVector(); }
  public static VectorOffset CreateFloatLogsVectorBlock(FlatBufferBuilder builder, IntPtr dataPtr, int sizeInBytes) { builder.StartVector(1, sizeInBytes, 1); builder.Add<Offset<librawfilereader.StatusLogFloatT>>(dataPtr, sizeInBytes); return builder.EndVector(); }
  public static void StartFloatLogsVector(FlatBufferBuilder builder, int numElems) { builder.StartVector(4, numElems, 4); }
  public static void AddBoolLogs(FlatBufferBuilder builder, VectorOffset boolLogsOffset) { builder.AddOffset(1, boolLogsOffset.Value, 0); }
  public static VectorOffset CreateBoolLogsVector(FlatBufferBuilder builder, Offset<librawfilereader.StatusLogBoolT>[] data) { builder.StartVector(4, data.Length, 4); for (int i = data.Length - 1; i >= 0; i--) builder.AddOffset(data[i].Value); return builder.EndVector(); }
  public static VectorOffset CreateBoolLogsVectorBlock(FlatBufferBuilder builder, Offset<librawfilereader.StatusLogBoolT>[] data) { builder.StartVector(4, data.Length, 4); builder.Add(data); return builder.EndVector(); }
  public static VectorOffset CreateBoolLogsVectorBlock(FlatBufferBuilder builder, ArraySegment<Offset<librawfilereader.StatusLogBoolT>> data) { builder.StartVector(4, data.Count, 4); builder.Add(data); return builder.EndVector(); }
  public static VectorOffset CreateBoolLogsVectorBlock(FlatBufferBuilder builder, IntPtr dataPtr, int sizeInBytes) { builder.StartVector(1, sizeInBytes, 1); builder.Add<Offset<librawfilereader.StatusLogBoolT>>(dataPtr, sizeInBytes); return builder.EndVector(); }
  public static void StartBoolLogsVector(FlatBufferBuilder builder, int numElems) { builder.StartVector(4, numElems, 4); }
  public static void AddIntLogs(FlatBufferBuilder builder, VectorOffset intLogsOffset) { builder.AddOffset(2, intLogsOffset.Value, 0); }
  public static VectorOffset CreateIntLogsVector(FlatBufferBuilder builder, Offset<librawfilereader.StatusLogIntT>[] data) { builder.StartVector(4, data.Length, 4); for (int i = data.Length - 1; i >= 0; i--) builder.AddOffset(data[i].Value); return builder.EndVector(); }
  public static VectorOffset CreateIntLogsVectorBlock(FlatBufferBuilder builder, Offset<librawfilereader.StatusLogIntT>[] data) { builder.StartVector(4, data.Length, 4); builder.Add(data); return builder.EndVector(); }
  public static VectorOffset CreateIntLogsVectorBlock(FlatBufferBuilder builder, ArraySegment<Offset<librawfilereader.StatusLogIntT>> data) { builder.StartVector(4, data.Count, 4); builder.Add(data); return builder.EndVector(); }
  public static VectorOffset CreateIntLogsVectorBlock(FlatBufferBuilder builder, IntPtr dataPtr, int sizeInBytes) { builder.StartVector(1, sizeInBytes, 1); builder.Add<Offset<librawfilereader.StatusLogIntT>>(dataPtr, sizeInBytes); return builder.EndVector(); }
  public static void StartIntLogsVector(FlatBufferBuilder builder, int numElems) { builder.StartVector(4, numElems, 4); }
  public static void AddStringLogs(FlatBufferBuilder builder, VectorOffset stringLogsOffset) { builder.AddOffset(3, stringLogsOffset.Value, 0); }
  public static VectorOffset CreateStringLogsVector(FlatBufferBuilder builder, Offset<librawfilereader.StatusLogStringT>[] data) { builder.StartVector(4, data.Length, 4); for (int i = data.Length - 1; i >= 0; i--) builder.AddOffset(data[i].Value); return builder.EndVector(); }
  public static VectorOffset CreateStringLogsVectorBlock(FlatBufferBuilder builder, Offset<librawfilereader.StatusLogStringT>[] data) { builder.StartVector(4, data.Length, 4); builder.Add(data); return builder.EndVector(); }
  public static VectorOffset CreateStringLogsVectorBlock(FlatBufferBuilder builder, ArraySegment<Offset<librawfilereader.StatusLogStringT>> data) { builder.StartVector(4, data.Count, 4); builder.Add(data); return builder.EndVector(); }
  public static VectorOffset CreateStringLogsVectorBlock(FlatBufferBuilder builder, IntPtr dataPtr, int sizeInBytes) { builder.StartVector(1, sizeInBytes, 1); builder.Add<Offset<librawfilereader.StatusLogStringT>>(dataPtr, sizeInBytes); return builder.EndVector(); }
  public static void StartStringLogsVector(FlatBufferBuilder builder, int numElems) { builder.StartVector(4, numElems, 4); }
  public static Offset<librawfilereader.StatusLogCollectionT> EndStatusLogCollectionT(FlatBufferBuilder builder) {
    int o = builder.EndTable();
    return new Offset<librawfilereader.StatusLogCollectionT>(o);
  }
}


static public class StatusLogCollectionTVerify
{
  static public bool Verify(Google.FlatBuffers.Verifier verifier, uint tablePos)
  {
    return verifier.VerifyTableStart(tablePos)
      && verifier.VerifyVectorOfTables(tablePos, 4 /*FloatLogs*/, librawfilereader.StatusLogFloatTVerify.Verify, false)
      && verifier.VerifyVectorOfTables(tablePos, 6 /*BoolLogs*/, librawfilereader.StatusLogBoolTVerify.Verify, false)
      && verifier.VerifyVectorOfTables(tablePos, 8 /*IntLogs*/, librawfilereader.StatusLogIntTVerify.Verify, false)
      && verifier.VerifyVectorOfTables(tablePos, 10 /*StringLogs*/, librawfilereader.StatusLogStringTVerify.Verify, false)
      && verifier.VerifyTableEnd(tablePos);
  }
}

}
