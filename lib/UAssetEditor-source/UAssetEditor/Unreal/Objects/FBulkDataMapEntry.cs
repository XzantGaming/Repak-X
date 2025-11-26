using UAssetEditor.Binary;

namespace UAssetEditor.Unreal.Objects;

// https://github.com/EpicGames/UnrealEngine/blob/585df42eb3a391efd295abd231333df20cddbcf3/Engine/Source/Runtime/CoreUObject/Public/Serialization/BulkDataCookedIndex.h#L33
public struct FBulkDataCookedIndex
{
	public sbyte Value;
}

public class FBulkDataMapEntry
{
	public const int SIZE = 32;
	
	public long SerialOffset;
    public long DuplicateSerialOffset;
    public long SerialSize;
    public int Flags;
    public FBulkDataCookedIndex CookedIndex;
    // uint8 Pad[3] = { 0, 0, 0 };

    public FBulkDataMapEntry(Reader reader)
    {
	    SerialOffset = reader.Read<long>();
	    DuplicateSerialOffset = reader.Read<long>();
	    SerialSize = reader.Read<long>();
	    Flags = reader.Read<int>();
	    CookedIndex = reader.Read<FBulkDataCookedIndex>();
	    reader.Position += 3;
    }

    public void Serialize(Writer writer)
    {
	    writer.Write(SerialOffset);
	    writer.Write(DuplicateSerialOffset);
	    writer.Write(SerialSize);
	    writer.Write(Flags);
	    writer.Write(CookedIndex);
	    writer.Position += 3;
    }
}