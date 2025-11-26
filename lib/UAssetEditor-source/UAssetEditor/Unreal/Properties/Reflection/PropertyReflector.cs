using System.Data;
using UAssetEditor.Unreal.Exports;
using UAssetEditor.Unreal.Properties.Structs;
using UAssetEditor.Unreal.Properties.Structs.Math;
using UAssetEditor.Unreal.Properties.Types;
using UAssetEditor.Binary;
using UAssetEditor.Classes;
using UAssetEditor.Unreal.Assets;
using UAssetEditor.Unreal.Misc;
using UAssetEditor.Unreal.Properties.Structs.Curves;
using UAssetEditor.Unreal.Properties.Structs.GameplayTags;
using System.Drawing;
using UAssetEditor.Unreal.Properties.Structs.Misc;
using UAssetEditor.Unreal.Properties.Structs.AI;

namespace UAssetEditor.Unreal.Properties.Reflection;

public class VersionedType
{
    private FPackageFileVersion _version;
    public Type Type;

    public VersionedType(EUnrealEngineObjectUE4Version version, Type type)
    {
        _version = FPackageFileVersion.CreateUE4Version(version);
        Type = type;
    }
    
    public VersionedType(EUnrealEngineObjectUE5Version version, Type type)
    {
        _version = FPackageFileVersion.CreateUE5Version(version);
        Type = type;
    }

    #region OPERATORS
    public static bool operator ==(VersionedType left, EUnrealEngineObjectUE4Version version)
    {
        return left._version == version;
    }
    
    public static bool operator !=(VersionedType left, EUnrealEngineObjectUE4Version version)
    {
        return left._version != version;
    }
    
    public static bool operator ==(VersionedType left, EUnrealEngineObjectUE5Version version)
    {
        return left._version == version;
    }
    
    public static bool operator !=(VersionedType left, EUnrealEngineObjectUE5Version version)
    {
        return left._version != version;
    }
    #endregion OPERATORS
}

public class UStructVer
{
    public readonly string Name;
    public readonly Type Default;
    private readonly VersionedType[] Versions;

    public UStructVer(string name, Type defaultType, params VersionedType[] versions)
    {
        Name = name;
        Default = defaultType;
        Versions = versions;
    }

    public Type Get(EUnrealEngineObjectUE4Version version)
    {
        foreach (var ver in Versions)
        {
            if (ver == version)
                return ver.Type;
        }

        Warning($"Could not find struct '{Name}' with the version: {version}. Returning default type.");
        return Default;
    }
    
    public Type Get(EUnrealEngineObjectUE5Version version)
    {
        foreach (var ver in Versions)
        {
            if (ver == version)
                return ver.Type;
        }

        Warning($"Could not find struct '{Name}' with the version: {version}. Returning default type.");
        return Default;
    }
    
    public Type Get(FPackageFileVersion version)
    {
        foreach (var ver in Versions)
        {
            if (ver == version.FileVersionUE4
                || ver == version.FileVersionUE5)
                return ver.Type;
        }

        Warning($"Could not find struct '{Name}' with the version: {version}. Returning default type.");
        return Default;
    }
}

// I know it's not really reflection, but I'm calling it that okay..
public static class PropertyReflector
{
    public static Dictionary<string, Type> PropertiesByName = new()
    {
        { "ArrayProperty", typeof(ArrayProperty) },
        { "BoolProperty", typeof(BoolProperty) },
        { "DoubleProperty", typeof(DoubleProperty) },
        { "EnumProperty", typeof(EnumProperty) },
        { "FloatProperty", typeof(FloatProperty) },
        { "Int8Property", typeof(ByteProperty) },
        { "ByteProperty", typeof(ByteProperty) },
        { "Int16Property", typeof(Int16Property) },
        { "IntProperty", typeof(IntProperty) },
        { "Int64Property", typeof(Int64Property) },
        { "UInt16Property", typeof(UInt16Property) },
        { "UInt64Property", typeof(UInt64Property) },
        { "StructProperty", typeof(StructProperty) },
        { "TextProperty", typeof(TextProperty) },
        { "NameProperty", typeof(NameProperty) },
        { "StrProperty", typeof(StrProperty) },
        { "SoftClassProperty", typeof(SoftObjectProperty) },
        { "SoftObjectProperty", typeof(SoftObjectProperty) },
        { "ScriptInterface", typeof(ObjectProperty) },
        { "ObjectProperty", typeof(ObjectProperty) },
        { "MapProperty", typeof(MapProperty) },
        { "MulticastDelegateProperty", typeof(MulticastDelegateProperty) }
    };
    
