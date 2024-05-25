using System;
using System.Collections.Generic;
using ThermoFisher.CommonCore.Data.FilterEnums;

namespace librawfilereader
{
    public enum InstrumentModelType
    {
        Unknown = -1,

        // Finnigan MAT
        MAT253,
        MAT900XP,
        MAT900XP_Trap,
        MAT95XP,
        MAT95XP_Trap,
        SSQ_7000,
        TSQ_7000,
        TSQ,

        // Thermo Electron
        Element_2,

        // Thermo Finnigan
        Delta_Plus_Advantage,
        Delta_Plus_XP,
        LCQ_Advantage,
        LCQ_Classic,
        LCQ_Deca,
        LCQ_Deca_XP_Plus,
        Neptune,
        DSQ,
        PolarisQ,
        Surveyor_MSQ,
        Tempus_TOF,
        Trace_DSQ,
        Triton,

        // Thermo Scientific
        LTQ,
        LTQ_Velos,
        LTQ_Velos_ETD,
        LTQ_Velos_Plus,
        LTQ_FT,
        LTQ_FT_Ultra,
        LTQ_Orbitrap,
        LTQ_Orbitrap_Classic,
        LTQ_Orbitrap_Discovery,
        LTQ_Orbitrap_XL,
        LTQ_Orbitrap_Velos,
        LTQ_Orbitrap_Velos_Pro,
        LTQ_Orbitrap_Elite,
        LXQ,
        LCQ_Fleet,
        ITQ_700,
        ITQ_900,
        ITQ_1100,
        GC_Quantum,
        LTQ_XL,
        LTQ_XL_ETD,
        LTQ_Orbitrap_XL_ETD,
        DFS,
        DSQ_II,
        ISQ,
        MALDI_LTQ_XL,
        MALDI_LTQ_Orbitrap,
        TSQ_Quantum,
        TSQ_Quantum_Access,
        TSQ_Quantum_Ultra,
        TSQ_Quantum_Ultra_AM,
        TSQ_Vantage,
        Element_XR,
        Element_GD,
        GC_IsoLink,
        Exactive,
        Exactive_Plus,
        Q_Exactive,
        Q_Exactive_Plus,
        Q_Exactive_HF,
        Q_Exactive_HF_X,
        Q_Exactive_UHMR,
        Surveyor_PDA,
        Accela_PDA,
        Orbitrap_Fusion,
        Orbitrap_Fusion_Lumos,
        Orbitrap_Fusion_ETD,
        Orbitrap_Ascend,
        Orbitrap_ID_X,
        TSQ_Quantiva,
        TSQ_Endura,
        TSQ_Altis,
        TSQ_Altis_Plus,
        TSQ_Quantis,
        TSQ_8000_Evo,
        TSQ_9000,
        Orbitrap_Exploris_120,
        Orbitrap_Exploris_240,
        Orbitrap_Exploris_480,
        Orbitrap_Eclipse,
        Orbitrap_GC,
        Orbitrap_Astral,
    }

    public enum MatchType
    {
        Exact,
        Contains,
        StartsWith,
        EndsWith,
        ExactNoSpaces
    };

    public struct InstrumentNameToModelMapping
    {

        public string Name;
        public InstrumentModelType ModelType;
        public MatchType MatchType;

        public InstrumentNameToModelMapping(string name, InstrumentModelType modelType, MatchType matchType)
        {
            Name = name;
            ModelType = modelType;
            MatchType = matchType;
        }
    }

    static class InstrumentModelHelpers
    {
        public static InstrumentModelType parseInstrumentModel(string instrumentModel) {
            var typeUpper = instrumentModel.ToUpper();
            var noSpaces = typeUpper.Replace(" ", "");
            foreach(var mapping in NameToModelMapping) {
                switch(mapping.MatchType) {
                    case MatchType.Exact: {
                        if(mapping.Name == typeUpper) {
                            return mapping.ModelType;
                        }
                        break;
                    }
                    case MatchType.ExactNoSpaces:
                    {
                        if (mapping.Name == typeUpper) {
                            return mapping.ModelType;
                        }
                        break;
                    }
                    case MatchType.Contains:
                    {
                        if (typeUpper.Contains(mapping.Name)) {
                            return mapping.ModelType;
                        }
                        break;
                    }
                    case MatchType.StartsWith: {
                        if (typeUpper.StartsWith(mapping.Name)) {
                            return mapping.ModelType;
                        }
                        break;
                    }
                    case MatchType.EndsWith: {
                        if (typeUpper.EndsWith(mapping.Name))
                        {
                            return mapping.ModelType;
                        }
                        break;
                    }
                    default: {
                        throw new System.Exception("Unknown match type");
                    }
                }
            }
            return InstrumentModelType.Unknown;
        }

