using System.Data;
using Serilog;
using UAssetEditor.Unreal.Names;
using UAssetEditor.Binary;
using UAssetEditor.Classes;
using UAssetEditor.Unreal.Assets;
using UAssetEditor.Unreal.Exports;
using UAssetEditor.Unreal.Properties.Structs.Misc;
using UAssetEditor.Utils;


namespace UAssetEditor.Unreal.Properties.Types;

// I did NOT feel like grabbing this from unrealengine repo - owen
// https://github.com/FabianFG/CUE4Parse/blob/0cd21c2d96068d29b812c9d0538de5654d8fbab5/CUE4Parse/UE4/Objects/Core/i18N/FText.cs

public enum EFormatArgumentType : sbyte
{
    Int,
    UInt,
    Float,
    Double,
    Text,
    Gender,

    // Add new enum types at the end only! They are serialized by index.
}

public class FFormatArgumentValue : UStruct
{
    public EFormatArgumentType Type;
    public object Value;

    public FFormatArgumentValue()
    { }

    public override void Read(Reader reader, PropertyData? data, Asset? asset = null, ESerializationMode mode = ESerializationMode.Normal)
    {
        Type = reader.Read<EFormatArgumentType>();
        switch (Type)
        {
            case EFormatArgumentType.Text:
                var text = new FText();
                text.Read(reader, data);
                Value = text;
                break;
            case EFormatArgumentType.Int:
                // TODO maybe versioning
                Value = reader.Read<long>();
                break;
            case EFormatArgumentType.UInt:
                Value = reader.Read<ulong>();
                break;
            case EFormatArgumentType.Double:
                Value = reader.Read<double>();
                break;
            case EFormatArgumentType.Float:
                Value = reader.Read<float>();
                break;
            default:
                throw new KeyNotFoundException();
        }
    }

    public override void Write(Writer writer, Asset? asset = null)
    {
        writer.Write(Type);

        if (Type == EFormatArgumentType.Text)
        {
            Value.As<FText>().Write(writer, asset);
            return;
        }
        
        writer.Write(Value);
    }
}

public enum ETextHistoryType : sbyte
{
    None = -1,
    Base = 0,
    NamedFormat,
    OrderedFormat,
    ArgumentFormat,
    AsNumber,
    AsPercent,
    AsCurrency,
    AsDate,
    AsTime,
    AsDateTime,
    Transform,
    StringTableEntry,
    TextGenerator,

    // Add new enum types at the end only! They are serialized by index.
}

// Basically this entire class was taken from CUE4Parse, I cba to rewrite all of this - owen
// https://github.com/FabianFG/CUE4Parse/blob/eb88d2ec617337324755745954eb29b09077868f/CUE4Parse/UE4/Objects/Core/i18N/FText.cs
public class FText : UStruct, IUnrealType
{
    [UField]
    public uint Flags;
    
    [UField]
    public ETextHistoryType Type;
    
    [UField]
    public FTextHistory History;

    public FText()
    { }

    [UValueGetter]
    public override string ToString()
    {
        return History.ToString();
    }

    public string Text
    {
        get => ToString();
        set => History.Text = value;
    }

    public override void Read(Reader reader, PropertyData? data, Asset? asset = null, ESerializationMode mode = ESerializationMode.Normal)
    {
        Flags = reader.Read<uint>();
        Type = reader.Read<ETextHistoryType>();
        History = Type switch
        {
            ETextHistoryType.Base => new FTextHistory.Base(reader),
            ETextHistoryType.NamedFormat => new FTextHistory.NamedFormat(reader),
            ETextHistoryType.OrderedFormat => new FTextHistory.OrderedFormat(reader),
            ETextHistoryType.ArgumentFormat => new FTextHistory.ArgumentFormat(reader),
            ETextHistoryType.AsNumber => new FTextHistory.FormatNumber(reader, Type),
            ETextHistoryType.AsPercent => new FTextHistory.FormatNumber(reader, Type),
            ETextHistoryType.AsCurrency => new FTextHistory.FormatNumber(reader, Type),
            ETextHistoryType.AsDate => new FTextHistory.AsDate(reader),
            ETextHistoryType.AsTime => new FTextHistory.AsTime(reader),
            ETextHistoryType.AsDateTime => new FTextHistory.AsDateTime(reader),
            ETextHistoryType.Transform => new FTextHistory.Transform(reader),
            ETextHistoryType.StringTableEntry => new FTextHistory.StringTableEntry(reader, asset?.NameMap 
                ?? throw new ArgumentNullException(nameof(asset), "Cannot be null because we need the NameMap to deserialize.")),
            ETextHistoryType.TextGenerator => new FTextHistory.TextGenerator(reader, asset?.NameMap 
                ?? throw new ArgumentNullException(nameof(asset), "Cannot be null because we need the NameMap to deserialize.")),
            _ => new FTextHistory.None(reader)
        };
    }

