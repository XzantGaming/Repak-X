using UAssetEditor.Binary;
using UAssetEditor.Unreal.Assets;
using UAssetEditor.Unreal.Properties;
using UAssetEditor.Unreal.Properties.Unversioned;
using UsmapDotNet;

namespace UAssetEditor.Unreal.Exports;

public class UStruct : UObject
{
    public string SuperType;

    public UStruct()
    {
        SuperType = "None";
    }

    public UStruct(string superType)
    {
        SuperType = superType;
    }
    
    public UStruct(UsmapSchema schema, Usmap mappings)
    {
        Name = schema.Name;
        SuperType = schema.SuperType ?? "None";

        while (true)
        {
            foreach (var prop in schema.Properties)
            {
                var property = new UProperty(prop.Data, prop.Name, null, prop.ArraySize, prop.SchemaIdx);

                for (int j = 0; j < property.ArraySize; j++)
                {
                    Properties.Add(property);
                }
            }

            if (schema.SuperType == null)
                return;

            var super = mappings.FindSchema(schema.SuperType);
            schema = super ?? throw new KeyNotFoundException($"Cannot find schema named '{schema.SuperType}'");
        }
    }
    
    public virtual void Read(Reader reader, PropertyData? data, Asset? asset = null,
        ESerializationMode mode = ESerializationMode.Normal)
    { } 

    public virtual void Write(Writer writer, Asset? asset = null)
    { }
}