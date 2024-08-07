// <auto-generated>
//  automatically generated by the FlatBuffers compiler, do not modify
// </auto-generated>

namespace librawfilereader
{

using global::System;
using global::System.Collections.Generic;
using global::Google.FlatBuffers;

public struct IsolationWindowT : IFlatbufferObject
{
  private Struct __p;
  public ByteBuffer ByteBuffer { get { return __p.bb; } }
  public void __init(int _i, ByteBuffer _bb) { __p = new Struct(_i, _bb); }
  public IsolationWindowT __assign(int _i, ByteBuffer _bb) { __init(_i, _bb); return this; }

  public double Lower { get { return __p.bb.GetDouble(__p.bb_pos + 0); } }
  public double Target { get { return __p.bb.GetDouble(__p.bb_pos + 8); } }
  public double Upper { get { return __p.bb.GetDouble(__p.bb_pos + 16); } }

  public static Offset<librawfilereader.IsolationWindowT> CreateIsolationWindowT(FlatBufferBuilder builder, double Lower, double Target, double Upper) {
    builder.Prep(8, 24);
    builder.PutDouble(Upper);
    builder.PutDouble(Target);
    builder.PutDouble(Lower);
    return new Offset<librawfilereader.IsolationWindowT>(builder.Offset);
  }
}


}
