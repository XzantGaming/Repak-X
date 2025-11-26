using System.Data;
using UAssetEditor.Unreal.Properties.Reflection;
using UAssetEditor.Unreal.Properties.Structs;
using UAssetEditor.Binary;
using UAssetEditor.Unreal.Assets;
using UAssetEditor.Utils;


namespace UAssetEditor.Unreal.Properties.Types;

public class StructProperty : AbstractProperty<object>
{
    public StructProperty()
    { }
    
    public StructProperty(object value)
    {
        Value = value;
    }
    
    public string Type = "None";

    /// <summary>
    /// Only applicable if value is an undefined struct (unversioned assets)
    /// </summary>
    public CustomStructHolder? Holder => Value?.AsOrDefault<CustomStructHolder>();
    
    public T? GetProperties<T>() where T : class
    {
        return Value as T;
    }
    
    public override string ToString()
    {
        if (Value is List<UProperty> properties)
            return $"[{properties.Count}]";
        
        return $"({Type})";
    }

    public override void Read(Reader reader, PropertyData? data, Asset? asset = null,
        ESerializationMode mode = ESerializationMode.Normal)
    {
        Type = data?.StructType ?? "None";
        Value = PropertyReflector.ReadStruct(reader, data, asset, mode);
    }

    public override void Write(Writer writer, UProperty property, Asset? asset = null, ESerializationMode mode = ESerializationMode.Normal)
    {
        if (Value is null)
            throw new NoNullAllowedException("Cannot write struct property without a non-null value.");

        var value = Value;
        if (value is CustomStructHolder holder)
            value = holder.Properties;
        
        PropertyReflector.WriteStruct(writer, value, property.Data ?? throw new NoNullAllowedException($"{nameof(property.Data)} cannot be null."), asset);
    }
}