    public override void Write(Writer writer, Asset? asset = null)
    {
        writer.Write(Flags);
        writer.Write(Type);
        History.Serialize(writer);
    }

    public abstract class FTextHistory : IUnrealType
    {
        public abstract string Text { get; set; }

        public override string ToString()
        {
            return Text;
        }

        public abstract void Serialize(Writer writer);
        
        public class None : FTextHistory
        {
            public string? CultureInvariantString;

            public override string Text
            {
                get => CultureInvariantString ?? "";
                set => CultureInvariantString = value;
            }

            public None(Reader reader)
            {
                var bHasCultureInvariantString = reader.ReadBool();
                if (bHasCultureInvariantString)
                    CultureInvariantString = FString.Read(reader);
            }

            public override void Serialize(Writer writer)
            {
                if (CultureInvariantString != null)
                {
                    writer.Write(1); // bHasCultureInvariantString = True
                    FString.Write(writer, CultureInvariantString);

                    return;
                }
            
                writer.Write(0); // bHasCultureInvariantString = False
            }
        }

        public class Base : FTextHistory
        {
            public string Namespace;
            public string Key;
            public string SourceString;
            // public string LocalizedString;
            
            public override string Text
            {
                get => SourceString ?? "";
                set => SourceString = value;
            }
        
            public Base(Reader reader)
            {
                Namespace = FString.Read(reader);
                Key = FString.Read(reader);
                SourceString = FString.Read(reader);
            }

            public override void Serialize(Writer writer)
            {
                FString.Write(writer, Namespace);
                FString.Write(writer, Key);
                FString.Write(writer, SourceString);
            }
        }

        public class NamedFormat : FTextHistory
        {
            public FText SourceFmt;

            public Dictionary<string, FFormatArgumentValue> Arguments;

            public override string Text
            {
                get => SourceFmt.ToString();
                set => SourceFmt.Text = value;
            }
            
            public NamedFormat(Reader reader)
            {
                SourceFmt = new FText();
                SourceFmt.Read(reader, null);
                var argCount = reader.Read<int>();
                Arguments = new Dictionary<string, FFormatArgumentValue>(argCount);

                for (int i = 0; i < argCount; i++)
                {
                    var key = FString.Read(reader);
                    var value = new FFormatArgumentValue();
                    value.Read(reader, null);
                    Arguments[key] = value;
                }
            }
            
            public override void Serialize(Writer writer)
            {
                SourceFmt.Write(writer);
                writer.Write(Arguments.Count);

                foreach (var arg in Arguments)
                {
                    FString.Write(writer, arg.Key);
                    arg.Value.Write(writer);
                }
            }
        }
        
        public class OrderedFormat : FTextHistory
        {
            public FText SourceFmt;
            public FFormatArgumentValue[] Arguments; /* called FFormatOrderedArguments in UE4 #1# */
            
            public override string Text
            {
                get => SourceFmt.ToString();
                set => SourceFmt.Text = value;
            }

            public OrderedFormat(Reader reader)
            {
                SourceFmt = new FText();
                SourceFmt.Read(reader, null);
                var count = reader.Read<int>();
                Arguments = reader.ReadArray(() =>
                {
                    var value = new FFormatArgumentValue();
                    value.Read(reader, null);
                    return value;
                }, count);
            }
            
