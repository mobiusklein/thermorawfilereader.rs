using System;
using System.IO;
using System.Runtime.InteropServices;

using ThermoFisher.CommonCore.Data.Business;
using ThermoFisher.CommonCore.Data.Interfaces;
using ThermoFisher.CommonCore.RawFileReader;

using Google.FlatBuffers;
using ThermoFisher.CommonCore.Data.FilterEnums;
using System.Collections;
using System.Collections.Generic;
using System.Reflection;

namespace librawfilereader
{

    public struct IsolationWindow
    {
        public double LowerMZ;
        public double TargetMZ;
        public double UpperMZ;

        public IsolationWindow(double isolationWidth, double monoisotopicMZ, double isolationOffset) {
            LowerMZ = monoisotopicMZ + isolationOffset - isolationWidth;
            UpperMZ = monoisotopicMZ + isolationOffset + isolationWidth;
            TargetMZ = monoisotopicMZ;
        }
    }

    public struct ActivationProperties
    {
        public DissociationMethod Dissociation;
        public double Energy;
    }

    public struct PrecursorProperties
    {
        public double MonoisotopicMZ;
        public int PrecursorCharge;
        public IsolationWindow IsolationWindow;
        public int MasterScanNumber;
        public ActivationProperties Activation;
    }

    [StructLayout(LayoutKind.Sequential)]
    public unsafe struct RawVec
    {
        public byte* Data;
        public nuint Len;
        public nuint Capacity;
    }

