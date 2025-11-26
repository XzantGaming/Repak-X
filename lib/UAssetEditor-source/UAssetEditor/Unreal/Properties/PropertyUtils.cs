using System.Data;
using UAssetEditor.Unreal.Properties.Reflection;
using UAssetEditor.Binary;
using UAssetEditor.Unreal.Assets;


namespace UAssetEditor.Unreal.Properties;

public struct NameValuePair
{
    public string Name;
    public object Value;

    public NameValuePair(string name, object value)
    {
        Name = name;
        Value = value;
    }
}

public static class PropertyUtils
{
    public static object ReadProperty(string type, Reader reader, PropertyData? data, Asset? asset = null, ESerializationMode mode = ESerializationMode.Normal)
    {
        if (string.IsNullOrEmpty(type))
            throw new ArgumentNullException($"'{nameof(type)}' cannot be null or empty.");
        
        return PropertyReflector.ReadProperty(type, reader, data, asset, mode);
    }
    
    public static void WriteProperty(Writer writer, UProperty prop, Asset? asset = null, ESerializationMode mode = ESerializationMode.Normal)
    {
        PropertyReflector.WriteProperty(writer, prop, asset, mode);
    }

    public static string GetStructType(this PropertyData data)
    {
        return data.Type switch
        {
            "StructProperty" => data.StructType,
            "ArrayProperty" => data.InnerType?.StructType,
            _ => data.Type
        } ?? throw new DataException($"Could not find structure type for '{data}'");
    }
}