            public override void Serialize(Writer writer)
            {
                SourceFmt.Write(writer);
                writer.Write(Arguments.Length);

                foreach (var arg in Arguments)
                    arg.Write(writer);
            }
        }
        
        public class ArgumentFormat : FTextHistory
        {
            public FText SourceFmt;
            public FFormatArgumentData[] Arguments;
            
            
            public override string Text
            {
                get => SourceFmt.ToString();
                set => SourceFmt.Text = value;
            }

            public ArgumentFormat(Reader reader)
            {
                SourceFmt = new FText();
                SourceFmt.Read(reader, null);
                var count = reader.Read<int>();
                Arguments = reader.ReadArray(() => new FFormatArgumentData(reader), count);
            }
            
            public override void Serialize(Writer writer)
            {
                SourceFmt.Write(writer);
                writer.Write(Arguments.Length);

                foreach (var arg in Arguments)
                    arg.Write(writer);
            }
        }

        public class FormatNumber : FTextHistory
        {
            public ETextHistoryType HistoryType;
            public string? CurrencyCode;
            public FFormatArgumentValue SourceValue;
            public bool bHasFormatOptions;
            public FNumberFormattingOptions? FormatOptions;
            public string TargetCulture;

            public override string Text
            {
                get => SourceValue.Value.ToString() ?? "None";
                set => throw new NotSupportedException(
                    "Editing the text value of FormatNumber is not supported. Please modify SourceValue directly.");
            }
            
            public FormatNumber(Reader reader, ETextHistoryType historyType)
            {
                HistoryType = historyType;
                if (HistoryType == ETextHistoryType.AsCurrency)
                {
                    CurrencyCode = FString.Read(reader);
                }

                SourceValue = new FFormatArgumentValue();
                SourceValue.Read(reader, null);

                bHasFormatOptions = reader.Read<bool>();
                if (bHasFormatOptions)
                {
                    FormatOptions = new FNumberFormattingOptions();
                    FormatOptions.Read(reader, null);
                }

                TargetCulture = FString.Read(reader);
            }
            
            public override void Serialize(Writer writer)
            {
                if (HistoryType == ETextHistoryType.AsCurrency)
                {
                    if (string.IsNullOrEmpty(CurrencyCode))
                        Warning("History Type is 'AsCurrency' but 'CurrencyCode' is null.");
                    
                    FString.Write(writer, CurrencyCode ?? string.Empty);
                }
                
                SourceValue.Write(writer);
                
                writer.Write(bHasFormatOptions);
                if (bHasFormatOptions)
                {
                    if (FormatOptions is null)
                        Warning("'bHasFormatOptions' is true but 'FormatOptions' is null.");
                    
                    FormatOptions?.Write(writer);
                }
                
                FString.Write(writer, TargetCulture);
            }
        }

        public class AsDate : FTextHistory
        {
            public FDateTime SourceDateTime;
            public EDateTimeStyle DateStyle;
            public string? TimeZone;
            public string TargetCulture;

            public override string Text
            {
                get => SourceDateTime.ToString();
                set => throw new NotSupportedException(
                    "Editing the text value of AsDate is not supported. Please modify SourceDateTime directly.");
            }
            
            public AsDate(Reader reader)
            {
                SourceDateTime = new FDateTime();
                SourceDateTime.Read(reader, null);
                
                DateStyle = reader.Read<EDateTimeStyle>();
                TimeZone = FString.Read(reader);
                TargetCulture = FString.Read(reader);
            }

            public override void Serialize(Writer writer)
            {
                SourceDateTime.Write(writer);
                writer.Write(DateStyle);
                FString.Write(writer, TimeZone ?? "");
                FString.Write(writer, TargetCulture);
            }
        }

        public class AsTime : FTextHistory
        {
            public FDateTime SourceDateTime;
            public EDateTimeStyle TimeStyle;
            public string TimeZone;
            public string TargetCulture;
            
