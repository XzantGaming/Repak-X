using System.Data;
using UAssetEditor.Binary;
using UAssetEditor.Unreal.Assets;
using UAssetEditor.Unreal.Names;
using UAssetEditor.Unreal.Properties;
using UAssetEditor.Unreal.Properties.Structs;
using UAssetEditor.Unreal.Properties.Unversioned;

namespace UAssetEditor.Unreal.Exports.Engine;

public enum ECurveTableMode : byte
{
    Empty,
    SimpleCurves,
    RichCurves
}

// https://github.com/FabianFG/CUE4Parse/blob/master/CUE4Parse/UE4/Assets/Exports/Engine/UCurveTable.cs
public class UCurveTable : UObject
{
    public Dictionary<FName, List<UProperty>> RowMap { get; set; } // FStructFallback is FRealCurve aka FSimpleCurve if CurveTableMode is SimpleCurves else FRichCurve
    public ECurveTableMode CurveTableMode { get; private set; }
    
    public UCurveTable(Asset asset) : base(asset)
    { }
    
    public override void Deserialize(long position)
    {
        base.Deserialize(position);

        var numRows = Owner!.Read<int>();
        
        var bUpgradingCurveTable = false; // PackageVersion < FFortniteMainBranchObjectVersion.Type.ShrinkCurveTableSize
        if (bUpgradingCurveTable)
            CurveTableMode = numRows > 0 ? ECurveTableMode.RichCurves : ECurveTableMode.Empty;
        else
            CurveTableMode = Owner.Read<ECurveTableMode>();
        
        RowMap = new Dictionary<FName, List<UProperty>>(numRows);
        for (var i = 0; i < numRows; i++)
        {
            var rowName = new FName(Owner, Owner.NameMap);
            var rowStruct = CurveTableMode switch
            {
                ECurveTableMode.SimpleCurves => "SimpleCurve",
                ECurveTableMode.RichCurves => "RichCurve",
                _ => ""
            };

            RowMap[rowName] = Owner.ReadProperties(rowStruct);
        }
    }
    
    public override void Serialize(Writer writer)
    {
        if (Owner?.Mappings == null)
            throw new NoNullAllowedException("Mappings must be present in order to serialize!");
        
        base.Serialize(writer);

        writer.Write(RowMap.Count);
        
        var bUpgradingCurveTable = false; // PackageVersion < FFortniteMainBranchObjectVersion.Type.ShrinkCurveTableSize
        if (!bUpgradingCurveTable)
            writer.Write(CurveTableMode);

        var rowStructName = CurveTableMode switch
        {
            ECurveTableMode.SimpleCurves => "SimpleCurve",
            ECurveTableMode.RichCurves => "RichCurve",
            _ => ""
        };

        var schema = Owner?.Mappings?.FindSchema(rowStructName);
        if (schema == null)
            throw new KeyNotFoundException($"Could not find schema named '{rowStructName}'");
        var rowStruct = new UStruct(schema, Owner!.Mappings!);
        
        foreach (var row in RowMap)
        {
            row.Key.Serialize(writer, Owner!.NameMap);
            UnversionedPropertyHandler.SerializeProperties((ZenAsset)Owner!, writer, rowStruct, row.Value);
        }
    }
}