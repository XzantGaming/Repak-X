using System.Data;
using UAssetEditor.Unreal.Names;
using UAssetEditor.Binary;
using UAssetEditor.Unreal.Assets;
using UAssetEditor.Unreal.Properties.Reflection;


namespace UAssetEditor.Unreal.Properties.Types;

public class EnumProperty : AbstractProperty<string>
{
    public EnumProperty()
    { }

    public EnumProperty(string value)
    {
        Value = value;
    }
    
    public override void Read(Reader reader, PropertyData? data, Asset? asset = null,
        ESerializationMode mode = ESerializationMode.Normal)
    {
        
        ArgumentNullException.ThrowIfNull(data);
        ArgumentNullException.ThrowIfNull(data.InnerType?.Type);
        ArgumentNullException.ThrowIfNull(asset);
        ArgumentNullException.ThrowIfNull(asset.Mappings);
        
        object enumObj = 0;

        switch (mode)
        {
            case ESerializationMode.Zero:
                break;
            case ESerializationMode.Normal:
                var value = PropertyUtils.ReadProperty(data.InnerType.Type, reader, data.InnerType, asset);
                if (value is not AbstractProperty prop)
                    throw new InvalidCastException("Property is not a AbstractProperty?");

                enumObj = prop.ValueAsObject ?? 0;
                break;
            case ESerializationMode.Map:
                Value = new FName(reader, asset.NameMap).Name;
                return;
        }
        
        var index = Convert.ToInt32(enumObj);
        var enumData = asset.Mappings.Enums.FirstOrDefault(x => x.Name == data.EnumName);
        
        if (enumData is null)
            throw new NullReferenceException($"Enum {data.EnumName} not found in mappings.");
        
        Value = enumData.Names.Length >= index
            ? enumData.Names[index]
            : throw new KeyNotFoundException($"Could not find a enum name ('{data.EnumName}') at index {index}.");
    }
    
    public override void Write(Writer writer, UProperty property, Asset? asset = null, ESerializationMode mode = ESerializationMode.Normal)
    {
        if (asset is null)
            throw new NoNullAllowedException($"'{nameof(asset)}' cannot be null");
        
        if (mode == ESerializationMode.Map)
        {
            if (property.Value is not EnumProperty name)
                throw new InvalidCastException("Value of enum must be EnumProperty.");
            
            new FName(name.Value!).Serialize(writer, asset.NameMap);
            return;
        }
        
        asset.CheckMappings();
        
        var enumData = asset.Mappings?.Enums.FirstOrDefault(x => x.Name == property.Data?.EnumName);
        
        if (enumData is null)
            throw new NullReferenceException($"Enum {property.Data?.EnumName} not found in mappings.");

        if (property.Value is not EnumProperty enumProperty)
            throw new InvalidCastException($"Property must be a enum property to serialize as a enum.");
        
        ArgumentNullException.ThrowIfNull(enumProperty.Value);
        
        var index = enumData.Names.ToList().IndexOf(enumProperty.Value);
        var enumType = property.Data?.InnerType?.Type;
        
        ArgumentNullException.ThrowIfNull(enumType);
        
        PropertyReflector.WriteProperty(writer, new UProperty(new PropertyData(enumType), "Dummy", enumType switch
        {
            "ByteProperty" => new ByteProperty((byte)index),
            "Int16Property" => new Int16Property((short)index),
            "Int64Property" => new Int64Property(index),
            "IntProperty" => new IntProperty(index),
            "UInt16Property" => new UInt16Property((ushort)index),
            "UInt64Property" => new UInt64Property((ulong)index)
        }));
    }
}