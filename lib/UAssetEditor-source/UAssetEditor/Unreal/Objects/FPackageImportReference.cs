namespace UAssetEditor.Unreal.Objects;

// https://github.com/FabianFG/CUE4Parse/blob/87020fa42ab70bb44a08bcd9f5d742ad70c97373/CUE4Parse/UE4/IO/Objects/FPackageImportReference.cs
public struct FPackageImportReference
{
    public uint ImportedPackageIndex;
    public uint ImportedPublicExportHashIndex;
}