using UAssetEditor.Binary;
using UAssetEditor.Unreal.Assets;


namespace UAssetEditor.Unreal.Properties.Types;

public class ArrayProperty : AbstractProperty<List<object>>
{
    public ArrayProperty()
    { }

    public ArrayProperty(List<object> value)
    {
        Value = value;
    }
    
    public void AddItem(object obj)
    {
        Value ??= [];
        Value.Add(obj);
    }
    
    public void RemoveItem(int index) => Value?.RemoveAt(index);
    
    public void RemoveItem(object item) => Value?.Remove(item);

    public object? GetItemAt(int index) => Value?[index] ?? null;

    public T? GetItemAt<T>(int index) where T : class
    {
        return Value?[index] as T ?? null;
    }

    public object? this[int index] => GetItemAt(index);
    
    public override string ToString()
    {
        return $"[{Value?.Count}]";
    }

    public override void Read(Reader reader, PropertyData? data, Asset? asset = null,
        ESerializationMode mode = ESerializationMode.Normal)
    {
        ArgumentNullException.ThrowIfNull(data);
        ArgumentNullException.ThrowIfNull(data.InnerType);
        ArgumentNullException.ThrowIfNull(data.InnerType.Type);

        Value = [];
        
        if (mode == ESerializationMode.Zero)
            return;
        
        var count = reader.Read<int>();

        for (int i = 0; i < count; i++)
        {
            var item = PropertyUtils.ReadProperty(data.InnerType.Type, reader, data.InnerType, asset);
            Value.Add(item);
        }
    }

    public override void Write(Writer writer, UProperty property, Asset? asset = null, ESerializationMode mode = ESerializationMode.Normal)
    {
        if (Value is null)
        {
            writer.Write(0);
            return;
        }
        
        writer.Write(Value.Count);
        
        foreach (var elm in Value)
        {
            switch (elm)
            {
                case AbstractProperty prop:
                    prop.Write(writer, property, asset);
                    break;
                default:
                    throw new ApplicationException("Array element is not a AbstractProperty!");
            }
        }
    }
}