        static InstrumentNameToModelMapping[] NameToModelMapping = new InstrumentNameToModelMapping[] {
    new InstrumentNameToModelMapping("MAT253", InstrumentModelType.MAT253, MatchType.ExactNoSpaces),
    new InstrumentNameToModelMapping("MAT900XP", InstrumentModelType.MAT900XP, MatchType.ExactNoSpaces),
    new InstrumentNameToModelMapping("MAT900XPTRAP", InstrumentModelType.MAT900XP_Trap, MatchType.ExactNoSpaces),
    new InstrumentNameToModelMapping("MAT95XP", InstrumentModelType.MAT95XP, MatchType.ExactNoSpaces),
    new InstrumentNameToModelMapping("MAT95XPTRAP", InstrumentModelType.MAT95XP_Trap, MatchType.ExactNoSpaces),
    new InstrumentNameToModelMapping("SSQ7000", InstrumentModelType.SSQ_7000, MatchType.ExactNoSpaces),
    new InstrumentNameToModelMapping("TSQ7000", InstrumentModelType.TSQ_7000, MatchType.ExactNoSpaces),
    new InstrumentNameToModelMapping("TSQ8000EVO", InstrumentModelType.TSQ_8000_Evo, MatchType.ExactNoSpaces),
    new InstrumentNameToModelMapping("TSQ9000", InstrumentModelType.TSQ_9000, MatchType.ExactNoSpaces),
    new InstrumentNameToModelMapping("TSQ", InstrumentModelType.TSQ, MatchType.Exact),
    new InstrumentNameToModelMapping("ELEMENT2", InstrumentModelType.Element_2, MatchType.ExactNoSpaces),
    new InstrumentNameToModelMapping("DELTA PLUSADVANTAGE", InstrumentModelType.Delta_Plus_Advantage, MatchType.Exact),
    new InstrumentNameToModelMapping("DELTAPLUSXP", InstrumentModelType.Delta_Plus_XP, MatchType.Exact),
    new InstrumentNameToModelMapping("LCQ ADVANTAGE", InstrumentModelType.LCQ_Advantage, MatchType.Exact),
    new InstrumentNameToModelMapping("LCQ CLASSIC", InstrumentModelType.LCQ_Classic, MatchType.Exact),
    new InstrumentNameToModelMapping("LCQ DECA", InstrumentModelType.LCQ_Deca, MatchType.Exact),
    new InstrumentNameToModelMapping("LCQ DECA XP", InstrumentModelType.LCQ_Deca_XP_Plus, MatchType.Exact),
    new InstrumentNameToModelMapping("LCQ DECA XP PLUS", InstrumentModelType.LCQ_Deca_XP_Plus, MatchType.Exact),
    new InstrumentNameToModelMapping("NEPTUNE", InstrumentModelType.Neptune, MatchType.Exact),
    new InstrumentNameToModelMapping("DSQ", InstrumentModelType.DSQ, MatchType.Exact),
    new InstrumentNameToModelMapping("POLARISQ", InstrumentModelType.PolarisQ, MatchType.Exact),
    new InstrumentNameToModelMapping("SURVEYOR MSQ", InstrumentModelType.Surveyor_MSQ, MatchType.Exact),
    new InstrumentNameToModelMapping("MSQ PLUS", InstrumentModelType.Surveyor_MSQ, MatchType.Exact),
    new InstrumentNameToModelMapping("TEMPUS TOF", InstrumentModelType.Tempus_TOF, MatchType.Exact),
    new InstrumentNameToModelMapping("TRACE DSQ", InstrumentModelType.Trace_DSQ, MatchType.Exact),
    new InstrumentNameToModelMapping("TRITON", InstrumentModelType.Triton, MatchType.Exact),
    new InstrumentNameToModelMapping("LTQ", InstrumentModelType.LTQ, MatchType.Exact),
    new InstrumentNameToModelMapping("LTQ XL", InstrumentModelType.LTQ_XL, MatchType.Exact),
    new InstrumentNameToModelMapping("LTQ FT", InstrumentModelType.LTQ_FT, MatchType.Exact),
    new InstrumentNameToModelMapping("LTQ-FT", InstrumentModelType.LTQ_FT, MatchType.Exact),
    new InstrumentNameToModelMapping("LTQ FT ULTRA", InstrumentModelType.LTQ_FT_Ultra, MatchType.Exact),
    new InstrumentNameToModelMapping("LTQ ORBITRAP", InstrumentModelType.LTQ_Orbitrap, MatchType.Exact),
    new InstrumentNameToModelMapping("LTQ ORBITRAP CLASSIC", InstrumentModelType.LTQ_Orbitrap_Classic, MatchType.Exact), // predict)d
    new InstrumentNameToModelMapping("LTQ ORBITRAP DISCOVERY", InstrumentModelType.LTQ_Orbitrap_Discovery, MatchType.Exact),
    new InstrumentNameToModelMapping("LTQ ORBITRAP XL", InstrumentModelType.LTQ_Orbitrap_XL, MatchType.Exact),
    new InstrumentNameToModelMapping("ORBITRAP VELOS PRO", InstrumentModelType.LTQ_Orbitrap_Velos_Pro, MatchType.Contains),
    new InstrumentNameToModelMapping("ORBITRAP VELOS", InstrumentModelType.LTQ_Orbitrap_Velos, MatchType.Contains),
    new InstrumentNameToModelMapping("ORBITRAP ELITE", InstrumentModelType.LTQ_Orbitrap_Elite, MatchType.Contains),
    new InstrumentNameToModelMapping("VELOS PLUS", InstrumentModelType.LTQ_Velos_Plus, MatchType.Contains),
    new InstrumentNameToModelMapping("VELOS PRO", InstrumentModelType.LTQ_Velos_Plus, MatchType.Contains),
    new InstrumentNameToModelMapping("LTQ VELOS", InstrumentModelType.LTQ_Velos, MatchType.Exact),
    new InstrumentNameToModelMapping("LTQ VELOS ETD", InstrumentModelType.LTQ_Velos_ETD, MatchType.Exact),
    new InstrumentNameToModelMapping("LXQ", InstrumentModelType.LXQ, MatchType.Exact),
    new InstrumentNameToModelMapping("LCQ FLEET", InstrumentModelType.LCQ_Fleet, MatchType.Exact),
    new InstrumentNameToModelMapping("ITQ 700", InstrumentModelType.ITQ_700, MatchType.Exact),
    new InstrumentNameToModelMapping("ITQ 900", InstrumentModelType.ITQ_900, MatchType.Exact),
    new InstrumentNameToModelMapping("ITQ 1100", InstrumentModelType.ITQ_1100, MatchType.Exact),
    new InstrumentNameToModelMapping("GC QUANTUM", InstrumentModelType.GC_Quantum, MatchType.Exact),
    new InstrumentNameToModelMapping("LTQ XL ETD", InstrumentModelType.LTQ_XL_ETD, MatchType.Exact),
    new InstrumentNameToModelMapping("LTQ ORBITRAP XL ETD", InstrumentModelType.LTQ_Orbitrap_XL_ETD, MatchType.Exact),
    new InstrumentNameToModelMapping("DFS", InstrumentModelType.DFS, MatchType.Exact),
    new InstrumentNameToModelMapping("DSQ II", InstrumentModelType.DSQ_II, MatchType.Exact),
    new InstrumentNameToModelMapping("ISQ SERIES", InstrumentModelType.ISQ, MatchType.Exact),
    new InstrumentNameToModelMapping("MALDI LTQ XL", InstrumentModelType.MALDI_LTQ_XL, MatchType.Exact),
    new InstrumentNameToModelMapping("MALDI LTQ ORBITRAP", InstrumentModelType.MALDI_LTQ_Orbitrap, MatchType.Exact),
    new InstrumentNameToModelMapping("TSQ QUANTUM", InstrumentModelType.TSQ_Quantum, MatchType.Exact),
    new InstrumentNameToModelMapping("TSQ QUANTUM ACCESS", InstrumentModelType.TSQ_Quantum_Access, MatchType.Contains),
    new InstrumentNameToModelMapping("TSQ QUANTUM ULTRA", InstrumentModelType.TSQ_Quantum_Ultra, MatchType.Exact),
    new InstrumentNameToModelMapping("TSQ QUANTUM ULTRA AM", InstrumentModelType.TSQ_Quantum_Ultra_AM, MatchType.Exact),
    new InstrumentNameToModelMapping("TSQ QUANTIVA", InstrumentModelType.TSQ_Quantiva, MatchType.Exact),
    new InstrumentNameToModelMapping("TSQ ENDURA", InstrumentModelType.TSQ_Endura, MatchType.Exact),
    new InstrumentNameToModelMapping("TSQ ALTIS", InstrumentModelType.TSQ_Altis, MatchType.Exact),
    new InstrumentNameToModelMapping("TSQ ALTIS PLUS", InstrumentModelType.TSQ_Altis_Plus, MatchType.Exact),
    new InstrumentNameToModelMapping("TSQ QUANTIS", InstrumentModelType.TSQ_Quantis, MatchType.Exact),
    new InstrumentNameToModelMapping("ELEMENT XR", InstrumentModelType.Element_XR, MatchType.Exact),
    new InstrumentNameToModelMapping("ELEMENT GD", InstrumentModelType.Element_GD, MatchType.Exact),
    new InstrumentNameToModelMapping("GC ISOLINK", InstrumentModelType.GC_IsoLink, MatchType.Exact),
    new InstrumentNameToModelMapping("ORBITRAP ID-X", InstrumentModelType.Orbitrap_ID_X, MatchType.Exact),
    new InstrumentNameToModelMapping("Q EXACTIVE PLUS", InstrumentModelType.Q_Exactive_Plus, MatchType.Contains),
    new InstrumentNameToModelMapping("Q EXACTIVE HF-X", InstrumentModelType.Q_Exactive_HF_X, MatchType.Contains),
    new InstrumentNameToModelMapping("Q EXACTIVE HF", InstrumentModelType.Q_Exactive_HF, MatchType.Contains),
    new InstrumentNameToModelMapping("Q EXACTIVE UHMR", InstrumentModelType.Q_Exactive_UHMR, MatchType.Contains),
    new InstrumentNameToModelMapping("Q EXACTIVE", InstrumentModelType.Q_Exactive, MatchType.Contains),
    new InstrumentNameToModelMapping("EXACTIVE PLUS", InstrumentModelType.Exactive_Plus, MatchType.Contains),
    new InstrumentNameToModelMapping("EXACTIVE", InstrumentModelType.Exactive, MatchType.Contains),
    new InstrumentNameToModelMapping("ORBITRAP EXPLORIS 120", InstrumentModelType.Orbitrap_Exploris_120, MatchType.Exact),
    new InstrumentNameToModelMapping("ORBITRAP EXPLORIS 240", InstrumentModelType.Orbitrap_Exploris_240, MatchType.Exact),
    new InstrumentNameToModelMapping("ORBITRAP EXPLORIS 480", InstrumentModelType.Orbitrap_Exploris_480, MatchType.Exact),
    new InstrumentNameToModelMapping("ORBITRAP GC", InstrumentModelType.Orbitrap_GC, MatchType.Contains),
    new InstrumentNameToModelMapping("ECLIPSE", InstrumentModelType.Orbitrap_Eclipse, MatchType.Contains),
    new InstrumentNameToModelMapping("ASTRAL", InstrumentModelType.Orbitrap_Astral, MatchType.Contains),
    new InstrumentNameToModelMapping("FUSION ETD", InstrumentModelType.Orbitrap_Fusion_ETD, MatchType.Contains),
    new InstrumentNameToModelMapping("FUSION LUMOS", InstrumentModelType.Orbitrap_Fusion_Lumos, MatchType.Contains),
    new InstrumentNameToModelMapping("FUSION", InstrumentModelType.Orbitrap_Fusion, MatchType.Contains),
    new InstrumentNameToModelMapping("ASCEND", InstrumentModelType.Orbitrap_Ascend, MatchType.Contains),
    new InstrumentNameToModelMapping("SURVEYOR PDA", InstrumentModelType.Surveyor_PDA, MatchType.Exact),
    new InstrumentNameToModelMapping("ACCELA PDA", InstrumentModelType.Accela_PDA, MatchType.Exact),
    };

