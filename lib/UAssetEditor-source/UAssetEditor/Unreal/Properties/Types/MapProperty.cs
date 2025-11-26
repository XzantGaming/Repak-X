using System.Data;
using UAssetEditor.Binary;
using UAssetEditor.Unreal.Assets;


namespace UAssetEditor.Unreal.Properties.Types;

public class MapProperty : AbstractProperty<Dictionary<object, object>>
{
    public MapProperty()
    { }
    
    public MapProperty(Dictionary<object, object> value)
    {
        Value = value;
    }

    public override string ToString()
    {
        return $"(KeyType '{KeyType?.Type ?? "None"}'):(ValueType '{ValueType?.Type ?? "None"}') [{Value?.Count}]";
    }

    public PropertyData? KeyType;
    public PropertyData? ValueType;
    
    public override void Read(Reader reader, PropertyData? data, Asset? asset = null,
        ESerializationMode mode = ESerializationMode.Normal)
    {
        ArgumentNullException.ThrowIfNull(data);
        ArgumentNullException.ThrowIfNull(data.InnerType);
        ArgumentNullException.ThrowIfNull(data.ValueType);
        //ArgumentNullException.ThrowIfNull(asset);

        KeyType = data.InnerType;
        ValueType = data.ValueType;

        if (data.InnerType.Type == null)
            throw new NoNullAllowedException("InnerType cannot be null.");
        
        var numKeysToRemove = reader.Read<int>();
        for (int i = 0; i < numKeysToRemove; i++)
        {
            PropertyUtils.ReadProperty(data.InnerType.Type, reader, null, asset);
        }

        var num = reader.Read<int>();
        Value = new Dictionary<object, object>();

        var keyType = data.InnerType!.Type!;
        var valueType = data.ValueType?.Type ?? throw new NoNullAllowedException("ValueType cannot be null.");

        for (int i = 0; i < num; i++)
        {
            var key = PropertyUtils.ReadProperty(keyType, reader, data.InnerType, asset, ESerializationMode.Map);
            var value = PropertyUtils.ReadProperty(valueType, reader, data.ValueType, asset, ESerializationMode.Map);
            Value.Add(key, value);
        }
    }

    public override void Write(Writer writer, UProperty property, Asset? asset = null, ESerializationMode mode = ESerializationMode.Normal)
    {
        ArgumentNullException.ThrowIfNull(KeyType);
        ArgumentNullException.ThrowIfNull(ValueType);
        
        var mapValue = (MapProperty)property.Value!;
        var map = mapValue.Value;
        
        writer.Write(0); // numKeysToRemove (dunno what this is)

        if (map is null)
        {
            writer.Write(0); // array num
            return;
        }
        
        writer.Write(map.Count); // array num
        foreach (var kvp in map)
        {
            PropertyUtils.WriteProperty(writer, new UProperty(KeyType, "Key", kvp.Key), asset, ESerializationMode.Map);
            PropertyUtils.WriteProperty(writer, new UProperty(ValueType, "Value", kvp.Value), asset, ESerializationMode.Map);
        }
    }
}