    public static List<UStructVer> DefinedStructsByName = new()
    {
        new UStructVer("Box2f", typeof(TBox2<float>)),
        new UStructVer("Box", typeof(FBox)),
        new UStructVer("Box2D", typeof(FBox2D)),
        new UStructVer("NavAgentSelector", typeof(FNavAgentSelector)),
        new UStructVer("Color", typeof(FColor)),
        new UStructVer("DateTime", typeof(FDateTime)),
        new UStructVer("FrameNumber", typeof(FFrameNumber)),
        new UStructVer("GameplayTagContainer", typeof(FGameplayTagContainer)),
        new UStructVer("InstancedStruct", typeof(FInstancedStruct)),
        new UStructVer("Vector", typeof(FVector3), 
            new VersionedType(EUnrealEngineObjectUE5Version.LARGE_WORLD_COORDINATES, typeof(FVector3_LARGE_WORLD_COORDINATES))),
        new UStructVer("Vector4", typeof(FVector4),
            new VersionedType(EUnrealEngineObjectUE5Version.LARGE_WORLD_COORDINATES, typeof(FVector4_LARGE_WORLD_COORDINATES))),
        new UStructVer("DeprecateSlateVector2D", typeof(FVector2D)),
        new UStructVer("Rotator", typeof(FRotator),
            new VersionedType(EUnrealEngineObjectUE5Version.LARGE_WORLD_COORDINATES, typeof(FRotator_LARGE_WORLD_COORDINATES))),
        new UStructVer("Quat", typeof(FQuat),
            new VersionedType(EUnrealEngineObjectUE5Version.LARGE_WORLD_COORDINATES, typeof(FQuat_LARGE_WORLD_COORDINATES))),
        new UStructVer("LinearColor", typeof(FLinearColor)),
        new UStructVer("Guid", typeof(FGuid)),
        new UStructVer("RichCurveKey", typeof(RichCurve.FRichCurveKey)),
        new UStructVer("SimpleCurveKey", typeof(SimpleCurve.FSimpleCurveKey)),
        new UStructVer("IntPoint", typeof(FIntPoint)),
        new UStructVer("IntVector2", typeof(TIntVector2<int>)),
        new UStructVer("UintVector2", typeof(TIntVector2<uint>)),
        new UStructVer("IntVector", typeof(FIntVector)),
        new UStructVer("Transform3f", typeof(FTransform)),
        new UStructVer("Plane", typeof(FPlane),
            new VersionedType(EUnrealEngineObjectUE5Version.LARGE_WORLD_COORDINATES, typeof(FPlane_LARGE_WORLD_COORDINATES))),
        new UStructVer("Int32Point", typeof(FIntPoint))
    };

    public static object ReadProperty(string type, Reader reader, PropertyData? data,
        Asset? asset = null, ESerializationMode mode = ESerializationMode.Normal)
    {
        foreach (var kvp in PropertiesByName)
        {
            if (type != kvp.Key)
                continue;

            if (Activator.CreateInstance(kvp.Value) is not AbstractProperty instance)
                throw new ApplicationException(
                    $"{kvp.Value.Name} is listed as a property but does not inherit 'AbstractProperty'.");
            
            instance.Read(reader, data, asset, mode);
            return instance;
        }

        throw new NotImplementedException($"Property type '{type}' is not implemented!");
    }
    
    public static void WriteProperty(Writer writer, UProperty prop, Asset? asset = null, ESerializationMode mode = ESerializationMode.Normal)
    {
        if (prop.Value is not AbstractProperty property)
            throw new ApplicationException($"{prop.Name} is not a 'AbstractProperty'.");
        
        property.Write(writer, prop, asset, mode);
    }
    
    public static object ReadStruct(Reader reader, PropertyData? data,
        Asset? asset = null, ESerializationMode mode = ESerializationMode.Normal)
    {
        ArgumentNullException.ThrowIfNull(data);
        ArgumentNullException.ThrowIfNull(asset);

        var type = data.StructType ?? data.InnerType?.StructType ??
            throw new NoNullAllowedException("Struct type cannot be null.");
        
        // TODO determine properly
        var structType = data.InnerType?.StructType ?? data.StructType;

        foreach (var ver in DefinedStructsByName)
        {
            if (structType != ver.Name)
                continue;

            var ustruct = ver.Get(asset.FileVersion);
            var instance = Activator.CreateInstance(ustruct);

            if (ustruct.IsValueType) // is it a struct
            {
                if (mode == ESerializationMode.Zero)
                    return instance; // return default
                
                var method = asset.GetType()
                    .GetMethods()
                    .FirstOrDefault(x => x.Name == "Read")!
                    .MakeGenericMethod(ustruct);

                return method.Invoke(asset, []) ?? throw new NoNullAllowedException($"{nameof(method)} returned null");
            }

            if (instance is not UStruct struc)
                throw new ApplicationException("Defined Struct is not a ValueType or UStruct type. Cannot deserialize!");
                    
            struc.Read(reader, data, asset, mode);
            return instance;
        }

        ArgumentNullException.ThrowIfNull(asset);
        
        return new CustomStructHolder(asset.ReadProperties(type));
    }

    public static void WriteStruct(Writer writer, object property, PropertyData data, Asset? asset = null)
    {
        ArgumentNullException.ThrowIfNull(asset);

        var structType = data.GetStructType();

        if (property is List<UProperty> properties)
        {
            asset.WriteProperties(writer, structType, properties);
            return;
        }

        var type = property.GetType();
        if (type.IsValueType) // is it a struct
        {
            var method = typeof(Writer).GetMethods()
                .FirstOrDefault(x => x.Name == "Write")!
                .MakeGenericMethod(type);
            method.Invoke(writer, [property]);
            
            return;
        }

        if (property is not UStruct struc)
            throw new ApplicationException("Defined Struct is not a ValueType or UStruct type. Cannot serialize!");

        struc.Write(writer, asset);
    }
}