        public static List<IonizationModeType> GetIonSourcesFor(InstrumentModelType type) {
            var ionSources = new List<IonizationModeType>();
            switch (type) {
                case InstrumentModelType.SSQ_7000:
                case InstrumentModelType.TSQ_7000:
                case InstrumentModelType.TSQ_8000_Evo:
                case InstrumentModelType.TSQ_9000:
                case InstrumentModelType.Surveyor_MSQ:
                case InstrumentModelType.LCQ_Advantage:
                case InstrumentModelType.LCQ_Classic:
                case InstrumentModelType.LCQ_Deca:
                case InstrumentModelType.LCQ_Deca_XP_Plus:
                case InstrumentModelType.LCQ_Fleet:
                case InstrumentModelType.LXQ:
                case InstrumentModelType.LTQ:
                case InstrumentModelType.LTQ_XL:
                case InstrumentModelType.LTQ_XL_ETD:
                case InstrumentModelType.LTQ_Velos:
                case InstrumentModelType.LTQ_Velos_ETD:
                case InstrumentModelType.LTQ_Velos_Plus:
                case InstrumentModelType.LTQ_FT:
                case InstrumentModelType.LTQ_FT_Ultra:
                case InstrumentModelType.LTQ_Orbitrap:
                case InstrumentModelType.LTQ_Orbitrap_Classic:
                case InstrumentModelType.LTQ_Orbitrap_Discovery:
                case InstrumentModelType.LTQ_Orbitrap_XL:
                case InstrumentModelType.LTQ_Orbitrap_XL_ETD:
                case InstrumentModelType.LTQ_Orbitrap_Velos:
                case InstrumentModelType.LTQ_Orbitrap_Velos_Pro:
                case InstrumentModelType.LTQ_Orbitrap_Elite:
                case InstrumentModelType.Exactive:
                case InstrumentModelType.Exactive_Plus:
                case InstrumentModelType.Q_Exactive:
                case InstrumentModelType.Q_Exactive_Plus:
                case InstrumentModelType.Q_Exactive_HF:
                case InstrumentModelType.Q_Exactive_HF_X:
                case InstrumentModelType.Q_Exactive_UHMR:
                case InstrumentModelType.Orbitrap_Exploris_120:
                case InstrumentModelType.Orbitrap_Exploris_240:
                case InstrumentModelType.Orbitrap_Exploris_480:
                case InstrumentModelType.Orbitrap_Eclipse:
                case InstrumentModelType.Orbitrap_Fusion:
                case InstrumentModelType.Orbitrap_Fusion_Lumos:
                case InstrumentModelType.Orbitrap_Fusion_ETD:
                case InstrumentModelType.Orbitrap_Ascend:
                case InstrumentModelType.Orbitrap_ID_X:
                case InstrumentModelType.Orbitrap_Astral:
                case InstrumentModelType.TSQ:
                case InstrumentModelType.TSQ_Quantum:
                case InstrumentModelType.TSQ_Quantum_Access:
                case InstrumentModelType.TSQ_Quantum_Ultra:
                case InstrumentModelType.TSQ_Quantum_Ultra_AM:
                case InstrumentModelType.TSQ_Quantiva:
                case InstrumentModelType.TSQ_Endura:
                case InstrumentModelType.TSQ_Altis:
                case InstrumentModelType.TSQ_Altis_Plus:
                case InstrumentModelType.TSQ_Quantis:
                    ionSources.Add(IonizationModeType.ElectroSpray);
                    break;
                case InstrumentModelType.DSQ:
                case InstrumentModelType.PolarisQ:
                case InstrumentModelType.ITQ_700:
                case InstrumentModelType.ITQ_900:
                case InstrumentModelType.ITQ_1100:
                case InstrumentModelType.Trace_DSQ:
                case InstrumentModelType.GC_Quantum:
                case InstrumentModelType.DFS:
                case InstrumentModelType.DSQ_II:
                case InstrumentModelType.ISQ:
                case InstrumentModelType.GC_IsoLink:
                case InstrumentModelType.Orbitrap_GC:
                    ionSources.Add(IonizationModeType.ElectronImpact);
                    break;
                case InstrumentModelType.MALDI_LTQ_XL:
                case InstrumentModelType.MALDI_LTQ_Orbitrap:
                    ionSources.Add(IonizationModeType.MatrixAssistedLaserDesorptionIonization);
                    break;

                case InstrumentModelType.Element_GD:
                    ionSources.Add(IonizationModeType.GlowDischarge);
                    break;

                case InstrumentModelType.Element_XR:
                case InstrumentModelType.Element_2:
                case InstrumentModelType.Delta_Plus_Advantage:
                case InstrumentModelType.Delta_Plus_XP:
                case InstrumentModelType.Neptune:
                case InstrumentModelType.Tempus_TOF:
                case InstrumentModelType.Triton:
                case InstrumentModelType.MAT253:
                case InstrumentModelType.MAT900XP:
                case InstrumentModelType.MAT900XP_Trap:
                case InstrumentModelType.MAT95XP:
                case InstrumentModelType.MAT95XP_Trap:
                    // TODO: get source information for these instruments
                    break;

                case InstrumentModelType.Surveyor_PDA:
                case InstrumentModelType.Accela_PDA:
                case InstrumentModelType.Unknown:
                default:
                    break;
            }
            return ionSources;
        }