            public override string Text
            {
                get => SourceDateTime.ToString();
                set => throw new NotSupportedException(
                    "Editing the text value of AsDate is not supported. Please modify SourceDateTime directly.");
            }

            public AsTime(Reader reader)
            {
                SourceDateTime = new FDateTime();
                SourceDateTime.Read(reader, null);
                TimeStyle = reader.Read<EDateTimeStyle>();
                TimeZone = FString.Read(reader);
                TargetCulture =  FString.Read(reader);
            }
            
            public override void Serialize(Writer writer)
            {
                SourceDateTime.Write(writer);
                writer.Write(TimeStyle);
                FString.Write(writer, TimeZone);
                FString.Write(writer, TargetCulture);
            }
        }

        public class AsDateTime : FTextHistory
        {
            public FDateTime SourceDateTime;
            public EDateTimeStyle DateStyle;
            public EDateTimeStyle TimeStyle;
            public string TimeZone;
            public string TargetCulture;
            
            public override string Text
            {
                get => SourceDateTime.ToString();
                set => throw new NotSupportedException(
                    "Editing the text value of AsDate is not supported. Please modify SourceDateTime directly.");
            }

            public AsDateTime(Reader reader)
            {
                SourceDateTime = new FDateTime();
                SourceDateTime.Read(reader, null);
                DateStyle = reader.Read<EDateTimeStyle>();
                TimeStyle = reader.Read<EDateTimeStyle>();
                TimeZone = FString.Read(reader);
                TargetCulture =  FString.Read(reader);
            }
            
            public override void Serialize(Writer writer)
            {
                SourceDateTime.Write(writer);
                writer.Write(DateStyle);
                writer.Write(TimeStyle);
                FString.Write(writer, TimeZone);
                FString.Write(writer, TargetCulture);
            }
        }

        public class Transform : FTextHistory
        {
            public FText SourceText;
            public ETransformType TransformType;
            
            public override string Text
            {
                get => SourceText.ToString();
                set => SourceText.Text = value;
            }

            public Transform(Reader reader)
            {
                SourceText = new FText();
                SourceText.Read(reader, null);
                TransformType = reader.Read<ETransformType>();
            }
            
            public override void Serialize(Writer writer)
            {
                SourceText.Write(writer);
                writer.Write(TransformType);
            }
        }

        public class StringTableEntry : FTextHistory
        {
            public FName TableId;
            public string Key;
            public string SourceString;
            public string LocalizedString;
            
            public override string Text
            {
                get => SourceString;
                set => SourceString = value;
            }
            
            public NameMapContainer NameMap; // For serialization

            public StringTableEntry(Reader reader, NameMapContainer nameMap)
            {
                NameMap = nameMap;
                TableId = new FName(reader, NameMap);
                Key = FString.Read(reader);
            }
            
            public override void Serialize(Writer writer)
            {
                TableId.Serialize(writer, NameMap);
                FString.Write(writer, Key);
            }
        }

        public class TextGenerator : FTextHistory
        {
            public FName GeneratorTypeID;
            public byte[]? GeneratorContents;
            
            public override string Text
            {
                get => GeneratorTypeID.Name;
                set => GeneratorTypeID.Name = value;
            }

            public NameMapContainer NameMap; // For serialization

            public TextGenerator(Reader reader, NameMapContainer nameMap)
            {
                NameMap = nameMap;
                GeneratorTypeID = new FName(reader, NameMap);
                
                /*if (!GeneratorTypeID.IsNone)
                {
                    // https://github.com/EpicGames/UnrealEngine/blob/4.26/Engine/Source/Runtime/Core/Private/Internationalization/TextHistory.cpp#L2916
                    // I don't understand what it does here
                }*/
            }
            
            public override void Serialize(Writer writer)
            {
                GeneratorTypeID.Serialize(writer, NameMap);
            }
        }
    }
    
    public class FFormatArgumentData : UStruct
    {
        public string ArgumentName;
        public FFormatArgumentValue ArgumentValue;

        public FFormatArgumentData(Reader reader)
        {
            ArgumentName = FString.Read(reader);
            ArgumentValue = new FFormatArgumentValue();
            ArgumentValue.Read(reader, null);
        }
    }
    
