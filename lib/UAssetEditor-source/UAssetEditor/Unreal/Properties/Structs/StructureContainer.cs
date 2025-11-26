using Serilog;
using UAssetEditor.Unreal.Exports;
using UAssetEditor.Classes.Containers;

namespace UAssetEditor.Unreal.Properties.Structs;

public class LoadedStructure(StructureContainer container, UStruct struc)
{
    public UStruct Structure => struc;

    public string Name => struc.Name;
    public UProperty[] Properties => struc.Properties.ToArray();
    
    public void Release()
    {
        container.Remove(this);
    }
}

public class StructureContainer() : Container<LoadedStructure>(new List<LoadedStructure>())
{
    public UStruct? this[string name]
    {
        get
        {
            foreach (var structure in Items)
            {
                if (structure.Name != name)
                    continue;

                return structure.Structure;
            }

            return null;
        }
    }


    public bool Contains(string name)
    {
        return Items.Any(structure => structure.Name == name);
    }

    public void Add(UStruct struc)
    {
        if (!Contains(struc.Name))
        {
            Items.Add(new LoadedStructure(this, struc));
            Log.Information($"Saved {struc.Name} for later use");
        }
    }
}