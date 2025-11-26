using UAssetEditor.Binary;
using UAssetEditor.Unreal.Assets;
using UsmapDotNet;


namespace UAssetEditor.Unreal.Properties;

public enum EPropertyDataType
{
    Class,
    Enum,
    Struct,
    Array,
    Map
}

public class PropertyData
{
    public string? Type;
    public string? EnumName;
    public string? StructType;
    public PropertyData? InnerType;
    public PropertyData? ValueType;

    public void SetClassData(string type)
    {
        Type = type;
        EnumName = null;
        StructType = null;
        InnerType = null;
        ValueType = null;
    }
    
    public void SetEnumData(string name)
    {
        Type = "EnumProperty";
        EnumName = name;
        StructType = null;
        InnerType = null;
        ValueType = null;
    }
    
    public void SetStructData(string structType)
    {
        Type = "StructProperty";
        EnumName = null;
        StructType = structType;
        InnerType = null;
        ValueType = null;
    }
    
    public PropertyData()
    { }

    public PropertyData(string type)
    {
        Type = type;
    }
    
    public PropertyData(UsmapPropertyData data)
    {
        Type = data.Type.ToString();
        EnumName = data.EnumName;
        StructType = data.StructType;
        
        if (data.InnerType != null)
            InnerType = new PropertyData(data.InnerType);
        
        if (data.ValueType != null)
            ValueType = new PropertyData(data.ValueType);
    }
}

public class UProperty
{
    public PropertyData? Data;
    public string Name;
    public object? Value;
    public byte ArraySize;
    public int SchemaIdx;
    public bool IsZero;

    public UProperty()
    {
        Name = "None";
    }
    
    public UProperty(UsmapPropertyData data, string name, object? value, byte arraySize = 1, int schemaIdx = -1, bool isZero = false)
    {
        Data = new PropertyData(data);
        Name = name;
        Value = value;
        ArraySize = arraySize;
        SchemaIdx = schemaIdx;
        IsZero = isZero;
    } 
    
    public UProperty(PropertyData data, string name, object? value, byte arraySize = 1, int schemaIdx = -1, bool isZero = false)
    {
        Data = data;
        Name = name;
        Value = value;
        ArraySize = arraySize;
        SchemaIdx = schemaIdx;
        IsZero = isZero;
    }

    public T? GetValue<T>() where T : class
    {
        return Value as T;
    }

    public override string ToString() => $"{Name}: {Value} ({Data.Type})";
}

public enum ESerializationMode
{
    Zero,
    Normal,
    Map
}

public abstract class AbstractProperty : ICloneable
{
    public string? Name { get; set; }
    public abstract object? ValueAsObject { get; }

    public string GetPropertyType() => GetType().Name;
    
    public virtual void Read(Reader reader, PropertyData? data, Asset? asset = null,
        ESerializationMode mode = ESerializationMode.Normal)
    { } 

    public virtual void Write(Writer writer, UProperty property, Asset? asset = null, ESerializationMode mode = ESerializationMode.Normal)
    { }

    public object Clone()
    {
        return MemberwiseClone();
    }
}

public abstract class AbstractProperty<T> : AbstractProperty
{
    public T? Value;
    
    public override object? ValueAsObject => Value;

    public override string ToString()
    {
        return Value?.ToString() ?? "None";
    }

    public AbstractProperty()
    { }
    
    public AbstractProperty(T value)
    {
        Value = value;
    }
}