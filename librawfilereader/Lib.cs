using System;
using System.IO;
using System.Runtime.InteropServices;

using ThermoFisher.CommonCore.Data.Business;
using ThermoFisher.CommonCore.Data.Interfaces;
using ThermoFisher.CommonCore.RawFileReader;
using ThermoFisher.CommonCore.Data.FilterEnums;
using ThermoFisher.CommonCore.RandomAccessReaderPlugin;

using Google.FlatBuffers;
using System.Collections.Generic;
using System.Runtime.CompilerServices;
using System.Linq;
using System.Text;
using System.Diagnostics;

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
        public List<double> CompensationVoltage = new List<double>();
        public MassAnalyzer Analyzer;
        public IonizationMode Ionization;
        public double LowMZ;
        public double HighMZ;
        public int ScanEventNumber;
        public float? Resolution;

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
        public static IonizationMode CastIonizationMode(IonizationModeType ionizationModeType)
        {
            int value = (int)ionizationModeType;
            return (IonizationMode)value;
        }

        public AcquisitionProperties(double injectionTime, List<double> compensationVoltage, MassAnalyzerType analyzer, double lowMZ, double highMZ, int scanEventNumber, float? resolution)
        {
            InjectionTime = injectionTime;
            CompensationVoltage = compensationVoltage ?? new List<double>();
            Analyzer = CastMassAnalyzer(analyzer);
            LowMZ = lowMZ;
            HighMZ = highMZ;
            ScanEventNumber = scanEventNumber;
            Resolution = resolution;
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
        Dictionary<short, uint> MSLevelCounts;

        /// <summary>
        /// An index look up mapping trailer keys by index that lets us avoid
        /// looping over all trailer entries
        /// </summary>
        Dictionary<string, int> TrailerMap;
        HeaderItem[] Headers;

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
            Manager = RawFileReaderAdapter.RandomAccessThreadedFileFactory(Path, RandomAccessFileManager.Instance);
            // Manager = RawFileReaderAdapter.ThreadedFileFactory(Path);
            // Handle = RawFileReaderAdapter.FileFactory(Path);
            Status = RawFileReaderError.Ok;
            InstrumentConfigsByComponents = new();
            PreviousMSLevels = new();
            TrailerMap = new();
            MSLevelCounts = new();
            Status = Configure();
        }

        IRawDataPlus GetHandleRaw()
        {
            var accessor = Manager.CreateThreadAccessor();
            return accessor;
            // return Handle;
        }

        IRawDataPlus GetHandle()
        {
            var accessor = GetHandleRaw();
            accessor.SelectInstrument(Device.MS, 1);
            accessor.IncludeReferenceAndExceptionData = true;
            return accessor;
            // return Handle;
        }

        public string FileErrorMessage() {
            var accessor = GetHandleRaw();
            string buffer = "";
            if (accessor.IsError) {
                if (accessor.FileError.HasError) {
                    buffer += accessor.FileError.ErrorMessage + "\n";
                }
                if (accessor.FileError.HasWarning) {
                    buffer += accessor.FileError.WarningMessage + "\n";
                }
            }
            return buffer;
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
            {MSOrderType.Ng, 2},
            {MSOrderType.Nl, 2},
            {MSOrderType.Par, 2},
        };

        /// <summary>
        /// Get the MS level of the scan as an integer
        /// </summary>
        /// <param name="filter">The scan filter for a given scan which holds much of the metadata</param>
        /// <returns>The MS level of the scan</returns>
        short MSLevelFromFilter(IScanFilter filter)
        {
            short msLevel;
            if (MSLevelMap.TryGetValue(filter.MSOrder, out msLevel))
            {
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
            if (cacheLookUp != null)
            {
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

        private const string InjectionTimeKey = "Ion Injection Time (ms)";
        private const string ScanEventKey = "Scan Evnet";
        private const string MasterScanKey = "Master Scan";
        private const string MonoisotopicMZKey = "Monoisotopic M/Z";
        private const string ChargeStateKey = "Charge State";
        private static readonly string[] IsolationLevelKeys = [
            "MS2 Isolation Width",
            "MS3 Isolation Width",
            "MS4 Isolation Width",
            "MS5 Isolation Width",
            "MS6 Isolation Width",
            "MS7 Isolation Width",
            "MS8 Isolation Width",
            "MS9 Isolation Width",
            "MS10 Isolation Width"
        ];

        private static readonly string OrbitrapResolutionKey = "Orbitrap Resolution";
        private static readonly string FTResolutionKey = "FT Resolution";

        bool GetShortTrailerExtraFor(IRawDataPlus accessor, int scanNumber, string key, out short value)
        {
            object tmp;
            HeaderItem header;
            int headerIdx;

            if (TrailerMap.TryGetValue(key, out headerIdx))
            {
                tmp = accessor.GetTrailerExtraValue(scanNumber, headerIdx);
                header = Headers[headerIdx];
                if (tmp != null)
                {
                    try {
                        switch (header.DataType)
                        {
                            case GenericDataTypes.SHORT:
                                {
                                    value = (short)tmp;
                                    return true;
                                }
                            case GenericDataTypes.LONG:
                                {
                                    value = Convert.ToInt16(tmp);
                                    return true;
                                }
                            case GenericDataTypes.ULONG:
                                {
                                    value = Convert.ToInt16(tmp);
                                    return true;
                                }
                            case GenericDataTypes.USHORT:
                                {
                                    value = (short)(ushort)tmp;
                                    return true;
                                }
                            default:
                                {
                                    value = Convert.ToInt16(tmp);
                                    return true;
                                }
                        }
                    } catch (InvalidCastException) {
                        value = Convert.ToInt16(tmp);
                        return true;
                    }

                }
            }
            value = 0;
            return false;
        }

        bool GetIntTrailerExtraFor(IRawDataPlus accessor, int scanNumber, string key, out int value, int defaultValue=0)
        {
            object tmp;
            HeaderItem header;
            int headerIdx;

            if (TrailerMap.TryGetValue(key, out headerIdx))
            {
                tmp = accessor.GetTrailerExtraValue(scanNumber, headerIdx);
                header = Headers[headerIdx];
                if (tmp != null)
                {
                    try {
                        switch (header.DataType)
                        {
                            case GenericDataTypes.SHORT:
                                {
                                    value = (short)tmp;
                                    return true;
                                }
                            case GenericDataTypes.LONG:
                                {
                                    value = (int)(long)tmp;
                                    return true;
                                }
                            case GenericDataTypes.ULONG:
                                {
                                    value = (int)(ulong)tmp;
                                    return true;
                                }
                            case GenericDataTypes.USHORT:
                                {
                                    value = (ushort)tmp;
                                    return true;
                                }
                            default:
                                {
                                    value = Convert.ToInt32(tmp);
                                    return true;
                                }
                        }
                    }
                    catch (InvalidCastException) {
                        value = Convert.ToInt32(tmp);
                        return true;
                    }
                }
            }
            value = defaultValue;
            return false;
        }

        bool GetDoubleTrailerExtraFor(IRawDataPlus accessor, int scanNumber, string key, out double value) {
            object tmp;
            HeaderItem header;
            int headerIdx;

            if (TrailerMap.TryGetValue(key, out headerIdx))
            {
                tmp = accessor.GetTrailerExtraValue(scanNumber, headerIdx);
                header = Headers[headerIdx];
                if (tmp != null)
                {
                    try {
                        switch (header.DataType)
                        {
                            case GenericDataTypes.FLOAT:
                                {
                                    value = (float)tmp;
                                    return true;
                                }
                            case GenericDataTypes.DOUBLE:
                                {
                                    value = (double)tmp;
                                    return true;
                                };
                            default:
                                {
                                    value = Convert.ToDouble(tmp);
                                    return true;
                                }
                        }
                    } catch (InvalidCastException) {
                        value = Convert.ToDouble(tmp);
                        return true;
                    }
                }
            }
            value = 0;
            return false;
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
            double resolution = 0.0;
            float? resolution_opt = null;

            GetDoubleTrailerExtraFor(accessor, scanNumber, InjectionTimeKey, out injectionTime);
            GetShortTrailerExtraFor(accessor, scanNumber, ScanEventKey, out scanEventNum);

            if (msLevel > 1)
            {
                GetIntTrailerExtraFor(accessor, scanNumber, MasterScanKey, out masterScanNumber, -1);
                GetDoubleTrailerExtraFor(accessor, scanNumber, MonoisotopicMZKey, out monoisotopicMZ);
                GetShortTrailerExtraFor(accessor, scanNumber, ChargeStateKey, out precursorCharge);
                GetDoubleTrailerExtraFor(accessor, scanNumber, IsolationLevelKeys[msLevel - 2], out isolationWidth);
            }

            if (!GetDoubleTrailerExtraFor(accessor, scanNumber, OrbitrapResolutionKey, out resolution)) {
                GetDoubleTrailerExtraFor(accessor, scanNumber, FTResolutionKey, out resolution);
            };
            resolution_opt = resolution == 0.0 ? null : (float)resolution;

            AcquisitionProperties acquisitionProperties = new AcquisitionProperties(injectionTime, new List<double>(), filter.MassAnalyzer, stats.LowMass, stats.HighMass, scanEventNum, resolution_opt);

            if (filter.CompensationVoltage == TriState.On)
            {
                for (int i = 0; i < filter.CompensationVoltageCount; i++)
                {
                    acquisitionProperties.CompensationVoltage.Add(filter.CompensationVoltageValue(i));
                }
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

        Offset<SpectrumData> StoreSpectrumData(int scanNumber, ScanStatistics stats, FlatBufferBuilder bufferBuilder, IRawDataPlus accessor, bool centroidSpectra)
        {
            // We have to write arrays in reverse order because FlatBuffers writes entries back-to-front.
            // By writing them in reverse here, we can read them out in the expected order on the other side
            // in Rust.
            Offset<SpectrumData> offset;
            if (centroidSpectra && !stats.IsCentroidScan)
            {
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
            else
            {
                var segScan = accessor.GetSegmentedScanFromScanNumber(scanNumber, null);

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

        Dictionary<(MassAnalyzer, IonizationMode), long> FindAllMassAnalyzers()
        {
            var analyzers = new Dictionary<(MassAnalyzer, IonizationMode), long>();
            var accessor = GetHandle();
            var events = accessor.ScanEvents;

            int counter = 0;
            for (var segmentIdx = 0; segmentIdx < events.Segments; segmentIdx++) {
                for(var eventIdx = 0; eventIdx < events.GetEventCount(segmentIdx); eventIdx++) {
                    var ev = events.GetEvent(segmentIdx, eventIdx);
                    var a = AcquisitionProperties.CastMassAnalyzer(ev.MassAnalyzer);
                    var i = AcquisitionProperties.CastIonizationMode(ev.IonizationMode);
                    if (analyzers.ContainsKey((a, i)))
                    {
                        continue;
                    }
                    analyzers.Add((a, i), counter);
                    counter += 1;
                }
            }
            return analyzers;
        }

        public ByteBuffer GetInstrumentInfo()
        {
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
            foreach (var ((analyzer, ionizer), i) in InstrumentConfigsByComponents)
            {
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

        public uint GetInstrumentMethodCount()  {
            var accessor = GetHandle();
            return (uint)accessor.InstrumentMethodsCount;
        }

        public ByteBuffer GetInstrumentMethodFor(int method) {
            var accessor = GetHandle();
            string methodText = accessor.GetInstrumentMethod(method);

            string[] displayNames = accessor.GetAllInstrumentFriendlyNamesFromInstrumentMethod();
            string[] names = accessor.GetAllInstrumentNamesFromInstrumentMethod();

            var builder = new FlatBufferBuilder(4096);
            var textOffset = builder.CreateString(methodText);
            StringOffset nameOffset = new();
            StringOffset displayNameOffset = new();
            if (method < names.Length) {
                nameOffset = builder.CreateString(names[method]);
                displayNameOffset = builder.CreateString(displayNames[method]);
            }

            InstrumentMethodT.StartInstrumentMethodT(builder);
            InstrumentMethodT.AddIndex(builder, (byte)method);
            InstrumentMethodT.AddText(builder, textOffset);
            if (method < names.Length) {
                InstrumentMethodT.AddName(builder, nameOffset);
                InstrumentMethodT.AddDisplayName(builder, displayNameOffset);
            }

            var description = InstrumentMethodT.EndInstrumentMethodT(builder);
            builder.Finish(description.Value);
            return builder.DataBuffer;
        }

        public Offset<ChromatogramData> StoreChromatogramData(ChromatogramSignal signal, FlatBufferBuilder bufferBuilder) {
            ChromatogramData.StartTimeVector(bufferBuilder, signal.Times.Count);
            foreach(var t in signal.Times.Reverse()) {
                bufferBuilder.AddDouble(t);
            }
            var timeOffset = bufferBuilder.EndVector();

            ChromatogramData.StartIntensityVector(bufferBuilder, signal.Times.Count);
            foreach (var i in signal.Intensities.Reverse()) {
                bufferBuilder.AddFloat((float)i);
            }
            var intensityOffset = bufferBuilder.EndVector();

            ChromatogramData.StartChromatogramData(bufferBuilder);
            ChromatogramData.AddTime(bufferBuilder, timeOffset);
            ChromatogramData.AddIntensity(bufferBuilder, intensityOffset);
            var offset = ChromatogramData.EndChromatogramData(bufferBuilder);
            return offset;
        }

        public ByteBuffer GetAdvancedPacketData(int scanNumber, bool includeSampledNoise=true) {
            var accessor = GetHandle();
            var packetData = accessor.GetAdvancedPacketData(scanNumber);
            var peakData = packetData.CentroidData;

            FlatBufferBuilder builder = new FlatBufferBuilder(1024);

            VectorOffset? noiseOffset = null;
            VectorOffset? baselineOffset = null;
            VectorOffset? massOffset = null;
            VectorOffset? chargeOffset = null;
            VectorOffset? resolutionsOffset = null;

            if (peakData.Noises != null) {
                ExtendedSpectrumDataT.StartNoiseVector(builder, peakData.Noises.Length);
                foreach (var b in peakData.Noises.Reverse())
                {
                    builder.AddFloat((float)b);
                }
                noiseOffset = builder.EndVector();
            }

            if (peakData.Baselines != null)
            {
                ExtendedSpectrumDataT.StartBaselineVector(builder, peakData.Baselines.Length);
                foreach (var b in peakData.Baselines.Reverse())
            {
                builder.AddFloat((float)b);
            }
                baselineOffset = builder.EndVector();
            }

            if (peakData.Masses != null) {
                ExtendedSpectrumDataT.StartMassVector(builder, peakData.Masses.Length);
                foreach (var b in peakData.Masses.Reverse())
                {
                    builder.AddDouble(b);
                }
                massOffset = builder.EndVector();
            }

            if (peakData.Charges != null) {
                ExtendedSpectrumDataT.StartChargeVector(builder, peakData.Charges.Length);
                foreach (var b in peakData.Charges.Reverse()) {
                    builder.AddFloat((float)b);
                }
                chargeOffset = builder.EndVector();
            }


            if (peakData.Resolutions != null)  {
                ExtendedSpectrumDataT.StartResolutionVector(builder, peakData.Resolutions.Length);
                foreach (var b in peakData.Resolutions.Reverse())
                {
                    builder.AddFloat((float)b);
                }
                resolutionsOffset = builder.EndVector();
            }

            VectorOffset? noiseBaselineOffset = null;
            VectorOffset? noiseMzOffset = null;
            VectorOffset? sampledNoiseOffset = null;
            if (includeSampledNoise) {
                var noiseData = packetData.NoiseData;
                ExtendedSpectrumDataT.StartSampledNoiseBaselineVector(builder, noiseData.Length);
                foreach (var b in noiseData.Reverse()) {
                    builder.AddFloat(b.Baseline);
                }
                noiseBaselineOffset = builder.EndVector();

                ExtendedSpectrumDataT.StartSampledNoiseMzVector(builder, noiseData.Length);
                foreach (var b in noiseData.Reverse())
                {
                    builder.AddFloat(b.Mass);
                }
                noiseMzOffset = builder.EndVector();

                ExtendedSpectrumDataT.StartSampledNoiseVector(builder, noiseData.Length);
                foreach (var b in noiseData.Reverse())
                {
                    builder.AddFloat(b.Noise);
                }
                sampledNoiseOffset = builder.EndVector();
            }

            ExtendedSpectrumDataT.StartExtendedSpectrumDataT(builder);
            if (noiseOffset.HasValue) ExtendedSpectrumDataT.AddNoise(builder, noiseOffset.Value);
            if (baselineOffset.HasValue) ExtendedSpectrumDataT.AddBaseline(builder, baselineOffset.Value);
            if (massOffset.HasValue) ExtendedSpectrumDataT.AddMass(builder, massOffset.Value);
            if (chargeOffset.HasValue) ExtendedSpectrumDataT.AddCharge(builder, chargeOffset.Value);
            if (resolutionsOffset.HasValue) ExtendedSpectrumDataT.AddResolution(builder, resolutionsOffset.Value);

            if (includeSampledNoise) {
                if (noiseBaselineOffset.HasValue)
                    ExtendedSpectrumDataT.AddSampledNoiseBaseline(builder, noiseBaselineOffset.Value);
                if (noiseMzOffset.HasValue)
                    ExtendedSpectrumDataT.AddSampledNoiseMz(builder, noiseMzOffset.Value);
                if (sampledNoiseOffset.HasValue)
                    ExtendedSpectrumDataT.AddSampledNoise(builder, sampledNoiseOffset.Value);
            }

            var offset = ExtendedSpectrumDataT.EndExtendedSpectrumDataT(builder);
            builder.Finish(offset.Value);
            return builder.DataBuffer;
        }

        public ByteBuffer GetRawTrailersForScan(int scanNumber) {
            var accessor = GetHandle();
            var trailers = accessor.GetTrailerExtraInformation(scanNumber);

            FlatBufferBuilder builder = new FlatBufferBuilder(1024);

            var trailerOffsets = new Offset<TrailerValueT>[trailers.Length];

            for(var i = 0; i < trailers.Length; i++) {
                var label = trailers.Labels[i];
                var trimmed = label.Substring(0, label.Length - 1);
                var labelOffset = builder.CreateString(trimmed);
                var valueOffset = builder.CreateString(trailers.Values[i]);
                var trailerOffset = TrailerValueT.CreateTrailerValueT(builder, labelOffset, valueOffset);
                trailerOffsets[i] = trailerOffset;
            }

            var trailersVecOffset = builder.CreateVectorOfTables(trailerOffsets);
            TrailerValuesT.StartTrailerValuesT(builder);
            TrailerValuesT.AddTrailers(builder, trailersVecOffset);
            var offset = TrailerValuesT.EndTrailerValuesT(builder);
            builder.Finish(offset.Value);
            return builder.DataBuffer;
        }

        public ByteBuffer GetSummaryTrace(TraceType traceType) {
            var accessor = GetHandle();
            var ticSettings = new ChromatogramTraceSettings(traceType);
            var tic = accessor.GetChromatogramDataEx([ticSettings], FirstSpectrum(), LastSpectrum());
            var signals = ChromatogramSignal.FromChromatogramData(tic);
            var signal = signals[0];

            var builder = new FlatBufferBuilder(4096);
            Offset<ChromatogramData> dataOffset = StoreChromatogramData(signal, builder);

            ChromatogramDescription.StartChromatogramDescription(builder);
            ChromatogramDescription.AddTraceType(builder, (TraceTypeT)(int)traceType);
            ChromatogramDescription.AddStartIndex(builder, FirstSpectrum());
            ChromatogramDescription.AddEndIndex(builder, LastSpectrum());
            ChromatogramDescription.AddData(builder, dataOffset);
            var description = ChromatogramDescription.EndChromatogramDescription(builder);
            builder.Finish(description.Value);
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

        Offset<AcquisitionT> StoreAcquisition(FlatBufferBuilder builder, AcquisitionProperties acquisitionProperties, IScanFilter filter)
        {
            VectorOffset voltagesOffset = default;
            if (acquisitionProperties.CompensationVoltage.Count > 0)
            {
                // Convert List<double> to float[] for FlatBuffer
                float[] voltageArray = acquisitionProperties.CompensationVoltage.Select(v => (float)v).ToArray();
                voltagesOffset = AcquisitionT.CreateCompensationVoltagesVector(builder, voltageArray);
            }
            AcquisitionT.StartAcquisitionT(builder);
            AcquisitionT.AddInjectionTime(builder, (float)acquisitionProperties.InjectionTime);
            if (acquisitionProperties.CompensationVoltage.Count > 0)
            {
                AcquisitionT.AddCompensationVoltages(builder, voltagesOffset);
            }
            AcquisitionT.AddLowMz(builder, acquisitionProperties.LowMZ);
            AcquisitionT.AddHighMz(builder, acquisitionProperties.HighMZ);
            AcquisitionT.AddMassAnalyzer(builder, acquisitionProperties.Analyzer);
            AcquisitionT.AddScanEvent(builder, acquisitionProperties.ScanEventNumber);
            AcquisitionT.AddIonizationMode(builder, AcquisitionProperties.CastIonizationMode(filter.IonizationMode));
            if (acquisitionProperties.Resolution.HasValue) {
                AcquisitionT.AddResolution(builder, acquisitionProperties.Resolution.Value);
            }
            var acquisitionOffset = AcquisitionT.EndAcquisitionT(builder);
            return acquisitionOffset;
        }

        Offset<PrecursorT> StorePrecursor(FlatBufferBuilder builder, PrecursorProperties precursorProps)
        {
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

        public ByteBuffer GetFileMetadata() {
            var accessor = GetHandle();
            var builder = new FlatBufferBuilder(1024);
            var description = accessor.FileHeader.FileDescription;
            var date = accessor.FileHeader.CreationDate.ToString("o");

            SampleInformation sampleInfo = accessor.SampleInformation;
            var sampleID = sampleInfo.SampleId;
            var sampleVial = sampleInfo.Vial;
            var sampleComment = sampleInfo.Comment;
            var sampleName = sampleInfo.SampleName;

            var counts = new uint[10];
            foreach(var (k, v) in MSLevelCounts) {
                counts[k - 1] = v;
            }

            var dateOffset = builder.CreateString(date);

            var sampleIDOffset = builder.CreateString(sampleID);
            var sampleNameOffset = builder.CreateString(sampleName);
            var sampleVialOffset = builder.CreateString(sampleVial);
            var sampleCommentOffset = builder.CreateString(sampleComment);

            var pathOffset = builder.CreateString(Path);
            var countsOffset = FileDescriptionT.CreateSpectraPerMsLevelVector(builder, counts);

            StringOffset[] headerOffsets = new StringOffset[Headers.Length];
            for(var i = 0; i < Headers.Length; i++) {
                var header = Headers[i];
                var offset = builder.CreateString(header.Label);
                headerOffsets[i] = offset;
            }
            var headersOffset = FileDescriptionT.CreateTrailerHeadersVector(builder, headerOffsets);

            FileDescriptionT.StartFileDescriptionT(builder);
            FileDescriptionT.AddCreationDate(builder, dateOffset);
            FileDescriptionT.AddSampleId(builder, sampleIDOffset);
            FileDescriptionT.AddSampleComment(builder, sampleCommentOffset);
            FileDescriptionT.AddSampleName(builder, sampleNameOffset);
            FileDescriptionT.AddSampleVial(builder, sampleVialOffset);
            FileDescriptionT.AddSourceFile(builder, pathOffset);
            FileDescriptionT.AddSpectraPerMsLevel(builder, countsOffset);
            FileDescriptionT.AddTrailerHeaders(builder, headersOffset);
            var fileDescOffset = FileDescriptionT.EndFileDescriptionT(builder);

            builder.Finish(fileDescOffset.Value);
            return builder.DataBuffer;
        }

        public ByteBuffer SpectrumDescriptionFor(int scanNumber) {
            return SpectrumDescriptionFor(scanNumber, IncludeSignal, CentroidSpectra);
        }

        public ByteBuffer SpectrumDataFor(int scanNumber, bool centroidSpectra)
        {
            var accessor = GetHandle();
            var stats = accessor.GetScanStatsForScanNumber(scanNumber);
            SpectrumMode mode;
            if (centroidSpectra)
            {
                mode = SpectrumMode.Centroid;
            }
            else
            {
                mode = stats.IsCentroidScan ? SpectrumMode.Centroid : SpectrumMode.Profile;
            }

            // Centroid spectra are usually compact, but profile spectra are much bigger
            var builder = new FlatBufferBuilder(mode == SpectrumMode.Centroid ? 8192 : 262144);
            Offset<SpectrumData> dataOffset;
            if (mode == SpectrumMode.Centroid && !centroidSpectra)
            {
                SpectrumData.StartMzVector(builder, 0);
                var mzOffset = builder.EndVector();
                SpectrumData.StartIntensityVector(builder, 0);
                var intOffset = builder.EndVector();
                dataOffset = SpectrumData.CreateSpectrumData(builder, mzOffset, intOffset);
            }
            else
            {
                dataOffset = StoreSpectrumData(scanNumber, stats, builder, accessor, centroidSpectra);
            }
            builder.Finish(dataOffset.Value);
            return builder.DataBuffer;
        }

        public ByteBuffer SpectrumDescriptionFor(int scanNumber, bool includeSignal, bool centroidSpectra)
        {
            var accessor = GetHandle();
            var stats = accessor.GetScanStatsForScanNumber(scanNumber);
            SpectrumMode mode;
            if (centroidSpectra)
            {
                mode = SpectrumMode.Centroid;
            }
            else
            {
                mode = stats.IsCentroidScan ? SpectrumMode.Centroid : SpectrumMode.Profile;
            }

            var filter = accessor.GetFilterForScanNumber(scanNumber);
            short level = MSLevelFromFilter(filter);
            Polarity polarity = GetPolarity(filter);

            // Centroid spectra are usually compact, but profile spectra are much bigger
            var builder = new FlatBufferBuilder(mode == SpectrumMode.Centroid ? 8192 : 262144);
            Offset<SpectrumData> dataOffset = new();

            if (includeSignal)
            {
                dataOffset = StoreSpectrumData(scanNumber, stats, builder, accessor, centroidSpectra);
            }

            ScanMode modeT = (ScanMode)(byte)filter.ScanMode;
            var filterString = filter.ToString();
            var filterStringOffset = builder.CreateString(filterString);

            var (precursorPropsOf, acquisitionProperties) = ExtractPrecursorAndTrailerMetadata(scanNumber, level, filter, accessor, stats);

            var acquisitionOffset = StoreAcquisition(builder, acquisitionProperties, filter);

            SpectrumDescription.StartSpectrumDescription(builder);
            if (includeSignal)
            {
                SpectrumDescription.AddData(builder, dataOffset);
            }
            SpectrumDescription.AddIndex(builder, stats.ScanNumber - 1);
            SpectrumDescription.AddMsLevel(builder, (byte)level);
            SpectrumDescription.AddPolarity(builder, polarity);
            SpectrumDescription.AddMode(builder, mode);
            SpectrumDescription.AddTime(builder, stats.StartTime);
            SpectrumDescription.AddFilterString(builder, filterStringOffset);
            SpectrumDescription.AddAcquisition(builder, acquisitionOffset);
            SpectrumDescription.AddMsOrder(builder, (MSOrder)filter.MSOrder);
            SpectrumDescription.AddScanMode(builder, modeT);
            if (level > 1)
            {
                var precursor = StorePrecursor(builder, (PrecursorProperties)precursorPropsOf);
                SpectrumDescription.AddPrecursor(builder, precursor);
            }
            var description = SpectrumDescription.EndSpectrumDescription(builder);
            builder.Finish(description.Value);
            return builder.DataBuffer;
        }

        public ByteBuffer StatusLogs() {
            var accessor = GetHandle();

            var builder = new FlatBufferBuilder(262144);

            var floatLogs = new Dictionary<string, StatusLog<double>>();
            var intLogs = new Dictionary<string, StatusLog<long>>();
            var stringLogs = new Dictionary<string, StatusLog<string>>();
            var boolLogs = new Dictionary<string, StatusLog<bool>>();

            var nEntries = accessor.GetStatusLogEntriesCount();

            for(var i = 0; i < nEntries; i++) {
                var logsFor = accessor.GetStatusLogEntry(i);

                foreach(var (datum, header) in logsFor.Values.Zip(accessor.GetStatusLogHeaderInformation())) {
                    var dType = header.DataType;
                    if (dType == GenericDataTypes.NULL)
                    {
                        continue;
                    }
                    switch (dType)
                    {
                        case GenericDataTypes.YESNO:
                        case GenericDataTypes.ONOFF:
                            {
                                if (!boolLogs.ContainsKey(header.Label))
                                {
                                    boolLogs.Add(header.Label, new StatusLog<bool>(header.Label));
                                }

                                boolLogs[header.Label].Add((bool)datum, logsFor.Time);
                                break;
                            }
                        case GenericDataTypes.Bool:
                            {
                                if (!boolLogs.ContainsKey(header.Label))
                                {
                                    boolLogs.Add(header.Label, new StatusLog<bool>(header.Label));
                                }

                                boolLogs[header.Label].Add((bool)datum, logsFor.Time);
                                break;
                            }
                        case GenericDataTypes.CHAR:
                            {
                                if (!stringLogs.ContainsKey(header.Label))
                                {
                                    stringLogs.Add(header.Label, new StatusLog<string>(header.Label));
                                }
                                stringLogs[header.Label].Add(datum.ToString(), logsFor.Time);
                                break;
                            }
                        case GenericDataTypes.CHAR_STRING:
                            {
                                if (!stringLogs.ContainsKey(header.Label))
                                {
                                    stringLogs.Add(header.Label, new StatusLog<string>(header.Label));
                                }
                                stringLogs[header.Label].Add(datum.ToString(), logsFor.Time);
                                break;
                            }
                        case GenericDataTypes.WCHAR_STRING:
                            {
                                if (!stringLogs.ContainsKey(header.Label))
                                {
                                    stringLogs.Add(header.Label, new StatusLog<string>(header.Label));
                                }
                                stringLogs[header.Label].Add((string)datum, logsFor.Time);
                                break;
                            }
                        case GenericDataTypes.FLOAT:
                            {
                                if (!floatLogs.ContainsKey(header.Label))
                                {
                                    floatLogs.Add(header.Label, new StatusLog<double>(header.Label));
                                }

                                floatLogs[header.Label].Add((float)datum, logsFor.Time);
                                break;
                            }
                        case GenericDataTypes.DOUBLE:
                            {
                                if (!floatLogs.ContainsKey(header.Label))
                                {
                                    floatLogs.Add(header.Label, new StatusLog<double>(header.Label));
                                }
                                floatLogs[header.Label].Add((double)datum, logsFor.Time);
                                break;
                            }
                        case GenericDataTypes.Int:
                            {
                                if (!intLogs.ContainsKey(header.Label))
                                {
                                    intLogs.Add(header.Label, new StatusLog<long>(header.Label));
                                }
                                intLogs[header.Label].Add((int)datum, logsFor.Time);
                                break;
                            }
                        case GenericDataTypes.ULONG:
                            {

                                if (!intLogs.ContainsKey(header.Label))
                                {
                                    intLogs.Add(header.Label, new StatusLog<long>(header.Label));
                                }
                                intLogs[header.Label].Add((uint)datum, logsFor.Time);
                                break;
                            }
                        case GenericDataTypes.SHORT:
                            {
                                if (!intLogs.ContainsKey(header.Label))
                                {
                                    intLogs.Add(header.Label, new StatusLog<long>(header.Label));
                                }
                                intLogs[header.Label].Add((short)datum, logsFor.Time);
                                break;
                            }
                        case GenericDataTypes.USHORT:
                            {
                                if (!intLogs.ContainsKey(header.Label))
                                {
                                    intLogs.Add(header.Label, new StatusLog<long>(header.Label));
                                }
                                intLogs[header.Label].Add((ushort)datum, logsFor.Time);
                                break;
                            }
                        default:
                            {
                                System.Console.Error.WriteLine("Skipping {0} {1}", header.Label, header.DataType);
                                break;
                            }
                    }
                }
            }

            var floatLogOffsets = new List<Offset<StatusLogFloatT>>();
            foreach(var log in floatLogs.Values) {
                var nameOffset = builder.CreateString(log.Name);
                var timeOffset = StatusLogFloatT.CreateTimesVector(builder, log.Time.ToArray());
                var dataOffset = StatusLogFloatT.CreateValuesVector(builder, log.Data.ToArray());
                var tOffset = StatusLogFloatT.CreateStatusLogFloatT(builder, nameOffset, timeOffset, dataOffset);
                floatLogOffsets.Add(tOffset);
            }

            var intLogOffsets = new List<Offset<StatusLogIntT>>();
            foreach (var log in intLogs.Values)
            {
                var nameOffset = builder.CreateString(log.Name);
                var timeOffset = StatusLogIntT.CreateTimesVector(builder, log.Time.ToArray());
                var dataOffset = StatusLogIntT.CreateValuesVector(builder, log.Data.ToArray());
                var tOffset = StatusLogIntT.CreateStatusLogIntT(builder, nameOffset, timeOffset, dataOffset);
                intLogOffsets.Add(tOffset);
            }

            var boolLogOffsets = new List<Offset<StatusLogBoolT>>();
            foreach (var log in boolLogs.Values)
            {
                var nameOffset = builder.CreateString(log.Name);
                var timeOffset = StatusLogBoolT.CreateTimesVector(builder, log.Time.ToArray());
                var dataOffset = StatusLogBoolT.CreateValuesVector(builder, log.Data.ToArray());
                var tOffset = StatusLogBoolT.CreateStatusLogBoolT(builder, nameOffset, timeOffset, dataOffset);
                boolLogOffsets.Add(tOffset);
            }

            var stringLogOffsets = new List<Offset<StatusLogStringT>>();
            foreach (var log in stringLogs.Values)
            {
                var nameOffset = builder.CreateString(log.Name);
                var timeOffset = StatusLogStringT.CreateTimesVector(builder, log.Time.ToArray());
                var stringOffsets = new StringOffset [log.Data.Count];
                for(var i = 0; i < log.Data.Count; i++) {
                    stringOffsets[i] = builder.CreateString(log.Data[i]);
                }
                var dataOffset = StatusLogStringT.CreateValuesVector(builder, stringOffsets);
                var tOffset = StatusLogStringT.CreateStatusLogStringT(builder, nameOffset, timeOffset, dataOffset);
                stringLogOffsets.Add(tOffset);
            }

            var boolVecOffset = StatusLogCollectionT.CreateBoolLogsVector(builder, boolLogOffsets.ToArray());
            var floatVecOffset = StatusLogCollectionT.CreateFloatLogsVector(builder, floatLogOffsets.ToArray());
            var intVecOffset = StatusLogCollectionT.CreateIntLogsVector(builder, intLogOffsets.ToArray());
            var stringVecOffset = StatusLogCollectionT.CreateStringLogsVector(builder, stringLogOffsets.ToArray());
            var offset = StatusLogCollectionT.CreateStatusLogCollectionT(builder, floatVecOffset, boolVecOffset, intVecOffset, stringVecOffset);

            builder.Finish(offset.Value);
            return builder.DataBuffer;
        }

        private void BuildScanTypeMap()
        {
            var accessor = GetHandle();
            Dictionary<short, uint> msLevelCounts = new() {
                {1, 0},
                {2, 0},
                {3, 0},
                {4, 0},
                {5, 0},
                {6, 0},
                {7, 0},
                {8, 0},
                {9, 0},
                {10, 0},
            };
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
            for (var i = FirstSpectrum(); i <= last; i++)
            {
                var filter = accessor.GetFilterForScanNumber(i);
                var msLevel = MSLevelFromFilter(filter);

                msLevelCounts[msLevel] += 1;

                List<int?> backwards = new();
                for (short j = 1; j < msLevel + 1; j++)
                {
                    var o = lastMSLevels[j];
                    backwards.Add(o);
                }
                previousMSLevels[i] = backwards;

                lastMSLevels[msLevel] = i;
            }

            PreviousMSLevels = previousMSLevels;
            MSLevelCounts = msLevelCounts;
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
            BuildScanTypeMap();

            var headers = accessor.GetTrailerExtraHeaderInformation();
            for (var i = 0; i < headers.Length; i++)
            {
                var header = headers[i];
                var label = header.Label.TrimEnd(':');
                TrailerMap[label] = i;
            }
            Headers = headers;
            return RawFileReaderError.Ok;
        }
    }

    public static class Exports
    {
        private static unsafe delegate*<nuint, RawVec*, void> RustAllocateMemory;

        private static Dictionary<IntPtr, RawFileReader> OpenHandles = new Dictionary<nint, RawFileReader>();
        private static IntPtr HandleCounter = 1;

        [UnmanagedCallersOnly(EntryPoint = "rawfilereader_set_memory_allocator")]
        public static unsafe void SetRustAllocateMemory(delegate*<nuint, RawVec*, void> rustAllocateMemory) => RustAllocateMemory = rustAllocateMemory;

        private unsafe static RawVec MemoryToRustVec(Span<byte> buffer, nuint size)
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

        [UnmanagedCallersOnly(EntryPoint = "rawfilereader_open")]
        public static unsafe IntPtr Open(IntPtr textPtr, int textLength)
        {
            var text = Marshal.PtrToStringUTF8(textPtr, textLength);
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
        [UnmanagedCallersOnly(EntryPoint = "rawfilereader_close")]
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

        [UnmanagedCallersOnly(EntryPoint = "rawfilereader_close_all")]
        public static unsafe void CloseAll()
        {
            OpenHandles.Clear();
        }

        [UnmanagedCallersOnly(EntryPoint = "rawfilereader_first_spectrum")]
        public static unsafe int FirstSpectrum(IntPtr handleToken)
        {
            RawFileReader reader = GetHandleForToken(handleToken);
            return reader.FirstSpectrum();
        }

        [UnmanagedCallersOnly(EntryPoint = "rawfilereader_last_spectrum")]
        public static unsafe int LastSpectrum(IntPtr handleToken)
        {
            RawFileReader reader = GetHandleForToken(handleToken);
            return reader.LastSpectrum();
        }

        [UnmanagedCallersOnly(EntryPoint = "rawfilereader_spectrum_count")]
        public static unsafe int SpectrumCount(IntPtr handleToken)
        {
            RawFileReader reader = GetHandleForToken(handleToken);
            return reader.SpectrumCount();
        }

        [UnmanagedCallersOnly(EntryPoint = "rawfilereader_status")]
        public static unsafe uint Status(IntPtr handleToken)
        {
            try
            {
                RawFileReader reader = GetHandleForToken(handleToken);
                return (uint)reader.Status;
            }
            catch
            {
                return (uint)RawFileReaderError.HandleNotFound;
            }
        }

        [UnmanagedCallersOnly(EntryPoint = "rawfilereader_get_signal_loading")]
        public static unsafe uint GetSignalLoading(IntPtr handleToken)
        {
            RawFileReader reader = GetHandleForToken(handleToken);
            return (uint)(reader.IncludeSignal ? 1 : 0);
        }

        [UnmanagedCallersOnly(EntryPoint = "rawfilereader_set_signal_loading")]
        public static unsafe void SetSignalLoading(IntPtr handleToken, uint value)
        {
            RawFileReader reader = GetHandleForToken(handleToken);
            if (value == 0)
            {
                reader.IncludeSignal = false;
            }
            else
            {
                reader.IncludeSignal = true;
            }
        }

        [UnmanagedCallersOnly(EntryPoint = "rawfilereader_get_centroiding")]
        public static unsafe uint GetCentroidSpectra(IntPtr handleToken)
        {
            RawFileReader reader = GetHandleForToken(handleToken);
            return (uint)(reader.CentroidSpectra ? 1 : 0);
        }

        [UnmanagedCallersOnly(EntryPoint = "rawfilereader_set_centroiding")]
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
        [UnmanagedCallersOnly(EntryPoint = "rawfilereader_spectrum_description_for")]
        public static unsafe RawVec SpectrumDescriptionFor(IntPtr handleToken, int scanNumber)
        {
            RawFileReader reader = GetHandleForToken(handleToken);
            var buffer = reader.SpectrumDescriptionFor(scanNumber);
            var bytes = buffer.ToSpan(buffer.Position, buffer.Length - buffer.Position);
            var size = bytes.Length;
            return MemoryToRustVec(bytes, (nuint)size);
        }

        /// <summary>
        /// Get a `SpectrumDescription` FlatBuffer message for a specific spectrum from a RAW file
        /// </summary>
        /// <param name="handleToken">The token corresponding to the `RawFileReader` handle</param>
        /// <param name="scanNumber">The scan number of the spectrum to retrieve</param>
        /// <param name="includeSignal">Whether or not to include the MS spectrum signal</param>
        /// <param name="centroidSpectra">Whether or not to retrieve the centroided spectrum signal</param>
        /// <returns>A `RawVec` representing Rust-allocated memory that holds the FlatBuffer message</returns>
        [UnmanagedCallersOnly(EntryPoint = "rawfilereader_spectrum_description_for_with_options")]
        public static unsafe RawVec SpectrumDescriptionForWithOptions(IntPtr handleToken, int scanNumber, int includeSignal, int centroidSpectra)
        {
            RawFileReader reader = GetHandleForToken(handleToken);
            var buffer = reader.SpectrumDescriptionFor(scanNumber, includeSignal != 0, centroidSpectra != 0);
            var bytes = buffer.ToSpan(buffer.Position, buffer.Length - buffer.Position);
            var size = bytes.Length;
            return MemoryToRustVec(bytes, (nuint)size);
        }

        /// <summary>
        /// Get a `SpectrumData` FlatBuffer message for a specific spectrum from a RAW file. May be empty if
        /// a profile spectrum is requested and profile data is not available.
        /// </summary>
        /// <param name="handleToken">The token corresponding to the `RawFileReader` handle</param>
        /// <param name="scanNumber">The scan number of the spectrum to retrieve</param>
        /// <param name="centroidSpectra">Whether or not to retrieve the centroided spectrum signal</param>
        /// <returns>A `RawVec` representing Rust-allocated memory that holds the FlatBuffer message</returns>
        [UnmanagedCallersOnly(EntryPoint = "rawfilereader_spectrum_data_for")]
        public static unsafe RawVec SpectrumDataFor(IntPtr handleToken, int scanNumber, int centroidSpectra)
        {
            RawFileReader reader = GetHandleForToken(handleToken);
            var buffer = reader.SpectrumDataFor(scanNumber, centroidSpectra != 0);
            var bytes = buffer.ToSpan(buffer.Position, buffer.Length - buffer.Position);
            var size = bytes.Length;
            return MemoryToRustVec(bytes, (nuint)size);
        }

        [UnmanagedCallersOnly(EntryPoint = "rawfilereader_advanced_packet_data_for")]
        public static unsafe RawVec AdvancedPacketDataFor(IntPtr handleToken, int scanNumber, int includeSampledNoise) {
            RawFileReader reader = GetHandleForToken(handleToken);
            var buffer = reader.GetAdvancedPacketData(scanNumber, includeSampledNoise != 0);
            var bytes = buffer.ToSpan(buffer.Position, buffer.Length - buffer.Position);
            var size = bytes.Length;
            return MemoryToRustVec(bytes, (nuint)size);
        }

        /// <summary>
        /// Get an `InstrumentModel` FlatBuffer message describing the instrument used to acquire a RAW file
        /// </summary>
        /// <param name="handleToken">The token corresponding to the `RawFileReader` handle</param>
        /// <returns>A `RawVec` representing Rust-allocated memory that holds the FlatBuffer message</returns>
        [UnmanagedCallersOnly(EntryPoint = "rawfilereader_instrument_model")]
        public static unsafe RawVec InstrumentModel(IntPtr handleToken)
        {
            RawFileReader reader = GetHandleForToken(handleToken);
            var buffer = reader.GetInstrumentInfo();
            var bytes = buffer.ToSpan(buffer.Position, buffer.Length - buffer.Position);
            var size = bytes.Length;
            return MemoryToRustVec(bytes, (nuint)size);
        }

        [UnmanagedCallersOnly(EntryPoint = "rawfilereader_file_description")]
        public static unsafe RawVec FileDescription(IntPtr handleToken) {
            RawFileReader reader = GetHandleForToken(handleToken);
            var buffer = reader.GetFileMetadata();
            var bytes = buffer.ToSpan(buffer.Position, buffer.Length - buffer.Position);
            var size = bytes.Length;
            return MemoryToRustVec(bytes, (nuint)size);
        }

        [UnmanagedCallersOnly(EntryPoint = "rawfilereader_instrument_method")]
        public static unsafe RawVec InstrumentMethod(IntPtr handleToken, int method) {
            RawFileReader reader = GetHandleForToken(handleToken);
            var buffer = reader.GetInstrumentMethodFor(method);
            var bytes = buffer.ToSpan(buffer.Position, buffer.Length - buffer.Position);
            var size = bytes.Length;
            return MemoryToRustVec(bytes, (nuint)size);
        }

        [UnmanagedCallersOnly(EntryPoint = "rawfilereader_instrument_method_count")]
        public static unsafe uint InstrumentMethodCount(IntPtr handleToken) {
            RawFileReader reader = GetHandleForToken(handleToken);
            return reader.GetInstrumentMethodCount();
        }

        [UnmanagedCallersOnly(EntryPoint = "rawfilereader_get_tic")]
        public static unsafe RawVec GetTIC(IntPtr handleToken) {
            RawFileReader reader = GetHandleForToken(handleToken);
            var buffer = reader.GetSummaryTrace(TraceType.TIC);
            var bytes = buffer.ToSpan(buffer.Position, buffer.Length - buffer.Position);
            var size = bytes.Length;
            return MemoryToRustVec(bytes, (nuint)size);
        }

        [UnmanagedCallersOnly(EntryPoint = "rawfilereader_get_bpc")]
        public static unsafe RawVec GetBPC(IntPtr handleToken)
        {
            RawFileReader reader = GetHandleForToken(handleToken);
            var buffer = reader.GetSummaryTrace(TraceType.BasePeak);
            var bytes = buffer.ToSpan(buffer.Position, buffer.Length - buffer.Position);
            var size = bytes.Length;
            return MemoryToRustVec(bytes, (nuint)size);
        }

        [UnmanagedCallersOnly(EntryPoint = "rawfilereader_get_raw_trailer_values_for")]
        public static unsafe RawVec GetRawTrailerValuesFor(IntPtr handleToken, int scanNumber) {
            RawFileReader reader = GetHandleForToken(handleToken);
            var buffer = reader.GetRawTrailersForScan(scanNumber);
            var bytes = buffer.ToSpan(buffer.Position, buffer.Length - buffer.Position);
            var size = bytes.Length;
            return MemoryToRustVec(bytes, (nuint)size);
        }

        [UnmanagedCallersOnly(EntryPoint = "rawfilereader_get_file_error_message")]
        public static unsafe RawVec GetErrorMessageFor(IntPtr handleToken) {
            RawFileReader reader = GetHandleForToken(handleToken);
            var message = reader.FileErrorMessage();
            // Bad things happen if the string is length zero.
            // If the string is empty, instead operate on a string
            // of just the nul byte.
            message = message.Length == 0 ? "\0" : message;
            var bytes = Encoding.UTF8.GetBytes(message);
            var bytesSpan = bytes.AsSpan();
            var size = bytes.Length;
            return MemoryToRustVec(bytesSpan, (nuint)size);
        }

        [UnmanagedCallersOnly(EntryPoint = "rawfilereader_get_status_logs")]
        public static unsafe RawVec GetStatusLogs(IntPtr handleToken) {
            RawFileReader reader = GetHandleForToken(handleToken);
            var buffer = reader.StatusLogs();
            var bytes = buffer.ToSpan(buffer.Position, buffer.Length - buffer.Position);
            var size = bytes.Length;
            return MemoryToRustVec(bytes, (nuint)size);
        }
    }
}
