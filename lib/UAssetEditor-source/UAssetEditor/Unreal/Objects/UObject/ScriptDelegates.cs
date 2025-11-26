// https://github.com/FabianFG/CUE4Parse/blob/master/CUE4Parse/UE4/Objects/UObject/ScriptDelegates.cs

using UAssetEditor.Binary;
using UAssetEditor.Unreal.Assets;
using UAssetEditor.Unreal.Names;

namespace UAssetEditor.Unreal.Objects.UObject;

/// <summary>
/// Script delegate base class.
/// </summary>
public class FScriptDelegate
{
    /// <summary>
    /// The object bound to this delegate, or null if no object is bound
    /// </summary>
    public FPackageIndex Object;

    /// <summary>
    /// Name of the function to call on the bound object
    /// </summary>
    public FName FunctionName;

    public FScriptDelegate(Asset asset)
    {
        Object = FPackageIndex.Read(asset);
        FunctionName = new FName(asset, asset.NameMap);
    }

    public FScriptDelegate(FPackageIndex obj, FName functionName)
    {
        Object = obj;
        FunctionName = functionName;
    }
}

/// <summary>
/// Script multi-cast delegate base class
/// </summary>
public class FMulticastScriptDelegate
{
    /// <summary>
    /// Ordered list functions to invoke when the Broadcast function is called
    /// </summary>
    public FScriptDelegate[] InvocationList;

    public FMulticastScriptDelegate(Asset asset)
    {
        var length = asset.Read<int>();
        InvocationList = asset.ReadArray(() => new FScriptDelegate(asset), length);
    }

    public FMulticastScriptDelegate(FScriptDelegate[] invocationList)
    {
        InvocationList = invocationList;
    }
}