    public enum RawFileReaderError : uint
    {
        Ok,
        FileNotFound,
        InvalidFormat,
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct RawFileReader : IDisposable
    {
        public string Path;
        public IRawDataPlus Handle;
        public RawFileReaderError Status;

        public RawFileReader(string path)
        {
            Path = path;
            Handle = RawFileReaderAdapter.FileFactory(Path);
            Status = RawFileReaderError.Ok;
            Status = Configure();
        }

        public int FirstSpectrum()
        {
            return Handle.RunHeaderEx.FirstSpectrum;
        }

        public int LastSpectrum()
        {
            return Handle.RunHeaderEx.LastSpectrum;
        }

        public int SpectrumCount()
        {
            return Handle.RunHeaderEx.SpectraCount;
        }

        public void Dispose()
        {
            Handle.Dispose();
        }

        private short MSLevelFromFilter(IScanFilter filter)
        {
            short msLevel = 1;
            {
                var level = filter.MSOrder;
                if (MSOrderType.Ms == level)
                {
                    msLevel = 1;
                }
                else if (MSOrderType.Ms2 == level)
                {
                    msLevel = 2;
                }
                else if (MSOrderType.Ms3 == level)
                {
                    msLevel = 3;
                }
                else if (MSOrderType.Ms4 == level)
                {
                    msLevel = 4;
                }
                else if (MSOrderType.Ms5 == level)
                {
                    msLevel = 5;
                }
            }
            return msLevel;
        }

        public int FindPreviousPrecursor(int scanNumber, short msLevel) {
            int i = scanNumber -  1;
            while(i > 0) {
                var filter = Handle.GetFilterForScanNumber(i);
                var levelOf = MSLevelFromFilter(filter);
                if (levelOf < msLevel) {
                    return i;
                } else {
                    i -= 1;
                }
            }

            return i;
        }

        public ActivationProperties ExtractActivation(int scanNumber, short msLevel, IScanFilter filter) {
            ActivationProperties activation = new ActivationProperties
            {
                Dissociation = DissociationMethod.Unknown,
                Energy = filter.GetEnergy(msLevel - 2)
            };

            var activationType = filter.GetActivation(msLevel - 2);
            if (ActivationType.CollisionInducedDissociation == activationType)
            {
                activation.Dissociation = DissociationMethod.CID;
            }
            else if (ActivationType.ElectronCaptureDissociation == activationType)
            {
                activation.Dissociation = DissociationMethod.ECD;
            }
            else if (ActivationType.HigherEnergyCollisionalDissociation == activationType)
            {
                activation.Dissociation = DissociationMethod.HCD;
            }
            else if (ActivationType.ElectronTransferDissociation == activationType)
            {
                activation.Dissociation = DissociationMethod.ETD;
            }
            else if (ActivationType.NegativeElectronTransferDissociation == activationType)
            {
                activation.Dissociation = DissociationMethod.NETD;
            }
            else if (ActivationType.MultiPhotonDissociation == activationType)
            {
                activation.Dissociation = DissociationMethod.MPD;
            }
            else if (ActivationType.ProtonTransferReaction == activationType)
            {
                activation.Dissociation = DissociationMethod.PTD;
            }
            return activation;
        }

        public PrecursorProperties ExtractPrecursorProperties(int scanNumber, short msLevel, IScanFilter filter)
        {
            var trailers = Handle.GetTrailerExtraInformation(scanNumber);

            var n = trailers.Length;
            double monoisotopicMZ = 0.0;
            int precursorCharge = 0;
            double isolationWidth = 0.0;
            int masterScanNumber = -1;
            for (int i = 0; i < n; i++)
            {
                var label = trailers.Labels[i].Trim(':');
                if (label == "Monoisotopic M/Z")
                {
                    monoisotopicMZ = double.Parse(trailers.Values[i]);
                }
                else if (label == "Charge State")
                {
                    precursorCharge = int.Parse(trailers.Values[i]);
                }
                else if (label.EndsWith("Isolation Width"))
                {
                    if (label.StartsWith("MS" + msLevel.ToString()))
                    {
                        isolationWidth = double.Parse(trailers.Values[i]) / 2.0;
                    }
                }
                else if (label == "Master Scan Number")
                {
                    masterScanNumber = int.Parse(trailers.Values[i]);
                }
            }

            if (msLevel > 1 && isolationWidth == 0.0)
            {
                isolationWidth = filter.GetIsolationWidth(msLevel - 2) / 2;
            }
            double isolationOffset = filter.GetIsolationWidthOffset(msLevel - 2);
            if (monoisotopicMZ == 0.0)
            {
                monoisotopicMZ = filter.GetMass(msLevel - 2);
            }

            if (masterScanNumber == -1) {
                masterScanNumber = FindPreviousPrecursor(scanNumber, msLevel);
            }

            ActivationProperties activation = ExtractActivation(scanNumber, msLevel, filter);
            IsolationWindow window = new IsolationWindow(isolationWidth, monoisotopicMZ, isolationOffset);
            PrecursorProperties props = new PrecursorProperties
            {
                PrecursorCharge = precursorCharge,
                MasterScanNumber = masterScanNumber,
                MonoisotopicMZ = monoisotopicMZ,
                IsolationWindow = window,
                Activation = activation
            };
            return props;
        }

        public Offset<SpectrumData> LoadSpectrumData(int scanNumber, ScanStatistics stats, FlatBufferBuilder bufferBuilder) {
            var segScan = Handle.GetSegmentedScanFromScanNumber(scanNumber, stats);

            var mzOffset = SpectrumData.CreateMzVector(bufferBuilder, segScan.Positions);
            SpectrumData.StartIntensityVector(bufferBuilder, segScan.PositionCount);
            foreach(var val in segScan.Intensities) {
                bufferBuilder.AddFloat((float)val);
            }
            var intensityOffset = bufferBuilder.EndVector();
            var offset = SpectrumData.CreateSpectrumData(bufferBuilder, mzOffset, intensityOffset);
            return offset;
        }

        public ByteBuffer SpectrumDescriptionFor(int scanNumber)
        {
            var stats = Handle.GetScanStatsForScanNumber(scanNumber);
            SpectrumMode mode = stats.IsCentroidScan ? SpectrumMode.Centroid : SpectrumMode.Profile;

            var filter = Handle.GetFilterForScanNumber(scanNumber);
            short level = MSLevelFromFilter(filter);

            Polarity polarity = Polarity.Positive;
            if (PolarityType.Positive == filter.Polarity)
            {
                polarity = Polarity.Positive;
            }
            else if (PolarityType.Negative == filter.Polarity)
            {
                polarity = Polarity.Negative;
            }
            else
            {
                polarity = Polarity.Unknown;
            }



            var builder = new FlatBufferBuilder(1024);

            var dataOffset = LoadSpectrumData(scanNumber, stats, builder);
            var filterString = filter.ToString();
            var filterStringOffset = builder.CreateString(filterString);
            SpectrumDescription.StartSpectrumDescription(builder);
            SpectrumDescription.AddData(builder, dataOffset);
            SpectrumDescription.AddIndex(builder, stats.ScanNumber - 1);
            SpectrumDescription.AddMsLevel(builder, (byte)level);
            SpectrumDescription.AddPolarity(builder, polarity);
            SpectrumDescription.AddMode(builder, mode);
            SpectrumDescription.AddFilterString(builder, filterStringOffset);
            if (level > 1)
            {
                var precursorProps = ExtractPrecursorProperties(scanNumber, level, filter);
                var precursor = PrecursorT.CreatePrecursorT(
                    builder,
                    precursorProps.MonoisotopicMZ,
                    (float)0.0,
                    precursorProps.PrecursorCharge,
                    precursorProps.MasterScanNumber,
                    precursorProps.IsolationWindow.LowerMZ,
                    precursorProps.IsolationWindow.TargetMZ,
                    precursorProps.IsolationWindow.UpperMZ,
                    precursorProps.Activation.Dissociation,
                    precursorProps.Activation.Energy
                );
                SpectrumDescription.AddPrecursor(builder, precursor);
            }
            var description = SpectrumDescription.EndSpectrumDescription(builder);
            builder.Finish(description.Value);
            return builder.DataBuffer;
        }

        private RawFileReaderError Configure()
        {
            if (!File.Exists(Path))
            {
                return RawFileReaderError.FileNotFound;
            }
            if (Handle.IsError)
            {
                return RawFileReaderError.InvalidFormat;
            }
            else
            {
                Handle.SelectInstrument(Device.MS, 1);
            }
            return RawFileReaderError.Ok;
        }
    }

