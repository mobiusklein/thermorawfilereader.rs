using System;
using System.IO;
using System.Runtime.InteropServices;

using ThermoFisher.CommonCore.Data.Business;
using ThermoFisher.CommonCore.Data.Interfaces;
using ThermoFisher.CommonCore.RawFileReader;

using Google.FlatBuffers;
using ThermoFisher.CommonCore.Data.FilterEnums;
using System.Collections.Generic;
using System.Runtime.CompilerServices;
using System.Linq;

namespace librawfilereader
{

    public struct IsolationWindow
    {
        public double LowerMZ;
        public double TargetMZ;
        public double UpperMZ;

        public IsolationWindow(double isolationWidth, double monoisotopicMZ, double isolationOffset)
        {
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

    public struct AcquisitionProperties
    {
        public double InjectionTime;
        public double? CompensationVoltage = null;
        public MassAnalyzer Analyzer;
        public IonizationMode Ionization;
        public double LowMZ;
        public double HighMZ;
        public int ScanEventNumber;

        public static MassAnalyzer CastMassAnalyzer(MassAnalyzerType analyzer)
        {
            switch (analyzer)
            {
                case MassAnalyzerType.MassAnalyzerASTMS:
                    {
                        return MassAnalyzer.ASTMS;
                    }
                case MassAnalyzerType.MassAnalyzerFTMS:
                    {
                        return MassAnalyzer.FTMS;
                    }
                case MassAnalyzerType.MassAnalyzerTOFMS:
                    {
                        return MassAnalyzer.TOFMS;
                    }
                case MassAnalyzerType.MassAnalyzerITMS:
                    {
                        return MassAnalyzer.ITMS;
                    }
                case MassAnalyzerType.MassAnalyzerSQMS:
                    {
                        return MassAnalyzer.SQMS;
                    }
                case MassAnalyzerType.MassAnalyzerSector:
                    {
                        return MassAnalyzer.Sector;
                    }
                case MassAnalyzerType.Any:
                    {
                        return MassAnalyzer.Unknown;
                    }
                default:
                    {
                        return MassAnalyzer.Unknown;
                    }
            };
        }
        public static IonizationMode CastIonizationMode(IonizationModeType ionizationModeType) {
            int value = (int)ionizationModeType;
            return (IonizationMode)value;
        }

        public AcquisitionProperties(double injectionTime, double? compensationVoltage, MassAnalyzerType analyzer, double lowMZ, double highMZ, int scanEventNumber)
        {
            InjectionTime = injectionTime;
            CompensationVoltage = compensationVoltage;
            Analyzer = CastMassAnalyzer(analyzer);
            LowMZ = lowMZ;
            HighMZ = highMZ;
            ScanEventNumber = scanEventNumber;
        }
    }

    [StructLayout(LayoutKind.Sequential)]
    public unsafe struct RawVec
    {
        public byte* Data;
        public nuint Len;
        public nuint Capacity;
    }

    /// <summary>
    /// A set of error codes to describe how creating and using a `RawFileReader` might fail (or not).
    ///
    /// This is FFI safe and mirrored by an equivalent enum on the other side.
    /// </summary>
    public enum RawFileReaderError : uint
    {
        Ok = 0,
        FileNotFound,
        InvalidFormat,
        HandleNotFound,

        Error = 999
    }

    /// <summary>
    /// Represent a reader for a Thermo RAW file that packages spectra and metadata
    /// into FlatBuffers for ease of exchange.
    /// </summary>
    public class RawFileReader : IDisposable
    {
        /// <summary>
        /// The path to read the RAW file from.
        /// </summary>
        public string Path;
        /// <summary>
        /// A mapping to look up the nearest previous spectrum of a given
        /// MS level
        /// </summary>
        Dictionary<int, List<int?>> PreviousMSLevels;

        /// <summary>
        /// An index look up mapping trailer keys by index that lets us avoid
        /// looping over all trailer entries
        /// </summary>
        Dictionary<string, int> TrailerMap;

        /// <summary>
        /// The actual Thermo-provided reader implementation
        /// </summary>
        // public IRawDataPlus Handle;
        public IRawFileThreadManager Manager;

        /// <summary>
        /// The status of the reader, determined when the file is first opened
        /// </summary>
        public RawFileReaderError Status;
        /// <summary>
        /// A static look up of instrument configurations to a unique identifier
        /// </summary>
        public Dictionary<(MassAnalyzer, IonizationMode), long> InstrumentConfigsByComponents;

        /// <summary>
        /// Whether or not to include the actual mass spectrum data in the message buffers.
        /// This is useful to disable if you just want to scoop up metadata.
        /// </summary>
        public bool IncludeSignal = true;

        /// <summary>
        /// Whether or not to pick peaks, simplifying profile spectra if they are not already stored
        /// as centroid peaks. Defaults to `false`.
        /// </summary>
        public bool CentroidSpectra = false;

        public RawFileReader(string path)
        {
            Path = path;
            Manager = RawFileReaderAdapter.ThreadedFileFactory(Path);
            // Handle = RawFileReaderAdapter.FileFactory(Path);
            Status = RawFileReaderError.Ok;
            InstrumentConfigsByComponents = new();
            PreviousMSLevels = new();
            TrailerMap = new();
            Status = Configure();
        }

        IRawDataPlus GetHandleRaw() {
            var accessor = Manager.CreateThreadAccessor();
            return accessor;
            // return Handle;
        }

        IRawDataPlus GetHandle()
        {
            var accessor = GetHandleRaw();
            accessor.SelectInstrument(Device.MS, 1);
            return accessor;
            // return Handle;
        }

        public int FirstSpectrum()
        {
            return GetHandle().RunHeaderEx.FirstSpectrum;
        }

        public int LastSpectrum()
        {
            return GetHandle().RunHeaderEx.LastSpectrum;
        }

        public int SpectrumCount()
        {
            return GetHandle().RunHeaderEx.SpectraCount;
        }

        public void Dispose()
        {
            Manager.Dispose();
            // Handle.Dispose();
        }

        /// <summary>
        /// A static lookup to convert Thermo's MSOrderType enum to an MS level integer
        /// </summary>
        private static Dictionary<MSOrderType, short> MSLevelMap = new Dictionary<MSOrderType, short>() {
            {MSOrderType.Ms, 1},
            {MSOrderType.Ms2, 2},
            {MSOrderType.Ms3, 3},
            {MSOrderType.Ms4, 4},
            {MSOrderType.Ms5, 5},
            {MSOrderType.Ms6, 6},
            {MSOrderType.Ms7, 7},
            {MSOrderType.Ms8, 8},
            {MSOrderType.Ms9, 9},
            {MSOrderType.Ms10, 10},
        };

        /// <summary>
        /// Get the MS level of the scan as an integer
        /// </summary>
        /// <param name="filter">The scan filter for a given scan which holds much of the metadata</param>
        /// <returns>The MS level of the scan</returns>
        short MSLevelFromFilter(IScanFilter filter)
        {
            short msLevel;
            if (MSLevelMap.TryGetValue(filter.MSOrder, out msLevel)) {
                return msLevel;
            }
            return 1;
        }

        /// <summary>
        /// Find the scan number of the precursor, which is assumed to be the most recent spectrum of a lower
        /// MS level, if it was not indicated some other way. This method is used when the Master Scan Number was
        /// not set in a trailer value.
        /// </summary>
        /// <param name="scanNumber">The scan number to search back from</param>
        /// <param name="msLevel">The MS level to search for lesser values from</param>
        /// <param name="accessor">The current RAW file accessor</param>
        /// <returns>The scan number of the most recent lower MS level spectrum</returns>
        int FindPreviousPrecursor(int scanNumber, short msLevel, IRawDataPlus accessor)
        {
            var cacheLookUp = PreviousMSLevels[scanNumber][msLevel - 1];
            if (cacheLookUp != null) {
                return cacheLookUp.Value;
            }
            int i = scanNumber - 1;
            while (i > 0)
            {
                var filter = accessor.GetFilterForScanNumber(i);
                var levelOf = MSLevelFromFilter(filter);
                if (levelOf < msLevel)
                {
                    return i;
                }
                else
                {
                    i -= 1;
                }
            }

            return i;
        }

        private static Dictionary<ActivationType, DissociationMethod> DissociationMethodMap = new Dictionary<ActivationType, DissociationMethod>() {
            {ActivationType.CollisionInducedDissociation, DissociationMethod.CID},
            {ActivationType.ElectronCaptureDissociation, DissociationMethod.ECD},
            {ActivationType.ElectronTransferDissociation, DissociationMethod.ETD},
            {ActivationType.HigherEnergyCollisionalDissociation, DissociationMethod.HCD},
            {ActivationType.NegativeElectronTransferDissociation, DissociationMethod.NETD},
            {ActivationType.MultiPhotonDissociation, DissociationMethod.MPD},
            {ActivationType.ProtonTransferReaction, DissociationMethod.PTD},
        };

        ActivationProperties ExtractActivation(int scanNumber, short msLevel, IScanFilter filter)
        {
            ActivationProperties activation = new ActivationProperties
            {
                Dissociation = DissociationMethod.Unknown,
                Energy = filter.GetEnergy(msLevel - 2)
            };

            var activationType = filter.GetActivation(msLevel - 2);
            DissociationMethodMap.TryGetValue(activationType, out activation.Dissociation);
            return activation;
        }

        (PrecursorProperties?, AcquisitionProperties) ExtractPrecursorAndTrailerMetadata(int scanNumber, short msLevel, IScanFilter filter, IRawDataPlus accessor, ScanStatistics stats)
        {
            var trailers = accessor.GetTrailerExtraInformation(scanNumber);

            var n = trailers.Length;
            double monoisotopicMZ = 0.0;
            short precursorCharge = 0;
            double isolationWidth = 0.0;
            double injectionTime = 0.0;
            int masterScanNumber = -1;
            short scanEventNum = 1;
            object tmp;

            var v = "Ion Injection Time (ms)";
            if (TrailerMap.ContainsKey(v))
            {
                tmp = accessor.GetTrailerExtraValue(scanNumber, TrailerMap[v]);
                if (tmp != null)
                {
                    injectionTime = Convert.ToDouble(tmp);
                }
            }

            v = "Scan Event";
            if (TrailerMap.ContainsKey(v)) {
                tmp = accessor.GetTrailerExtraValue(scanNumber, TrailerMap[v]);
                if (tmp != null)
                {
                    scanEventNum = Convert.ToInt16(tmp);
                }
            }

            if (msLevel > 1) {
                if(TrailerMap.ContainsKey("Master Scan Number")) {
                    tmp = accessor.GetTrailerExtraValue(scanNumber, TrailerMap["Master Scan Number"]);
                    if (tmp != null)
                    {
                        masterScanNumber = Convert.ToInt32(tmp);
                    }
                }

                if (TrailerMap.ContainsKey("Monoisotopic M/Z"))
                {
                    tmp = accessor.GetTrailerExtraValue(scanNumber, TrailerMap["Monoisotopic M/Z"]);
                    if (tmp != null)
                    {
                        monoisotopicMZ = Convert.ToDouble(tmp);
                    }
                }

                if (TrailerMap.ContainsKey("Charge State")) {
                    tmp = accessor.GetTrailerExtraValue(scanNumber, TrailerMap["Charge State"]);
                    if (tmp != null) {
                        precursorCharge = Convert.ToInt16(tmp);
                    }
                }

                v = String.Format("MS{0} Isolation Width", msLevel);
                if (TrailerMap.ContainsKey(v))
                {
                    tmp = accessor.GetTrailerExtraValue(scanNumber, TrailerMap[v]);
                    if (tmp != null)
                    {
                        isolationWidth = Convert.ToDouble(tmp);
                    }
                }
            }

            AcquisitionProperties acquisitionProperties = new AcquisitionProperties(injectionTime, null, filter.MassAnalyzer, stats.LowMass, stats.HighMass, scanEventNum);

            if (filter.CompensationVoltage == TriState.On)
            {
                acquisitionProperties.CompensationVoltage = filter.CompensationVoltageValue(msLevel - 1);
            }

            if (msLevel > 1 && isolationWidth == 0.0)
            {
                isolationWidth = filter.GetIsolationWidth(msLevel - 2) / 2;
            }
            if (msLevel > 1)
            {
                double isolationOffset = filter.GetIsolationWidthOffset(msLevel - 2);
                if (monoisotopicMZ == 0.0)
                {
                    monoisotopicMZ = filter.GetMass(msLevel - 2);
                }

                if (masterScanNumber == -1)
                {
                    masterScanNumber = FindPreviousPrecursor(scanNumber, msLevel, accessor);
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
                return (props, acquisitionProperties);
            }
            else
            {
                return (null, acquisitionProperties);
            }
        }

        Offset<SpectrumData> StoreSpectrumData(int scanNumber, ScanStatistics stats, FlatBufferBuilder bufferBuilder, IRawDataPlus accessor, IScanFilter filter)
        {
            Offset<SpectrumData> offset;
            if (CentroidSpectra && !stats.IsCentroidScan) {
                var stream = accessor.GetCentroidStream(scanNumber, true);
                var centroids = stream.GetCentroids();

                SpectrumData.StartMzVector(bufferBuilder, centroids.Count);
                foreach (var val in centroids.Reverse())
                {
                    bufferBuilder.AddDouble(val.Mass);
                }
                var mzOffset = bufferBuilder.EndVector();

                SpectrumData.StartIntensityVector(bufferBuilder, centroids.Count);
                foreach (var val in centroids.Reverse())
                {
                    bufferBuilder.AddFloat((float)val.Intensity);
                }
                var intensityOffset = bufferBuilder.EndVector();

                offset = SpectrumData.CreateSpectrumData(bufferBuilder, mzOffset, intensityOffset);
            }
            else {
                var segScan = accessor.GetSegmentedScanFromScanNumber(scanNumber, stats);

                SpectrumData.StartMzVector(bufferBuilder, segScan.PositionCount);
                foreach (var val in segScan.Positions.Reverse())
                {
                    bufferBuilder.AddDouble(val);
                }
                var mzOffset = bufferBuilder.EndVector();

                SpectrumData.StartIntensityVector(bufferBuilder, segScan.PositionCount);
                foreach (var val in segScan.Intensities.Reverse())
                {
                    bufferBuilder.AddFloat((float)val);
                }
                var intensityOffset = bufferBuilder.EndVector();
                offset = SpectrumData.CreateSpectrumData(bufferBuilder, mzOffset, intensityOffset);
            }
            return offset;
        }

        Dictionary<(MassAnalyzer, IonizationMode), long> FindAllMassAnalyzers() {
            var analyzers = new Dictionary<(MassAnalyzer, IonizationMode), long>();
            var accessor = GetHandle();
            var filters = accessor.GetFilters();
            int counter = 0;
            foreach(var filter in filters) {
                var a = AcquisitionProperties.CastMassAnalyzer(filter.MassAnalyzer);
                var i = AcquisitionProperties.CastIonizationMode(filter.IonizationMode);
                if(analyzers.ContainsKey((a, i))) {
                    continue;
                }
                analyzers.Add((a, i), counter);
                counter += 1;
            }
            return analyzers;
        }

        public ByteBuffer GetInstrumentInfo() {
            var accessor = GetHandle();
            var instrument = accessor.GetInstrumentData();

            FlatBufferBuilder builder = new FlatBufferBuilder(128);
            var name = builder.CreateString(instrument.Name);
            var version = builder.CreateString(instrument.HardwareVersion);
            var model = builder.CreateString(instrument.Model);
            var serialNumber = builder.CreateString(instrument.SerialNumber);
            var softwareVersion = builder.CreateString(instrument.SoftwareVersion);

            var n = InstrumentConfigsByComponents.Count;
            InstrumentModelT.StartConfigurationsVector(builder, n);
            foreach(var ((analyzer, ionizer), i) in InstrumentConfigsByComponents) {
                var conf = InstrumentConfigurationT.CreateInstrumentConfigurationT(builder, analyzer, ionizer);

            }
            var confOffsets = builder.EndVector();
            InstrumentModelT.StartInstrumentModelT(builder);
            InstrumentModelT.AddConfigurations(builder, confOffsets);
            InstrumentModelT.AddHardwareVersion(builder, version);
            InstrumentModelT.AddName(builder, name);
            InstrumentModelT.AddModel(builder, model);
            InstrumentModelT.AddSerialNumber(builder, serialNumber);
            InstrumentModelT.AddSoftwareVersion(builder, softwareVersion);
            var off = InstrumentModelT.EndInstrumentModelT(builder);
            builder.Finish(off.Value);
            return builder.DataBuffer;
        }

        Polarity GetPolarity(IScanFilter filter)
        {
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
            return polarity;
        }

        Offset<AcquisitionT> StoreAcquisition(FlatBufferBuilder builder, AcquisitionProperties acquisitionProperties, IScanFilter filter) {
            AcquisitionT.StartAcquisitionT(builder);
            AcquisitionT.AddInjectionTime(builder, (float)acquisitionProperties.InjectionTime);
            if (acquisitionProperties.CompensationVoltage.HasValue)
            {
                AcquisitionT.AddCompensationVoltage(builder, (float)acquisitionProperties.CompensationVoltage.Value);
            }
            AcquisitionT.AddLowMz(builder, acquisitionProperties.LowMZ);
            AcquisitionT.AddHighMz(builder, acquisitionProperties.HighMZ);
            AcquisitionT.AddMassAnalyzer(builder, acquisitionProperties.Analyzer);
            AcquisitionT.AddScanEvent(builder, acquisitionProperties.ScanEventNumber);
            AcquisitionT.AddIonizationMode(builder, AcquisitionProperties.CastIonizationMode(filter.IonizationMode));
            var acquisitionOffset = AcquisitionT.EndAcquisitionT(builder);
            return acquisitionOffset;
        }

        Offset<PrecursorT> StorePrecursor(FlatBufferBuilder builder, PrecursorProperties precursorProps) {
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
            return precursor;
        }

        public ByteBuffer SpectrumDescriptionFor(int scanNumber)
        {
            var accessor = GetHandle();
            var stats = accessor.GetScanStatsForScanNumber(scanNumber);
            SpectrumMode mode;
            if (CentroidSpectra) {
                mode = SpectrumMode.Centroid;
            } else {
                mode = stats.IsCentroidScan ? SpectrumMode.Centroid : SpectrumMode.Profile;
            }

            var filter = accessor.GetFilterForScanNumber(scanNumber);
            short level = MSLevelFromFilter(filter);
            Polarity polarity = GetPolarity(filter);


            var builder = new FlatBufferBuilder(1024);
            Offset<SpectrumData> dataOffset = new();

            if (IncludeSignal) {
                dataOffset = StoreSpectrumData(scanNumber, stats, builder, accessor, filter);
            }
            var filterString = filter.ToString();
            var filterStringOffset = builder.CreateString(filterString);

            var (precursorPropsOf, acquisitionProperties) = ExtractPrecursorAndTrailerMetadata(scanNumber, level, filter, accessor, stats);

            var acquisitionOffset = StoreAcquisition(builder, acquisitionProperties, filter);

            SpectrumDescription.StartSpectrumDescription(builder);
            if (IncludeSignal) {
                SpectrumDescription.AddData(builder, dataOffset);
            }
            SpectrumDescription.AddIndex(builder, stats.ScanNumber - 1);
            SpectrumDescription.AddMsLevel(builder, (byte)level);
            SpectrumDescription.AddPolarity(builder, polarity);
            SpectrumDescription.AddMode(builder, mode);
            SpectrumDescription.AddTime(builder, stats.StartTime);
            SpectrumDescription.AddFilterString(builder, filterStringOffset);
            SpectrumDescription.AddAcquisition(builder, acquisitionOffset);
            if (level > 1)
            {
                var precursor = StorePrecursor(builder, (PrecursorProperties)precursorPropsOf);
                SpectrumDescription.AddPrecursor(builder, precursor);
            }
            var description = SpectrumDescription.EndSpectrumDescription(builder);
            builder.Finish(description.Value);
            return builder.DataBuffer;
        }

        private Dictionary<int, List<int?>> BuildScanTypeMap() {
            var accessor = GetHandle();
            Dictionary<int, List<int?>> previousMSLevels = new();
            Dictionary<short, int?> lastMSLevels = new() {
                {1, null},
                {2, null},
                {3, null},
                {4, null},
                {5, null},
                {6, null},
                {7, null},
                {8, null},
                {9, null},
                {10, null},
            };

            var last = LastSpectrum();
            for(var i = FirstSpectrum(); i <= last; i++) {
                var filter = accessor.GetFilterForScanNumber(i);
                var msLevel = MSLevelFromFilter(filter);

                List<int?> backwards = new();
                for(short j = 1; j < msLevel + 1; j++) {
                    var o = lastMSLevels[j];
                    backwards.Add(o);
                }
                previousMSLevels[i] = backwards;

                lastMSLevels[msLevel] = i;
            }

            return previousMSLevels;
        }

        private RawFileReaderError Configure()
        {
            var accessor = GetHandleRaw();
            if (!File.Exists(Path))
            {
                return RawFileReaderError.FileNotFound;
            }
            if (accessor.IsError)
            {
                return RawFileReaderError.InvalidFormat;
            }
            else
            {
                accessor.SelectInstrument(Device.MS, 1);
            }
            InstrumentConfigsByComponents = FindAllMassAnalyzers();
            PreviousMSLevels = BuildScanTypeMap();

            var headers = accessor.GetTrailerExtraHeaderInformation();
            for(var i = 0; i < headers.Length; i++) {
                var header =  headers[i];
                var label = header.Label.TrimEnd(':');
                TrailerMap[label] = i;
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

        private unsafe static RawVec MemoryToRustVec(Span<byte> buffer, nuint size) {
            var vec = new RawVec();
            RustAllocateMemory(size, &vec);

            if ((IntPtr)vec.Data == IntPtr.Zero)
            {
                return vec;
            }
            fixed (byte* buf = buffer)
            {
                vec.Len = size;
                Unsafe.CopyBlock(vec.Data, buf, (uint)size);
                // Buffer.MemoryCopy(
                //     buf, vec.Data, vec.Capacity, vec.Len
                // );

            }
            return vec;
        }

        private unsafe static RawVec BufferToRustVec(byte[] buffer, nuint size)
        {
            var vec = new RawVec();
            RustAllocateMemory(size, &vec);

            if ((IntPtr)vec.Data == IntPtr.Zero)
            {
                return vec;
            }

            fixed (byte* buf = buffer)
            {
                vec.Len = size;
                Unsafe.CopyBlock(vec.Data, buf, (uint)size);
                // Buffer.MemoryCopy(
                //     buf, vec.Data, vec.Capacity, vec.Len
                // );

            }
            return vec;
        }

        [UnmanagedCallersOnly]
        public static unsafe IntPtr Open(IntPtr textPtr, int textLength)
        {
            var text = Marshal.PtrToStringAnsi(textPtr, textLength);
            var handle = new RawFileReader(text);
            IntPtr handleToken;
            lock (OpenHandles)
            {
                handleToken = HandleCounter;
                HandleCounter += 1;
                OpenHandles[handleToken] = handle;
            }
            return handleToken;
        }

        private static RawFileReader GetHandleForToken(IntPtr handleToken)
        {
            RawFileReader handle;
            lock (OpenHandles)
            {
                handle = OpenHandles[handleToken];
            }
            return handle;
        }

        /// <summary>
        /// Close the underlying handle, removing it from the map.
        /// </summary>
        /// <param name="handleToken"></param>
        [UnmanagedCallersOnly]
        public static unsafe void Close(IntPtr handleToken)
        {
            lock (OpenHandles)
            {
                if (OpenHandles.ContainsKey(handleToken))
                {
                    OpenHandles.Remove(handleToken);
                }
            }
        }

        [UnmanagedCallersOnly]
        public static unsafe void CloseAll()
        {
            OpenHandles.Clear();
        }

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

        [UnmanagedCallersOnly]
        public static unsafe uint Status(IntPtr handleToken)
        {
            try {
                RawFileReader reader = GetHandleForToken(handleToken);
                return (uint)reader.Status;
            } catch {
                return (uint) RawFileReaderError.HandleNotFound;
            }
        }

        [UnmanagedCallersOnly]
        public static unsafe uint GetSignalLoading(IntPtr handleToken) {
            RawFileReader reader = GetHandleForToken(handleToken);
            return (uint)(reader.IncludeSignal ? 1 : 0);
        }

        [UnmanagedCallersOnly]
        public static unsafe void SetSignalLoading(IntPtr handleToken, uint value) {
            RawFileReader reader = GetHandleForToken(handleToken);
            if (value == 0) {
                reader.IncludeSignal = false;
            } else {
                reader.IncludeSignal = true;
            }
        }

        [UnmanagedCallersOnly]
        public static unsafe uint GetCentroidSpectra(IntPtr handleToken)
        {
            RawFileReader reader = GetHandleForToken(handleToken);
            return (uint)(reader.CentroidSpectra ? 1 : 0);
        }

        [UnmanagedCallersOnly]
        public static unsafe void SetCentroidSpectra(IntPtr handleToken, uint value)
        {
            RawFileReader reader = GetHandleForToken(handleToken);
            if (value == 0)
            {
                reader.CentroidSpectra = false;
            }
            else
            {
                reader.CentroidSpectra = true;
            }
        }

        /// <summary>
        /// Get a `SpectrumDescription` FlatBuffer message for a specific spectrum from a RAW file
        /// </summary>
        /// <param name="handleToken">The token corresponding to the `RawFileReader` handle</param>
        /// <param name="scanNumber">The scan number of the spectrum to retrieve</param>
        /// <returns>A `RawVec` representing Rust-allocated memory that holds the FlatBuffer message</returns>
        [UnmanagedCallersOnly]
        public static unsafe RawVec SpectrumDescriptionFor(IntPtr handleToken, int scanNumber)
        {
            RawFileReader reader = GetHandleForToken(handleToken);
            var buffer = reader.SpectrumDescriptionFor(scanNumber);
            var bytes = buffer.ToSpan(buffer.Position, buffer.Length - buffer.Position);
            var size = bytes.Length;
            return MemoryToRustVec(bytes, (nuint)size);
        }

        /// <summary>
        /// Get an `InstrumentModel` FlatBuffer message describing the instrument used to acquire a RAW file
        /// </summary>
        /// <param name="handleToken">The token corresponding to the `RawFileReader` handle</param>
        /// <returns>A `RawVec` representing Rust-allocated memory that holds the FlatBuffer message</returns>
        [UnmanagedCallersOnly]
        public static unsafe RawVec InstrumentModel(IntPtr handleToken) {
            RawFileReader reader = GetHandleForToken(handleToken);
            var buffer = reader.GetInstrumentInfo();
            var bytes = buffer.ToSpan(buffer.Position, buffer.Length - buffer.Position);
            var size = bytes.Length;
            return MemoryToRustVec(bytes, (nuint)size);
        }
    }
}
