using System.Data;
using UAssetEditor.Unreal.Exports;
using UAssetEditor.Unreal.Objects;
using UAssetEditor.Binary;
using UAssetEditor.Unreal.Assets;
using UAssetEditor.Unreal.Properties.Unversioned;

namespace UAssetEditor.Unreal.Properties.Structs;

// TODO
public class FInstancedStruct : UStruct
{
    public FPackageIndex Index;
    public byte[]? Buffer;

    public override void Read(Reader reader, PropertyData? data, Asset? asset = null,
        ESerializationMode mode = ESerializationMode.Normal)
    {
        Index = new FPackageIndex(asset, reader.Read<int>());
        var serialSize = reader.Read<int>();

        var className = Index.ResolvedObject?.Name.ToString();
        var struc = asset?.Mappings?.FindSchema(className ?? "None");
        
        if (struc != null)
        {
            Class = new UStruct(struc, asset!.Mappings!);

            var start = reader.Position;
            Properties = asset.ReadProperties(className);

            var actualSize = reader.Position - start;
            if (actualSize != serialSize)
                Warning($"FInstanedStruct ({className}) was expected to read {serialSize} bytes, but actually read {actualSize}");
        }
        else
        {
            Buffer = reader.ReadBytes(serialSize);
        }
    }

    public override void Write(Writer writer, Asset? asset = null)
    {
        writer.Write(Index.Index);

        if (!Properties.Any())
        {
            if (Buffer == null)
                throw new NoNullAllowedException("Since we never deserialized properties the buffer of FInstancedStruct cannot be null!");
            
            writer.Write(Buffer.Length);
            writer.WriteBytes(Buffer);
        }
        
        if (asset is null)
            throw new NoNullAllowedException("Asset cannot be null!");
        
        var properties = new Writer();
        
        asset.WriteProperties(properties, Index.ResolvedObject?.Name.ToString() 
                                          ?? throw new NoNullAllowedException("ResolvedObject cannot be null!"), Properties);
        
        writer.Write((int)properties.Length); // Write Buffer.Length
        properties.CopyTo(writer); // Write Buffer
    }
}