    public static class Exports
    {
        private static unsafe delegate*<nuint, RawVec*, void> RustAllocateMemory;

        private static Dictionary<IntPtr, RawFileReader> OpenHandles = new Dictionary<nint, RawFileReader>();
        private static IntPtr HandleCounter = 1;

        [UnmanagedCallersOnly]
        public static unsafe void SetRustAllocateMemory(delegate*<nuint, RawVec*, void> rustAllocateMemory) => RustAllocateMemory = rustAllocateMemory;

        private unsafe static RawVec BufferToRustVec(byte[] buffer, nuint size)
        {
            var vec = new RawVec();
            RustAllocateMemory(size, &vec);

            if ((IntPtr)vec.Data == IntPtr.Zero) {
                return vec;
            }

            fixed (byte* buf = buffer)
            {
                vec.Len = size;
                Buffer.MemoryCopy(
                    buf, vec.Data, vec.Capacity, vec.Len
                );

            }
            return vec;
        }

        [UnmanagedCallersOnly]
        public static unsafe IntPtr Open(IntPtr textPtr, int textLength)
        {
            var text = Marshal.PtrToStringAnsi(textPtr, textLength);
            var handle = new RawFileReader(text);
            IntPtr handleToken;
            lock (OpenHandles) {
                handleToken = HandleCounter;
                HandleCounter += 1;
                OpenHandles[handleToken] = handle;
            }
            return handleToken;
        }

        private static RawFileReader GetHandleForToken(IntPtr handleToken) {
            RawFileReader handle;
            lock (OpenHandles) {
                handle = OpenHandles[handleToken];
            }
            return handle;
        }

        [UnmanagedCallersOnly]
        public static unsafe void Close(IntPtr handleToken)
        {
            lock (OpenHandles) {
                if (OpenHandles.ContainsKey(handleToken)) {
                    OpenHandles.Remove(handleToken);
                }
            }
        }

        public static unsafe void CloseAll()
        {
            OpenHandles.Clear();
        }


        public delegate int SpectrumIndexFn(IntPtr handleToken);

        [UnmanagedCallersOnly]
        public static unsafe int FirstSpectrum(IntPtr handleToken)
        {
            RawFileReader reader = GetHandleForToken(handleToken);
            return reader.FirstSpectrum();
        }

        [UnmanagedCallersOnly]
        public static unsafe int LastSpectrum(IntPtr handleToken)
        {
            RawFileReader reader = GetHandleForToken(handleToken);
            return reader.LastSpectrum();
        }

        [UnmanagedCallersOnly]
        public static unsafe int SpectrumCount(IntPtr handleToken)
        {
            RawFileReader reader = GetHandleForToken(handleToken);
            return reader.SpectrumCount();
        }

        public delegate RawFileReaderError StatusFn(IntPtr handleToken);

        public static unsafe uint Status(IntPtr handleToken)
        {
            RawFileReader reader = GetHandleForToken(handleToken);
            return (uint)reader.Status;
        }

        public delegate RawVec BufferFn(IntPtr handleToken, int scanNumber);

        [UnmanagedCallersOnly]
        public static unsafe RawVec SpectrumDescriptionFor(IntPtr handleToken, int scanNumber)
        {
            RawFileReader reader = GetHandleForToken(handleToken);
            var buffer = reader.SpectrumDescriptionFor(scanNumber);
            var bytes = buffer.ToSizedArray();
            var size = bytes.Length;
            return BufferToRustVec(bytes, (nuint)size);
        }
    }
}