        public static List<MassAnalyzerType> GetMassAnalyzerFor(InstrumentModelType type) {
            var analyzers = new List<MassAnalyzerType>();
            switch (type) {
                case InstrumentModelType.Exactive:
                case InstrumentModelType.Exactive_Plus:
                case InstrumentModelType.Q_Exactive:
                case InstrumentModelType.Q_Exactive_Plus:
                case InstrumentModelType.Q_Exactive_HF_X:
                case InstrumentModelType.Q_Exactive_HF:
                case InstrumentModelType.Q_Exactive_UHMR:
                case InstrumentModelType.Orbitrap_Exploris_120:
                case InstrumentModelType.Orbitrap_Exploris_240:
                case InstrumentModelType.Orbitrap_Exploris_480:
                    analyzers.Add(MassAnalyzerType.MassAnalyzerFTMS);
                    return analyzers;

                case InstrumentModelType.LTQ_Orbitrap:
                case InstrumentModelType.LTQ_Orbitrap_Classic:
                case InstrumentModelType.LTQ_Orbitrap_Discovery:
                case InstrumentModelType.LTQ_Orbitrap_XL:
                case InstrumentModelType.LTQ_Orbitrap_XL_ETD:
                case InstrumentModelType.MALDI_LTQ_Orbitrap:
                case InstrumentModelType.LTQ_Orbitrap_Velos:
                case InstrumentModelType.LTQ_Orbitrap_Velos_Pro:
                case InstrumentModelType.LTQ_Orbitrap_Elite:
                case InstrumentModelType.Orbitrap_Fusion:
                case InstrumentModelType.Orbitrap_Fusion_Lumos:
                case InstrumentModelType.Orbitrap_Fusion_ETD:
                case InstrumentModelType.Orbitrap_Ascend:
                case InstrumentModelType.Orbitrap_ID_X:
                case InstrumentModelType.Orbitrap_Eclipse:
                case InstrumentModelType.Orbitrap_GC:
                    {
                        analyzers.Add(MassAnalyzerType.MassAnalyzerFTMS);
                        analyzers.Add(MassAnalyzerType.MassAnalyzerITMS);
                        return analyzers;
                    }

                case InstrumentModelType.Orbitrap_Astral:
                    {
                        analyzers.Add(MassAnalyzerType.MassAnalyzerFTMS);
                        analyzers.Add(MassAnalyzerType.MassAnalyzerASTMS);
                        return analyzers;
                    }

                case InstrumentModelType.LTQ_FT:
                case InstrumentModelType.LTQ_FT_Ultra:
                    {
                        analyzers.Add(MassAnalyzerType.MassAnalyzerFTMS);
                        analyzers.Add(MassAnalyzerType.MassAnalyzerITMS);
                        return analyzers;
                    }

                case InstrumentModelType.SSQ_7000:
                case InstrumentModelType.Surveyor_MSQ:
                case InstrumentModelType.DSQ:
                case InstrumentModelType.DSQ_II:
                case InstrumentModelType.ISQ:
                case InstrumentModelType.Trace_DSQ:
                case InstrumentModelType.GC_IsoLink:
                    analyzers.Add(MassAnalyzerType.MassAnalyzerTQMS);
                    return analyzers;

                // case InstrumentModelType.TSQ_7000:
                // case InstrumentModelType.TSQ_8000_Evo:
                // case InstrumentModelType.TSQ_9000:
                // case InstrumentModelType.TSQ:
                // case InstrumentModelType.TSQ_Quantum:
                // case InstrumentModelType.TSQ_Quantum_Access:
                // case InstrumentModelType.TSQ_Quantum_Ultra:
                // case InstrumentModelType.TSQ_Quantum_Ultra_AM:
                // case InstrumentModelType.GC_Quantum:
                // case InstrumentModelType.TSQ_Quantiva:
                // case InstrumentModelType.TSQ_Endura:
                // case InstrumentModelType.TSQ_Altis:
                // case InstrumentModelType.TSQ_Altis_Plus:
                // case InstrumentModelType.TSQ_Quantis:
                //     return MassAnalyzerType_Triple_Quadrupole;

                case InstrumentModelType.LCQ_Advantage:
                case InstrumentModelType.LCQ_Classic:
                case InstrumentModelType.LCQ_Deca:
                case InstrumentModelType.LCQ_Deca_XP_Plus:
                case InstrumentModelType.LCQ_Fleet:
                case InstrumentModelType.PolarisQ:
                case InstrumentModelType.ITQ_700:
                case InstrumentModelType.ITQ_900:
                    analyzers.Add(MassAnalyzerType.MassAnalyzerITMS);
                    return analyzers;

                case InstrumentModelType.LTQ:
                case InstrumentModelType.LXQ:
                case InstrumentModelType.LTQ_XL:
                case InstrumentModelType.LTQ_XL_ETD:
                case InstrumentModelType.ITQ_1100:
                case InstrumentModelType.MALDI_LTQ_XL:
                case InstrumentModelType.LTQ_Velos:
                case InstrumentModelType.LTQ_Velos_ETD:
                case InstrumentModelType.LTQ_Velos_Plus:
                    analyzers.Add(MassAnalyzerType.MassAnalyzerITMS);
                    return analyzers;

                // case InstrumentModelType.DFS:
                // case InstrumentModelType.MAT253:
                // case InstrumentModelType.MAT900XP:
                // case InstrumentModelType.MAT900XP_Trap:
                // case InstrumentModelType.MAT95XP:
                // case InstrumentModelType.MAT95XP_Trap:
                //     options.Add((MassAnalyzerType.MassAnalyzerITMS, null));
                //     return options;

                // case InstrumentModelType.Tempus_TOF:
                //     return MassAnalyzerType_TOF;

                // case InstrumentModelType.Element_XR:
                // case InstrumentModelType.Element_2:
                // case InstrumentModelType.Element_GD:
                // case InstrumentModelType.Delta_Plus_Advantage:
                // case InstrumentModelType.Delta_Plus_XP:
                // case InstrumentModelType.Neptune:
                // case InstrumentModelType.Triton:
                //     // TODO: get mass analyzer information for these instruments
                //     return MassAnalyzerType_Unknown;

                case InstrumentModelType.Surveyor_PDA:
                case InstrumentModelType.Accela_PDA:
                case InstrumentModelType.Unknown:
                default:
                    break;
            }
            return analyzers;
        }

    }

}