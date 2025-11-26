using System.Data;
using UAssetEditor.Binary;
using UAssetEditor.Classes;
using UAssetEditor.Summaries;
using UAssetEditor.Unreal.Assets;
using UAssetEditor.Unreal.Exports.Engine;
using UAssetEditor.Unreal.Misc;
using UAssetEditor.Unreal.Properties;
using UAssetEditor.Unreal.Properties.Unversioned;

namespace UAssetEditor.Unreal.Exports;

public class UObject
{
    public string Name;
    public Lazy<UObject?> Outer;
    public UStruct? Class;
    public Lazy<ResolvedObject?> Super;
    public Lazy<ResolvedObject?> Template;
    public EObjectFlags Flags;
    public FGuid? ObjectGuid { get; private set; }

    public List<UProperty> Properties;

    public Asset? Owner;

    public UObject()
    {
        Name = "None";
        Properties = new();
    }
    
    public UObject(Asset asset) : this()
    {
        Owner = asset;
    }

    // TODO other export types
    public static UObject ConstructObject(Asset asset, string className)
    {
        return className switch
        {
            "CurveTable" => new UCurveTable(asset),
            _ => new UObject(asset)
        };
    }

    public override string ToString()
    {
        return $"{Class?.Name ?? "None"}.{Name}";
    }

    public UProperty? this[string name]
    {
        get => Properties.FirstOrDefault(x => x.Name == name);
        set
        {
            var property = Properties.FirstOrDefault(x => x.Name == name);
            
            if (property != null)
            {
                if (value == null)
                {
                    Properties.Remove(property);
                    return;
                }
                
                property.Data = value.Data;
                property.Value = value.Value;
                property.ArraySize = value.ArraySize;
                property.IsZero = value.IsZero;
                property.SchemaIdx = value.SchemaIdx;
                return;
            }

            if (value == null)
                return;
            
            Properties.Add(value);
        }
    }

    public virtual void Deserialize(long position)
    {
        if (Owner is null)
            throw new NoNullAllowedException("Cannot deserialize UObject without a valid asset to work with.");

        if (Class is null)
            throw new NoNullAllowedException($"Class cannot be null.");

        Owner.Position = position;
        
        if (Owner.HasUnversionedProperties)
        {
            Properties = UnversionedPropertyHandler.DeserializeProperties(Owner, Class);
        }
        else
        {
            // TODO tagged properties
        }
        
        if (!Flags.HasFlag(EObjectFlags.RF_ClassDefaultObject) && Owner.Read<int>() == 1)
        {
            ObjectGuid = Owner.Read<FGuid>();
        }
    }
    
    public virtual void Serialize(Writer writer)
    {
        if (Owner is null)
            throw new NoNullAllowedException("Cannot serialize UObject without a valid asset to work with.");

        if (Class is null)
            throw new NoNullAllowedException($"Class cannot be null.");
        
        if (Owner.HasUnversionedProperties)
        {
            UnversionedPropertyHandler.SerializeProperties((ZenAsset)Owner, writer, Class, Properties);
        }
        else
        {
            // TODO tagged properties
        }
        
        if (!Flags.HasFlag(EObjectFlags.RF_ClassDefaultObject))
        {
            var shouldWriteGuid = ObjectGuid != null;
            writer.Write(shouldWriteGuid ? 1 : 0); // Boolean as Int32
            
            if (shouldWriteGuid)
                writer.Write(ObjectGuid);
        }
    }
}