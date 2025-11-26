namespace UAssetEditor.Unreal.Exports;

public enum EExportCommandType : uint
{
    ExportCommandType_Create,
    ExportCommandType_Serialize,
    ExportCommandType_Count
};

public struct FExportBundleEntry
{
    public uint LocalExportIndex;
    public EExportCommandType CommandType;
}