    public class FNumberFormattingOptions : UStruct
    {
        public enum ERoundingMode : sbyte
        {
            /** Rounds to the nearest place, equidistant ties go to the value which is closest to an even value: 1.5 becomes 2, 0.5 becomes 0 */
            HalfToEven,

            /** Rounds to nearest place, equidistant ties go to the value which is further from zero: -0.5 becomes -1.0, 0.5 becomes 1.0 */
            HalfFromZero,

            /** Rounds to nearest place, equidistant ties go to the value which is closer to zero: -0.5 becomes 0, 0.5 becomes 0. */
            HalfToZero,

            /** Rounds to the value which is further from zero, "larger" in absolute value: 0.1 becomes 1, -0.1 becomes -1 */
            FromZero,

            /** Rounds to the value which is closer to zero, "smaller" in absolute value: 0.1 becomes 0, -0.1 becomes 0 */
            ToZero,

            /** Rounds to the value which is more negative: 0.1 becomes 0, -0.1 becomes -1 */
            ToNegativeInfinity,

            /** Rounds to the value which is more positive: 0.1 becomes 1, -0.1 becomes 0 */
            ToPositiveInfinity,


            // Add new enum types at the end only! They are serialized by index.
        }
        
        private const int _DBL_DIG = 15;
        private const int _DBL_MAX_10_EXP = 308;

        public bool AlwaysSign;
        public bool UseGrouping;
        public ERoundingMode RoundingMode;
        public int MinimumIntegralDigits;
        public int MaximumIntegralDigits;
        public int MinimumFractionalDigits;
        public int MaximumFractionalDigits;

        public FNumberFormattingOptions()
        {
            AlwaysSign = false;
            UseGrouping = true;
            RoundingMode = ERoundingMode.HalfToEven;
            MinimumIntegralDigits = 1;
            MaximumIntegralDigits = _DBL_MAX_10_EXP + _DBL_DIG + 1;
            MinimumFractionalDigits = 0;
            MaximumFractionalDigits = 3;
        }

        public override void Read(Reader reader, PropertyData? data, Asset? asset = null, ESerializationMode mode = ESerializationMode.Normal)
        {
            AlwaysSign = reader.ReadBoolean();
            UseGrouping = reader.ReadBoolean();
            RoundingMode = reader.Read<ERoundingMode>();
            MinimumIntegralDigits = reader.Read<int>();
            MaximumIntegralDigits = reader.Read<int>();
            MinimumFractionalDigits = reader.Read<int>();
            MaximumFractionalDigits = reader.Read<int>();
        }

        public override void Write(Writer writer, Asset? asset = null)
        {
            writer.Write(AlwaysSign);
            writer.Write(UseGrouping);
            writer.Write(RoundingMode);
            writer.Write(MinimumIntegralDigits);
            writer.Write(MaximumIntegralDigits);
            writer.Write(MinimumFractionalDigits);
            writer.Write(MaximumFractionalDigits);
        }
    }
    
    public enum EDateTimeStyle : sbyte
    {
        Default,
        Short,
        Medium,
        Long,

        Full
        // Add new enum types at the end only! They are serialized by index.
    }
    
    public enum ETransformType : byte
    {
        ToLower = 0,
        ToUpper,

        // Add new enum types at the end only! They are serialized by index.
    }
}

public class TextProperty : AbstractProperty<FText>
{
    public TextProperty()
    { }
    
    public TextProperty(FText value)
    {
        Value = value;
    }
    
    public override void Read(Reader reader, PropertyData? data, Asset? asset = null,
        ESerializationMode mode = ESerializationMode.Normal)
    {
        Value = new FText();
        Value.Read(reader, data, asset, mode);
    }

    public override void Write(Writer writer, UProperty property, Asset? asset = null, ESerializationMode mode = ESerializationMode.Normal)
    {
        if (Value is null)
            throw new NoNullAllowedException("Value of FText cannot be null.");
        
        Value.Write(writer, asset);
    }
}